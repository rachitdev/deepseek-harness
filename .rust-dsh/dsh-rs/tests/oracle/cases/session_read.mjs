// Slice 1: session/state, events, wait against both hosts.
// Each host gets its own empty session in its own working directory.
export default async ({ call, both, scrub, hosts }) => {
  const made = {}
  for (const host of ['ts', 'rs']) {
    const reply = await call(host, 'session/create', { request: { cwd: '/tmp' } })
    if (!reply.ok) throw new Error(`${host} session/create: ${JSON.stringify(reply)}`)
    made[host] = reply.value.sessionId
  }
  // Replace each host's session id with one token so replies compare equal.
  scrub(v => JSON.parse(JSON.stringify(v).replaceAll(made.ts, '<sid>').replaceAll(made.rs, '<sid>')))
  // The event's timestamp is the host's own clock, so it is masked.
  scrub(v => JSON.parse(JSON.stringify(v).replace(/"time":\d+/g, '"time":0')))
  const sid = host => made[host]
  const each = async (method, build, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    return { ts, rs, label }
  }
  const results = []
  const check = async (method, build, label) => results.push(await each(method, build, label))

  await check('session/state', h => ({ request: { sessionId: sid(h) } }), 'blank session')
  await check('session/state', () => ({ request: { sessionId: 'ghost' } }), 'unknown session')
  await check('session/state', () => ({ request: { sessionId: 5 } }), 'non-string id')
  await check('session/state', () => ({ request: {} }), 'missing id')
  await check('session/state', h => ({ request: { sessionId: sid(h) }, extra: 1 }), 'unexpected field')
  await check('session/events', h => ({ request: { address: { kind: 'session', sessionId: sid(h) } } }), 'blank page')
  await check('session/events', () => ({ request: { address: { kind: 'session', sessionId: 'ghost' } } }), 'unknown')
  for (const bad of [{ throughSeq: -2 }, { throughSeq: 1.5 }, { beforeSeq: -1 }, { maxMessages: 0 }, { throughSeq: 99 },
    { turnWindow: { minMessages: 0, minTurns: 1 } }, { maxMessages: 2, turnWindow: { minMessages: 3, minTurns: 1 } }]) {
    await check('session/events', h => ({ request: { address: { kind: 'session', sessionId: sid(h) }, ...bad } }), JSON.stringify(bad))
  }
  await check('session/events', () => ({ request: { address: { kind: 'subagent', parentSessionId: 'p', childSessionId: 'c', mode: 'unknown' } } }), 'subagent absent')
  await check('session/events', () => ({ request: { address: { kind: 'subagent', parentSessionId: 'p', childSessionId: 'c', mode: 'weird' } } }), 'bad mode')
  await check('session/events', () => ({ request: { address: { kind: 'nope' } } }), 'bad kind')
  await check('session/wait', h => ({ request: { sessionId: sid(h), timeoutMs: 2000 } }), 'blank wait')
  await check('session/wait', () => ({ request: { sessionId: 'ghost' } }), 'unknown wait')
  for (const bad of [{ timeoutMs: 0 }, { timeoutMs: -1 }, { timeoutMs: 2147483648 }, { afterSeq: -2 }, { maxMessages: 0 }, { timeoutMs: 1.5 }]) {
    await check('session/wait', h => ({ request: { sessionId: sid(h), ...bad } }), JSON.stringify(bad))
  }
  const serialise = v => JSON.stringify(v)
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  let differing = 0
  for (const r of results) {
    // The event's timestamp is the host's own clock, so it is masked.
    const clean = (v, id) => JSON.parse(JSON.stringify(v).replaceAll(id, '<sid>').replace(/"time":\d+/g, '"time":0'))
    const a = serialise(canon(clean(r.ts, made.ts)))
    const b = serialise(canon(clean(r.rs, made.rs)))
    if (a !== b) { differing++; console.log(`DIFF ${r.label}\n  ts: ${a}\n  rs: ${b}`) }
  }
  console.log(`cases ${results.length}, differing ${differing}`)
}
