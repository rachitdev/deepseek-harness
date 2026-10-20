/**
 * Live model-route failover over the Agent request-recovery extension point.
 * @module @deepseek-ai/dsh-decision-consumers/model-failover
 */

import type { Context, Events } from '@deepseek-ai/cordis'
import type { Agent, RequestErrorAction } from '@deepseek-ai/dsh-agent'
import {
  ReasoningEffortId,
  type LlmCallConfig,
  type TokenUsage,
} from '@deepseek-ai/dsh-llm'
import type {} from '@deepseek-ai/dsh-tool-subagent/model-selection-settings'
import z from '@deepseek-ai/schemastery'
import { arbitrateModelFailover } from './failover-arbiter.ts'
import type { FailoverCandidateProbe } from './types.ts'

/** Exact model route eligible to take over a failed Agent request. */
export interface FailoverModelRoute {
  /** Registered LLM provider id. */
  readonly provider: string
  /** Provider-owned model id. */
  readonly model: string
  /** Optional adapter-owned effort applied after takeover. */
  readonly reasoningEffort?: string
}

/** Live failover policy. Every pacing and selection value is composition-configurable. */
export interface ModelFailoverConfig {
  /** Whether request-error recovery may switch models. */
  readonly enabled?: boolean
  /** Configured fallback routes, used when no enabled allowed-model setting is available. */
  readonly candidates?: FailoverModelRoute[]
  /** Union the live `subagent-model-selection.allowedModels` list with `candidates` when that setting is enabled. */
  readonly useAllowedModels?: boolean
  /**
   * Failure codes that start probing. Unclassified provider errors (`PI_AI_ERROR`)
   * are included by default so a dead route is not retried in place.
   */
  readonly triggerCodes?: string[]
  /** Maximum healthy or failed routes probed concurrently, in source order. */
  readonly maxCandidates?: number
  /** Deadline for each candidate probe. */
  readonly probeTimeoutMs?: number
  /** Maximum output tokens for one probe. */
  readonly probeMaxTokens?: number
  /** Maximum user-task characters sent to the decision provider. */
  readonly maxTaskChars?: number
  /** Maximum probe-output characters retained for arbitration. */
  readonly maxProbeOutputChars?: number
}

const DEFAULT_TRIGGER_CODES = Object.freeze([
  'QUOTA',
  'RATE_LIMIT',
  'SERVER',
  'TIMEOUT',
  'TRANSPORT',
  'AUTH',
  'PI_AI_ERROR',
])

/**
 * Lowest `VALIDITY_RUBRIC` position a takeover route may hold. Reachability is already proven by
 * the probe returning text, so takeover is decided by the judge's graded quality score instead of
 * a second boolean question that re-judges evidence the harness observed directly; this floor only
 * rejects a winner the judge placed at "broken-or-hallucinated". The winning choice probability is
 * telemetry, never a gate: it expresses how strongly the judge separated routes that are all
 * healthy, and refusing takeover on a near-uniform ranking would leave a dead route in place.
 */
const MIN_TAKEOVER_VALIDITY = 1

/** Runtime schema for {@link ModelFailoverConfig}. */
export const ModelFailoverConfig: z<ModelFailoverConfig> = z.object({
  enabled: z.boolean().default(false),
  candidates: z.array(z.object({
    provider: z.string().min(1).required(),
    model: z.string().min(1).required(),
    reasoningEffort: z.string().min(1),
  })).default([]),
  useAllowedModels: z.boolean().default(true),
  triggerCodes: z.array(z.string().min(1)).default([...DEFAULT_TRIGGER_CODES]),
  maxCandidates: z.number().step(1).min(1).default(4),
  probeTimeoutMs: z.number().step(1).min(1).default(8_000),
  probeMaxTokens: z.number().step(1).min(1).default(96),
  maxTaskChars: z.number().step(1).min(1).default(2_000),
  maxProbeOutputChars: z.number().step(1).min(1).default(2_000),
})

interface ResolvedModelFailoverConfig {
  readonly candidates: readonly FailoverModelRoute[]
  readonly useAllowedModels: boolean
  readonly triggerCodes: ReadonlySet<string>
  readonly maxCandidates: number
  readonly probeTimeoutMs: number
  readonly probeMaxTokens: number
  readonly maxTaskChars: number
  readonly maxProbeOutputChars: number
}

interface AgentFailoverState {
  turn: number
  step: number
  readonly failedRoutes: Set<string>
  activeRoute?: FailoverModelRoute
  /** Recovery already settled for {@link recoverySignature}. */
  recovery?: Promise<RequestErrorAction>
  /** Sorted failed-route set the cached recovery was arbitrated against. */
  recoverySignature?: string
}

function resolveConfig(config: ModelFailoverConfig): ResolvedModelFailoverConfig {
  return {
    candidates: config.candidates ?? [],
    useAllowedModels: config.useAllowedModels ?? true,
    triggerCodes: new Set(config.triggerCodes ?? DEFAULT_TRIGGER_CODES),
    maxCandidates: config.maxCandidates ?? 4,
    probeTimeoutMs: config.probeTimeoutMs ?? 8_000,
    probeMaxTokens: config.probeMaxTokens ?? 96,
    maxTaskChars: config.maxTaskChars ?? 2_000,
    maxProbeOutputChars: config.maxProbeOutputChars ?? 2_000,
  }
}

function routeKey(route: Pick<FailoverModelRoute, 'provider' | 'model'>): string {
  return `${route.provider}\0${route.model}`
}

function deduplicateRoutes(routes: readonly FailoverModelRoute[]): FailoverModelRoute[] {
  const seen = new Set<string>()
  const unique: FailoverModelRoute[] = []
  for (const route of routes) {
    const key = routeKey(route)
    if (seen.has(key)) continue
    seen.add(key)
    unique.push({ ...route })
  }
  return unique
}

function availableRoutes(ctx: Context, config: ResolvedModelFailoverConfig): FailoverModelRoute[] {
  const configured = config.candidates
  if (config.useAllowedModels) {
    const setting = ctx.get('subagentModelSelection')?.current()
    if (setting?.enabled === true && setting.allowedModels.length > 0) {
      return deduplicateRoutes([...setting.allowedModels, ...configured])
    }
  }
  return deduplicateRoutes(configured)
}

function latestUserTask(agent: Agent, maxChars: number): string {
  const messages = agent.session.deriveMessages()
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index]
    if (message?.role !== 'user' || message.source.kind !== 'user') continue
    const text = message.content
      .filter(block => block.type === 'text')
      .map(block => block.text)
      .join('\n')
      .trim()
    if (text.length > 0) return text.slice(0, maxChars)
  }
  return 'Continue the current coding-agent task safely.'
}

function tokenCount(usage: TokenUsage | undefined): number {
  if (usage === undefined) return 0
  return usage.totalTokens
    ?? usage.inputTokens + usage.outputTokens
      + (usage.cacheReadTokens ?? 0) + (usage.cacheWriteTokens ?? 0)
}

async function probeCandidate(
  ctx: Context,
  route: FailoverModelRoute,
  task: string,
  config: ResolvedModelFailoverConfig,
  signal: AbortSignal,
): Promise<FailoverCandidateProbe> {
  const started = performance.now()
  const probeSignal = AbortSignal.any([
    signal,
    AbortSignal.timeout(config.probeTimeoutMs),
  ])
  let output = ''
  let usage: TokenUsage | undefined
  try {
    for await (const chunk of ctx.llm.stream({
      provider: route.provider,
      model: route.model,
      ...(route.reasoningEffort === undefined
        ? {}
        : { reasoningEffort: ReasoningEffortId(route.reasoningEffort) }),
      maxTokens: config.probeMaxTokens,
      // Request-only input: a bounded projection of the durable user/message
      // event, so every probe input remains reconstructable from the Session log.
      messages: [{ role: 'user', content: [{ type: 'text', text: task }] }],
      signal: probeSignal,
    })) {
      if (chunk.type === 'text-delta' && output.length < config.maxProbeOutputChars) {
        output += chunk.text.slice(0, config.maxProbeOutputChars - output.length)
      } else if (chunk.type === 'usage') {
        usage = chunk.usage
      } else if (chunk.type === 'finish'
        && (chunk.reason.kind === 'error' || chunk.reason.kind === 'aborted')) {
        return {
          provider: route.provider,
          model: route.model,
          latencyMs: Math.round(performance.now() - started),
          tokenCount: tokenCount(usage),
          probeOutput: output,
          error: `${chunk.reason.failure.code}: ${chunk.reason.failure.message}`,
        }
      }
    }
  } catch (error: unknown) {
    return {
      provider: route.provider,
      model: route.model,
      latencyMs: Math.round(performance.now() - started),
      tokenCount: tokenCount(usage),
      probeOutput: output,
      error: error instanceof Error ? error.message : String(error),
    }
  }
  const trimmed = output.trim()
  return {
    provider: route.provider,
    model: route.model,
    latencyMs: Math.round(performance.now() - started),
    tokenCount: tokenCount(usage),
    probeOutput: trimmed,
    ...(trimmed.length === 0 ? { error: 'candidate returned no text' } : {}),
  }
}

function firstHealthyRoute(
  probes: readonly FailoverCandidateProbe[],
  candidates: readonly FailoverModelRoute[],
): FailoverModelRoute | undefined {
  return candidates.find(route =>
    probes.some(probe =>
      probe.provider === route.provider
      && probe.model === route.model
      && probe.error === undefined
      && probe.probeOutput.length > 0))
}

function switchedConfig(config: LlmCallConfig, route: FailoverModelRoute): LlmCallConfig {
  const { reasoningEffort: _inheritedEffort, ...withoutInheritedEffort } = config
  return {
    ...withoutInheritedEffort,
    provider: route.provider,
    model: route.model,
    ...(route.reasoningEffort === undefined
      ? {}
      : { reasoningEffort: ReasoningEffortId(route.reasoningEffort) }),
  }
}

/**
 * Install parallel canary probing, JEV arbitration, and same-step route takeover.
 * @param ctx - Context carrying Agent, LLM, and decision services.
 * @param policy - Failover policy resolved from plugin configuration.
 */
export function installModelFailover(ctx: Context, policy: ModelFailoverConfig): void {
  if (policy.enabled !== true) return
  if (policy.candidates === undefined || policy.candidates.length === 0) {
    throw new Error('enabled model failover requires at least one fallback candidate route')
  }
  if (deduplicateRoutes(policy.candidates).length !== policy.candidates.length) {
    throw new Error('model failover candidate routes must be unique provider/model pairs')
  }
  const config = resolveConfig(policy)
  const lifetime = new AbortController()
  const states = new WeakMap<Agent, AgentFailoverState>()
  const active = new Set<Promise<RequestErrorAction>>()

  function stateFor(agent: Agent, turn: number, step: number): AgentFailoverState {
    const current = states.get(agent)
    if (current !== undefined && current.turn === turn && current.step === step) return current
    const next = { turn, step, failedRoutes: new Set<string>() }
    states.set(agent, next)
    return next
  }

  const disposeRequest = ctx.on('agent/request', async ({ agent }, next) => {
    const proposed = await next()
    const route = states.get(agent)?.activeRoute
    return route === undefined ? proposed : switchedConfig(proposed, route)
  })

  async function recover(
    payload: Parameters<Events['agent/request-error']>[0],
    next: () => Promise<RequestErrorAction>,
  ): Promise<RequestErrorAction> {
    const { agent, turn, step, provider, failure, signal } = payload
    if (!config.triggerCodes.has(failure.code) || signal.aborted || lifetime.signal.aborted) {
      return next()
    }
    const header = agent.session.requestHeader()
    if (header === undefined || header.config.provider !== provider) {
      ctx.logger.warn(
        `model failover: skipped ${failure.code} on ${provider} `
        + `(header=${header === undefined ? 'missing' : `${header.config.provider}/${header.config.model}`})`,
      )
      return next()
    }

    const state = stateFor(agent, turn, step)
    state.failedRoutes.add(routeKey({ provider, model: header.config.model }))
    // One probe and decision round covers every repeat failure of the same route set, so generic
    // same-route retries do not each pay for a fresh canary fan-out. A newly failed takeover route
    // changes the set and earns its own arbitration.
    const signature = [...state.failedRoutes].sort().join('|')
    if (state.recovery !== undefined && state.recoverySignature === signature) return state.recovery

    const recovery = (async (): Promise<RequestErrorAction> => {
      const fusedSignal = AbortSignal.any([signal, lifetime.signal])
      // `AbortSignal.aborted` is readonly, so TypeScript carries the `false` narrowing from one
      // check across the awaits below; reading it through a call observes a mid-flight abort.
      const cancelled = (): boolean => fusedSignal.aborted
      const candidates = availableRoutes(ctx, config)
        .filter(route => !state.failedRoutes.has(routeKey(route)))
        .slice(0, config.maxCandidates)
      if (candidates.length === 0) return next()

      const task = latestUserTask(agent, config.maxTaskChars)
      const probes = await Promise.all(candidates.map(route =>
        probeCandidate(ctx, route, task, config, fusedSignal)))
      if (cancelled()) return

      let decision
      try {
        decision = await arbitrateModelFailover(ctx.decision, {
          failedModel: header.config.model,
          failureReason: `${failure.code}: ${failure.message}`,
          currentTask: task,
          candidateProbes: probes,
        }, fusedSignal)
      } catch (error: unknown) {
        // JEV/Laya outage must not pin the turn to a cooling or 429 route.
        // A live canary already proved a replacement; take the first healthy
        // probe in candidate order instead of same-route backoff.
        const fallback = firstHealthyRoute(probes, candidates)
        if (fallback === undefined) {
          ctx.logger.warn('model failover arbitration failed; preserving same-route recovery: %o', error)
          return next()
        }
        state.activeRoute = fallback
        ctx.logger.warn(
          `model failover: ${provider}/${header.config.model} -> ${fallback.provider}/${fallback.model} `
          + '(judge-unavailable, first-healthy-probe; %o)',
          error,
        )
        return { kind: 'retry' }
      }
      if (cancelled()) return
      const selected = candidates.find(route =>
        route.provider === decision.recommendedProvider && route.model === decision.recommendedModel)
      if (selected === undefined || decision.codeValidityScore < MIN_TAKEOVER_VALIDITY) {
        return next()
      }

      state.activeRoute = selected
      ctx.logger.info(
        `model failover: ${provider}/${header.config.model} -> ${selected.provider}/${selected.model} `
        + `(judge=${decision.providerId}, confidence=${decision.confidence.toFixed(3)})`,
      )
      return { kind: 'retry' }
    })()
    state.recovery = recovery
    state.recoverySignature = signature
    active.add(recovery)
    try {
      return await recovery
    } finally {
      active.delete(recovery)
    }
  }

  const disposeError = ctx.on('agent/request-error', (payload, next) => recover(payload, next))
  const disposeSession = ctx.on('session/event', (session, event) => {
    if (event.type !== 'turn/end'
      && (event.type !== 'assistant/message' || event.data.interrupted === true)) return
    const agent = ctx.get('agents')?.get(session.id)
    if (agent?.session === session) states.delete(agent)
  })
  const disposeAgent = ctx.on('agent/disposed', ({ agent }) => { states.delete(agent) })

  ctx.effect(() => async () => {
    disposeRequest()
    disposeError()
    disposeSession()
    disposeAgent()
    lifetime.abort(new Error('model failover plugin disposed'))
    await Promise.allSettled([...active])
  }, 'decision-consumers: abort and drain model failover')
}
