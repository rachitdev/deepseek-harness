/**
 * Delegation facts across residencies: the policy attribution the observer
 * writes back to its caller, and the lifecycle that retires a child's record
 * when the manager releases it or drains. Both specs drive the real manager
 * create → settle → cold-resume path rather than a hand-built fixture.
 */

import { afterEach, describe, expect, it, vi } from 'vitest'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { Context } from '@deepseek-ai/cordis'
import type { Agent } from '@deepseek-ai/dsh-agent'
import AgentLoop from '@deepseek-ai/dsh-agent-loop'
import { mountAgentLoopTestDependencies } from '@deepseek-ai/dsh-agent-loop-testkit'
import type { GenerateOptions } from '@deepseek-ai/dsh-llm'
import SandboxPolicyService from '@deepseek-ai/dsh-sandbox-policy'
import { SessionId } from '@deepseek-ai/dsh-session'
import * as JsonlSessionPersistence from '@deepseek-ai/dsh-session-persistence-jsonl'
import * as SubagentSpawn from '@deepseek-ai/dsh-subagent-spawn-in-process'
import { queueHostSubagentPrompt } from '@deepseek-ai/dsh-subagent/internal'
import { defineTool } from '@deepseek-ai/dsh-tools'
import ApprovalService, { setApprovalPolicy } from '@deepseek-ai/dsh-user-approval'
import type { ApprovalOutcome } from '@deepseek-ai/dsh-user-approval'
import { MockAdapter, textResponse, toolCallResponse } from '../../../core/agent-loop/tests/mock-adapter.ts'
import SubagentRuntime from '../src/index.ts'
import { delegationFactsState, managerState, subagentManager } from './manager-internals.ts'
import { loadStoredSession } from './persistence-helpers.ts'
import { TestSessionQuery } from './test-session-query.ts'
import { mountWorkingDirectoryFixture } from './working-directory-fixture.ts'

type Script = ConstructorParameters<typeof MockAdapter>[0]
type Response = ReturnType<typeof textResponse>

const contexts: Context[] = []
const roots: string[] = []

afterEach(async () => {
  for (const ctx of contexts.splice(0).reverse()) await ctx.fiber.dispose()
  for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true })
})

/**
 * Answer each child request from its own sequence while the parent's turns
 * (which the settlement notices wake) answer with plain text. Both agents
 * share one adapter, and only a parent request carries a settlement notice.
 * @param responses - child responses in request order.
 * @returns scripted adapter entries, one per model request.
 */
function scriptedChild(responses: readonly Response[]): Script {
  let index = 0
  const dispatch = (options: GenerateOptions): Response => {
    if (options.messages.some(message => message.source?.kind === 'subagent-settled')) {
      return textResponse('parent acknowledged')
    }
    const response = responses[index] ?? textResponse('child done')
    index += 1
    return response
  }
  return Array.from({ length: 12 }, () => dispatch)
}

/** Boot the continuable stack with the tools and approval chain the specs drive. */
async function setup(script: Script) {
  const ctx = new Context()
  contexts.push(ctx)
  await mountAgentLoopTestDependencies(ctx)
  const root = mkdtempSync(join(tmpdir(), 'dsh-delegation-facts-'))
  roots.push(root)
  await ctx.plugin(JsonlSessionPersistence, { root })
  await ctx.plugin(SandboxPolicyService, { mode: 'workspace-write', workspaceRoot: root })
  await ctx.plugin(ApprovalService)
  await ctx.plugin(AgentLoop, { agents: [] })
  await ctx.plugin(TestSessionQuery)
  await mountWorkingDirectoryFixture(ctx)
  await ctx.plugin(SubagentRuntime)
  await ctx.plugin(SubagentSpawn, { providerName: 'spawn' })
  ctx.llm.registerAdapter(['mock'], new MockAdapter(script))
  for (const name of ['guarded', 'switch']) registerTool(ctx, name)
  const parent = await ctx.agentLoop.create(SessionId('parent'), { provider: 'mock', model: 'mock' })
  return { ctx, parent }
}

/** Register one model-callable tool whose behavior comes from the specs' hooks. */
function registerTool(
  ctx: Context,
  name: string,
  execute?: (signal: AbortSignal, agent: Agent | undefined) => void | Promise<void>,
): void {
  ctx.tools.register(defineTool({
    name,
    description: `${name} test tool`,
    parameters: {},
    output: {
      schema: { type: 'object', additionalProperties: false, properties: {} },
      render: () => [{ type: 'text', text: `${name} done` }],
    },
    execute: async (_args, exec) => {
      await execute?.(exec.signal, exec.agent)
      return {}
    },
  }))
}

function startSpec(parent: Agent) {
  return {
    provider: 'spawn',
    label: 'child task',
    request: { prompt: [{ type: 'text' as const, text: 'child task' }], parent },
    signal: new AbortController().signal,
  }
}

/** The prompt text a request carries, used to route parent and grandchild turns. */
function promptText(options: GenerateOptions): string {
  const user = options.messages.find(message => message.source?.kind === 'user')
  return (user?.content ?? []).map(block => block.type === 'text' ? block.text : '').join(' ')
}

/**
 * Answer the delegating child and the grandchild it spawns from separate
 * queues, so the shared adapter cannot hand either agent the other's response.
 * @param child - responses for the child's own requests, in order.
 * @param grandchild - responses for the grandchild's requests, in order.
 * @returns scripted adapter entries, one per model request.
 */
function routedChild(child: readonly Response[], grandchild: readonly Response[]): Script {
  const queues = { child: [...child], grandchild: [...grandchild] }
  const dispatch = (options: GenerateOptions): Response => {
    if (options.messages.some(message => message.source?.kind === 'subagent-settled')) {
      return textResponse('parent acknowledged')
    }
    const queue = promptText(options).includes('grandchild') ? queues.grandchild : queues.child
    return queue.shift() ?? textResponse('done')
  }
  return Array.from({ length: 12 }, () => dispatch)
}

/** Settle when the gate opens or the tool call is cancelled. */
function releaseWhen(gate: Promise<unknown>, signal: AbortSignal): Promise<void> {
  if (signal.aborted) return Promise.resolve()
  return new Promise<void>((resolve) => {
    signal.addEventListener('abort', () => { resolve() }, { once: true })
    void gate.then(() => { resolve() })
  })
}

/** Wait until a child's Activation is gone, i.e. its handle finished disposal. */
async function waitNoActivation(ctx: Context, childId: SessionId): Promise<void> {
  await vi.waitFor(() => {
    expect(ctx.agents.get(childId)).toBeUndefined()
  }, { timeout: 15_000 })
}

/** Every delegation notice the parent's own log holds. */
function delegationNotices(parent: Agent) {
  return parent.session.snapshotEvents().filter(event =>
    event.type === 'user/message' && event.data.source.kind === 'subagent-delegation')
}

/** Require the retained delegation facts of one child for assertions. */
function factsOf(ctx: Context, childId: SessionId) {
  const facts = delegationFactsState(ctx).get(childId)
  if (facts === undefined) throw new Error(`expected retained delegation facts for "${childId}"`)
  return facts
}

/** Read one child's durable approval outcomes from its stored log. */
async function decidedOutcomes(ctx: Context, childId: SessionId): Promise<unknown[]> {
  const stored = await loadStoredSession(ctx.sessionPersistence, childId)
  return stored.events.filter(event => event.type === 'approval/decided').map(event => event.data.outcome)
}

describe('delegation facts across residencies', () => {
  it('keeps a child that switched to ask unreported after a cold resume', { timeout: 20_000 }, async () => {
    const { ctx, parent } = await setup(scriptedChild([
      toolCallResponse('switch-1', 'switch', {}),
      textResponse('first answer'),
      toolCallResponse('guarded-1', 'guarded', {}),
      textResponse('resumed answer'),
    ]))
    ctx.on('tools/pre-execute', async (exec, next) => {
      if (exec.agent === parent) return next()
      // The child answers a policy question mid-turn, as a UI control does.
      if (exec.name === 'switch' && exec.agent !== undefined) {
        setApprovalPolicy(exec.agent.session, 'ask')
        return next()
      }
      return exec.name === 'guarded' ? { kind: 'ask', reason: 'needs a human' } : next()
    })
    ctx.on('approval/request', () => Promise.resolve<ApprovalOutcome>('rejected'))

    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await waitNoActivation(ctx, started.childId)
    // The observed switch leaves neither an attribution nor a report, so the
    // record retires and the resume inherits nothing.
    expect(delegationFactsState(ctx).has(started.childId)).toBe(false)

    await queueHostSubagentPrompt(
      ctx.subagents, parent, started.childId,
      [{ type: 'text', text: 'continue please' }], { kind: 'user' }, new AbortController().signal,
    )
    await waitNoActivation(ctx, started.childId)

    // An answerer rejection under `ask` belongs to the answerer, not the pin.
    expect(await decidedOutcomes(ctx, started.childId)).toEqual(['rejected'])
    expect(delegationNotices(parent)).toEqual([])
  })

  it('retires a released child so a later residency cannot report through it', { timeout: 20_000 }, async () => {
    const entered = Promise.withResolvers<undefined>()
    const held = Promise.withResolvers<undefined>()
    const { ctx, parent } = await setup(scriptedChild([
      toolCallResponse('block-1', 'block', {}),
      toolCallResponse('guarded-1', 'guarded', {}),
      textResponse('after release'),
    ]))
    registerTool(ctx, 'block', async (signal) => {
      entered.resolve(undefined)
      await releaseWhen(held.promise, signal)
    })
    ctx.on('tools/pre-execute', async (exec, next) =>
      exec.name === 'guarded' ? { kind: 'ask' } : next())

    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await entered.promise
    expect(delegationFactsState(ctx).get(started.childId)).toMatchObject({ pinned: true })

    await ctx.subagents.drainChildren(parent, [started.childId])
    held.resolve(undefined)
    expect(delegationFactsState(ctx).has(started.childId)).toBe(false)
    expect(ctx.agents.get(started.childId)).toBeUndefined()

    await queueHostSubagentPrompt(
      ctx.subagents, parent, started.childId,
      [{ type: 'text', text: 'continue please' }], { kind: 'user' }, new AbortController().signal,
    )
    await waitNoActivation(ctx, started.childId)

    // The delegated pin still refuses the ask, but a retired record cannot
    // attribute it, so the parent hears nothing and the empty record retires
    // with the residency that minted it.
    expect(await decidedOutcomes(ctx, started.childId)).toEqual(['rejected'])
    expect(delegationNotices(parent)).toEqual([])
    expect(delegationFactsState(ctx).has(started.childId)).toBe(false)
  })

  it('leaves no facts behind when child creation fails', { timeout: 20_000 }, async () => {
    const { ctx, parent } = await setup(scriptedChild([textResponse('child done')]))
    const workingDirectory = ctx.get('workingDirectory')
    if (workingDirectory === undefined) throw new Error('expected the working-directory fixture')
    // Creation fails after the descriptor and delegated policy were appended.
    vi.spyOn(workingDirectory, 'set').mockRejectedValue(new Error('cwd rejected'))

    await expect(ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' }))
      .rejects.toThrow('cwd rejected')

    expect(delegationFactsState(ctx).size).toBe(0)
  })

  it('keeps a settled child facts when a resume fails admission', { timeout: 20_000 }, async () => {
    const { ctx, parent } = await setup(scriptedChild([
      toolCallResponse('guarded-1', 'guarded', {}),
      textResponse('first answer'),
      toolCallResponse('guarded-2', 'guarded', {}),
      textResponse('resumed answer'),
    ]))
    ctx.on('tools/pre-execute', async (exec, next) =>
      exec.name === 'guarded' ? { kind: 'ask' } : next())

    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await waitNoActivation(ctx, started.childId)
    expect(factsOf(ctx, started.childId)).toMatchObject({ pinned: true })
    expect([...factsOf(ctx, started.childId).reported]).toEqual(['guarded'])
    expect(delegationNotices(parent)).toHaveLength(1)

    // A resume that fails admission must not discard the retained facts.
    const resolveModelInfo = vi.spyOn(ctx.llm, 'resolveModelInfo')
      .mockResolvedValue({ inputModalities: ['text'] } as never)
    await expect(queueHostSubagentPrompt(ctx.subagents, parent, started.childId, [
      { type: 'text', text: 'see this' },
      {
        type: 'image' as const,
        attachment: {
          attachmentId: 'att-1' as never, mediaType: 'image/png' as const, bytes: 1, width: 1, height: 1,
        },
      },
    ], { kind: 'user' }, new AbortController().signal))
      .rejects.toMatchObject({ code: 'MODEL_DOES_NOT_SUPPORT_IMAGES' })
    resolveModelInfo.mockRestore()

    expect(factsOf(ctx, started.childId)).toMatchObject({ pinned: true })
    expect([...factsOf(ctx, started.childId).reported]).toEqual(['guarded'])

    // The next residency still refuses to report the same tool a second time.
    await queueHostSubagentPrompt(
      ctx.subagents, parent, started.childId,
      [{ type: 'text', text: 'continue please' }], { kind: 'user' }, new AbortController().signal,
    )
    await waitNoActivation(ctx, started.childId)
    expect(await decidedOutcomes(ctx, started.childId)).toEqual(['rejected', 'rejected'])
    expect(delegationNotices(parent)).toHaveLength(1)
  })

  it('clears the facts even when teardown fails during a drain', { timeout: 20_000 }, async () => {
    const entered = Promise.withResolvers<undefined>()
    const held = Promise.withResolvers<undefined>()
    const { ctx, parent } = await setup(scriptedChild([
      textResponse('first answer'),
      toolCallResponse('block-1', 'block', {}),
    ]))
    registerTool(ctx, 'block', async (signal) => {
      entered.resolve(undefined)
      await releaseWhen(held.promise, signal)
    })

    // One child settles and keeps its facts; the other is still resident when
    // the drain begins and its teardown fails.
    const settled = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await waitNoActivation(ctx, settled.childId)
    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await entered.promise
    expect(delegationFactsState(ctx).size).toBe(2)
    const resident = managerState(ctx).resident.get(started.childId)
    if (resident === undefined || resident.kind !== 'local') throw new Error('expected a resident local child')
    const disposeFailure = vi.spyOn(resident.handle, 'dispose')
      .mockRejectedValue(new Error('handle teardown failed'))

    await expect(subagentManager(ctx).drain()).rejects.toMatchObject({ code: 'ACTIVATION_TEARDOWN_FAILED' })
    expect(delegationFactsState(ctx).size).toBe(0)

    // The mocked failure never released the handle, so release it here.
    disposeFailure.mockRestore()
    held.resolve(undefined)
    await resident.handle.dispose()
  })

  it('leaves no record for a child it only resumes', { timeout: 20_000 }, async () => {
    const { ctx, parent } = await setup(scriptedChild([
      textResponse('first answer'),
      toolCallResponse('guarded-1', 'guarded', {}),
      textResponse('resumed answer'),
    ]))
    ctx.on('tools/pre-execute', async (exec, next) =>
      exec.name === 'guarded' ? { kind: 'ask' } : next())

    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await waitNoActivation(ctx, started.childId)
    // Stand in for a child a previous process created: no record here at all.
    delegationFactsState(ctx).delete(started.childId)

    await queueHostSubagentPrompt(
      ctx.subagents, parent, started.childId,
      [{ type: 'text', text: 'continue please' }], { kind: 'user' }, new AbortController().signal,
    )
    await waitNoActivation(ctx, started.childId)

    // The resumed pin still refuses the ask, and the epoch inherited nothing to
    // attribute or remember, so it leaves no record behind.
    expect(await decidedOutcomes(ctx, started.childId)).toEqual(['rejected'])
    expect(delegationNotices(parent)).toEqual([])
    expect(delegationFactsState(ctx).has(started.childId)).toBe(false)
  })

  it('retires a resumed descendant record when a scoped teardown releases it', { timeout: 20_000 }, async () => {
    const entered = Promise.withResolvers<undefined>()
    const held = Promise.withResolvers<undefined>()
    const childHeld = Promise.withResolvers<undefined>()
    const spawned = Promise.withResolvers<SessionId>()
    const { ctx, parent } = await setup(routedChild(
      [toolCallResponse('nested-1', 'nested', {}), textResponse('child answer')],
      [textResponse('grandchild answer'), toolCallResponse('block-1', 'block', {})],
    ))
    registerTool(ctx, 'nested', async (signal, child) => {
      if (child === undefined) throw new Error('expected a caller agent')
      const started = await ctx.subagents.startActivation({
        provider: 'spawn',
        label: 'grandchild task',
        request: { prompt: [{ type: 'text', text: 'grandchild task' }], parent: child },
        signal: new AbortController().signal,
        delivery: 'parent',
      })
      spawned.resolve(started.childId)
      // Keep the delegating child mid-turn so it stays resident as the root of
      // the scoped teardown and as the grandchild's live direct parent.
      await releaseWhen(childHeld.promise, signal)
    })
    registerTool(ctx, 'block', async (signal) => {
      entered.resolve(undefined)
      await releaseWhen(held.promise, signal)
    })

    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    const grandchildId = await spawned.promise
    await waitNoActivation(ctx, grandchildId)
    expect(factsOf(ctx, grandchildId)).toMatchObject({ pinned: true })
    const child = ctx.agents.get(started.childId)
    if (child === undefined) throw new Error('expected the child agent to stay resident')

    // Hold the grandchild in a resumed epoch, so releasing it is a scoped
    // teardown of a record this epoch inherited rather than created.
    await queueHostSubagentPrompt(
      ctx.subagents, child, grandchildId,
      [{ type: 'text', text: 'continue please' }], { kind: 'user' }, new AbortController().signal,
    )
    await entered.promise

    await ctx.subagents.drainDescendants([child])
    held.resolve(undefined)

    // The scoped teardown retires the grandchild's record and keeps its root.
    expect(delegationFactsState(ctx).has(grandchildId)).toBe(false)
    expect(ctx.agents.get(grandchildId)).toBeUndefined()
    expect(ctx.agents.get(started.childId)).toBe(child)
    expect(delegationNotices(child)).toEqual([])
    expect(delegationNotices(parent)).toEqual([])
    // The drained subtree stays closed, so no later residency of the grandchild
    // is admitted and no rejection could reach either parent through it.
    await expect(ctx.subagents.startActivation({
      provider: 'spawn',
      label: 'grandchild task',
      request: { prompt: [{ type: 'text', text: 'grandchild task' }], parent: child },
      signal: new AbortController().signal,
      delivery: 'parent',
    })).rejects.toMatchObject({ code: 'DRAINING' })

    childHeld.resolve(undefined)
    await waitNoActivation(ctx, started.childId)
  })

  it('clears the facts of settled children when the manager drains', { timeout: 20_000 }, async () => {
    const { ctx, parent } = await setup(scriptedChild([textResponse('first answer')]))

    const started = await ctx.subagents.startActivation({ ...startSpec(parent), delivery: 'parent' })
    await waitNoActivation(ctx, started.childId)
    expect(delegationFactsState(ctx).size).toBe(1)

    await subagentManager(ctx).drain()
    expect(delegationFactsState(ctx).size).toBe(0)
  })
})
