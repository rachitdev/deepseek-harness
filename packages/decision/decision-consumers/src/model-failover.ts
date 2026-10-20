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
  /** Ranked primary routes probed together during start racing and failure recovery. */
  readonly candidates?: FailoverModelRoute[]
  /**
   * Lower-tier routes probed only if every eligible primary candidate fails. JEV can select among
   * these survivors, but a healthy primary can never lose to a floor route.
   */
  readonly fallbackCandidates?: FailoverModelRoute[]
  /** Union the live `subagent-model-selection.allowedModels` list with `candidates` when that setting is enabled. */
  readonly useAllowedModels?: boolean
  /**
   * Probe every candidate route in parallel before the very first LLM call of a new agent and
   * adopt whichever healthy responder JEV ranks best, instead of always starting on `candidates[0]`.
   * The winner stays active for the rest of the session (cleared only by a fresh in-turn failure,
   * which still recovers through the existing per-turn `recover` path below). Off by default: it
   * adds one full probe-and-arbitrate round trip to the first message of every session.
   */
  readonly raceOnStart?: boolean
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
  // Catalog miss. Not a malformed body; the route cannot be served.
  'MODEL_UNAVAILABLE',
  // Configured/resolved route cannot be dispatched by the provider or adapter.
  'UNKNOWN_MODEL',
  'MODEL_UNSUPPORTED',
  // Reasoning effort option rejected before streaming.
  'UNSUPPORTED_REASONING_EFFORT',
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
  fallbackCandidates: z.array(z.object({
    provider: z.string().min(1).required(),
    model: z.string().min(1).required(),
    reasoningEffort: z.string().min(1),
  })).default([]),
  useAllowedModels: z.boolean().default(true),
  raceOnStart: z.boolean().default(false),
  triggerCodes: z.array(z.string().min(1)).default([...DEFAULT_TRIGGER_CODES]),
  maxCandidates: z.number().step(1).min(1).default(4),
  probeTimeoutMs: z.number().step(1).min(1).default(8_000),
  probeMaxTokens: z.number().step(1).min(1).default(96),
  maxTaskChars: z.number().step(1).min(1).default(2_000),
  maxProbeOutputChars: z.number().step(1).min(1).default(2_000),
})

interface ResolvedModelFailoverConfig {
  readonly candidates: readonly FailoverModelRoute[]
  readonly fallbackCandidates: readonly FailoverModelRoute[]
  readonly useAllowedModels: boolean
  readonly raceOnStart: boolean
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
    fallbackCandidates: config.fallbackCandidates ?? [],
    useAllowedModels: config.useAllowedModels ?? true,
    raceOnStart: config.raceOnStart ?? false,
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

function availablePrimaryRoutes(ctx: Context, config: ResolvedModelFailoverConfig): FailoverModelRoute[] {
  const configured = config.candidates
  if (config.useAllowedModels) {
    const setting = ctx.get('subagentModelSelection')?.current()
    if (setting?.enabled === true && setting.allowedModels.length > 0) {
      return deduplicateRoutes([...setting.allowedModels, ...configured])
    }
  }
  return deduplicateRoutes(configured)
}

function availableFallbackRoutes(config: ResolvedModelFailoverConfig): FailoverModelRoute[] {
  const primaryKeys = new Set(config.candidates.map(routeKey))
  return deduplicateRoutes(config.fallbackCandidates).filter(route => !primaryKeys.has(routeKey(route)))
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

/** Health evidence for a route that streamed reasoning or tool calls but no text within the probe budget. */
const REASONING_ONLY_PROBE_OUTPUT = '[route is live: streamed reasoning or tool calls; no text within the probe budget]'

function liveWithoutText(
  route: FailoverModelRoute,
  started: number,
  usage: TokenUsage | undefined,
): FailoverCandidateProbe {
  return {
    provider: route.provider,
    model: route.model,
    latencyMs: Math.round(performance.now() - started),
    tokenCount: tokenCount(usage),
    probeOutput: REASONING_ONLY_PROBE_OUTPUT,
  }
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
  // A reasoning model spends a tiny probe budget thinking, and some gateways then
  // close that capped stream without a terminal marker. Reasoning or tool-call
  // deltas still prove the route is serving this task, so they count as liveness.
  let streamed = false
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
      } else if (chunk.type === 'reasoning-delta' || chunk.type === 'tool-call-delta') {
        streamed = true
      } else if (chunk.type === 'usage') {
        usage = chunk.usage
      } else if (chunk.type === 'finish'
        && (chunk.reason.kind === 'error' || chunk.reason.kind === 'aborted')) {
        if (output.length === 0 && streamed && chunk.reason.kind === 'error') {
          return liveWithoutText(route, started, usage)
        }
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
    // Only the probe's own deadline may count as liveness; a caller abort never does.
    if (output.length === 0 && streamed && !signal.aborted) {
      return liveWithoutText(route, started, usage)
    }
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
  if (trimmed.length === 0 && streamed) return liveWithoutText(route, started, usage)
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
      && isHealthyProbe(probe)))
}

/** A probe proves a route healthy when it finished without error and produced evidence to judge. */
function isHealthyProbe(probe: FailoverCandidateProbe): boolean {
  return probe.error === undefined && probe.probeOutput.length > 0
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
  const allConfigured = [...policy.candidates, ...(policy.fallbackCandidates ?? [])]
  if (deduplicateRoutes(allConfigured).length !== allConfigured.length) {
    throw new Error('model failover primary and fallback routes must be unique provider/model pairs')
  }
  const config = resolveConfig(policy)
  const lifetime = new AbortController()
  const states = new WeakMap<Agent, AgentFailoverState>()
  /** Route selected by the session-start race or a later failover; sticky until agent disposal. */
  const sessionRoutes = new WeakMap<Agent, FailoverModelRoute>()
  /** Coalesce concurrent first requests so one agent pays for only one start race. */
  const startRaces = new WeakMap<Agent, Promise<FailoverModelRoute | undefined>>()
  const active = new Set<Promise<RequestErrorAction>>()

  function stateFor(agent: Agent, turn: number, step: number): AgentFailoverState {
    const current = states.get(agent)
    if (current !== undefined && current.turn === turn && current.step === step) return current
    const next = { turn, step, failedRoutes: new Set<string>() }
    states.set(agent, next)
    return next
  }

  /**
   * Probe `routes` in ranked windows of `maxCandidates` and stop at the first window holding a
   * healthy route. A tier longer than one window (for example a large allowedModels union) is
   * walked in full, so a single cooling provider family at the head cannot hide every live
   * route behind it. Probes of every walked window are returned for diagnostics.
   */
  async function probeTier(
    routes: readonly FailoverModelRoute[],
    task: string,
    signal: AbortSignal,
  ): Promise<{ candidates: FailoverModelRoute[]; probes: FailoverCandidateProbe[] }> {
    const candidates: FailoverModelRoute[] = []
    const probes: FailoverCandidateProbe[] = []
    for (let offset = 0; offset < routes.length && !signal.aborted; offset += config.maxCandidates) {
      const window = routes.slice(offset, offset + config.maxCandidates)
      const results = await Promise.all(
        window.map(route => probeCandidate(ctx, route, task, config, signal)),
      )
      candidates.push(...window)
      probes.push(...results)
      if (results.some(isHealthyProbe)) break
    }
    return { candidates, probes }
  }

  async function probePrimaryThenFallback(
    primary: readonly FailoverModelRoute[],
    fallback: readonly FailoverModelRoute[],
    task: string,
    signal: AbortSignal,
  ): Promise<{ candidates: FailoverModelRoute[]; probes: FailoverCandidateProbe[] }> {
    const first = await probeTier(primary, task, signal)
    if (first.probes.some(isHealthyProbe) || signal.aborted) return first
    const floor = await probeTier(fallback, task, signal)
    // Keep primary probe failures alongside the floor for arbitration diagnostics.
    return {
      candidates: [...first.candidates, ...floor.candidates],
      probes: [...first.probes, ...floor.probes],
    }
  }

  async function raceCandidatesAtStart(
    agent: Agent,
    signal: AbortSignal,
  ): Promise<FailoverModelRoute | undefined> {
    const fusedSignal = AbortSignal.any([signal, lifetime.signal])
    const task = latestUserTask(agent, config.maxTaskChars)
    const { candidates, probes } = await probePrimaryThenFallback(
      availablePrimaryRoutes(ctx, config),
      availableFallbackRoutes(config),
      task,
      fusedSignal,
    )
    if (fusedSignal.aborted) return undefined

    const healthy = probes.filter(isHealthyProbe)
    if (healthy.length === 0) return undefined
    if (healthy.length === 1) {
      const [single] = healthy
      if (single === undefined) return undefined
      return candidates.find(c => c.provider === single.provider && c.model === single.model)
    }

    try {
      const decision = await arbitrateModelFailover(ctx.decision, {
        failedModel: 'session-start-race',
        failureReason: 'SESSION_START_PARALLEL_RACE',
        currentTask: task,
        candidateProbes: probes,
      }, fusedSignal)
      const winner = candidates.find(
        r => r.provider === decision.recommendedProvider && r.model === decision.recommendedModel,
      )
      if (winner !== undefined && decision.codeValidityScore >= MIN_TAKEOVER_VALIDITY) {
        ctx.logger.info(
          `model race on start: winner ${winner.provider}/${winner.model} `
          + `(judge=${decision.providerId}, confidence=${decision.confidence.toFixed(3)})`,
        )
        return winner
      }
    } catch (err: unknown) {
      ctx.logger.warn('model race on start: arbitration error; using first healthy probe: %o', err)
    }
    const fallback = firstHealthyRoute(probes, candidates)
    if (fallback !== undefined) {
      ctx.logger.info(`model race on start: winner ${fallback.provider}/${fallback.model} (first healthy probe)`)
    }
    return fallback
  }

  const configuredRoutes = new WeakMap<Agent, Pick<FailoverModelRoute, 'provider' | 'model'>>()
  const disposeRequest = ctx.on('agent/request', async ({ agent, turn, step, signal }, next) => {
    const state = states.get(agent)
    const inTurnRoute = state !== undefined && state.turn === turn && state.step === step
      ? state.activeRoute
      : undefined
    let route = inTurnRoute ?? sessionRoutes.get(agent)
    if (route === undefined && config.raceOnStart && !signal.aborted && !lifetime.signal.aborted) {
      let pendingRace = startRaces.get(agent)
      if (pendingRace === undefined) {
        pendingRace = raceCandidatesAtStart(agent, signal)
        startRaces.set(agent, pendingRace)
      }
      try {
        const winner = await pendingRace
        if (winner !== undefined) {
          sessionRoutes.set(agent, winner)
          route = winner
        } else if (startRaces.get(agent) === pendingRace) {
          // No candidate survived (or the request that started the race was
          // itself aborted, e.g. an interrupted first turn). Drop the cached
          // race so the next request gets its own fresh attempt instead of
          // silently running unraced for the rest of the session.
          startRaces.delete(agent)
        }
      } catch (err: unknown) {
        if (startRaces.get(agent) === pendingRace) startRaces.delete(agent)
        ctx.logger.warn('model race on start failed: %o', err)
      }
    }
    const proposed = await next()
    if (!configuredRoutes.has(agent) && proposed.provider !== undefined && proposed.model !== undefined) {
      configuredRoutes.set(agent, { provider: proposed.provider, model: proposed.model })
    }
    const configured = configuredRoutes.get(agent)
    const persisted = agent.session.requestHeader()?.config
    // The loop seed is either the original route or the last logged header.
    // Only a proposal that matches neither is an explicit listener switch.
    const explicitSwitch = configured !== undefined
      && (proposed.provider !== configured.provider || proposed.model !== configured.model)
      && (persisted === undefined
        || proposed.provider !== persisted.provider
        || proposed.model !== persisted.model)
    if (explicitSwitch) {
      const adopted = { provider: proposed.provider, model: proposed.model }
      if (sessionRoutes.has(agent) || inTurnRoute !== undefined) sessionRoutes.set(agent, adopted)
      if (state?.activeRoute !== undefined && state.turn === turn && state.step === step) delete state.activeRoute
      return proposed
    }
    if (route !== undefined) return switchedConfig(proposed, route)
    if (configured !== undefined
      && (proposed.provider !== configured.provider || proposed.model !== configured.model)) {
      return switchedConfig(proposed, configured)
    }
    return proposed
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
    const failedModel = payload.model ?? header?.config.model
    // A prepare-time failure has a payload model but no request header yet.
    if (failedModel === undefined || (header !== undefined && header.config.provider !== provider)) {
      ctx.logger.warn(
        `model failover: skipped ${failure.code} on ${provider} `
        + `(header=${header === undefined ? 'missing' : `${header.config.provider}/${header.config.model}`}, model=${failedModel ?? 'missing'})`,
      )
      return next()
    }

    const state = stateFor(agent, turn, step)
    state.failedRoutes.add(routeKey({ provider, model: failedModel }))
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
      const primary = availablePrimaryRoutes(ctx, config)
        .filter(route => !state.failedRoutes.has(routeKey(route)))
      const task = latestUserTask(agent, config.maxTaskChars)
      const fallback = availableFallbackRoutes(config)
        .filter(route => !state.failedRoutes.has(routeKey(route)))
      const { candidates, probes } = await probePrimaryThenFallback(primary, fallback, task, fusedSignal)
      if (cancelled()) return
      if (candidates.length === 0) return next()

      let decision
      try {
        decision = await arbitrateModelFailover(ctx.decision, {
          failedModel,
          failureReason: `${failure.code}: ${failure.message}`,
          currentTask: task,
          candidateProbes: probes,
        }, fusedSignal)
      } catch (error: unknown) {
        // Judge outage must not pin the turn to a cooling or 429 route.
        // A live canary already proved a replacement; take the first healthy
        // probe in candidate order instead of same-route backoff.
        const fallback = firstHealthyRoute(probes, candidates)
        if (fallback === undefined) {
          ctx.logger.warn('model failover arbitration failed; preserving same-route recovery: %o', error)
          return next()
        }
        state.activeRoute = fallback
        ctx.logger.warn(
          `model failover: ${provider}/${failedModel} -> ${fallback.provider}/${fallback.model} `
          + '(judge-unavailable, first-healthy-probe; %o)',
          error,
        )
        return { kind: 'retry' }
      }
      if (cancelled()) return
      const selected = candidates.find(route =>
        route.provider === decision.recommendedProvider && route.model === decision.recommendedModel)
      if (selected === undefined || decision.codeValidityScore < MIN_TAKEOVER_VALIDITY) {
        // A judge that names no candidate must not close the turn while a probe
        // already returned text. An explicit broken-code verdict still refuses.
        const fallback = selected === undefined ? firstHealthyRoute(probes, candidates) : undefined
        if (fallback === undefined) return next()
        state.activeRoute = fallback
        ctx.logger.warn(
          `model failover: ${provider}/${failedModel} -> ${fallback.provider}/${fallback.model} `
          + '(judge named no candidate, first-healthy-probe)',
        )
        return { kind: 'retry' }
      }

      state.activeRoute = selected
      ctx.logger.info(
        `model failover: ${provider}/${failedModel} -> ${selected.provider}/${selected.model} `
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
  const disposeAgent = ctx.on('agent/disposed', ({ agent }) => {
    states.delete(agent)
    sessionRoutes.delete(agent)
    configuredRoutes.delete(agent)
    startRaces.delete(agent)
  })

  ctx.effect(() => async () => {
    disposeRequest()
    disposeError()
    disposeSession()
    disposeAgent()
    lifetime.abort(new Error('model failover plugin disposed'))
    await Promise.allSettled([...active])
  }, 'decision-consumers: abort and drain model failover')
}
