/**
 * `@deepseek-ai/dsh-decision-consumers`: suite of 16 specialized consumers built on `ctx.decision`:
 * 1. `decideEgoStep`: ego-browser navigation stepping with calibrated confidence gating
 * 2. `assessToolRisk`: pre-approval tool call risk scoring and permission gating
 * 3. `routeTurnModel`: turn-level complexity routing between fast and frontier model tiers
 * 4. `filterRetrievalCandidates`: relevance scoring and noise filtering for retrieved context
 * 5. `scoreCompactionItems`: transcript retention prioritization for context compaction
 * 6. `detectLoopStall`: probabilistic loop stall and repetition detection
 * 7. `evaluateSubagent`: delegated subagent execution evaluation and verdict
 * 8. `selectVerificationGates`: pre-push git diff triage and minimal gate selection
 * 9. live model failover: parallel LLM canaries, decision arbitration, and same-step route takeover
 * 10. `selectSkill`: per-turn skill catalog routing with confidence and risk gates
 * 11. `guardFetchedContent`: prompt-injection and credential-harvest screening for fetched web pages
 * 12. `assessInstructionDrift`: advisory judgement that work diverged from repository instructions
 * 13. `decideBrowserAction`: dynamic browser operation/target selection in one speculative request
 * 14. live recall filter: relevance and stored-injection screening of plugin-injected memory recall
 * 15. `cascadeSkillSelection`: two-stage category-then-skill selection for progressive disclosure
 * 16. live turn triage: per-turn model tier selection before generation, advisory or enforced
 * @module @deepseek-ai/dsh-decision-consumers
 */

import type { Context } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import { installModelFailover, ModelFailoverConfig } from './model-failover.ts'
import { installRecallFilter, RecallFilterConfig } from './recall-filter.ts'
import { installTurnTriage, TurnTriageConfig } from './turn-triage.ts'

export * from './types.ts'
export { decideEgoStep, DEFAULT_EGO_CONFIDENCE_THRESHOLD } from './ego-driver.ts'
export type { EgoStepDriverOptions } from './ego-driver.ts'
export { assessToolRisk } from './tool-risk-gate.ts'
export type { ToolRiskGateOptions } from './tool-risk-gate.ts'
export { routeTurnModel } from './model-router.ts'
export type { ModelRouterOptions } from './model-router.ts'
export { filterRetrievalCandidates } from './retrieval-filter.ts'
export type { RetrievalFilterOptions } from './retrieval-filter.ts'
export { scoreCompactionItems } from './compaction-scorer.ts'
export type { CompactionScorerOptions } from './compaction-scorer.ts'
export { detectLoopStall } from './loop-stall-detector.ts'
export type { LoopStallDetectorOptions } from './loop-stall-detector.ts'
export { evaluateSubagent } from './subagent-verdict.ts'
export type { SubagentVerdictOptions } from './subagent-verdict.ts'
export { selectVerificationGates, SUITE_COMMANDS } from './gate-selector.ts'
export { selectSkill } from './skill-selector.ts'
export type { SkillSelectorOptions } from './skill-selector.ts'
export { adjudicateSkillSelection } from './skill-adjudicator.ts'
export type {
  SkillAdjudication,
  SkillAdjudicationOptions,
  SkillAdjudicationRequest,
} from './skill-adjudicator.ts'
export { cascadeSkillSelection } from './skill-cascade.ts'
export { installTurnTriage, triageTurn, TurnTriageConfig } from './turn-triage.ts'
export type { TurnTriage, TurnTriagePolicy, TurnTriageQuery, TurnTriageTier } from './turn-triage.ts'
export type { SkillCascade, SkillCascadeCategory, SkillCascadeRequest } from './skill-cascade.ts'
export { guardFetchedContent } from './fetch-guardrail.ts'
export type { FetchGuardrailOptions } from './fetch-guardrail.ts'
export { assessInstructionDrift } from './instruction-drift.ts'
export type { InstructionDriftOptions } from './instruction-drift.ts'
export { decideBrowserAction } from './browser-action-policy.ts'
export type { BrowserActionPolicyOptions } from './browser-action-policy.ts'
export { arbitrateModelFailover } from './failover-arbiter.ts'
export { installModelFailover, ModelFailoverConfig } from './model-failover.ts'
export type { FailoverModelRoute } from './model-failover.ts'
export { screenRecallItems } from './recall-screen.ts'
export type { RecallScreenOptions, RecallScreenQuery } from './recall-screen.ts'
export { parseRecallBlock, renderRecallBlock } from './recall-block.ts'
export type { ParsedRecallBlock, RecallBlockItem, RecallItemRendering } from './recall-block.ts'
export {
  installRecallFilter,
  omittedItemsNote,
  RecallFilterConfig,
  WITHHELD_INJECTION_MARKER,
  WITHHELD_UNSCREENABLE_MARKER,
} from './recall-filter.ts'

/** Cordis plugin name. */
export const name = 'decision-consumers'

/**
 * Injects the decision and LLM seams. The Agent registry is read optionally: only the live
 * failover binding's turn-end cleanup needs it, so a composition without one still gets the
 * classification consumers instead of waiting for a service it never mounts.
 */
export const inject = ['decision', 'llm']

/** Configuration interface for consumer defaults and live bindings. */
export interface Config {
  /** Default confidence threshold for ego-browser driving. */
  readonly egoConfidenceThreshold?: number
  /** Optional active-model recovery policy. */
  readonly failover?: ModelFailoverConfig
  /** Optional screening of plugin-injected memory recall. */
  readonly recallFilter?: RecallFilterConfig
  /** Optional per-turn model tier triage before generation. */
  readonly turnTriage?: TurnTriageConfig
}

/** Runtime schema for decision-consumer configuration. */
export const Config: z<Config> = z.object({
  egoConfidenceThreshold: z.number().min(0).max(1).default(0.7),
  failover: ModelFailoverConfig,
  recallFilter: RecallFilterConfig,
  turnTriage: TurnTriageConfig,
})

/**
 * Register configured live consumers for this context.
 * @param ctx - Plugin context carrying decision, LLM, and Agent services.
 * @param config - Consumer defaults, optional live failover policy, recall filtering, and turn triage.
 */
export function apply(ctx: Context, config: Config = {}): void {
  if (config.failover !== undefined) installModelFailover(ctx, config.failover)
  if (config.recallFilter?.enabled === true) installRecallFilter(ctx, config.recallFilter)
  if (config.turnTriage?.enabled === true) installTurnTriage(ctx, config.turnTriage)
}
