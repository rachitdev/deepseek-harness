// Archiving a session whose reminders are armed: refused naming them, then
// archived with stopActivity, which deletes them. Run after schedule_reset.sh.
export default async ({ call }) => {
  const env = process.env
  const sessions = { ts: env.TS_S1, rs: env.RS_S1 }
  const others = { ts: env.TS_S2, rs: env.RS_S2 }
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object' ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const mask = (v, h) => JSON.parse(JSON.stringify(v).replaceAll(sessions[h], '<S1>').replaceAll(others[h], '<S2>').replace(/\d{4}-\d\d-\d\dT[\d:.]+Z/g, '<T>'))
  const diffs = []; let cases = 0
  const step = async (label, method, build) => {
    const [a, b] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    cases++
    const x = JSON.stringify(canon(mask(a, 'ts'))), y = JSON.stringify(canon(mask(b, 'rs')))
    if (x !== y) diffs.push(`${label}\n  ts: ${x.slice(0, 700)}\n  rs: ${y.slice(0, 700)}`)
  }
  const S = h => ({ request: { sessionId: sessions[h] } })
  await step('archive with reminders', 'workspace/archiveSession', S)
  await step('list unchanged by refusal', 'schedule/list', S)
  await step('archive stopActivity', 'workspace/archiveSession', h => ({ request: { sessionId: sessions[h], stopActivity: true } }))
  await step('list after', 'schedule/list', S)
  await step('catalog after', 'schedule/catalog', () => ({}))
  await step('archive again', 'workspace/archiveSession', S)
  await step('unarchive', 'workspace/unarchiveSession', S)
  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
