import { Context } from '@deepseek-ai/cordis'
import { SESSION_FORMAT_VERSION, SessionId, SessionLogOffset, SessionSeq } from '@deepseek-ai/dsh-session'
import type { SessionEvent, SessionHeader } from '@deepseek-ai/dsh-session'
import { SessionFormatUnsupportedError, SessionLogCorruptError } from '@deepseek-ai/dsh-session-persistence'
import * as JsonlSessionPersistence from '@deepseek-ai/dsh-session-persistence-jsonl'
import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterEach, describe, expect, it } from 'vitest'
import { eventLines, toHeaderLine } from '../src/format.ts'
import { readSessionLogPrefix } from '../src/reader.ts'
import { compressZstdFrame } from '../src/zstd.ts'

const { decodeSessionLog, readSessionLog } = JsonlSessionPersistence

const fixtureDir = fileURLToPath(new URL('./fixtures/', import.meta.url))
const roots: string[] = []
const contexts: Context[] = []

afterEach(async () => {
  await Promise.all(contexts.splice(0).map(ctx => ctx.fiber.dispose()))
  await Promise.all(roots.splice(0).map(root => rm(root, { recursive: true, force: true })))
})

/** One current-format header record as the writer stores it. */
function headerRecord(id: string, version = SESSION_FORMAT_VERSION): Record<string, unknown> {
  return {
    type: 'session', version, id, createdAt: 1,
    ...(version >= 2 ? { isSeeded: false } : {}), delegationDepth: 0,
  }
}

const turnStart = { type: 'turn/start', seq: 0, time: 1, data: { turn: 1 } }
const turnEnd = { type: 'turn/end', seq: 1, time: 2, data: { turn: 1, reason: { kind: 'completed' } } }

/** Frame every record as its own complete Zstandard frame, as one batch per frame. */
async function frameRecords(records: readonly unknown[]): Promise<Buffer> {
  const frames: Buffer[] = []
  for (const record of records) frames.push(await compressZstdFrame(`${JSON.stringify(record)}\n`))
  return Buffer.concat(frames)
}

/** One RFC 8878 skippable frame carrying arbitrary user data. */
function skippableFrame(body: Buffer, magic = 0x184D2A50): Buffer {
  const header = Buffer.alloc(8)
  header.writeUInt32LE(magic, 0)
  header.writeUInt32LE(body.length, 4)
  return Buffer.concat([header, body])
}

describe('readSessionLog', () => {
  it('decodes every concatenated frame of one log into header-first records', async () => {
    const records = [headerRecord('multi-frame'), turnStart, turnEnd]

    const decoded = await readSessionLog(await frameRecords(records))

    expect(decoded.version).toBe(SESSION_FORMAT_VERSION)
    expect(decoded.records).toEqual(records)
  })

  it('reassembles a record split across adjacent frames', async () => {
    const text = `${JSON.stringify(headerRecord('split-record'))}\n${JSON.stringify(turnStart)}\n`
    const split = text.indexOf('turn/start') + 4

    const decoded = await readSessionLog(Buffer.concat([
      await compressZstdFrame(text.slice(0, split)),
      await compressZstdFrame(text.slice(split)),
    ]))

    expect(decoded.records).toEqual([headerRecord('split-record'), turnStart])
  })

  it.each([0, 1, 2, 3, 4])('decodes the golden v%s fixture under its own format codec', async (version) => {
    const decoded = await readSessionLog(await readFile(join(fixtureDir, `session-log-v${version}.jsonl.zstd`)))

    expect(decoded.version).toBe(version)
    expect(decoded.records).toEqual([
      headerRecord(`golden-v${version}`, version),
      turnStart,
      turnEnd,
    ])
  })

  it('reads an uncompressed newline-delimited log', async () => {
    const bytes = Buffer.from(`${JSON.stringify(headerRecord('plain-text'))}\n${JSON.stringify(turnStart)}\n`)

    const decoded = await readSessionLog(bytes)

    expect(decoded.records).toEqual([headerRecord('plain-text'), turnStart])
  })

  it('scans the log a mounted writer committed', async () => {
    const root = await mkdtemp(join(tmpdir(), 'dsh-session-log-reader-'))
    roots.push(root)
    const ctx = new Context()
    contexts.push(ctx)
    await ctx.plugin(JsonlSessionPersistence, { root })
    const header: SessionHeader = {
      version: SESSION_FORMAT_VERSION,
      id: SessionId('reader-round-trip'),
      createdAt: 1,
      cwd: '/work',
      isSeeded: false,
      delegationDepth: 0,
    }
    const events: SessionEvent[] = [
      { type: 'turn/start', seq: SessionSeq(0), time: 1, data: { turn: 1 } },
      { type: 'turn/end', seq: SessionSeq(1), time: 2, data: { turn: 1, reason: { kind: 'completed' } } },
    ]
    const handle = await ctx.sessionPersistence.create(header)
    await handle.append(events)
    await handle.close()

    const files = (await readdir(root, { recursive: true })).filter(name => name.endsWith('.jsonl.zstd'))
    expect(files).toHaveLength(1)
    const decoded = await readSessionLog(await readFile(join(root, files[0] as string)))

    expect(decoded.version).toBe(SESSION_FORMAT_VERSION)
    expect(decoded.records).toEqual([
      JSON.parse(JSON.stringify(toHeaderLine(header, SessionLogOffset(0)))) as Record<string, unknown>,
      ...eventLines(events).split('\n').map(line => JSON.parse(line) as Record<string, unknown>),
    ])
  })

  it('refuses a log whose header comes from a newer format version', async () => {
    const newer = { ...headerRecord('from-the-future', SESSION_FORMAT_VERSION + 1) }

    await expect(readSessionLog(await frameRecords([newer]))).rejects.toThrow(
      new SessionFormatUnsupportedError(
        `stored Session uses newer format v${SESSION_FORMAT_VERSION + 1}; this build writes v${SESSION_FORMAT_VERSION}`,
      ),
    )
  })

  it('refuses a header this build cannot read', async () => {
    const malformed = { type: 'session', version: SESSION_FORMAT_VERSION, id: 'no-created-at', isSeeded: false }

    await expect(readSessionLog(await frameRecords([malformed]))).rejects.toMatchObject({
      name: 'SessionLogCorruptError',
      kind: 'no-header',
      damage: { line: 1 },
    })
  })

  it('refuses a torn final frame', async () => {
    const header = await compressZstdFrame(`${JSON.stringify(headerRecord('torn'))}\n`)
    const partial = await compressZstdFrame(`${JSON.stringify(turnStart)}\n`)
    const bytes = Buffer.concat([header, partial.subarray(0, partial.length - 3)])

    const failure: unknown = await readSessionLog(bytes).catch((error: unknown) => error)

    expect(failure).toBeInstanceOf(SessionLogCorruptError)
    expect(failure).toMatchObject({ kind: 'torn-frame', damage: { offset: header.length } })
    expect((failure as Error).message).toBe(`corrupt session log: incomplete Zstandard frame at byte ${header.length}`)
  })

  it('refuses an unparsable record', async () => {
    const bytes = await frameRecords([headerRecord('bad-json')])

    await expect(readSessionLog(Buffer.concat([bytes, await compressZstdFrame('{not-json}\n')])))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'invalid-json', damage: { line: 2 } })
  })

  it('refuses a record that is not a JSON object', async () => {
    const bytes = await frameRecords([headerRecord('not-an-object')])

    await expect(readSessionLog(Buffer.concat([bytes, await compressZstdFrame('42\n')])))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'not-object', damage: { line: 2 } })
  })

  it('marks an unterminated trailing line apart from mid-file corruption', async () => {
    const committed = `${JSON.stringify(headerRecord('torn-line'))}\n${JSON.stringify(turnStart)}\n`
    const appending = `${committed}${JSON.stringify({ type: 'turn/end', seq: 2 }).slice(0, 12)}`

    await expect(readSessionLog(Buffer.from(appending)))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'invalid-json', damage: { line: 3, trailing: true } })
    await expect(readSessionLog(Buffer.from(`${JSON.stringify(headerRecord('mid-file'))}\n{oops\n`)))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'invalid-json', damage: { line: 2 } })
  })

  it('refuses an empty log', async () => {
    await expect(readSessionLog(Buffer.alloc(0)))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'no-header', damage: { offset: 0 } })
  })

  it('refuses a later frame with a reserved header bit', async () => {
    const headerFrame = await compressZstdFrame(`${JSON.stringify(headerRecord('reserved-bit'))}\n`)
    const reservedHeader = Buffer.concat([ZSTD_MAGIC_BYTES, Buffer.from([0x08])])

    await expect(readSessionLog(Buffer.concat([headerFrame, reservedHeader]))).rejects.toMatchObject({
      name: 'SessionLogCorruptError',
      kind: 'invalid-container',
      damage: { offset: headerFrame.length + 4 },
    })
  })

  it('refuses a later frame with a reserved block type', async () => {
    const headerFrame = await compressZstdFrame(`${JSON.stringify(headerRecord('reserved-block'))}\n`)
    const reservedBlock = Buffer.concat([ZSTD_MAGIC_BYTES, Buffer.from([0x20, 0x00, 0x07, 0x00, 0x00])])

    await expect(readSessionLog(Buffer.concat([headerFrame, reservedBlock]))).rejects.toMatchObject({
      name: 'SessionLogCorruptError',
      kind: 'invalid-container',
      damage: { offset: headerFrame.length + 6 },
    })
  })

  it('refuses container damage after a valid first frame', async () => {
    const headerFrame = await compressZstdFrame(`${JSON.stringify(headerRecord('mid-stream'))}\n`)
    const damaged = Buffer.concat([headerFrame, breakFrameMagic(await compressZstdFrame(`${JSON.stringify(turnStart)}\n`), 0)])

    await expect(readSessionLog(damaged))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'invalid-container', damage: { offset: headerFrame.length } })
  })

  it('refuses bytes that open neither a Zstandard stream nor a JSONL record', async () => {
    const damagedMagic = await compressZstdFrame(`${JSON.stringify(headerRecord('bad-magic'))}\n`)
    const bytes = Buffer.concat([Buffer.from([0x00]), damagedMagic.subarray(1)])

    await expect(readSessionLog(bytes)).rejects.toMatchObject({
      name: 'SessionLogCorruptError',
      kind: 'unrecognized-container',
      damage: { offset: 1 },
    })
    await expect(readSessionLog(bytes)).rejects.toThrow('opens neither a Zstandard stream nor a JSONL record')
  })

  it('names the header record when the first record is not one', async () => {
    await expect(readSessionLog(await frameRecords([turnStart])))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'no-header', damage: { line: 1 } })
  })

  it('ignores blank lines between records', async () => {
    const text = `${JSON.stringify(headerRecord('blank-lines'))}\n\n${JSON.stringify(turnStart)}\n\n`

    const decoded = await readSessionLog(Buffer.from(text))

    expect(decoded.records).toEqual([headerRecord('blank-lines'), turnStart])
  })

  it('consumes a skippable frame that opens the log', async () => {
    const bytes = Buffer.concat([
      skippableFrame(Buffer.from('user data the reader never interprets')),
      await frameRecords([headerRecord('skippable-lead'), turnStart]),
    ])

    const decoded = await readSessionLog(bytes)

    expect(decoded.version).toBe(SESSION_FORMAT_VERSION)
    expect(decoded.records).toEqual([headerRecord('skippable-lead'), turnStart])
  })

  it('consumes a skippable frame between record frames', async () => {
    const bytes = Buffer.concat([
      await compressZstdFrame(`${JSON.stringify(headerRecord('skippable-mid'))}\n`),
      skippableFrame(Buffer.alloc(0), 0x184D2A5F),
      await compressZstdFrame(`${JSON.stringify(turnStart)}\n`),
    ])

    const decoded = await readSessionLog(bytes)

    expect(decoded.records).toEqual([headerRecord('skippable-mid'), turnStart])
  })

  it('reports a log of only skippable frames as header-less', async () => {
    const bytes = Buffer.concat([skippableFrame(Buffer.from('nothing committed')), skippableFrame(Buffer.from(''))])

    await expect(readSessionLog(bytes))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'no-header', damage: { offset: 0 } })
  })

  it('refuses a skippable frame whose declared body runs past the log', async () => {
    const header = Buffer.alloc(8)
    header.writeUInt32LE(0x184D2A50, 0)
    header.writeUInt32LE(64, 4)

    await expect(readSessionLog(Buffer.concat([header, Buffer.alloc(4)])))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'torn-frame', damage: { offset: 0 } })
  })
})

/** Run one reader call and return the typed corruption evidence it reports. */
async function corruptEvidence(run: () => Promise<unknown>): Promise<{
  readonly kind: string
  readonly damage: { offset?: number; line?: number; trailing?: boolean }
}> {
  try {
    await run()
  } catch (error: unknown) {
    if (!(error instanceof SessionLogCorruptError)) throw error
    return { kind: error.kind, damage: error.damage }
  }
  throw new Error('the reader accepted a damaged log')
}

const ZSTD_MAGIC_BYTES = Buffer.from([0x28, 0xB5, 0x2F, 0xFD])

/** Count the structural reads a prefix read performs, without changing what it reads. */
function countingBytes(bytes: Buffer): { readonly proxy: Buffer; readonly reads: () => number } {
  let reads = 0
  const counted = new Set(['readUInt32LE', 'readUInt8', 'readUIntLE'])
  const proxy = new Proxy(bytes, {
    get(target, property) {
      const value: unknown = Reflect.get(target, property, target)
      if (typeof value !== 'function') return value
      const call = value as (...args: number[]) => number
      if (typeof property === 'string' && counted.has(property)) {
        return (...args: number[]): number => {
          reads += 1
          return call.apply(target, args)
        }
      }
      return (...args: number[]): unknown => Reflect.apply(call, target, args)
    },
  })
  return { proxy, reads: () => reads }
}

/** Replace the frame magic at `offset` with bytes that open nothing. */
function breakFrameMagic(bytes: Buffer, offset: number): Buffer {
  const damaged = Buffer.from(bytes)
  damaged.writeUInt32LE(0x00000000, offset)
  return damaged
}

describe('SessionLogCorruptError evidence', () => {
  it('reports no-header with the offset of a log that holds no record', async () => {
    expect(await corruptEvidence(() => readSessionLog(Buffer.alloc(0))))
      .toEqual({ kind: 'no-header', damage: { offset: 0 } })
  })

  it('reports unrecognized-container with the offset of the leading byte', async () => {
    expect(await corruptEvidence(() => readSessionLog(Buffer.from([0x00, 0x28, 0xB5]))))
      .toEqual({ kind: 'unrecognized-container', damage: { offset: 1 } })
  })

  it('reports torn-frame with the start of the interrupted frame', async () => {
    const header = await compressZstdFrame(`${JSON.stringify(headerRecord('evidence-torn'))}\n`)
    const torn = await compressZstdFrame(`${JSON.stringify(turnStart)}\n`)

    expect(await corruptEvidence(() => readSessionLog(Buffer.concat([header, torn.subarray(0, -3)]))))
      .toEqual({ kind: 'torn-frame', damage: { offset: header.length } })
  })

  it('reports prefix-short with the decoded length that fell short', async () => {
    const header = `${JSON.stringify(headerRecord('evidence-short'))}\n`

    expect(await corruptEvidence(() => readSessionLogPrefix(Buffer.from(header), header.length + 1)))
      .toEqual({ kind: 'prefix-short', damage: { offset: header.length } })
  })

  it('reports invalid-json with its line and whether that line was unterminated', async () => {
    const header = `${JSON.stringify(headerRecord('evidence-json'))}\n`
    const committed = `${header}${JSON.stringify(turnStart)}\n`

    expect(await corruptEvidence(() => readSessionLog(Buffer.from(`${header}{oops\n`))))
      .toEqual({ kind: 'invalid-json', damage: { line: 2 } })
    expect(await corruptEvidence(() => readSessionLog(Buffer.from(`${committed}{"type":"turn/`))))
      .toEqual({ kind: 'invalid-json', damage: { line: 3, trailing: true } })
  })

  it('reports invalid-container with the offset of a broken frame', async () => {
    const header = await compressZstdFrame(`${JSON.stringify(headerRecord('evidence-container'))}\n`)

    expect(await corruptEvidence(() => readSessionLog(Buffer.concat([header, Buffer.alloc(4)]))))
      .toEqual({ kind: 'invalid-container', damage: { offset: header.length } })
  })

  it('reports not-object with the line that parsed to a non-record', async () => {
    const committed = `${JSON.stringify(headerRecord('evidence-object'))}\n[1,2]\n`

    expect(await corruptEvidence(() => readSessionLog(Buffer.from(committed))))
      .toEqual({ kind: 'not-object', damage: { line: 2 } })
  })
})

describe('decodeSessionLog', () => {
  it('decodes the framing without classifying the stored version', async () => {
    const projected: Record<string, unknown> = {
      type: 'session', version: 0, id: '{{session:1}}', createdAt: 1, cwd: '{{cwd}}/workspace', delegationDepth: 0,
    }

    const records = await decodeSessionLog(await frameRecords([projected, turnStart]))

    expect(records).toEqual([projected, turnStart])
    await expect(readSessionLog(await frameRecords([projected]))).rejects.toMatchObject({
      name: 'SessionLogCorruptError',
      kind: 'no-header',
    })
  })

  it('applies the same framing checks as the versioned reader', async () => {
    await expect(decodeSessionLog(Buffer.from('{not-json}\n')))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'invalid-json', damage: { line: 1 } })
  })
})

describe('readSessionLogPrefix', () => {
  it('returns the requested prefix of an uncompressed log', async () => {
    const text = `${JSON.stringify(headerRecord('prefix-plain'))}\n${JSON.stringify(turnStart)}\n`

    const prefix = await readSessionLogPrefix(Buffer.from(text), `${JSON.stringify(headerRecord('prefix-plain'))}\n`.length)

    expect(prefix.toString('utf8')).toBe(`${JSON.stringify(headerRecord('prefix-plain'))}\n`)
  })

  it('decodes only the frames that cover a compressed prefix', async () => {
    const header = `${JSON.stringify(headerRecord('prefix-zstd'))}\n`
    const bytes = Buffer.concat([
      await compressZstdFrame(header),
      await compressZstdFrame(`${JSON.stringify(turnStart)}\n`),
    ])

    expect((await readSessionLogPrefix(bytes, header.length)).toString('utf8')).toBe(header)
  })

  it('refuses a prefix longer than the decoded log', async () => {
    const header = `${JSON.stringify(headerRecord('short-prefix'))}\n`
    const compressed = await compressZstdFrame(header)
    const plain = Buffer.from(header)

    await expect(readSessionLogPrefix(compressed, header.length + 1))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'prefix-short' })
    await expect(readSessionLogPrefix(plain, header.length + 1))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'prefix-short' })
  })

  it('refuses a prefix length that is not a non-negative safe integer', async () => {
    const plain = Buffer.from(`${JSON.stringify(headerRecord('bad-prefix'))}\n`)

    await expect(readSessionLogPrefix(plain, -1))
      .rejects.toThrow('session log prefix length must be a non-negative safe integer, got -1')
  })

  it('refuses a prefix whose complete frames do not cover it', async () => {
    const header = `${JSON.stringify(headerRecord('torn-prefix'))}\n`
    const compressed = await compressZstdFrame(header)

    await expect(readSessionLogPrefix(compressed.subarray(0, compressed.length - 3), 8))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'torn-frame', damage: { offset: 0 } })
    await expect(readSessionLogPrefix(compressed.subarray(0, compressed.length - 3), header.length))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'torn-frame' })
  })

  it('returns an empty prefix from an empty log', async () => {
    expect(await readSessionLogPrefix(Buffer.alloc(0), 0)).toEqual(Buffer.alloc(0))
  })

  it('returns an empty prefix without reading a damaged container', async () => {
    const torn = (await compressZstdFrame(`${JSON.stringify(headerRecord('zero-prefix'))}\n`)).subarray(0, 6)

    expect(await readSessionLogPrefix(torn, 0)).toEqual(Buffer.alloc(0))
    expect(await readSessionLogPrefix(Buffer.from([0x00, 0x01]), 0)).toEqual(Buffer.alloc(0))
  })

  it('decodes a prefix that follows a leading skippable frame', async () => {
    const header = `${JSON.stringify(headerRecord('skippable-prefix'))}\n`
    const bytes = Buffer.concat([skippableFrame(Buffer.from('metadata')), await compressZstdFrame(header)])

    expect((await readSessionLogPrefix(bytes, header.length)).toString('utf8')).toBe(header)
    await expect(readSessionLogPrefix(skippableFrame(Buffer.from('metadata')), 1))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'prefix-short', damage: { offset: 0 } })
  })

  it('returns the proven prefix when a later frame is corrupt', async () => {
    const header = `${JSON.stringify(headerRecord('prefix-damage'))}\n`
    const headerFrame = await compressZstdFrame(header)
    const damaged = Buffer.concat([headerFrame, breakFrameMagic(await compressZstdFrame('\n'), 0)])

    expect((await readSessionLogPrefix(damaged, header.length)).toString('utf8')).toBe(header)
  })

  it('refuses a prefix that needs the corrupt frame', async () => {
    const header = `${JSON.stringify(headerRecord('prefix-damage'))}\n`
    const headerFrame = await compressZstdFrame(header)
    const damaged = Buffer.concat([headerFrame, breakFrameMagic(await compressZstdFrame('\n'), 0)])

    await expect(readSessionLogPrefix(damaged, header.length + 1))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'invalid-container', damage: { offset: headerFrame.length } })
  })

  it('reads each frame header once when a prefix covers a long log', async () => {
    const records = Array.from({ length: 48 }, (_, index) => `${JSON.stringify({
      type: 'turn/start', seq: index, time: index, data: { turn: index },
    })}\n`)
    const frames: Buffer[] = []
    for (const record of records) frames.push(await compressZstdFrame(record))
    const bytes = Buffer.concat(frames)
    const plaintext = records.join('')
    const { proxy, reads } = countingBytes(bytes)

    expect((await readSessionLogPrefix(proxy, plaintext.length)).toString('utf8')).toBe(plaintext)
    // One header read per frame plus a constant per frame is linear; rescanning
    // from the container start would read O(frames^2) headers here.
    expect(reads()).toBeLessThanOrEqual(records.length * 8)
  })

  it('decodes a proven prefix beside a half-written frame', async () => {
    const header = `${JSON.stringify(headerRecord('mid-append'))}\n`
    const first = await compressZstdFrame(header)
    const partial = await compressZstdFrame(`${JSON.stringify(turnStart)}\n`)
    const bytes = Buffer.concat([first, partial.subarray(0, partial.length - 3)])

    expect((await readSessionLogPrefix(bytes, header.length)).toString('utf8')).toBe(header)
    await expect(readSessionLogPrefix(bytes, header.length + 1))
      .rejects.toMatchObject({ name: 'SessionLogCorruptError', kind: 'torn-frame', damage: { offset: first.length } })
  })
})
