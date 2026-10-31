#!/usr/bin/env node
// Runs the REAL @deepseek-ai/dsh-tool-schedule tool bodies over a battery of
// inputs against the real @deepseek-ai/dsh-schedule domain, with a clock fixed at
// 2026-03-01T12:00:00.000Z and an in-memory task set, and prints each outcome.
// tests/schedule_tools_fixture.rs replays the same battery on the Rust tools.
//
//   node --import tsx/esm tests/oracle/schedule_tools_ts.mjs > tests/fixtures/schedule_tools_ts.json
import * as D from '/Users/rachit/code/deepseek-harness/packages/schedule/schedule/src/domain.ts'
import { resolveScheduleUpdate } from '/Users/rachit/code/deepseek-harness/packages/schedule/schedule/src/update.ts'
import { apply } from '/Users/rachit/code/deepseek-harness/packages/schedule/tool-schedule/src/index.ts'

const NOW = Date.parse('2026-03-01T12:00:00.000Z')
Date.now = () => NOW

let counter = 0
const tasks = new Map() // id -> { sessionId, record, status }
const schedule = {
  async list({ sessionId }) { return [...tasks.values()].filter(t => t.sessionId === sessionId && t.status === 'active').map(t => t.record) },
  async catalog() { return [...tasks.values()].map(t => ({ ...t.record, sessionId: t.sessionId, status: t.status })) },
  async create(sessionId, request) {
    const id = `schedule-${++counter}`
    const title = request.title
    let record
    if (request.after_seconds !== undefined) record = D.createAfterScheduleRecord(id, request.prompt, request.after_seconds, NOW, title)
    else if (request.at !== undefined) record = D.createAtScheduleRecord(id, request.prompt, request.at, NOW, title)
    else if (request.every_seconds !== undefined) record = D.createEveryScheduleRecord(id, request.prompt, request.every_seconds, NOW, title)
    else if (request.daily !== undefined) record = D.createDailyScheduleRecord(id, request.prompt, request.daily, NOW, title)
    else if (request.weekly !== undefined) record = D.createWeeklyScheduleRecord(id, request.prompt, request.weekly, NOW, title)
    else if (request.cron !== undefined) record = D.createCronScheduleRecord(id, request.prompt, request.cron, NOW, title)
    else throw new D.ScheduleInputError('invalid_selector', 'Exactly one reminder selector is required.')
    tasks.set(id, { sessionId, record, status: 'active' })
    return record
  },
  async delete({ sessionId, id }) {
    const t = tasks.get(id)
    if (!t || t.sessionId !== sessionId) return { id, deleted: false, code: 'schedule_not_found' }
    tasks.delete(id)
    return { id, deleted: true }
  },
  async update(request) {
    const t = tasks.get(request.id)
    if (!t || t.sessionId !== request.sessionId) return { id: request.id, updated: false, code: 'schedule_not_found' }
    if (t.status === 'inactive') return { id: request.id, updated: false, code: 'schedule_ended' }
    const result = resolveScheduleUpdate(t.record, request.expected, request.change, NOW, request)
    if ('record' in result && result.updated) tasks.set(request.id, { ...t, record: result.record })
    return result
  },
}

const registered = new Map()
apply({
  inject: (_deps, fn) => fn({ tools: { register: tool => { registered.set(tool.name, tool) } }, schedule }),
})

const agentFor = (id, depth = 0) => ({ id, options: { subagentDepth: depth }, session: { id, header: { delegationDepth: depth } } })
async function call(name, args, agent = agentFor('s1')) {
  const tool = registered.get(name)
  const exec = { signal: new AbortController().signal, agent }
  try { return JSON.parse(JSON.stringify(await tool.execute(args, exec))) } catch (e) { return { threw: String(e).slice(0, 160) } }
}

const battery = []
const step = async (tool, args, label) => { battery.push({ label, tool, args, result: await call(tool, args) }) }
const P = { prompt: 'p', title: 'T' }
for (const extra of [
  { after_seconds: 600 }, { after_seconds: 0 }, { after_seconds: 1.5 }, { after_seconds: -3 },
  { at: '2026-03-02T09:00:00Z' }, { at: '2020-01-01T00:00:00Z' }, { at: 'junk' }, { at: { date: '2026-03-08', time: '02:30:00', time_zone: 'America/New_York' } },
  { at: { date: '2026-03-08', time: '03:30:00', time_zone: 'America/New_York' } }, { at: { date: '2026-03-02', time: '09:00:00', time_zone: 'Nope/Zone' } },
  { every_seconds: 60 }, { every_seconds: 59 }, { every_seconds: 1.5 }, { every_seconds: 3600 },
  { daily: { time: '09:00:00', time_zone: 'Asia/Kolkata' } }, { daily: { time: '9:00', time_zone: 'UTC' } }, { daily: { time: '09:00:00' } }, { daily: 'x' },
  { weekly: { time: '09:00:00', time_zone: 'UTC', weekdays: [1, 3] } }, { weekly: { time: '09:00:00', time_zone: 'UTC', weekdays: [] } }, { weekly: { time: '09:00:00', time_zone: 'UTC', weekdays: [1, 1] } },
  { cron: { expression: '*/15 9-17 * * 1-5', time_zone: 'Europe/London' } }, { cron: { expression: 'bad', time_zone: 'UTC' } }, { cron: { expression: '0 0 31 2 *', time_zone: 'UTC' } },
  {}, { after_seconds: 5, every_seconds: 60 }, { after_seconds: 5, zzz: 1 }, { zzz: 1 },
]) await step('schedule_create', { ...P, ...extra }, `create ${JSON.stringify(extra)}`)
for (const bad of [{ prompt: '  ', title: 'T', after_seconds: 5 }, { prompt: 'p', title: '  ', after_seconds: 5 }, { prompt: 'p', title: 'x'.repeat(121), after_seconds: 5 }, { prompt: 'p', title: 'x'.repeat(120), after_seconds: 5 }, { prompt: ' p ', title: ' T ', after_seconds: 5 }]) {
  await step('schedule_create', bad, `create ${JSON.stringify(bad).slice(0, 60)}`)
}
await step('schedule_list', {}, 'list')
const ids = (await call('schedule_list', {})).map?.(r => r.id) ?? []
const first = ids[0]
for (const args of [
  { id: first, title: 'Renamed' }, { id: first, prompt: 'new' }, { id: first, title: ' ' }, { id: first, prompt: ' ' }, { id: first, title: 'x'.repeat(121) },
  { id: first }, { id: first, at: '2031-01-01T00:00:00Z' }, { id: first, at: '2031-01-01T00:00:00Z', every_seconds: 60 }, { id: first, every_seconds: 90 }, { id: first, every_seconds: 10 },
  { id: first, daily: { time: '10:00:00', time_zone: 'UTC' } }, { id: first, cron: { expression: '*/10 * * * *', time_zone: 'UTC' } }, { id: first, cron: { expression: 'x', time_zone: 'UTC' } },
  { id: ' x', title: 'y' }, { id: '', title: 'y' }, { id: 'schedule-none', title: 'y' }, { id: first, after_seconds: 5 }, { id: first, zzz: 1 },
]) await step('schedule_update', args, `update ${JSON.stringify(args).replace(first, '<first>').slice(0, 70)}`)
await step('schedule_list', {}, 'list after updates')
for (const args of [{ id: ' x' }, { id: '' }, { id: 'schedule-none' }, { id: first }, { id: first }]) await step('schedule_delete', args, `delete ${JSON.stringify(args).replace(first, '<first>')}`)
await step('schedule_list', {}, 'list after delete')
// a subagent caller
const child = agentFor('c1', 1)
for (const [tool, args] of [['schedule_create', { ...P, after_seconds: 5 }], ['schedule_list', {}], ['schedule_delete', { id: 'x' }], ['schedule_update', { id: 'x', title: 'y' }]]) {
  battery.push({ label: `subagent ${tool}`, tool, args, subagent: true, result: await call(tool, args, child) })
}
console.log(JSON.stringify(battery))
