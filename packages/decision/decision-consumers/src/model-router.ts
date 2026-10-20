/**
 * `ModelRouter`: selects model tier (fast/cheap vs frontier) per conversation turn using
 * `ctx.decision`. Bounded classification avoids spending premium reasoning turns on simple lookups
 * or straightforward file edits.
 * @module @deepseek-ai/dsh-decision-consumers/model-router
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { ModelRouteDecision } from './types.ts'

/** Configured models for router tiers. */
export interface ModelRouterOptions {
  /** Fast tier model id; defaults to 'deepseek-chat'. */
  readonly fastModel?: string
  /** Frontier tier model id; defaults to 'deepseek-reasoner'. */
  readonly frontierModel?: string
  /** Complexity score threshold (0..3) above which frontier tier is selected; defaults to 1.8. */
  readonly complexityThreshold?: number
}

/**
 * Route a conversation turn to the optimal model tier.
 * @param decisionService - the active decision seam service.
 * @param turn - user instruction and turn context.
 * @param options - tier configuration and thresholds.
 * @param signal - optional cancellation signal.
 * @returns the routing decision with calibrated complexity metrics.
 */
export async function routeTurnModel(
  decisionService: DecisionService,
  turn: {
    readonly userMessage: string
    readonly contextSummary?: string
  },
  options: ModelRouterOptions = {},
  signal?: AbortSignal,
): Promise<ModelRouteDecision> {
  const fastModel = options.fastModel ?? 'deepseek-chat'
  const frontierModel = options.frontierModel ?? 'deepseek-reasoner'
  const complexityThreshold = options.complexityThreshold ?? 1.8

  const state = [
    `User Instruction: ${turn.userMessage}`,
    `Recent Context Summary: ${turn.contextSummary ?? 'New turn / fresh session'}`,
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        tier: {
          type: 'choice',
          instructions: 'Which execution tier is most suitable for fulfilling this user instruction?',
          criteria: {
            fast: 'Routine file lookup, direct single-location code edit, factual explanation, or simple command',
            frontier: 'Architecture design, intricate debugging, multi-file refactor, or complex trade-off reasoning',
          },
        },
        complexity: {
          type: 'score',
          instructions: 'How complex is the required reasoning or problem solving?',
          criteria: [
            'trivial-lookup',
            'straightforward-mutation',
            'moderate-refactor',
            'deep-algorithmic-reasoning',
          ],
        },
        deep_reasoning: {
          type: 'boolean',
          instructions: 'Does this task require prolonged deliberate reasoning or complex chain of thought?',
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const tierAns = result.answers['tier']
  const compAns = result.answers['complexity']
  const deepAns = result.answers['deep_reasoning']

  if (tierAns?.type !== 'choice' || compAns?.type !== 'score' || deepAns?.type !== 'boolean') {
    throw new Error('Decision provider returned unexpected answer types for model routing')
  }

  const complexityScore = compAns.score
  const needsDeepReasoning = deepAns.probability >= 0.5
  const tierConfidence = tierAns.probabilities[tierAns.choice] ?? 0

  const selectedTier: 'fast' | 'frontier' =
    tierAns.choice === 'frontier' || complexityScore >= complexityThreshold || needsDeepReasoning
      ? 'frontier'
      : 'fast'

  return {
    selectedTier,
    recommendedModel: selectedTier === 'frontier' ? frontierModel : fastModel,
    confidence: tierConfidence,
    complexityScore,
    needsDeepReasoning,
    latencyMs,
    providerId: result.providerId,
  }
}
