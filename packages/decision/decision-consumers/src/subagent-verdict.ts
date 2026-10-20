/**
 * `SubagentVerdict`: evaluates a delegated subagent's execution against its original task brief
 * and acceptance criteria using `ctx.decision`. Replaces unstructured LLM evaluation with typed,
 * calibrated verdicts: accept, retry with targeted feedback, or escalate.
 * @module @deepseek-ai/dsh-decision-consumers/subagent-verdict
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { SubagentEvaluation } from './types.ts'

/** Options configuring subagent evaluation thresholds. */
export interface SubagentVerdictOptions {
  /** Quality score (0..3) required for automatic acceptance; defaults to 2.0. */
  readonly minQualityScore?: number
  /** Minimum criteria-met probability for acceptance; defaults to 0.80. */
  readonly criteriaMetThreshold?: number
}

/**
 * Evaluate a subagent's returned output against its delegation brief.
 * @param decisionService - the active decision seam service.
 * @param task - brief, acceptance criteria, and subagent output.
 * @param options - threshold options.
 * @param signal - optional cancellation signal.
 * @returns the structured subagent verdict with quality score and confidence.
 */
export async function evaluateSubagent(
  decisionService: DecisionService,
  task: {
    readonly brief: string
    readonly acceptanceCriteria: readonly string[]
    readonly subagentOutput: string
  },
  options: SubagentVerdictOptions = {},
  signal?: AbortSignal,
): Promise<SubagentEvaluation> {
  const minQuality = options.minQualityScore ?? 2.0
  const criteriaThreshold = options.criteriaMetThreshold ?? 0.8

  const state = [
    `Task Brief: ${task.brief}`,
    'Acceptance Criteria:',
    ...task.acceptanceCriteria.map((c, i) => `${String(i + 1)}. ${c}`),
    'Subagent Output:',
    task.subagentOutput.slice(0, 2000),
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        verdict: {
          type: 'choice',
          instructions: 'What verdict should be issued for this subagent execution?',
          criteria: {
            accept: 'The output completely satisfies the brief and all acceptance criteria',
            retry: 'Minor omissions or fixable errors; a second pass with feedback can resolve them',
            escalate: 'Fundamental failure, incorrect approach, or subagent lacks capability',
          },
        },
        quality: {
          type: 'score',
          instructions: 'How high is the technical quality and completeness of the solution?',
          criteria: [
            'unusable-hallucination-or-crash',
            'partially-implemented-missing-key-pieces',
            'acceptable-solid-implementation',
            'exceptional-exceeds-brief-with-clean-evidence',
          ],
        },
        criteria_met: {
          type: 'boolean',
          instructions: 'Are all explicit acceptance criteria demonstrably satisfied by the output?',
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const verdAns = result.answers['verdict']
  const qualAns = result.answers['quality']
  const critAns = result.answers['criteria_met']

  if (verdAns?.type !== 'choice' || qualAns?.type !== 'score' || critAns?.type !== 'boolean') {
    throw new Error('Decision provider returned unexpected answer types for subagent evaluation')
  }

  const qualityScore = qualAns.score
  const criteriaMet = critAns.probability >= criteriaThreshold
  const confidence = verdAns.probabilities[verdAns.choice] ?? 0

  let verdict: SubagentEvaluation['verdict']
  if (criteriaMet && qualityScore >= minQuality && verdAns.choice === 'accept') {
    verdict = 'accept'
  } else if (qualityScore < 1.0 || verdAns.choice === 'escalate') {
    verdict = 'escalate'
  } else {
    verdict = 'retry'
  }

  let feedbackRecommendation: string | undefined
  if (verdict === 'retry') {
    feedbackRecommendation = `Subagent scored quality ${qualityScore.toFixed(1)}/3.0 (criteria met probability ${critAns.probability.toFixed(2)}). Request focused correction on unsatisfied criteria.`
  }

  return {
    verdict,
    confidence,
    qualityScore,
    criteriaMet,
    ...(feedbackRecommendation !== undefined ? { feedbackRecommendation } : {}),
    latencyMs,
    providerId: result.providerId,
  }
}
