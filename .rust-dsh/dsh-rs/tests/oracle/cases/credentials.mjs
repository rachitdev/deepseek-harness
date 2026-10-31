// Slice 3: credentials/describe, set, unset against both hosts, each with its own
// credential document. Process-environment names are shared by neither host
// (they are not launched with the probe names), so `env` never appears.
export default async ({ call, hosts }) => {
  let differing = 0
  let cases = 0
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const check = async (method, args, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, args), call('rs', method, args)])
    cases++
    const a = JSON.stringify(canon(ts)); const b = JSON.stringify(canon(rs))
    if (a !== b) { differing++; console.log(`DIFF ${label}\n  ts: ${a.slice(0, 500)}\n  rs: ${b.slice(0, 500)}`) }
  }
  const D = 'DSH_ORACLE_'
  await check('credentials/describe', { refs: [] }, 'empty')
  await check('credentials/describe', { refs: [`${D}A`, `${D}B`] }, 'unset pair')
  await check('credentials/set', { ref: `${D}A`, value: 'one' }, 'set')
  await check('credentials/describe', { refs: [`${D}A`, `${D}B`] }, 'after set')
  await check('credentials/set', { ref: `${D}A`, value: 'two' }, 'overwrite')
  await check('credentials/set', { ref: `${D}S`, value: '  ' }, 'spaces')
  await check('credentials/set', { ref: `${D}Q`, value: 'a: b # c' }, 'yaml-special value')
  await check('credentials/describe', { refs: [`${D}A`, `${D}S`, `${D}Q`] }, 'after more')
  await check('credentials/unset', { ref: `${D}A` }, 'unset')
  await check('credentials/unset', { ref: `${D}A` }, 'unset again')
  await check('credentials/unset', { ref: 'NEVER_SET_ANYWHERE' }, 'unset missing')
  await check('credentials/describe', { refs: [`${D}A`] }, 'after unset')
  await check('credentials/describe', { refs: ['__proto__', 'constructor'] }, 'prototype names')
  await check('credentials/set', { ref: '__proto__', value: 'v' }, 'set __proto__')
  await check('credentials/describe', { refs: ['__proto__'] }, 'after __proto__')
  await check('credentials/unset', { ref: '__proto__' }, 'unset __proto__')
  for (const refs of [['bad one'], ['ok', '1x'], ['bad one', 'ok', '1x'], [''], ['a-b'], ['é']]) {
    await check('credentials/describe', { refs }, `bad refs ${JSON.stringify(refs)}`)
  }
  await check('credentials/describe', { refs: Array.from({ length: 65 }, (_, i) => `R${i}`) }, '65 refs')
  await check('credentials/describe', { refs: ['bad one', ...Array.from({ length: 65 }, (_, i) => `R${i}`)] }, 'bad + 65')
  await check('credentials/describe', { refs: Array.from({ length: 64 }, (_, i) => `R${i}`) }, '64 refs')
  await check('credentials/describe', { refs: ['A', 'A'] }, 'duplicate')
  for (const [ref, value] of [['bad one', 'v'], ['', 'v'], ['1x', 'v'], ['OK_X', ''], ['bad one', ''], ['a.b', 'v']]) {
    await check('credentials/set', { ref, value }, `bad set ${JSON.stringify([ref, value])}`)
  }
  for (const ref of ['bad one', '', '1x']) await check('credentials/unset', { ref }, `bad unset ${JSON.stringify(ref)}`)
  for (const args of [{ ref: 5, value: 'v' }, { ref: 'OK', value: 5 }, { ref: 'OK' }, { value: 'v' }, { ref: 'OK', value: 'v', x: 1 }, {}]) {
    await check('credentials/set', args, `set types ${JSON.stringify(args)}`)
  }
  for (const args of [{ ref: 5 }, {}, { ref: 'OK', x: 1 }]) await check('credentials/unset', args, `unset types ${JSON.stringify(args)}`)
  for (const args of [{ refs: 'x' }, { refs: [5] }, { refs: [null] }, {}, { refs: [], x: 1 }, { refs: {} }]) {
    await check('credentials/describe', args, `describe types ${JSON.stringify(args)}`)
  }
  console.log(`cases ${cases}, differing ${differing}`)
}
