// Slice 9: account/*. Neither host is signed in. `startSignIn` is excluded:
// the TS host opens a Platform sign-in attempt, which RS does not implement.
export default async ({ call }) => {
  let cases = 0; const diffs = []
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const check = async (method, args, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, args), call('rs', method, args)])
    cases++
    const a = JSON.stringify(canon(ts)); const b = JSON.stringify(canon(rs))
    if (a !== b) diffs.push(`${label}\n  ts: ${a.slice(0, 400)}\n  rs: ${b.slice(0, 400)}`)
  }
  const C = { version: '1', locale: 'en', timezoneOffsetSeconds: 0 }
  await check('account/getState', {}, 'state')
  await check('account/hasRunningAccountTasks', {}, 'running tasks')
  for (const m of ['getBalance', 'getProfile', 'getUnnotifiedBonuses', 'signOut']) {
    await check(`account/${m}`, { client: C }, `${m} ok`)
    await check(`account/${m}`, { client: { ...C, extra: true } }, `${m} extra key`)
    await check(`account/${m}`, { client: { version: '1', locale: 'en' } }, `${m} missing tz`)
    await check(`account/${m}`, { client: { ...C, timezoneOffsetSeconds: '0' } }, `${m} tz string`)
    await check(`account/${m}`, { client: { ...C, version: 1 } }, `${m} version number`)
    await check(`account/${m}`, { client: {} }, `${m} empty client`)
    await check(`account/${m}`, { client: null }, `${m} null client`)
    await check(`account/${m}`, { client: 'x' }, `${m} string client`)
    await check(`account/${m}`, {}, `${m} no args`)
    await check(`account/${m}`, { client: C, accountId: 'a' }, `${m} unexpected`)
  }
  for (const [a, o] of [['a', 'o'], ['', ''], ['x', '']]) await check('account/ackBonusNotified', { accountId: a, orderId: o, client: C }, `ack ${a}/${o}`)
  await check('account/ackBonusNotified', { accountId: 5, orderId: 'o', client: C }, 'ack accountId number')
  await check('account/ackBonusNotified', { orderId: 'o', client: C }, 'ack missing accountId')
  for (const id of ['x', '', 'abc-123']) await check('account/cancelSignIn', { attemptId: id }, `cancel ${JSON.stringify(id)}`)
  await check('account/cancelSignIn', { attemptId: 5 }, 'cancel number')
  await check('account/cancelSignIn', {}, 'cancel missing')
  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
