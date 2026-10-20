/**
 * `LoopStallDetector`: analyzes recent agent actions and tool results using `ctx.decision` to
 * detect repetitive loops, oscillation, or thrashing without forward progress. Replaces crude
 * string-matching heuristics with probabilistic loop classification.
 * @module @deepseek-ai/dsh-decision-consumers/loop-stall-detector
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { LoopStallVerdict, RecentAgentAction } from './types.ts'

/** Options configuring stall detection sensitivity. */
export interface LoopStallDetectorOptions {
  /** Probability threshold (0..1) above which a stall warning is triggered; defaults to 0.65. */
  readonly stallWarningThreshold?: number
  /** Probability threshold (0..1) above which loop interruption is recommended; defaults to 0.85. */
  readonly stallInterruptThreshold?: number
}

/**
 * Assess whether recent agent actions indicate a stalled or thrashing loop.
 * @param decisionService - the active decision seam service.
 * @param actions - recent sequence of actions and results (typically last 3-8 turns).
 * @param taskGoal - the high-level task goal being pursued.
 * @param options - threshold options.
 * @param signal - optional cancellation signal.
 * @returns the structured loop stall verdict and recommended intervention.
 */
export async function detectLoopStall(
  decisionService: DecisionService,
  actions: readonly RecentAgentAction[],
  taskGoal: string,
  options: LoopStallDetectorOptions = {},
  signal?: AbortSignal,
): Promise<LoopStallVerdict> {
  const warnThreshold = options.stallWarningThreshold ?? 0.65
  const interruptThreshold = options.stallInterruptThreshold ?? 0.85

  if (actions.length < 2) {
    return {
      stalled: false,
      loopState: 'progressing',
      stallSeverity: 0,
      stallProbability: 0,
      suggestedIntervention: 'continue',
      latencyMs: 0,
      providerId: 'none',
    }
  }

  const state = [
    `Task Goal: ${taskGoal}`,
    'Recent Actions (chronological, oldest to newest):',
    ...actions.map((a, i) =>
      `Turn ${String(i + 1)}: tool=${a.tool} input=${a.inputSummary} result=${a.resultSummary}${a.error === true ? ' [FAILED]' : ''}`),
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        loop_state: {
          type: 'choice',
          instructions: 'What is the dynamic state of this action sequence?',
          criteria: {
            progressing: 'Gaining new information, discovering errors productively, or advancing toward goal',
            repeating_error: 'Repeating the exact same failed command or error pattern without modifying strategy',
            oscillating: 'Flipping back and forth between two mutually incompatible states or edits',
            exhausted_leads: 'Trying endless minor variations of an approach that is demonstrably blocked',
          },
        },
        is_stalled: {
          type: 'boolean',
          instructions: 'Is the agent making zero genuine forward progress toward the task goal?',
        },
        severity: {
          type: 'score',
          instructions: 'How severe is the lack of progress or loop deadlock?',
          criteria: [
            'healthy-progress',
            'minor-transient-friction',
            'distinct-loop-detected',
            'hopeless-infinite-loop',
          ],
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const stateAns = result.answers['loop_state']
  const stallAns = result.answers['is_stalled']
  const sevAns = result.answers['severity']

  if (stateAns?.type !== 'choice' || stallAns?.type !== 'boolean' || sevAns?.type !== 'score') {
    throw new Error('Decision provider returned unexpected answer types for loop stall detection')
  }

  const stallProb = stallAns.probability
  const stalled = stallProb >= warnThreshold || stateAns.choice !== 'progressing'

  let suggestedIntervention: LoopStallVerdict['suggestedIntervention'] = 'continue'
  if (stallProb >= interruptThreshold || sevAns.score >= 2.5) {
    suggestedIntervention = 'interrupt'
  } else if (stallProb >= warnThreshold || sevAns.score >= 1.5) {
    suggestedIntervention = 'warn'
  }

  return {
    stalled,
    loopState: stateAns.choice,
    stallSeverity: sevAns.score,
    stallProbability: stallProb,
    suggestedIntervention,
    latencyMs,
    providerId: result.providerId,
  }
}
