/**
 * Whole-log reader for the JSONL session container. A committed session log is
 * newline-delimited JSON records inside one Zstandard stream of independently
 * decodable frames; this module decodes every frame, splits the records, and
 * validates the header-first format version through the format catalog. Callers
 * that inspect a stored log use this reader instead of inventing their own
 * decompression and framing.
 * @module dsh-session-persistence-jsonl/reader
 */

import { sessionFormatCatalog } from '@deepseek-ai/dsh-session-format-catalog'
import { isSessionFormatJsonObject } from '@deepseek-ai/dsh-session-format'
import { SessionFormatUnsupportedError, SessionLogCorruptError } from '@deepseek-ai/dsh-session-persistence'
import { decompressZstdFrame, isSkippableFrameMagic, scanZstdFrames, type ZstdScanOptions } from './zstd.ts'

const ZSTD_FRAME_MAGIC = 0xFD2FB528
const JSONL_OPENING_BYTE = 0x7B
/** The reader's container: skippable frames are members, never damage. */
const CONTAINER_SCAN: ZstdScanOptions = { allowSkippable: true }

/** One physical JSONL record decoded from a session log; the first record is the header. */
export type SessionLogRecord = Record<string, unknown>

/** Header-first decoded content of one complete session log. */
export interface SessionLogRecords {
  /** Format version stored in the header record: the current version or a supported historical one. */
  readonly version: number
  /** Header record first, then every following record in file order. */
  readonly records: readonly [SessionLogRecord, ...SessionLogRecord[]]
}

/**
 * Read one complete session log into its header-first records.
 *
 * The container is detected from its leading bytes: a Zstandard stream is
 * decoded across every concatenated frame while skippable frames are consumed
 * unread, and raw newline-delimited UTF-8 is read directly. Records are split on
 * newlines, blank lines are ignored, and the first record must be a
 * `type: 'session'` header whose stored version this build classifies as
 * current, migration-required, or unsupported. A torn final frame, broken frame
 * structure anywhere in the container, and any unparsable or non-object record
 * reject as corruption — a whole-log read never runs while a later write is
 * appending, so damage anywhere in the file is damage in the log this call was
 * given.
 *
 * @param bytes - complete log bytes as stored on disk.
 * @returns the stored format version and every record, header first.
 */
export async function readSessionLog(bytes: Buffer): Promise<SessionLogRecords> {
  const records = await decodeSessionLog(bytes)
  const header = records[0]
  if (header['type'] !== 'session') {
    throw new SessionLogCorruptError('no-header', 'first record is not a session header', { line: 1 })
  }
  const classification = sessionFormatCatalog.readHeader(header)
  switch (classification.status) {
    case 'current':
    case 'migration-required':
      return { version: classification.storedVersion, records }
    case 'unsupported':
      throw new SessionFormatUnsupportedError(classification.reason)
    case 'malformed':
      throw new SessionLogCorruptError(
        'no-header',
        `header record is not a valid session header: ${classification.reason}`,
        { line: 1 },
      )
    /* v8 ignore next 2 -- the header classification union is closed. */
    default:
      throw new Error(`unhandled Session format header classification: ${String(classification)}`)
  }
}

/**
 * Decode the framing of one session log into its header-first records without
 * classifying the stored format version.
 *
 * Framing and version classification are separate questions: this function
 * reports what the container holds, while {@link readSessionLog} additionally
 * decides whether this build can interpret that version. Callers that only need
 * the record boundaries — a fixture whose recorded header carries projection
 * tokens the current format validator rejects, for example — use this and own
 * the classification question themselves.
 *
 * @param bytes - complete log bytes as stored on disk.
 * @returns every record, header first, after the same framing checks as {@link readSessionLog}.
 */
export async function decodeSessionLog(bytes: Buffer): Promise<readonly [SessionLogRecord, ...SessionLogRecord[]]> {
  return parseRecords(await decodeSessionLogPlaintext(bytes))
}

/**
 * Decode the leading JSONL plaintext of one session log without reading beyond
 * the frames that cover it.
 *
 * Appended write batches start after a record boundary, so the leading
 * `plaintextBytes` of a longer log are exactly the bytes of the generation that
 * wrote them. A log whose decoded plaintext is shorter than the requested prefix
 * rejects as corruption. Because a concurrent writer may be mid-frame at the end
 * of the bytes this call received, an incomplete frame is damage only when the
 * complete frames before it decode to fewer than `plaintextBytes`; past that
 * point the requested prefix is already proven and the half-written tail is not
 * this call's evidence. For the same reason the scan stops at the frame that
 * covers the prefix, so a corrupted frame past it does not fail this call. A
 * zero-length prefix needs no bytes at all, so it returns an empty buffer without
 * reading the container.
 *
 * @param bytes - log bytes as stored on disk, possibly captured mid-append.
 * @param plaintextBytes - exact uncompressed prefix length to decode.
 * @returns the decoded prefix, exactly `plaintextBytes` long.
 */
export async function readSessionLogPrefix(bytes: Buffer, plaintextBytes: number): Promise<Buffer> {
  if (!Number.isSafeInteger(plaintextBytes) || plaintextBytes < 0) {
    throw new Error(`session log prefix length must be a non-negative safe integer, got ${String(plaintextBytes)}`)
  }
  if (plaintextBytes === 0) return Buffer.alloc(0)
  if (isZstdContainer(bytes)) {
    const { chunks, decoded } = await decodeFrames(bytes, plaintextBytes)
    if (decoded < plaintextBytes) throw prefixTooShort(decoded, plaintextBytes)
    return Buffer.concat(chunks, decoded).subarray(0, plaintextBytes)
  }
  assertJsonlContainer(bytes)
  if (bytes.length < plaintextBytes) throw prefixTooShort(bytes.length, plaintextBytes)
  return bytes.subarray(0, plaintextBytes)
}

/**
 * Decode the frames of one Zstandard stream that cover a plaintext prefix.
 *
 * A prefix read resumes the structural scan after each decoded frame and stops
 * at the frame that satisfies `targetBytes`, so its work is linear in the frames
 * the prefix covers and structural damage beyond it never enters this call's
 * evidence. A container scan for a whole log reads every frame and reports any
 * damage it finds.
 *
 * @param bytes - log bytes that begin with a Zstandard container frame.
 * @param targetBytes - stop once the decoded length reaches this prefix length; every frame when omitted.
 * @returns decoded plaintext chunks in frame order, possibly longer than `targetBytes`, and their total length.
 */
async function decodeFrames(
  bytes: Buffer,
  targetBytes?: number,
): Promise<{ readonly chunks: Buffer[]; readonly decoded: number }> {
  const chunks: Buffer[] = []
  let decoded = 0
  if (targetBytes === undefined) {
    const { frames, tornStart } = scanZstdFrames(bytes, Number.POSITIVE_INFINITY, CONTAINER_SCAN)
    for (const frame of frames) {
      const chunk = await decompressZstdFrame(bytes.subarray(frame.start, frame.end))
      chunks.push(chunk)
      decoded += chunk.length
    }
    if (tornStart !== undefined) throw tornFrame(tornStart)
    return { chunks, decoded }
  }
  let offset = 0
  for (;;) {
    const scan = scanZstdFrames(bytes, 1, { ...CONTAINER_SCAN, start: offset })
    const frame = scan.frames[0]
    if (frame === undefined) {
      if (scan.tornStart !== undefined) throw tornFrame(scan.tornStart)
      return { chunks, decoded }
    }
    const chunk = await decompressZstdFrame(bytes.subarray(frame.start, frame.end))
    chunks.push(chunk)
    decoded += chunk.length
    offset = frame.end
    if (decoded >= targetBytes) return { chunks, decoded }
  }
}

function tornFrame(offset: number): SessionLogCorruptError {
  return new SessionLogCorruptError('torn-frame', `incomplete Zstandard frame at byte ${offset}`, { offset })
}

function prefixTooShort(decoded: number, plaintextBytes: number): SessionLogCorruptError {
  return new SessionLogCorruptError(
    'prefix-short',
    `${decoded} decoded plaintext bytes are shorter than the expected ${plaintextBytes}`,
    { offset: decoded },
  )
}

/**
 * Reject bytes that are neither a Zstandard stream nor JSONL.
 *
 * A Zstandard log with a damaged magic would otherwise reach the record parser
 * and misreport itself as an unparsable line; naming the container instead keeps
 * the diagnostic about what is actually wrong.
 * @param bytes - log bytes that failed the Zstandard magic check.
 */
function assertJsonlContainer(bytes: Buffer): void {
  const offset = bytes.findIndex(byte => byte > 0x20)
  const firstByte = offset === -1 ? undefined : bytes[offset]
  if (firstByte === undefined) {
    throw new SessionLogCorruptError('no-header', 'log holds no records', { offset: 0 })
  }
  if (firstByte !== JSONL_OPENING_BYTE) {
    throw new SessionLogCorruptError(
      'unrecognized-container',
      `leading byte 0x${firstByte.toString(16)} opens neither a Zstandard stream nor a JSONL record`,
      { offset },
    )
  }
}

/**
 * Report whether the log bytes open a Zstandard container.
 *
 * A skippable frame is a container member with user data where compressed
 * plaintext would be, so leading with one is a valid log container rather than
 * damage.
 * @param bytes - log bytes to classify by their leading magic.
 */
function isZstdContainer(bytes: Buffer): boolean {
  if (bytes.length < 4) return false
  const magic = bytes.readUInt32LE(0)
  return magic === ZSTD_FRAME_MAGIC || isSkippableFrameMagic(magic)
}

/** Decode one concatenated Zstandard stream into its complete JSONL plaintext. */
async function decodeSessionLogPlaintext(bytes: Buffer): Promise<Buffer> {
  if (!isZstdContainer(bytes)) {
    assertJsonlContainer(bytes)
    return bytes
  }
  return Buffer.concat((await decodeFrames(bytes)).chunks)
}

/** Split decoded plaintext into JSON records, ignoring blank lines. */
function parseRecords(plaintext: Buffer): readonly [SessionLogRecord, ...SessionLogRecord[]] {
  const records: SessionLogRecord[] = []
  const text = plaintext.toString('utf8')
  const lines = text.split('\n')
  const unterminated = lines.length
  for (const [index, line] of lines.entries()) {
    if (line.trim() === '') continue
    records.push(parseRecord(line, index + 1, index + 1 === unterminated && !text.endsWith('\n')))
  }
  const [header, ...rest] = records
  if (header === undefined) throw new SessionLogCorruptError('no-header', 'log holds no records', { offset: 0 })
  return [header, ...rest]
}

/**
 * Parse one JSONL record.
 * @param line - one non-blank plaintext line.
 * @param lineNumber - 1-based position in the decoded plaintext.
 * @param trailing - whether this line is the unterminated final line of the log.
 * @returns the parsed record.
 */
function parseRecord(line: string, lineNumber: number, trailing: boolean): SessionLogRecord {
  let decoded: unknown
  try {
    decoded = JSON.parse(line)
  } catch {
    throw new SessionLogCorruptError('invalid-json', `line ${lineNumber} is not valid JSON`, {
      line: lineNumber,
      ...(trailing ? { trailing: true } : {}),
    })
  }
  if (!isSessionFormatJsonObject(decoded)) {
    throw new SessionLogCorruptError('not-object', `line ${lineNumber} is not a JSON object`, { line: lineNumber })
  }
  return decoded
}
