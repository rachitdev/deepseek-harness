/** Forked Web agents inherit the standard preset and refuse every reminder tool. */
import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { expect, it } from 'vitest'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import { parseSessionLog, prepareSessionSnapshotFixtureForComparison } from '@deepseek-ai/dsh-llm-replay'
import type { Session, SessionEvent } from '@deepseek-ai/dsh-session'
import {
  type SnapshotCapability, assertToolPins, fixtureContext, normalizeSessionSnapshots, normalizedSystemPrompts, normalizedToolSchemas,
  formatSystemPromptSnapshot, readHeaderPin, recordedApprovalPolicy, writeHeaderPin,
  redactSessionSnapshotIds, refreshFixtureReplacements,
  scrubSessionSnapshot, stabilizeFixtureMessageIds, stabilizeRefreshLog, tokenizeSessionFixtureCwd,
} from '@deepseek-ai/dsh-session-snapshot'
import type {} from '@deepseek-ai/dsh-api-session-controller'
import type {} from '@deepseek-ai/dsh-schedule'
import { SUBAGENT_DELEGATION_CONTEXT } from '@deepseek-ai/dsh-subagent'
import {
  fixtureUserPrompts, launchWebScaffold, normalizeWebSessionVolatiles, rawSessionLog, readPersistedEvents,
  recordedSessionFixturePath, selectedSessionFixture, webSnapshotMode,
} from './scaffold.ts'
import { REPO_ROOT } from './support.ts'

const DIR = fileURLToPath(new URL('../../../snapshots/web/subagent-fork-schedule', import.meta.url))
const PIN = DIR
const MODE = webSnapshotMode()

it.skipIf(MODE === 'record')('replays inherited schedule tools and refusals in a seeded Web fork', async () => {
  const paths = await Promise.all(['session.v4.jsonl', 'session.1.v4.jsonl']
    .map(file => selectedSessionFixture(join(DIR, file))))
  let expected = await Promise.all(paths.map(path => readFile(path, 'utf8')))
  const scaffold = await launchWebScaffold({
    replayFixture: paths[0]!, replayChildFixtures: [paths[1]!], compareReplaySession: false,
  })
  try {
    const sessions: Session[] = []
    scaffold.ctx.on('session/created', (session) => { sessions.push(session) })
    const { sessionId } = await scaffold.ctx.sessionController.create({ cwd: scaffold.workspaceCwd })
    const parent = scaffold.ctx.agents.get(sessionId)
    if (parent === undefined) throw new Error('created parent has no Agent')
    const prompt = fixtureUserPrompts(expected[0]!)[0]
    if (prompt === undefined) throw new Error('parent fixture has no user prompt')
    parent.followup(createUserMessage({ content: [{ type: 'text', text: prompt }], source: { kind: 'user' } }))
    await parent.whenIdle()

    const childPrompt = parseSessionLog(expected[1]!).findLast(event => event.type === 'user/message'
      && event.data.source.kind === 'user')
    if (childPrompt?.type !== 'user/message') throw new Error('child fixture has no user prompt')
    const activation = await scaffold.ctx.subagents.startActivation({
      provider: 'fork', label: 'Check child reminder refusal', delivery: 'caller',
      signal: new AbortController().signal,
      request: { parent, prompt: [...childPrompt.data.content] },
    })
    try {
      await activation.result
    } finally {
      await activation.dispose()
    }
    await scaffold.ctx.sessions.flush(parent.session)
    expect(sessions).toHaveLength(2)
    const child = sessions[1]!
    expect(child.header).toMatchObject({ parentSession: parent.id, isSeeded: true, agentPreset: 'standard' })
    expect(child.inheritedEventCount).toBeGreaterThan(0)
    for (const session of sessions) {
      expect(await readPersistedEvents(scaffold, session.id)).toEqual(session.snapshotEvents())
    }
    const raw = sessions.map(session => normalizeWebSessionVolatiles(rawSessionLog(session), scaffold.workspaceCwd))
    const context = { sessionIds: sessions.map(session => String(session.id)), cwd: scaffold.workspaceCwd }
    // A delegated child pins its approval policy to 'never' (child-agent.ts
    // captureDelegatedPolicyOverrides), so its schemas drop the approval-gated
    // parameters while every other request field and the tool order still match.
    const toolNames = (header: unknown): unknown => {
      const copy = structuredClone(header) as { tools?: unknown }
      if (Array.isArray(copy.tools)) copy.tools = (copy.tools as { name: string }[]).map(tool => tool.name)
      return copy
    }
    expect(toolNames(child.requestHeader())).toEqual(toolNames(parent.session.requestHeader()))
    expect(JSON.stringify(parent.session.requestHeader())).toContain('sandbox_permissions')
    expect(JSON.stringify(child.requestHeader())).not.toContain('sandbox_permissions')
    // The capability the child runs under is asserted where it is decided: the parent's
    // declared policy, the delegation event that fixes the child's, and the statement the
    // child actually reads. Without these, the two schema sets above would pin a difference
    // nothing explains.
    expect(recordedApprovalPolicy(rawSessionLog(parent.session))).toBe('ask')
    // The fork seed carries the parent's ask first; delegation then fixes the child's own policy.
    const delegation = child.snapshotEvents().findLast(event =>
      event.type === 'approval/policy' && event.data.source === 'delegation')
    expect(delegation?.data).toMatchObject({ policy: 'never', source: 'delegation' })
    // The seed states the confinement the child was created under, so its own log explains the
    // scope it ran in without consulting the parent's.
    const seededMode = child.snapshotEvents()
      .filter((event): event is Extract<SessionEvent, { type: 'sandbox/mode' }> => event.type === 'sandbox/mode')
      .findLast(event => event.data.source === 'delegation')
    expect(seededMode?.data).toMatchObject({ source: 'delegation' })
    expect(['read-only', 'workspace-write', 'danger-full-access']).toContain(seededMode?.data.mode)
    const runtimeContext = child.snapshotEvents()
      .filter((event): event is Extract<SessionEvent, { type: 'user/message' }> => event.type === 'user/message')
      .filter(event => event.data.source.kind === 'runtime-context')
    expect(JSON.stringify(runtimeContext.at(-1)?.data.content)).toContain(SUBAGENT_DELEGATION_CONTEXT)
    // This case owns its pins: a fork boot seeds its own composition, so borrowing
    // another case's schema pin would assert a header this case never emits. Refresh
    // writes them from this boot before the comparison, then re-reads the same files.
    const prompts = normalizedSystemPrompts(raw[0]!, context).map(prompt => prompt
      .split(REPO_ROOT).join('{{sourceRoot}}').split(scaffold.baseUrl).join('{{webUrl}}'))
    // Each request header runs under the approval policy recorded before it: the parent and
    // the child's inherited fork header under `ask` (escalation), the delegated child's own
    // turns under `never` (unconfined). Consecutive identical revisions collapse, so the pin
    // holds two revisions, and the parent-versus-child capability difference stays visible.
    const headerRevisions = (log: string): { set: unknown[]; capability: SnapshotCapability }[] => {
      const schemaSets = normalizedToolSchemas(log, context)
      const capabilities: SnapshotCapability[] = []
      let policy: string | undefined
      for (const line of log.split('\n')) {
        if (line.trim().length === 0) continue
        const record = JSON.parse(line) as { type?: string; data?: { policy?: string; header?: { tools?: unknown } } }
        if (record.type === 'approval/policy') policy = record.data?.policy
        if (record.type === 'request/header' && Array.isArray(record.data?.header?.tools)) {
          capabilities.push(policy === 'never' ? 'unconfined' : 'escalation')
        }
      }
      expect(capabilities, 'one recorded approval policy per tool-bearing request header').toHaveLength(schemaSets.length)
      return schemaSets.map((set, index) => ({ set, capability: capabilities[index] as SnapshotCapability }))
    }
    const revisions = [...headerRevisions(raw[0] as string), ...headerRevisions(raw[1] as string)]
      .filter((revision, index, all) => index === 0
        || revision.capability !== all[index - 1]!.capability
        || JSON.stringify(revision.set) !== JSON.stringify(all[index - 1]!.set))
    const sets = revisions.map(revision => revision.set)
    const setCapabilities = revisions.map(revision => revision.capability)
    expect(setCapabilities, 'the parent pins escalation and its delegated child pins unconfined')
      .toEqual(['escalation', 'unconfined'])
    const pinLocation = { snapshotsDir: join(REPO_ROOT, 'snapshots', 'web'), ownerDir: PIN, headerClass: 'web-fork' }
    if (MODE === 'refresh') {
      await Promise.all([
        writeFile(join(PIN, 'system-prompt.expected.md'),
          formatSystemPromptSnapshot(prompts[0] as string, prompts.slice(1))),
        writeHeaderPin(pinLocation, 'escalation', sets, process.env.DSH_SNAPSHOT_CATALOG_PARTS, setCapabilities),
      ])
    }
    const promptPin = await readFile(join(PIN, 'system-prompt.expected.md'), 'utf8')
    // A refresh run writes the pin and its catalog fragment, so the committed
    // catalog cannot agree with it yet; the generator merges the fragment next.
    if (MODE === 'refresh') {
      for (const log of raw) {
        for (const prompt of normalizedSystemPrompts(log, context)) {
          expect(formatSystemPromptSnapshot(prompt.split(REPO_ROOT).join('{{sourceRoot}}')
            .split(scaffold.baseUrl).join('{{webUrl}}'))).toBe(promptPin)
        }
      }
    } else {
      const { pins, catalog } = await readHeaderPin(PIN, join(REPO_ROOT, 'docs', 'tool-catalog.json'))
      expect(pins.class).toBe('web/web-fork')
      expect(pins.sets.map(set => set.capability)).toEqual(setCapabilities)
      assertToolPins(sets, { label: `${PIN}: tool-schema pin`, className: pins.class, pins, catalog })
      for (const log of raw) {
        for (const prompt of normalizedSystemPrompts(log, context)) {
          expect(formatSystemPromptSnapshot(prompt.split(REPO_ROOT).join('{{sourceRoot}}')
            .split(scaffold.baseUrl).join('{{webUrl}}'))).toBe(promptPin)
        }
      }
    }
    const results = child.snapshotEvents().slice(child.inheritedEventCount)
      .filter(event => event.type === 'tool/result')
    expect(results.map(event => event.data.message.content)).toEqual(Array.from({ length: 4 }, () => [
      { type: 'text', text: '{"code":"subagent_session","message":"A delegated subagent cannot use reminders."}' },
    ]))
    expect(await scaffold.ctx.schedule.list({ sessionId: child.id })).toEqual([])

    if (MODE === 'refresh') {
      const replacements = refreshFixtureReplacements(raw.map((content, index) => ({
        id: sessions[index]!.id, createdAt: sessions[index]!.header.createdAt, content,
      })), expected)
      expected = redactSessionSnapshotIds(stabilizeFixtureMessageIds(raw.map((log, index) =>
        tokenizeSessionFixtureCwd(scrubSessionSnapshot(stabilizeRefreshLog(
          prepareSessionSnapshotFixtureForComparison(log), expected[index]!, replacements, context,
        ))).split(scaffold.harnessHome).join('{{harnessHome}}')), expected))
      await Promise.all(expected.map((log, index) => writeFile(
        recordedSessionFixturePath(paths[index]!, sessions[index]!.header.version), log,
      )))
    }
    expect(normalizeSessionSnapshots(raw, context))
      .toEqual(normalizeSessionSnapshots(expected, fixtureContext(expected[0]!)))
  } finally {
    await scaffold.close()
  }
})
