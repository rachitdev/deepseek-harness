// Slice 4, officeToPdf: the document preview's conversion.
//
// The generation is this host's own, and the file's version is opaque, so both
// are masked; the failure reasons and the messages are compared as they are.
const env = process.env
const sessions = { ts: env.TS_S1, rs: env.RS_S1 }

export default async ({ call }) => {
  let cases = 0
  const diffs = []
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const mask = (v, h) => JSON.parse(JSON.stringify(v)
    .replaceAll(sessions[h], '<S1>')
    .replace(/"version":"[^"]*"/g, '"version":"<V>"')
    .replace(/"bytes":\d+/g, '"bytes":"<N>"')
    .replace(/"parts":\[[^\]]*\]/g, '"parts":"<PARTS>"')
    .replace(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g, '<UUID>'))
  const check = async (method, build, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    cases++
    const a = JSON.stringify(canon(mask(ts, 'ts')))
    const b = JSON.stringify(canon(mask(rs, 'rs')))
    if (a !== b) diffs.push(`${label}\n  ts: ${a.slice(0, 700)}\n  rs: ${b.slice(0, 700)}`)
    return { ts, rs }
  }

  await check('officeToPdf/generation', () => ({}), 'generation')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: 'ghost', path: 'a.docx', priority: 'foreground' }), 'render unknown scope')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: sessions[h], path: 'a.txt', priority: 'foreground' }), 'render unsupported format')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: sessions[h], path: 'missing.docx', priority: 'foreground' }), 'render missing document')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: sessions[h], path: 'fake.docx', priority: 'foreground' }), 'render a document that is not one')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: sessions[h], path: 'sample.docx', priority: 'foreground' }), 'render a real document')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: sessions[h], path: 'sample.docx', priority: 'nope' }), 'render a bad priority')
  await check('officeToPdf/render', h => ({ workspaceFileScopeId: sessions[h], path: 'sample.docx' }), 'render without a priority')

  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
