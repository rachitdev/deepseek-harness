import { afterEach, describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import AgentRegistry from '@deepseek-ai/dsh-agent'
import AgentLoop from '@deepseek-ai/dsh-agent-loop'
import DecisionRuntime from '@deepseek-ai/dsh-decision'
import type { DecisionAnswer, DecisionProvider, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import LlmRuntime, {
  createUserMessage,
  isAgentLoopRequest,
  LlmAdapter,
  resolveRetryPolicy,
} from '@deepseek-ai/dsh-llm'
import type {
  GenerateOptions,
  ResolvedRetryPolicy,
  StreamChunk,
} from '@deepseek-ai/dsh-llm'
import * as retry from '@deepseek-ai/dsh-llm-retry'
import SessionStore, { SessionId } from '@deepseek-ai/dsh-session'
import SessionProjectionRegistry from '@deepseek-ai/dsh-session-projection'
import SystemPrompt from '@deepseek-ai/dsh-system-prompt'
import SubagentModelSelectionConfig from '@deepseek-ai/dsh-tool-subagent/model-selection-settings'
import ToolRuntime from '@deepseek-ai/dsh-tools'
import * as consumers from '../src/index.ts'

function response(text: string, tokens: number): StreamChunk[] {
  return [
    { type: 'block-start', index: 0, blockType: 'text' },
    { type: 'text-delta', index: 0, text },
    { type: 'block-end', index: 0, block: { type: 'text', text } },
    { type: 'usage', usage: { inputTokens: 10, outputTokens: tokens - 10, totalTokens: tokens } },
    { type: 'finish', reason: { kind: 'stop' } },
  ]
}

function failure(code = 'RATE_LIMIT'): StreamChunk {
  return {
    type: 'finish',
    reason: {
      kind: 'error',
      failure: { code, message: `${code} from scripted route`, status: code === 'RATE_LIMIT' ? 429 : 500 },
    },
  }
}

interface AdapterOptions {
  readonly activeFailures?: number
  readonly activeFailureCode?: string
  readonly failedProbes?: ReadonlySet<string>
  readonly emptyProbes?: ReadonlySet<string>
  readonly agentFailures?: Readonly<Record<string, number>>
  readonly blockProbes?: boolean
}

class FailoverAdapter extends LlmAdapter {
  readonly requests: GenerateOptions[] = []
  readonly probesEntered = Promise.withResolvers<undefined>()
  probeAbortCount = 0
  private activeFailures: number
  private readonly agentFailures = new Map<string, number>()
  private readonly options: AdapterOptions
  private probesStarted = 0
  private readonly retryPolicy = resolveRetryPolicy({
    mode: 'normal',
    maxRetries: 2,
    retryableCodes: ['RATE_LIMIT', 'SERVER'],
    backoff: { initialDelayMs: 1, maxDelayMs: 1, jitterRatio: 0 },
  }, 'failover test policy')

  constructor(options: AdapterOptions = {}) {
    super()
    this.options = options
    this.activeFailures = options.activeFailures ?? 1
    for (const [model, count] of Object.entries(options.agentFailures ?? {})) {
      this.agentFailures.set(model, count)
    }
  }

  override providerRetryPolicy(_provider: string): ResolvedRetryPolicy {
    return this.retryPolicy
  }

  async * stream(options: GenerateOptions): AsyncIterable<StreamChunk> {
    this.requests.push(options)
    const agentRequest = isAgentLoopRequest(options)
    if (agentRequest && options.model === 'active' && this.activeFailures > 0) {
      this.activeFailures -= 1
      yield failure(this.options.activeFailureCode)
      return
    }
    const remainingFailures = this.agentFailures.get(options.model) ?? 0
    if (agentRequest && remainingFailures > 0) {
      this.agentFailures.set(options.model, remainingFailures - 1)
      yield failure('SERVER')
      return
    }
    if (!agentRequest && this.options.blockProbes === true) {
      this.probesStarted += 1
      if (this.probesStarted === 2) this.probesEntered.resolve(undefined)
      await new Promise<void>((resolve) => {
        if (options.signal?.aborted === true) {
          resolve()
          return
        }
        options.signal?.addEventListener('abort', () => { resolve() }, { once: true })
      })
      this.probeAbortCount += 1
      throw options.signal?.reason ?? new Error('probe aborted')
    }
    if (!agentRequest && this.options.failedProbes?.has(options.model) === true) {
      yield failure('SERVER')
      return
    }
    if (!agentRequest && this.options.emptyProbes?.has(options.model) === true) {
      yield { type: 'usage', usage: { inputTokens: 5, outputTokens: 0 } }
      yield { type: 'finish', reason: { kind: 'stop' } }
      return
    }
    if (agentRequest && options.model === 'active') {
      yield* response('same route recovered', 20)
      return
    }
    const chunks = options.model === 'candidate-b'
      ? response('export const reverse = (x: Uint8Array) => x.toReversed()', 24)
      : response('function reverse(x) { return x }', 38)
    yield* chunks
  }
}

interface ProviderOptions {
  readonly choice?: string
  readonly confidence?: number
  readonly codeValidity?: number
  readonly throws?: boolean
}

class SelectingProvider implements DecisionProvider {
  readonly id = 'test-jev'
  readonly requests: DecisionRequest[] = []
  private readonly options: ProviderOptions

  constructor(options: ProviderOptions = {}) {
    this.options = options
  }

  available(): boolean {
    return true
  }

  evaluate(request: DecisionRequest, _signal?: AbortSignal): Promise<DecisionResult> {
    this.requests.push(request)
    if (this.options.throws === true) return Promise.reject(new Error('judge unavailable'))
    const question = request.questions.best_provider
    if (question?.type !== 'choice') throw new Error('expected best_provider choice')
    const choices = Object.keys(question.criteria)
    const chosenWorker = this.options.choice === undefined
      ? choices.at(-1)
      : Object.entries(question.criteria)
        .find(([, description]) => description.includes(`/${this.options.choice}: latency=`))?.[0]
    if (chosenWorker === undefined) throw new Error('scripted model was not offered')
    const confidence = this.options.confidence ?? 0.94
    const remainder = choices.length === 1 ? 0 : (1 - confidence) / (choices.length - 1)
    const answers: DecisionResult['answers'] = Object.fromEntries(Object.entries(request.questions).map(([key, value]): [string, DecisionAnswer] => {
      switch (value.type) {
        case 'choice':
          return [key, {
            type: 'choice',
            choice: chosenWorker,
            probabilities: Object.fromEntries(choices.map(worker => [
              worker,
              worker === chosenWorker ? confidence : remainder,
            ])),
          }]
        case 'score':
          return [key, {
            type: 'score',
            score: key.startsWith('code_validity_')
              ? this.options.codeValidity ?? 2.8
              : 3,
            probabilities: { '0': 0, '1': 0, '2': 0.2, '3': 0.8 },
          }]
        case 'boolean':
          throw new Error(`takeover arbitration must not ask the boolean question "${key}"`)
      }
    }))
    return Promise.resolve({ providerId: this.id, answers })
  }
}

let context: Context | undefined

afterEach(async () => {
  await context?.fiber.dispose()
  context = undefined
})

async function harness(
  provider: SelectingProvider,
  adapter: FailoverAdapter,
  failover: Partial<consumers.ModelFailoverConfig> = {},
): Promise<{
  readonly ctx: Context
  readonly failoverFiber: Awaited<ReturnType<Context['plugin']>>
}> {
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
  await ctx.plugin(retry)
  const failoverFiber = await ctx.plugin(consumers, {
    failover: {
      enabled: true,
      candidates: [
        { provider: 'mock', model: 'candidate-a' },
        { provider: 'mock', model: 'candidate-b' },
      ],
      useAllowedModels: false,
      triggerCodes: ['RATE_LIMIT', 'SERVER'],
      maxCandidates: 4,
      probeTimeoutMs: 1_000,
      probeMaxTokens: 80,
      maxTaskChars: 1_000,
      ...failover,
    },
  })
  await ctx.plugin(AgentLoop, { agents: [] })
  ctx.llm.registerAdapter(['mock'], adapter)
  return { ctx, failoverFiber }
}

async function start(ctx: Context, id: string): ReturnType<Context['agentLoop']['create']> {
  const agent = await ctx.agentLoop.create(SessionId(id), {
    provider: 'mock',
    model: 'active',
  })
  agent.followup(createUserMessage({
    content: [{ type: 'text', text: 'Implement the byte reversal safely.' }],
    source: { kind: 'user' },
  }))
  return agent
}

function requestRoutes(adapter: FailoverAdapter): Array<{ model: string; agentLoop: boolean }> {
  return adapter.requests.map(request => ({
    model: request.model,
    agentLoop: isAgentLoopRequest(request),
  }))
}

describe('live model failover', () => {
  it('probes in parallel, asks the decision provider, hot-swaps, and retries the same step', async () => {
    const provider = new SelectingProvider()
    const adapter = new FailoverAdapter()
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-failover')

    await agent.whenIdle()

    expect(requestRoutes(adapter)).toEqual([
      { model: 'active', agentLoop: true },
      { model: 'candidate-a', agentLoop: false },
      { model: 'candidate-b', agentLoop: false },
      { model: 'candidate-b', agentLoop: true },
    ])
    expect(provider.requests[0]?.state).toContain('Implement the byte reversal safely.')
    expect(agent.session.snapshotEvents().some(event => event.type === 'llm/retry')).toBe(false)
    expect(agent.session.requestHeader()?.config).toMatchObject({ provider: 'mock', model: 'candidate-b' })
    expect(agent.session.deriveMessages().at(-1)).toMatchObject({
      role: 'assistant',
      source: { kind: 'model', provider: 'mock', model: 'candidate-b' },
    })
  })

  it('delegates to same-route retry when every canary fails', async () => {
    const provider = new SelectingProvider()
    const adapter = new FailoverAdapter({ failedProbes: new Set(['candidate-a', 'candidate-b']) })
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-all-probes-failed')

    await agent.whenIdle()

    expect(provider.requests).toHaveLength(0)
    expect(requestRoutes(adapter).at(-1)).toEqual({ model: 'active', agentLoop: true })
    expect(agent.session.snapshotEvents().filter(event => event.type === 'llm/retry')).toHaveLength(1)
    expect(agent.session.deriveMessages().at(-1)).toMatchObject({
      content: [{ type: 'text', text: 'same route recovered' }],
    })
  })

  it('takes over when the judge cannot strongly separate several healthy routes', async () => {
    const provider = new SelectingProvider({ confidence: 0.34 })
    const adapter = new FailoverAdapter()
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-near-uniform-ranking')

    await agent.whenIdle()

    expect(requestRoutes(adapter).at(-1)).toEqual({ model: 'candidate-b', agentLoop: true })
    expect(agent.session.snapshotEvents().some(event => event.type === 'llm/retry')).toBe(false)
    expect(agent.session.requestHeader()?.config).toMatchObject({ provider: 'mock', model: 'candidate-b' })
  })

  it('keeps generic recovery when the judge rates every healthy route as broken', async () => {
    const provider = new SelectingProvider({ codeValidity: 0.4 })
    const adapter = new FailoverAdapter()
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-broken-code-verdict')

    await agent.whenIdle()

    expect(requestRoutes(adapter).at(-1)).toEqual({ model: 'active', agentLoop: true })
    expect(agent.session.snapshotEvents().filter(event => event.type === 'llm/retry')).toHaveLength(1)
  })

  it('takes the first healthy probe when JEV/Laya cannot judge', async () => {
    const provider = new SelectingProvider({ throws: true })
    const adapter = new FailoverAdapter()
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-judge-unavailable')

    await agent.whenIdle()

    expect(requestRoutes(adapter)).toEqual([
      { model: 'active', agentLoop: true },
      { model: 'candidate-a', agentLoop: false },
      { model: 'candidate-b', agentLoop: false },
      { model: 'candidate-a', agentLoop: true },
    ])
    expect(agent.session.snapshotEvents().some(event => event.type === 'llm/retry')).toBe(false)
    expect(agent.session.requestHeader()?.config).toMatchObject({ provider: 'mock', model: 'candidate-a' })
  })

  it('reads candidates from the live allowedModels setting when enabled', async () => {
    const provider = new SelectingProvider({ choice: 'allowed-b' })
    const adapter = new FailoverAdapter()
    const { ctx } = await harness(provider, adapter, { useAllowedModels: true })
    await ctx.plugin(SubagentModelSelectionConfig, {
      enabled: true,
      allowedModels: [
        { provider: 'mock', model: 'allowed-a' },
        { provider: 'mock', model: 'allowed-b' },
      ],
    })
    const agent = await start(ctx, 'jev-allowed-models')

    await agent.whenIdle()

    expect(requestRoutes(adapter)).toEqual([
      { model: 'active', agentLoop: true },
      { model: 'allowed-a', agentLoop: false },
      { model: 'allowed-b', agentLoop: false },
      { model: 'candidate-a', agentLoop: false },
      { model: 'candidate-b', agentLoop: false },
      { model: 'allowed-b', agentLoop: true },
    ])
    expect(agent.session.requestHeader()?.config.model).toBe('allowed-b')
  })

  it('takes over on unclassified provider errors that cannot stay on the dead route', async () => {
    const provider = new SelectingProvider()
    const adapter = new FailoverAdapter({ activeFailureCode: 'PI_AI_ERROR' })
    const { ctx } = await harness(provider, adapter, { triggerCodes: ['PI_AI_ERROR'] })
    const agent = await start(ctx, 'jev-unclassified-provider-error')

    await agent.whenIdle()

    expect(requestRoutes(adapter).at(-1)).toEqual({ model: 'candidate-b', agentLoop: true })
    expect(agent.session.snapshotEvents().some(event => event.type === 'llm/retry')).toBe(false)
    expect(agent.session.requestHeader()?.config.model).toBe('candidate-b')
  })

  it('does not cycle back to a route that failed after takeover', async () => {
    const provider = new SelectingProvider({ choice: 'candidate-b' })
    const adapter = new FailoverAdapter({ agentFailures: { 'candidate-b': 1 } })
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-no-route-cycle')

    await agent.whenIdle()

    expect(requestRoutes(adapter)).toEqual([
      { model: 'active', agentLoop: true },
      { model: 'candidate-a', agentLoop: false },
      { model: 'candidate-b', agentLoop: false },
      { model: 'candidate-b', agentLoop: true },
      { model: 'candidate-a', agentLoop: false },
      { model: 'candidate-a', agentLoop: true },
    ])
    expect(provider.requests).toHaveLength(1)
    expect(agent.session.requestHeader()?.config.model).toBe('candidate-a')
  })

  it('ignores failure codes outside the configured takeover policy', async () => {
    const provider = new SelectingProvider()
    const adapter = new FailoverAdapter({ activeFailureCode: 'INVALID_REQUEST' })
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-non-trigger')

    await agent.whenIdle()

    expect(requestRoutes(adapter)).toEqual([{ model: 'active', agentLoop: true }])
    expect(provider.requests).toHaveLength(0)
    expect(agent.session.snapshotEvents().some(event => event.type === 'llm/retry')).toBe(false)
  })

  it('treats an empty canary output as failed and uses the remaining survivor', async () => {
    const provider = new SelectingProvider()
    const adapter = new FailoverAdapter({ emptyProbes: new Set(['candidate-a']) })
    const { ctx } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-empty-probe')

    await agent.whenIdle()

    expect(provider.requests).toHaveLength(0)
    expect(agent.session.requestHeader()?.config.model).toBe('candidate-b')
  })

  it('aborts and drains in-flight canaries when the plugin is disposed', async () => {
    const provider = new SelectingProvider()
    const adapter = new FailoverAdapter({ blockProbes: true })
    const { ctx, failoverFiber } = await harness(provider, adapter)
    const agent = await start(ctx, 'jev-disposal')
    await adapter.probesEntered.promise

    await failoverFiber.dispose()
    await agent.whenIdle()

    expect(adapter.probeAbortCount).toBe(2)
    expect(provider.requests).toHaveLength(0)
    expect(agent.session.requestHeader()?.config.model).toBe('active')
    expect(agent.session.deriveMessages().at(-1)).toMatchObject({
      content: [{ type: 'text', text: 'same route recovered' }],
    })
  })
})
