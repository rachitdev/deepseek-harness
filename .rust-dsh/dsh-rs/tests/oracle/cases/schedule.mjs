// Slice 2: schedule/list, catalog, history, delete, deleteInactive, update.
// Both hosts start from the same store (tests/oracle/reset.sh): five
// reminders in session 1, one in session 2, one ended, and an hourly one with
// five saved deliveries. Ids and instants that depend on the creation moment are
// compared structurally (a placeholder for each) because the two stores were
// seeded a few milliseconds apart.
const env = process.env
const sessions = { ts: [env.TS_S1, env.TS_S2], rs: [env.RS_S1, env.RS_S2] }

export default async ({ call }) => {
  let cases = 0
  const diffs = []
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const mask = (v, host) => JSON.parse(JSON.stringify(v)
    .replaceAll(sessions[host][0], '<S1>').replaceAll(sessions[host][1], '<S2>')
    .replace(/\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z/g, '<T>'))
  const check = async (method, build, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    cases++
    const a = JSON.stringify(canon(mask(ts, 'ts'))); const b = JSON.stringify(canon(mask(rs, 'rs')))
    if (a !== b) diffs.push(`${label}\n  ts: ${a.slice(0, 600)}\n  rs: ${b.slice(0, 600)}`)
    return { ts, rs }
  }
  const s = (i, h) => sessions[h][i]
  const req = (h, extra) => ({ request: { sessionId: s(0, h), ...extra } })
  const record = async (h, id) => (await call(h, 'schedule/list', req(h, {}))).value.find(r => r.id === id)

  await check('schedule/list', h => req(h, {}), 'list session 1')
  await check('schedule/list', h => ({ request: { sessionId: s(1, h) } }), 'list session 2')
  await check('schedule/list', () => ({ request: { sessionId: 'ghost' } }), 'list unknown')
  await check('schedule/list', () => ({ request: { sessionId: '' } }), 'list empty id')
  await check('schedule/catalog', () => ({}), 'catalog')
  for (const limit of [1, 2, 3, 5, 100]) await check('schedule/history', h => req(h, { id: 'schedule-b', limit }), `history limit ${limit}`)
  for (const bad of [0, 101, -1, 1.5]) await check('schedule/history', h => req(h, { id: 'schedule-b', limit: bad }), `history bad limit ${bad}`)
  await check('schedule/history', h => req(h, { id: 'schedule-b', limit: 2, before: 'msg-3' }), 'history before msg-3')
  await check('schedule/history', h => req(h, { id: 'schedule-b', limit: 2, before: 'nope' }), 'history unknown cursor')
  await check('schedule/history', h => req(h, { id: 'schedule-a', limit: 5 }), 'history empty')
  await check('schedule/history', h => req(h, { id: 'schedule-g', limit: 5 }), 'history ended')
  await check('schedule/history', h => req(h, { id: 'schedule-f', limit: 5 }), 'history other session')
  await check('schedule/history', h => req(h, { id: 'nope', limit: 5 }), 'history unknown id')
  await check('schedule/history', () => ({ request: { id: 'x', limit: 5, sessionId: 'ghost' } }), 'history unknown session')
  for (const bad of [{ id: 'x', limit: 'a' }, { id: 'x' }, { limit: 5 }, { id: 5, limit: 5 }]) {
    await check('schedule/history', h => req(h, bad), `history malformed ${JSON.stringify(bad)}`)
  }

  // update: needs the stored record as `expected`, which differs per host in its instants only.
  const updates = async (label, id, change, content = {}) => {
    const [ts, rs] = await Promise.all([record('ts', id), record('rs', id)])
    const [a, b] = await Promise.all([
      call('ts', 'schedule/update', req('ts', { id, expected: ts, ...(change ? { change } : {}), ...content })),
      call('rs', 'schedule/update', req('rs', { id, expected: rs, ...(change ? { change } : {}), ...content })),
    ])
    cases++
    const x = JSON.stringify(canon(mask(a, 'ts'))); const y = JSON.stringify(canon(mask(b, 'rs')))
    if (x !== y) diffs.push(`${label}\n  ts: ${x.slice(0, 600)}\n  rs: ${y.slice(0, 600)}`)
  }
  await updates('update title', 'schedule-a', undefined, { title: 'Renamed' })
  await updates('update prompt', 'schedule-a', undefined, { prompt: 'new prompt' })
  await updates('update both', 'schedule-a', undefined, { title: 'Both', prompt: 'both' })
  await updates('update nothing', 'schedule-a', undefined, {})
  await updates('update same title', 'schedule-a', undefined, { title: 'Both' })
  await updates('update blank title', 'schedule-a', undefined, { title: '   ' })
  await updates('update long title', 'schedule-a', undefined, { title: 'x'.repeat(121) })
  await updates('update blank prompt', 'schedule-a', undefined, { prompt: ' ' })
  await updates('every to 7200', 'schedule-b', { kind: 'every', every_seconds: 7200 })
  await updates('every to 30', 'schedule-b', { kind: 'every', every_seconds: 30 })
  await updates('every to junk', 'schedule-b', { kind: 'every', every_seconds: 'x' })
  await updates('daily same', 'schedule-c', { kind: 'daily', daily: { time: '09:30:00', time_zone: 'Asia/Kolkata' } })
  await updates('daily alias zone', 'schedule-c', { kind: 'daily', daily: { time: '09:30:00', time_zone: 'Asia/Calcutta' } })
  await updates('daily new time', 'schedule-c', { kind: 'daily', daily: { time: '10:00:00', time_zone: 'UTC' } })
  await updates('daily bad time', 'schedule-c', { kind: 'daily', daily: { time: '25:00:00', time_zone: 'UTC' } })
  await updates('weekly to other days', 'schedule-d', { kind: 'weekly', weekly: { time: '16:00:00', time_zone: 'America/New_York', weekdays: [1, 2] } })
  await updates('weekly same', 'schedule-d', { kind: 'weekly', weekly: { time: '16:00:00', time_zone: 'America/New_York', weekdays: [5] } })
  await updates('cron same canonical', 'schedule-e', { kind: 'cron', cron: { expression: '0-30/30 * * * *', time_zone: 'UTC' } })
  await updates('cron changed', 'schedule-e', { kind: 'cron', cron: { expression: '*/10 * * * *', time_zone: 'UTC' } })
  await updates('cron bad', 'schedule-e', { kind: 'cron', cron: { expression: 'nope', time_zone: 'UTC' } })
  await updates('at to past', 'schedule-a', { kind: 'at', at: '2020-01-01T00:00:00Z' })
  await updates('at junk', 'schedule-a', { kind: 'at', at: 'junk' })
  await updates('bad change shape', 'schedule-a', { kind: 'nope' })
  await updates('change with extra key', 'schedule-a', { kind: 'at', at: '2030-01-01T00:00:00Z', extra: 1 })
  await updates('ended reminder', 'schedule-g', undefined, { title: 'x' })
  await updates('unknown id', 'schedule-zzz', undefined, { title: 'x' })
  // conflict and malformed expected
  for (const [label, expected] of [['conflict', { id: 'schedule-a', kind: 'at', title: 'Other', prompt: 'p', scheduledAt: '2030-01-01T00:00:00.000Z' }], ['bad expected', { id: 'x' }]]) {
    await check('schedule/update', h => req(h, { id: 'schedule-c', expected, title: 'x' }), label)
  }
  await check('schedule/update', h => req(h, { id: 'schedule-c', title: 'x' }), 'missing expected')
  await check('schedule/update', h => req(h, { id: 'schedule-c', expected: 5, title: 'x' }), 'expected not object')
  await check('schedule/update', h => ({ request: { sessionId: 'ghost', id: 'schedule-c', expected: { id: 'x' } } }), 'update unknown session')

  // delete
  await check('schedule/delete', h => req(h, { id: 'schedule-f' }), 'delete other session reminder')
  await check('schedule/delete', h => req(h, { id: 'nope' }), 'delete unknown')
  await check('schedule/delete', h => req(h, { id: 'schedule-g' }), 'delete ended')
  await check('schedule/delete', h => req(h, { id: 'schedule-g' }), 'delete ended again')
  await check('schedule/delete', h => req(h, { id: 'schedule-a' }), 'delete active')
  await check('schedule/list', h => req(h, {}), 'list after deletes')
  await check('schedule/catalog', () => ({}), 'catalog after deletes')
  await check('schedule/deleteInactive', () => ({}), 'deleteInactive when none ended')
  for (const bad of [{ id: 5 }, {}, { id: 'x', extra: 1 }]) await check('schedule/delete', h => req(h, bad), `delete malformed ${JSON.stringify(bad)}`)

  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
