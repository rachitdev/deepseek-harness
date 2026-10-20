/**
 * `EgoStepDriver`: drives browser navigation steps via `ctx.decision`. Evaluates interactive
 * elements as a Choice question alongside ride-along Boolean questions (page blocked, task
 * complete) and a risk score, then gates action on calibrated probability before executing or
 * escalating.
 * @module @deepseek-ai/dsh-decision-consumers/ego-driver
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { EgoCandidateElement, EgoStepDecision } from './types.ts'

/** Default confidence threshold below which the driver escalates to an LLM turn or human. */
export const DEFAULT_EGO_CONFIDENCE_THRESHOLD = 0.7

/** Options configuring step decision policy. */
export interface EgoStepDriverOptions {
  /** Minimum winning choice probability to click automatically; defaults to 0.70. */
  readonly confidenceThreshold?: number
  /** Probability above which a page is treated as blocked by a wall/captcha; defaults to 0.60. */
  readonly blockedThreshold?: number
  /** Probability above which the task is treated as already complete; defaults to 0.80. */
  readonly completionThreshold?: number
}

/**
 * Decide the next action for an ego-browser navigation step.
 * @param decisionService - the active decision seam service.
 * @param params - step parameters: task goal, page description, and candidates.
 * @param options - gating thresholds.
 * @param signal - optional cancellation signal.
 * @returns the structured step decision with confidence metrics.
 */
export async function decideEgoStep(
  decisionService: DecisionService,
  params: {
    readonly taskGoal: string
    readonly pageState: string
    readonly candidates: readonly EgoCandidateElement[]
  },
  options: EgoStepDriverOptions = {},
  signal?: AbortSignal,
): Promise<EgoStepDecision> {
  const confidenceThreshold = options.confidenceThreshold ?? DEFAULT_EGO_CONFIDENCE_THRESHOLD
  const blockedThreshold = options.blockedThreshold ?? 0.6
  const completionThreshold = options.completionThreshold ?? 0.8

  if (params.candidates.length < 2) {
    throw new Error('EgoStepDriver requires at least two candidate elements to decide between')
  }

  const criteria: Record<string, string> = {}
  for (const candidate of params.candidates) {
    criteria[candidate.id] = candidate.description !== undefined && candidate.description.length > 0
      ? `${candidate.label} (${candidate.description})`
      : candidate.label
  }

  const state = [
    `Task Goal: ${params.taskGoal}`,
    `Page State: ${params.pageState}`,
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        element: {
          type: 'choice',
          instructions: 'Which element should be clicked next to advance the task goal?',
          criteria,
        },
        task_complete: {
          type: 'boolean',
          instructions: 'Is the task goal already completely achieved on the current page?',
        },
        page_blocked: {
          type: 'boolean',
          instructions: 'Is the page blocked by an error, captcha, login barrier, or access denial?',
        },
        risk_level: {
          type: 'score',
          instructions: 'How risky or destructive is advancing on this page?',
          criteria: ['safe-navigation', 'form-mutation', 'destructive-or-financial'],
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const elemAns = result.answers['element']
  const doneAns = result.answers['task_complete']
  const blockAns = result.answers['page_blocked']
  const riskAns = result.answers['risk_level']

  if (elemAns?.type !== 'choice' || doneAns?.type !== 'boolean' || blockAns?.type !== 'boolean' || riskAns?.type !== 'score') {
    throw new Error('Decision provider returned unexpected answer types for ego step')
  }

  const taskComplete = doneAns.probability >= completionThreshold
  const pageBlocked = blockAns.probability >= blockedThreshold
  const confidence = elemAns.probabilities[elemAns.choice] ?? 0

  let action: EgoStepDecision['action']
  if (pageBlocked) {
    action = { type: 'escalate', reason: `Page appears blocked (confidence ${String(blockAns.probability)})` }
  } else if (taskComplete) {
    action = { type: 'complete' }
  } else if (confidence >= confidenceThreshold) {
    action = { type: 'click', elementId: elemAns.choice }
  } else {
    action = {
      type: 'escalate',
      reason: `Low decision confidence: pick "${elemAns.choice}" at ${String(confidence)} below threshold ${String(confidenceThreshold)}`,
    }
  }

  return {
    action,
    confidence,
    probabilities: elemAns.probabilities,
    taskComplete,
    pageBlocked,
    riskLevel: riskAns.score,
    latencyMs,
    providerId: result.providerId,
  }
}
