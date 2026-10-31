// Slice 4: subagents/prompt and interruptByParent. Neither host has a live
// child here, so these compare the validation order and every refusal; the
// delivery path is covered by tests/subagents_gui.rs.
export default async ({ call }) => {
  let cases = 0; const diffs = []
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const made = {}
  for (const host of ['ts', 'rs']) {
    const r = await call(host, 'session/create', { request: { cwd: '/tmp' } })
    made[host] = r.value.sessionId
  }
  const other = {}
  for (const host of ['ts', 'rs']) other[host] = (await call(host, 'session/create', { request: { cwd: '/tmp' } })).value.sessionId
  const swap = (v, host) => JSON.parse(JSON.stringify(v).replaceAll(made[host], '<P>').replaceAll(other[host], '<O>'))
  const check = async (method, build, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    cases++
    const a = JSON.stringify(canon(swap(ts, 'ts'))); const b = JSON.stringify(canon(swap(rs, 'rs')))
    if (a !== b) diffs.push(`${label}\n  ts: ${a.slice(0, 420)}\n  rs: ${b.slice(0, 420)}`)
  }
  const prompt = (parent, child, extra = {}) => host => ({ request: {
    requestId: 'r', parentSessionId: parent(host), childSessionId: child(host), mode: 'continuable', delivery: 'queue',
    content: [{ type: 'text', text: 'hi' }], ...extra,
  } })
  const P = h => made[h]; const O = h => other[h]; const lit = s => () => s
  await check('subagents/prompt', prompt(lit('p1'), lit('c1')), 'parent not live')
  await check('subagents/prompt', prompt(lit(''), lit('c1')), 'empty parent')
  await check('subagents/prompt', prompt(lit('p1'), lit('')), 'empty child')
  await check('subagents/prompt', prompt(lit(''), lit('')), 'both empty')
  await check('subagents/prompt', prompt(P, lit('ghost')), 'live parent, ghost child')
  await check('subagents/prompt', prompt(P, O), 'live parent, root session as child')
  await check('subagents/prompt', prompt(P, P), 'parent as its own child')
  await check('subagents/prompt', prompt(P, lit('ghost'), { delivery: 'steer' }), 'steer ghost')
  await check('subagents/prompt', prompt(P, lit('ghost'), { content: [] }), 'empty content')
  await check('subagents/prompt', prompt(P, lit('ghost'), { content: [{ type: 'text', text: '' }] }), 'empty text')
  const zones = ['UTC', 'utc', 'Utc', 'Europe/Paris', 'europe/paris', 'EUROPE/PARIS', 'Asia/Calcutta', 'Asia/Kolkata', 'US/Pacific',
    'Etc/GMT+5', 'Etc/UTC', 'America/Argentina/Buenos_Aires', 'Nope/Zone', 'Europe/Nowhere', 'America/New_York ', ' UTC', 'UTC ',
    'EST5EDT', 'GMT', 'Z', 'Europe', 'Europe/', '/Paris', 'Asia/Ho_Chi_Minh', 'Africa/Asmera', 'Etc/GMT-14', 'Etc/GMT+12',
    'Antarctica/DumontDUrville', 'America/Port-au-Prince', 'Europe/Kiev', 'Europe/Kyiv', '', 'Europe//Paris', '1Europe/Paris',
    'Europe/Pa ris', 'America/Indiana/Indianapolis', 'America/Indianapolis', 'Pacific/Kanton', 'Pacific/Enderbury', 'Asia/Katmandu',
    'Asia/Kathmandu', 'America/Godthab', 'America/Nuuk', 'Europe/Belfast', 'Zulu', 'Universal', 'Etc/Zulu', 'Etc/Universal', 'Etc/GMT',
    'Etc/GMT0', 'Etc/Greenwich', 'GB', 'Cuba', 'Egypt', 'Japan', 'US/Eastern', 'Australia/ACT', 'Canada/Pacific', 'Mexico/General',
    'Brazil/East', 'Chile/Continental', 'Asia/Rangoon', 'Asia/Yangon', 'Asia/Saigon', 'America/Buenos_Aires', 'Atlantic/Faeroe',
    'Europe/Kiev ', 'europe/KYIV', 'ASIA/KOLKATA', 'Asia/ Kolkata', 'Asia/Kolkata/', 'a/b', 'A/B', 'x/y/z', 'Etc/GMT+0', 'Etc/GMT-0',
    'Etc/GMT+13', 'Etc/GMT-15', 'W-SU', 'Navajo', 'Asia/Tokyo', 'Asia/Kolkata\n', 'Europe/Paris\t']
  for (const zone of zones) {
    await check('subagents/prompt', prompt(lit('p1'), lit('c1'), { clientTimeZone: zone }), `zone ${JSON.stringify(zone)}`)
  }
  for (const [c, p] of [['c1', 'p1'], ['', 'p1'], ['c1', ''], ['', ''], ['ghost', 'ghost']]) {
    await check('subagents/interruptByParent', () => ({ childSessionId: c, parentSessionId: p, mode: 'continuable' }), `interrupt ${c}/${p}`)
  }
  await check('subagents/interruptByParent', h => ({ childSessionId: O(h), parentSessionId: P(h), mode: 'continuable' }), 'interrupt a root session as child')
  await check('subagents/interruptByParent', h => ({ childSessionId: P(h), parentSessionId: P(h), mode: 'continuable' }), 'interrupt self')
  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
