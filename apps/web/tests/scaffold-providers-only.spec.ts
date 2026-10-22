import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { zstdCompressSync } from 'node:zlib'
import { describe, expect, it } from 'vitest'
import { SessionLogCorruptError } from '@deepseek-ai/dsh-session-persistence'
import { assertProvidersOnlyFixture } from './scaffold.ts'

const caseDirectory = fileURLToPath(new URL('../../../snapshots/web/plan-narrow-viewport', import.meta.url))

/**
 * Read the newest committed generation of the case's recording.
 * The directory gains a higher-version file when the format moves, so a literal name would
 * silently pin the spec to a superseded fixture.
 * @returns the chosen fixture's bytes.
 */
async function fixtureBytes(): Promise<Buffer> {
  const names = (await readdir(caseDirectory)).filter(name => /^session\.v\d+\.jsonl$/u.test(name))
  const newest = names.sort((left, right) => Number(/(\d+)/u.exec(left)?.[1]) - Number(/(\d+)/u.exec(right)?.[1])).at(-1)
  if (newest === undefined) throw new Error('the case commits no recording')
  return readFile(join(caseDirectory, newest))
}

describe('providers-only replay fixture guard', () => {
  it('accepts the recorded fixture and reads its header through the session reader', async () => {
    const records = await assertProvidersOnlyFixture(await fixtureBytes())

    expect(records[0]).toMatchObject({ type: 'session' })
  })

  // A Zstandard fixture must reach the reader still framed: decoding it to text first would
  // replace every non-UTF-8 byte and turn a valid container into unrecognized damage.
  it('accepts a Zstandard-framed fixture', async () => {
    const compressed = zstdCompressSync(await fixtureBytes())

    await expect(assertProvidersOnlyFixture(compressed)).resolves.toBeDefined()
  })

  it('refuses a fixture whose first record is not a session header', async () => {
    await expect(assertProvidersOnlyFixture(Buffer.from('{"type":"turn/start","seq":0,"time":1,"data":{"turn":1}}\n')))
      .rejects.toThrow('replayProvidersOnly fixture must open with a session header row')
  })

  it('refuses a fixture that frames no further than its first line', async () => {
    const damaged = `${JSON.stringify({ type: 'session', version: 3 })}\n{truncated`

    await expect(assertProvidersOnlyFixture(Buffer.from(damaged)))
      .rejects.toBeInstanceOf(SessionLogCorruptError)
  })
})
