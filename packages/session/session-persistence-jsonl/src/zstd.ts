/**
 * Zstandard frame primitives for the JSONL persistence backend. The backend
 * owns a concatenated-frame container so it can append and recover batches
 * without exposing compression mechanics through the persistence seam.
 * @module dsh-session-persistence-jsonl/zstd
 */

import {
  constants, zstdCompress, zstdDecompress, type ZstdOptions,
} from 'node:zlib'
import { promisify } from 'node:util'
import { SessionLogCorruptError } from '@deepseek-ai/dsh-session-persistence'
import { NodePrivateZstdFrameDecoder } from './zstd-private-decoder.ts'
import { PublicZstdFrameDecoder } from './zstd-public-decoder.ts'

const ZSTD_MAGIC = 0xFD2FB528
const ZSTD_SKIPPABLE_MAGIC_MIN = 0x184D2A50
const ZSTD_SKIPPABLE_MAGIC_MAX = 0x184D2A5F
const zstdCompressAsync = promisify(zstdCompress)
const zstdDecompressAsync = promisify(zstdDecompress)
const CHECKSUM_OPTIONS: ZstdOptions = {
  params: { [constants.ZSTD_c_checksumFlag]: 1 },
}
const INCOMPLETE_FRAME_OPTIONS: ZstdOptions = {
  finishFlush: constants.ZSTD_e_flush,
}

/** Byte range occupied by one structurally complete Zstandard frame. */
export interface ZstdFrameRange {
  /** Inclusive frame start. */
  start: number
  /** Exclusive frame end. */
  end: number
}

/** Structural scan options for a concatenated Zstandard stream. */
export interface ZstdScanOptions {
  /**
   * Consume RFC 8878 skippable frames as container members and omit them from
   * `frames`, instead of rejecting their magic. A skippable frame carries user
   * data, never compressed plaintext, so a decoder has nothing to decode.
   */
  readonly allowSkippable?: boolean
  /**
   * First byte offset to scan, for a caller continuing after a frame it already
   * decoded. Every reported range, torn start, and failure offset stays absolute
   * in `buffer`. Must be an integer in `0..buffer.length`; a value outside that
   * range throws `RangeError`, because it is a caller error rather than damage
   * in the log.
   */
  readonly start?: number
}

/** Structural scan result for a concatenated Zstandard stream. */
export interface ZstdFrameScan {
  /** Complete frames in file order. */
  frames: ZstdFrameRange[]
  /** Start of an incomplete final frame, when EOF interrupts one. */
  tornStart?: number
}

/**
 * Report whether one frame magic opens a Zstandard skippable frame.
 * @param magic - the 4-byte little-endian magic as read from the stream.
 * @returns whether the magic is one of the sixteen skippable-frame values.
 */
export function isSkippableFrameMagic(magic: number): boolean {
  return magic >= ZSTD_SKIPPABLE_MAGIC_MIN && magic <= ZSTD_SKIPPABLE_MAGIC_MAX
}

/**
 * Resolve the offset a scan begins at, rejecting a caller error instead of
 * reporting it as a container with no frames.
 * @param buffer - the bytes about to be scanned, which bound the valid offsets.
 * @param start - the requested first offset, when the caller supplied one.
 * @returns the first offset to scan.
 */
function scanStart(buffer: Buffer, start: number | undefined): number {
  if (start === undefined) return 0
  if (!Number.isSafeInteger(start) || start < 0 || start > buffer.length) {
    throw new RangeError(`ZstdScanOptions.start must be an integer within 0..${buffer.length}, got ${String(start)}`)
  }
  return start
}

/**
 * Locate complete frames without decompressing their blocks. Invalid complete
 * structure rejects as `SessionLogCorruptError` with kind `invalid-container`
 * and the byte offset of the failed check; EOF inside the final frame returns
 * its start for repair.
 * @param buffer - complete bytes currently present in the session artifact.
 * @param maxFrames - optional complete-frame limit for metadata-only readers.
 * @param options - scanning behavior: whether skippable frames are container members to consume, and where to resume.
 * @returns complete frame ranges and an optional incomplete-final-frame start.
 * @throws RangeError when `options.start` is not an integer within the buffer.
 */
export function scanZstdFrames(
  buffer: Buffer,
  maxFrames = Number.POSITIVE_INFINITY,
  options: ZstdScanOptions = {},
): ZstdFrameScan {
  const frames: ZstdFrameRange[] = []
  let offset = scanStart(buffer, options.start)

  while (offset < buffer.length) {
    const start = offset
    if (buffer.length - offset < 4) return { frames, tornStart: start }
    const magic = buffer.readUInt32LE(offset)
    if (options.allowSkippable === true && isSkippableFrameMagic(magic)) {
      // Skippable frame: magic, a 4-byte body length, then that body.
      if (buffer.length - offset < 8) return { frames, tornStart: start }
      const bodyBytes = buffer.readUInt32LE(offset + 4)
      if (buffer.length - offset - 8 < bodyBytes) return { frames, tornStart: start }
      offset += 8 + bodyBytes
      continue
    }
    if (magic !== ZSTD_MAGIC) {
      throw new SessionLogCorruptError('invalid-container', `invalid frame magic at byte ${offset}`, { offset })
    }
    offset += 4

    if (offset === buffer.length) return { frames, tornStart: start }
    const descriptor = buffer.readUInt8(offset)
    offset += 1
    if ((descriptor & 0x18) !== 0) {
      throw new SessionLogCorruptError('invalid-container', `reserved frame-header bit at byte ${offset - 1}`, {
        offset: offset - 1,
      })
    }

    const contentSizeFlag = descriptor >>> 6
    const singleSegment = (descriptor & 0x20) !== 0
    const checksum = (descriptor & 0x04) !== 0
    const dictionaryFlag = descriptor & 0x03
    const dictionaryBytes = dictionaryFlag === 3 ? 4 : dictionaryFlag
    const contentSizeBytes = contentSizeFlag === 0
      ? (singleSegment ? 1 : 0)
      : 1 << contentSizeFlag
    const remainingHeaderBytes = (singleSegment ? 0 : 1) + dictionaryBytes + contentSizeBytes
    if (buffer.length - offset < remainingHeaderBytes) return { frames, tornStart: start }
    offset += remainingHeaderBytes

    for (;;) {
      if (buffer.length - offset < 3) return { frames, tornStart: start }
      const blockHeader = buffer.readUIntLE(offset, 3)
      offset += 3
      const lastBlock = (blockHeader & 1) !== 0
      const blockType = (blockHeader >>> 1) & 0x03
      const blockSize = blockHeader >>> 3
      if (blockType === 0x03) {
        throw new SessionLogCorruptError('invalid-container', `reserved block type at byte ${offset - 3}`, {
          offset: offset - 3,
        })
      }
      const payloadBytes = blockType === 0x01 ? 1 : blockSize
      if (buffer.length - offset < payloadBytes) return { frames, tornStart: start }
      offset += payloadBytes
      if (lastBlock) break
    }

    if (checksum) {
      if (buffer.length - offset < 4) return { frames, tornStart: start }
      offset += 4
    }
    frames.push({ start, end: offset })
    if (frames.length === maxFrames) return { frames }
  }

  return { frames }
}

/**
 * Compress one independently decodable, checksummed Zstandard frame.
 * @param input - JSONL bytes for a header or durable event batch.
 * @returns the complete encoded frame.
 */
export async function compressZstdFrame(input: Buffer | string): Promise<Buffer> {
  return zstdCompressAsync(input, CHECKSUM_OPTIONS)
}

/**
 * Decompress one complete frame and validate its checksum.
 * @param input - one structurally complete Zstandard frame.
 * @returns the frame plaintext.
 */
export async function decompressZstdFrame(input: Buffer): Promise<Buffer> {
  return zstdDecompressAsync(input)
}

/** Common lifecycle for interchangeable synchronous multi-frame decoders. */
export interface ZstdFrameDecoder {
  /**
   * Decode and checksum complete frames in source order. Each yielded buffer
   * remains valid only until the iterator advances to the next frame.
   * @param source - concatenated Zstandard frame bytes.
   * @param frames - structurally complete ranges within `source`.
   * @returns one plaintext buffer per frame.
   */
  decode(source: Buffer, frames: readonly ZstdFrameRange[]): Generator<Buffer, void, void>
  /** Release decoder-owned resources; repeated calls are harmless. */
  close(): void
}

/**
 * Select the shared private decoder when the running Node 22/24/26 shape is
 * compatible, otherwise preserve correctness with the public one-shot API.
 * @returns a synchronous decoder with an implementation-independent lifecycle.
 */
export function createZstdFrameDecoder(): ZstdFrameDecoder {
  return NodePrivateZstdFrameDecoder.create() ?? new PublicZstdFrameDecoder()
}

/**
 * Recover available plaintext from a structurally incomplete final frame.
 * `ZSTD_e_flush` deliberately suppresses final-frame and checksum completion;
 * callers must establish the torn frame boundary before using this helper.
 * @param input - available bytes from a known incomplete Zstandard frame.
 * @returns plaintext produced from the available input.
 */
export async function decompressZstdPrefix(input: Buffer): Promise<Buffer> {
  return zstdDecompressAsync(input, INCOMPLETE_FRAME_OPTIONS)
}
