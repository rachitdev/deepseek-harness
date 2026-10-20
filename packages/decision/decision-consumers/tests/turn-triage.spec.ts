import { afterEach, describe, expect, it, vi, type MockInstance } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import AgentRegistry from '@deepseek-ai/dsh-agent'
import AgentLoop from '@deepseek-ai/dsh-agent-loop'
import DecisionRuntime, { DecisionError } from '@deepseek-ai/dsh-decision'
import type { DecisionProvider, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import LlmRuntime, { createUserMessage, LlmAdapter, ReasoningEffortId, ToolCallId, type GenerateOptions, type LlmResolvedModelInfo, type StreamChunk } from '@deepseek-ai/dsh-llm'
import SessionStore, { SessionId } from '@deepseek-ai/dsh-session'
import SessionProjectionRegistry from '@deepseek-ai/dsh-session-projection'
import SystemPrompt from '@deepseek-ai/dsh-system-prompt'
import ToolRuntime, { defineContentToolFixture } from '@deepseek-ai/dsh-tools'
import * as consumers from '../src/index.ts'
import { createMockDecisionService } from './helpers.ts'

class RecordingAdapter extends LlmAdapter {
  readonly requests: GenerateOptions[] = []
  /** Answer the first request with one call to the `noop` tool, so the turn takes a second step. */
  callToolFirst = false

  override resolveModel(provider: string, model: string): Promise<LlmResolvedModelInfo> {
    return Promise.resolve({ provider, id: model, name: model, reasoning: { efforts: [{ id: ReasoningEffortId('high'), name: 'High' }] } })
  }

  async * stream(options: GenerateOptions): AsyncIterable<StreamChunk> {
    this.requests.push(options)
    if (this.callToolFirst && this.requests.length === 1) {
      yield { type: 'block-start', index: 0, blockType: 'tool-call' }
      yield { type: 'block-end', index: 0, block: { type: 'tool-call', id: ToolCallId('c1'), name: 'noop', arguments: '{}' } }
      yield { type: 'finish', reason: { kind: 'tool-calls' } }
      return
    }
    yield { type: 'block-start', index: 0, blockType: 'text' }
    yield { type: 'text-delta', index: 0, text: 'ok' }
    yield { type: 'block-end', index: 0, block: { type: 'text', text: 'ok' } }
    yield { type: 'finish', reason: { kind: 'stop' } }
  }
}

type ProviderMode = 'judge' | 'fail' | 'hang'

class TriageProvider implements DecisionProvider {
  readonly id = 'test-jev'
  readonly requests: DecisionRequest[] = []
  constructor(private readonly mode: ProviderMode = 'judge', private readonly confidence = 0.9) {}

  available(): boolean {
    return true
  }

  evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult> {
    this.requests.push(request)
    if (this.mode === 'fail') return Promise.reject(new DecisionError('provider down', 'DECISION_PROVIDER_ERROR'))
    if (this.mode === 'hang') {
      return new Promise((_resolve, reject) => {
        signal?.addEventListener('abort', () => { reject(new DecisionError('aborted', 'DECISION_ABORTED')) }, { once: true })
      })
    }
    const choice = request.state.includes('refactor') ? 'frontier' : 'fast'
    return Promise.resolve({
      providerId: this.id,
      answers: { tier: { type: 'choice', choice, probabilities: { [choice]: this.confidence } } },
    })
  }
}

const FAST: consumers.TurnTriageTier = { name: 'fast', description: 'Quick lookups and simple answers', provider: 'mock', model: 'fast-model' }
const TIERS: consumers.TurnTriageTier[] = [
  FAST,
  { name: 'standard', description: 'Routine edits', provider: 'mock', model: 'standard-model' },
  { name: 'frontier', description: 'Hard multi-step work', provider: 'mock', model: 'frontier-model', reasoningEffort: 'high' },
]

let context: Context | undefined

afterEach(async () => {
  await context?.fiber.dispose()
  context = undefined
})

async function harness(
  provider: TriageProvider,
  turnTriage: consumers.TurnTriageConfig,
): Promise<{ readonly ctx: Context; readonly adapter: RecordingAdapter }> {
  const ctx = new Context()
  context = ctx
  await ctx.plugin(LlmRuntime)
  await ctx.plugin(SessionStore)
  await ctx.plugin(SessionProjectionRegistry)
  await ctx.plugin(SystemPrompt)
  await ctx.plugin(ToolRuntime)
  await ctx.plugin(AgentRegistry)
  await ctx.plugin(DecisionRuntime)
  ctx.effect(() => ctx.decision.registerProvider(provider))
  await ctx.plugin(consumers, { turnTriage: Object.assign({ enabled: true, tiers: TIERS, fallbackTier: 'standard' }, turnTriage) })
  await ctx.plugin(AgentLoop, { agents: [] })
  const adapter = new RecordingAdapter()
  ctx.llm.registerAdapter(['mock'], adapter)
  return { ctx, adapter }
}

async function runTurn(ctx: Context, id: string, text: string): Promise<ReturnType<Context['agentLoop']['create']>> {
  const agent = await ctx.agentLoop.create(SessionId(id), { provider: 'mock', model: 'configured-model' })
  agent.followup(createUserMessage({ content: [{ type: 'text', text }], source: { kind: 'user' } }))
  await agent.whenIdle()
  return agent
}

type LogMethods = Record<'info' | 'warn', (...args: unknown[]) => void>

function loggerSpy(ctx: Context, level: keyof LogMethods): MockInstance<LogMethods[keyof LogMethods]> {
  return vi.spyOn(Object.getPrototypeOf(ctx.logger) as LogMethods, level)
}

describe('live turn triage', () => {
  it('logs the verdict without changing the model in advisory mode', async () => {
    const provider = new TriageProvider()
    const { ctx, adapter } = await harness(provider, {})
    const info = loggerSpy(ctx, 'info')

    const agent = await runTurn(ctx, 'triage-advisory', 'Please refactor the session store.')
    const verdict = await vi.waitFor(() => {
      const call = info.mock.calls.find(([format]) => String(format).startsWith('turn triage (%s)'))
      expect(call).toBeDefined()
      return call
    })
    info.mockRestore()

    expect(verdict?.slice(1, 9)).toEqual(['advisory', agent.id, 1, 'frontier', 'frontier', '0.90', 'mock', 'frontier-model'])

    expect(provider.requests).toHaveLength(1)
    expect(provider.requests[0]?.state).toContain('Please refactor the session store.')
    expect(adapter.requests.map(request => request.model)).toEqual(['configured-model'])
    expect(agent.session.requestHeader()?.config.model).toBe('configured-model')
  })

  it('routes the turn to the chosen tier and records it in the request header in enforce mode', async () => {
    const { ctx, adapter } = await harness(new TriageProvider(), { mode: 'enforce' })

    const agent = await runTurn(ctx, 'triage-enforce', 'Please refactor the session store.')

    expect(adapter.requests.map(request => request.model)).toEqual(['frontier-model'])
    expect(agent.session.requestHeader()?.config).toMatchObject({ provider: 'mock', model: 'frontier-model', reasoningEffort: 'high' })
  })

  it('drops an inherited reasoning effort when the tier sets none', async () => {
    const { ctx } = await harness(new TriageProvider(), { mode: 'enforce' })

    const agent = await ctx.agentLoop.create(SessionId('triage-fast'), { provider: 'mock', model: 'configured-model', reasoningEffort: ReasoningEffortId('high') })
    agent.followup(createUserMessage({ content: [{ type: 'text', text: 'What time is it?' }], source: { kind: 'user' } }))
    await agent.whenIdle()

    expect(agent.session.requestHeader()?.config.model).toBe('fast-model')
    expect(agent.session.requestHeader()?.config.reasoningEffort).toBeUndefined()
  })

  it('uses the fallback tier when the verdict is below the confidence floor', async () => {
    const { ctx, adapter } = await harness(new TriageProvider('judge', 0.4), { mode: 'enforce', minConfidence: 0.5 })

    await runTurn(ctx, 'triage-low-confidence', 'What time is it?')

    expect(adapter.requests.map(request => request.model)).toEqual(['standard-model'])
  })

  it.each(['fail', 'hang'] as const)('keeps the configured model when the provider %s', async (mode) => {
    const { ctx, adapter } = await harness(new TriageProvider(mode), { mode: 'enforce', timeoutMs: 20 })
    const warn = loggerSpy(ctx, 'warn')

    await runTurn(ctx, `triage-${mode}`, 'Please refactor the session store.')
    const warned = warn.mock.calls.some(([format]) => format === 'turn triage: agent %s turn %d failed; keeping the configured model: %o')
    warn.mockRestore()

    expect(warned).toBe(true)
    expect(adapter.requests.map(request => request.model)).toEqual(['configured-model'])
  })

  it('judges a turn once even when it takes several steps', async () => {
    const provider = new TriageProvider()
    const { ctx, adapter } = await harness(provider, { mode: 'enforce' })
    adapter.callToolFirst = true
    ctx.tools.register(defineContentToolFixture({ name: 'noop', description: '', parameters: {}, execute: () => Promise.resolve([{ type: 'text', text: 'done' }]) }))

    await runTurn(ctx, 'triage-once', 'Please refactor the session store.')

    expect(provider.requests).toHaveLength(1)
    expect(adapter.requests.map(request => request.model)).toEqual(['frontier-model', 'frontier-model'])
  })

  it('skips delegated agents unless subagents are included', async () => {
    const provider = new TriageProvider()
    const { ctx, adapter } = await harness(provider, { mode: 'enforce' })

    const handle = await ctx.agents.create({
      sessionId: SessionId('triage-child'),
      meta: { origin: 'subagent', delegationDepth: 1 },
      agentOptions: { provider: 'mock', model: 'configured-model' },
    })
    handle.agent.followup(createUserMessage({ content: [{ type: 'text', text: 'Please refactor it.' }], source: { kind: 'user' } }))
    await handle.agent.whenIdle()
    await handle.dispose()

    expect(provider.requests).toHaveLength(0)
    expect(adapter.requests.map(request => request.model)).toEqual(['configured-model'])
  })

  it('makes no decision call for a step without direct user text', async () => {
    const provider = new TriageProvider()
    const { ctx, adapter } = await harness(provider, { mode: 'enforce' })

    const agent = await ctx.agentLoop.create(SessionId('triage-no-user'), { provider: 'mock', model: 'configured-model' })
    agent.followup(createUserMessage({ content: [{ type: 'text', text: 'note' }], source: { kind: 'system-notice' } as never }))
    await agent.whenIdle()

    expect(provider.requests).toHaveLength(0)
    expect(adapter.requests.map(request => request.model)).toEqual(['configured-model'])
  })

  it.each<[consumers.TurnTriageConfig, RegExp]>([
    [{ tiers: TIERS.slice(0, 1) }, /at least two tiers/],
    [{ tiers: [FAST, FAST] }, /"fast" is duplicated/],
    [{ fallbackTier: 'missing' }, /fallbackTier "missing" names no configured tier/],
  ])('rejects invalid configuration %#', (override, message) => {
    const ctx = new Context()
    expect(() => { consumers.installTurnTriage(ctx, Object.assign({ enabled: true, tiers: TIERS, fallbackTier: 'standard' }, override)) }).toThrow(message)
  })
})

describe('triageTurn', () => {
  const policy = { tiers: TIERS, fallback: TIERS[1]!, minConfidence: 0.5 }

  it('includes the previous reply tail in the judged state', async () => {
    let state = ''
    const service = createMockDecisionService((request) => {
      state = request.state
      return Promise.resolve({ providerId: 'p', answers: { tier: { type: 'choice', choice: 'fast', probabilities: { fast: 0.8 } } } })
    })

    const triage = await consumers.triageTurn(service, { request: 'go ahead', previousReply: 'Shall I refactor it?' }, policy)

    expect(state).toBe('User request: go ahead\nPrevious assistant reply (tail): Shall I refactor it?')
    expect(triage).toEqual({ tier: TIERS[0], chosen: 'fast', confidence: 0.8, providerId: 'p' })
  })

  it('treats a missing probability as zero confidence', async () => {
    const service = createMockDecisionService(() => Promise.resolve({
      providerId: 'p',
      answers: { tier: { type: 'choice', choice: 'frontier', probabilities: {} } },
    }))

    expect((await consumers.triageTurn(service, { request: 'x', previousReply: '' }, policy)).tier).toBe(TIERS[1])
  })

  it.each([
    [{ tier: { type: 'boolean', probability: 1 } }, /expected choice answer/],
    [{ tier: { type: 'choice', choice: 'other', probabilities: {} } }, /unknown tier "other"/],
  ] as const)('rejects an unusable answer %#', async (answers, message) => {
    const service = createMockDecisionService(() => Promise.resolve({ providerId: 'p', answers }))

    await expect(consumers.triageTurn(service, { request: 'x', previousReply: '' }, policy)).rejects.toThrow(message)
  })
})

describe('apply', () => {
  it('installs no live listener for disabled recall filtering and triage', () => {
    const ctx = new Context()
    const on = vi.spyOn(ctx, 'on')

    consumers.apply(ctx, { recallFilter: { enabled: false }, turnTriage: { enabled: false } })

    expect(on).not.toHaveBeenCalled()
  })
})
