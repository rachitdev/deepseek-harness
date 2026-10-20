/**
 * Live turn triage: before generation, one decision call sorts each user turn into a configured
 * model tier (for example fast, standard, frontier). In `advisory` mode the verdict is only logged;
 * in `enforce` mode the turn's model requests switch to the tier's route through `agent/request`,
 * where the resulting provider and model are recorded in the session's request header.
 * @module @deepseek-ai/dsh-decision-consumers/turn-triage
 */

import type { Context } from '@deepseek-ai/cordis'
import type { Agent, PreStepDecision } from '@deepseek-ai/dsh-agent'
import type { DecisionService } from '@deepseek-ai/dsh-decision'
import { ReasoningEffortId, type LlmCallConfig } from '@deepseek-ai/dsh-llm'
import z from '@deepseek-ai/schemastery'
import { previousReplyText, requestText } from './turn-text.ts'

/** One triage tier and the model route its turns use. */
export interface TurnTriageTier {
  /** Tier id offered to the decision provider as a choice option. */
  name: string
  /** Kinds of work that belong in this tier; the decision provider judges against it. */
  description: string
  /** Registered LLM provider id. */
  provider: string
  /** Provider-owned model id. */
  model: string
  /** Optional adapter-owned reasoning effort for this tier. */
  reasoningEffort?: string
}

/** Live turn-triage policy. */
export interface TurnTriageConfig {
  /** Whether turns are triaged. */
  enabled?: boolean
  /** `advisory` logs the verdict only; `enforce` switches the turn's model requests to the tier route. */
  mode?: 'advisory' | 'enforce'
  /** Ordered tiers; at least two, with unique names. */
  tiers?: TurnTriageTier[]
  /** Tier used when the winning tier's probability is below `minConfidence`; must name a tier. */
  fallbackTier?: string
  /** Winning-tier probability below which `fallbackTier` is used instead. */
  minConfidence?: number
  /** Whether delegated (subagent) agents are triaged too. */
  includeSubagents?: boolean
  /** Maximum user-request characters sent to the decision provider. */
  maxQueryChars?: number
  /** Maximum characters of the previous assistant reply sent as context; 0 omits it. */
  maxContextChars?: number
  /** Deadline for the decision call; on expiry the turn keeps its configured model. */
  timeoutMs?: number
}

const TurnTriageTier: z<TurnTriageTier> = z.object({
  name: z.string().min(1).required(),
  description: z.string().min(1).required(),
  provider: z.string().min(1).required(),
  model: z.string().min(1).required(),
  reasoningEffort: z.string().min(1),
})

/** Runtime schema for {@link TurnTriageConfig}. */
export const TurnTriageConfig: z<TurnTriageConfig> = z.object({
  enabled: z.boolean().default(false),
  mode: z.union(['advisory', 'enforce'] as const).default('advisory'),
  tiers: z.array(TurnTriageTier).default([]),
  fallbackTier: z.string().min(1),
  minConfidence: z.number().min(0).max(1).default(0.5),
  includeSubagents: z.boolean().default(false),
  maxQueryChars: z.number().step(1).min(1).default(2_000),
  maxContextChars: z.number().step(1).min(0).default(800),
  timeoutMs: z.number().step(1).min(1).default(3_000),
})

/** The triage verdict for one turn. */
export interface TurnTriage {
  /** Tier the turn is assigned to after the confidence fallback. */
  readonly tier: TurnTriageTier
  /** Tier the decision provider chose, before the confidence fallback. */
  readonly chosen: string
  /** Probability of the chosen tier. */
  readonly confidence: number
  /** Decision provider that answered. */
  readonly providerId: string
}

/** Request text the triage judges. */
export interface TurnTriageQuery {
  /** Direct user text of the turn. */
  readonly request: string
  /** Tail of the previous assistant reply; empty when absent or disabled. */
  readonly previousReply: string
}

/** Validated tier policy {@link triageTurn} judges against. */
export interface TurnTriagePolicy {
  /** Candidate tiers offered as choice options. */
  readonly tiers: readonly TurnTriageTier[]
  /** Tier used when the winning probability is below `minConfidence`. */
  readonly fallback: TurnTriageTier
  /** Winning-tier probability below which `fallback` is used. */
  readonly minConfidence: number
}

/**
 * Sort one user turn into a tier with a single decision call.
 * @param decisionService - the active decision seam service.
 * @param query - the turn's request text and optional previous-reply context.
 * @param policy - tiers, fallback tier, and confidence floor.
 * @param signal - optional cancellation signal.
 * @returns the verdict, with low-confidence choices replaced by the fallback tier.
 */
export async function triageTurn(
  decisionService: DecisionService,
  query: TurnTriageQuery,
  policy: TurnTriagePolicy,
  signal?: AbortSignal,
): Promise<TurnTriage> {
  const result = await decisionService.evaluate({
    state: [
      `User request: ${query.request}`,
      ...query.previousReply.length > 0 ? [`Previous assistant reply (tail): ${query.previousReply}`] : [],
    ].join('\n'),
    questions: {
      tier: {
        type: 'choice',
        instructions: 'Which tier of work does fulfilling this user request require?',
        criteria: Object.fromEntries(policy.tiers.map(tier => [tier.name, tier.description])),
      },
    },
  }, signal)
  const answer = result.answers['tier']
  if (answer?.type !== 'choice') throw new Error('turn triage expected choice answer "tier"')
  const chosen = policy.tiers.find(tier => tier.name === answer.choice)
  if (chosen === undefined) throw new Error(`turn triage provider chose unknown tier "${answer.choice}"`)
  const confidence = answer.probabilities[answer.choice] ?? 0
  return {
    tier: confidence < policy.minConfidence ? policy.fallback : chosen,
    chosen: chosen.name,
    confidence,
    providerId: result.providerId,
  }
}

/**
 * Resolve and validate the triage policy; misconfiguration throws at plugin load.
 * @param raw - raw `turnTriage` configuration.
 * @returns the resolved configuration and tier policy.
 */
function resolveTurnTriage(raw: TurnTriageConfig): { config: Required<Omit<TurnTriageConfig, 'fallbackTier'>>; policy: TurnTriagePolicy } {
  const config = TurnTriageConfig(raw) as Required<TurnTriageConfig>
  if (config.tiers.length < 2) throw new Error('turn triage requires at least two tiers')
  const names = new Set<string>()
  for (const tier of config.tiers) {
    if (names.has(tier.name)) throw new Error(`turn triage tier "${tier.name}" is duplicated`)
    names.add(tier.name)
  }
  const fallback = config.tiers.find(tier => tier.name === config.fallbackTier)
  if (fallback === undefined) throw new Error(`turn triage fallbackTier "${config.fallbackTier}" names no configured tier`)
  return { config, policy: { tiers: config.tiers, fallback, minConfidence: config.minConfidence } }
}

function routedConfig(config: LlmCallConfig, tier: TurnTriageTier): LlmCallConfig {
  const { reasoningEffort: _inheritedEffort, ...withoutInheritedEffort } = config
  return {
    ...withoutInheritedEffort,
    provider: tier.provider,
    model: tier.model,
    ...tier.reasoningEffort === undefined ? {} : { reasoningEffort: ReasoningEffortId(tier.reasoningEffort) },
  }
}

/**
 * Install live turn triage: a pre-step listener starts one decision call per agent turn, and in
 * `enforce` mode an `agent/request` listener routes the turn's requests to the verdict's tier.
 * A failed, timed-out, or step-cancelled triage leaves the turn on its configured model.
 * @param ctx - Context carrying the decision service.
 * @param raw - triage policy from plugin configuration.
 */
export function installTurnTriage(ctx: Context, raw: TurnTriageConfig): void {
  const { config, policy } = resolveTurnTriage(raw)
  const pending = new WeakMap<Agent, { readonly turn: number; readonly verdict: Promise<TurnTriage | undefined> }>()

  const start = (agent: Agent, turn: number, decision: PreStepDecision, signal: AbortSignal): void => {
    if (decision.kind !== 'enter' || pending.get(agent)?.turn === turn) return
    if (!config.includeSubagents && (agent.session.header.delegationDepth ?? 0) > 0) return
    const request = requestText(decision.messages, config.maxQueryChars)
    if (request.length === 0) return
    const query = { request, previousReply: previousReplyText(agent, config.maxContextChars) }
    const started = performance.now()
    const deadline = AbortSignal.any([signal, AbortSignal.timeout(config.timeoutMs)])
    const verdict = triageTurn(ctx.decision, query, policy, deadline).then((triage) => {
      ctx.logger.info(
        'turn triage (%s): agent %s turn %d -> %s (chose %s at %s) route %s/%s in %dms via %s',
        config.mode, agent.id, turn, triage.tier.name, triage.chosen, triage.confidence.toFixed(2),
        triage.tier.provider, triage.tier.model, Math.round(performance.now() - started), triage.providerId,
      )
      return triage
    }, (error: unknown) => {
      ctx.logger.warn('turn triage: agent %s turn %d failed; keeping the configured model: %o', agent.id, turn, error)
      return undefined
    })
    pending.set(agent, { turn, verdict })
  }

  ctx.on('agent/pre-step', async ({ agent, turn, signal }, next) => {
    const decision = await next()
    start(agent, turn, decision, signal)
    return decision
  })
  if (config.mode === 'enforce') {
    ctx.on('agent/request', async ({ agent, turn }, next) => {
      const proposed = await next()
      const entry = pending.get(agent)
      const triage = entry?.turn === turn ? await entry.verdict : undefined
      return triage === undefined ? proposed : routedConfig(proposed, triage.tier)
    })
  }
  ctx.on('agent/disposed', ({ agent }) => {
    pending.delete(agent)
  })
}
