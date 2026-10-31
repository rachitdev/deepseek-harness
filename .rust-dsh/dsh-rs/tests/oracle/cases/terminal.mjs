// Slice 3, terminal: the unary `terminal/*` endpoints.
//
// Both hosts start from the same state (tests/oracle/reset.sh): two
// sessions each, no terminals. Ids and the session ids differ per host, so the
// session ids are masked; everything else must match exactly.
const env = process.env
const sessions = { ts: env.TS_S1, rs: env.RS_S1 }
const other = { ts: env.TS_S2, rs: env.RS_S2 }

export default async ({ call }) => {
  let cases = 0
  const diffs = []
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const mask = (v, h) => JSON.parse(JSON.stringify(v)
    .replaceAll(sessions[h], '<S1>')
    .replaceAll(other[h], '<S2>'))
  const check = async (method, build, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    cases++
    const a = JSON.stringify(canon(mask(ts, 'ts')))
    const b = JSON.stringify(canon(mask(rs, 'rs')))
    if (a !== b) diffs.push(`${label}\n  ts: ${a.slice(0, 700)}\n  rs: ${b.slice(0, 700)}`)
    return { ts, rs }
  }
  const req = (h, extra) => ({ request: { sessionId: sessions[h], ...extra } })

  await check('terminal/shells', h => ({ agentId: sessions[h] }), 'shells')
  await check('terminal/shells', h => ({ agentId: other[h] }), 'shells other session')
  await check('terminal/shells', h => ({ agentId: 'ghost' }), 'shells unknown session')
  await check('terminal/environment', h => ({ agentId: sessions[h] }), 'environment')
  await check('terminal/environment', h => ({ agentId: 'ghost' }), 'environment unknown session')
  await check('terminal/list', h => ({ sessionId: sessions[h] }), 'list empty')
  await check('terminal/list', h => ({ sessionId: 'ghost' }), 'list unknown session')

  await check('terminal/create', h => ({ agentId: sessions[h], request: { id: 't1', cols: 80, rows: 24 } }), 'create')
  await check('terminal/create', h => ({ agentId: sessions[h], request: { id: 't1', cols: 80, rows: 24 } }), 'create again')
  await check('terminal/list', h => ({ sessionId: sessions[h] }), 'list after create')
  await check('terminal/create', h => ({ agentId: sessions[h], request: { id: 't1', cols: 81, rows: 25 } }), 'create same id other size')
  for (const bad of [
    { id: 'bad id!', cols: 80, rows: 24 }, { id: '', cols: 80, rows: 24 }, { id: 'x'.repeat(129), cols: 80, rows: 24 },
    { id: 't2', cols: 1, rows: 24 }, { id: 't2', cols: 80, rows: 0 }, { id: 't2', cols: 501, rows: 24 },
    { id: 't2', cols: 80, rows: 201 }, { id: 't2', cols: 80.5, rows: 24 }, { id: 't2', cols: 80, rows: 24.5 },
    { id: 't2', cols: 80, rows: 24, shellPath: '/bin/nope' },
  ]) {
    await check('terminal/create', h => ({ agentId: sessions[h], request: bad }), `create bad ${JSON.stringify(bad)}`)
  }
  await check('terminal/create', h => ({ agentId: sessions[h], request: { id: 't3', cols: 80, rows: 24, extra: 1 } }), 'create extra key')
  await check('terminal/create', h => ({ agentId: 'ghost', request: { id: 't9', cols: 80, rows: 24 } }), 'create unknown session')
  await check('terminal/list', h => ({ sessionId: sessions[h] }), 'list after creates')

  await check('terminal/rename', h => ({ agentId: sessions[h], id: 't1', title: '  My term  ' }), 'rename')
  await check('terminal/rename', h => ({ agentId: sessions[h], id: 't1', title: '   ' }), 'rename blank')
  await check('terminal/rename', h => ({ agentId: sessions[h], id: 't1', title: 'x'.repeat(121) }), 'rename long')
  await check('terminal/rename', h => ({ agentId: sessions[h], id: 't1', title: 'x'.repeat(120) }), 'rename 120')
  await check('terminal/rename', h => ({ agentId: sessions[h], id: 'nope', title: 'x' }), 'rename unknown')
  await check('terminal/rename', h => ({ agentId: 'ghost', id: 't1', title: 'x' }), 'rename unknown session')
  await check('terminal/list', h => ({ sessionId: sessions[h] }), 'list after rename')

  await check('terminal/close', h => ({ agentId: sessions[h], id: 't1' }), 'close')
  await check('terminal/close', h => ({ agentId: sessions[h], id: 't1' }), 'close again')
  await check('terminal/close', h => ({ agentId: sessions[h], id: 'zzz' }), 'close unknown')
  await check('terminal/create', h => ({ agentId: sessions[h], request: { id: 't1', cols: 80, rows: 24 } }), 'create after close')
  await check('terminal/list', h => ({ sessionId: sessions[h] }), 'list after close')

  // The session limit: eight terminals, the ninth is refused.
  for (let i = 0; i < 9; i++) {
    await check('terminal/create', h => ({ agentId: sessions[h], request: { id: `m${i}`, cols: 80, rows: 24 } }), `limit m${i}`)
  }
  await check('terminal/list', h => ({ sessionId: sessions[h] }), 'list at the limit')

  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
