// Validates a session log with the real TS reader + current-format validator.
// usage: node ts_validate.mjs <log.jsonl | log.jsonl.zstd> [repo-root]
// prints one JSON line: {ok:true,events,version} | {ok:false,error}
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const file = process.argv[2]
const repo = resolve(process.argv[3] ?? new URL('../../../../', import.meta.url).pathname)
const root = `${repo}/packages/session/`
try {
  const { decodeSessionLog } = await import(`${root}session-persistence-jsonl/lib/index.js`)
  const { sessionFormatCatalog } = await import(`${root}session-format-catalog/lib/index.js`)
  const records = await decodeSessionLog(readFileSync(file))
  const restore = sessionFormatCatalog.createRestore(records[0], { validation: 'current', recovery: 'strict' })
  for (const row of records.slice(1)) restore.decodeRow(row)
  const artifact = restore.finish()
  console.log(JSON.stringify({ ok: true, events: artifact.events.length, version: artifact.header.version }))
} catch (error) {
  console.log(JSON.stringify({ ok: false, error: String(error?.message ?? error).split('\n')[0] }))
}
