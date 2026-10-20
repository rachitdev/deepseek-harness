/**
 * JEV arbitration for parallel canary probes after an active model-route failure.
 * @module @deepseek-ai/dsh-decision-consumers/failover-arbiter
 */

import type { DecisionQuestion, DecisionService } from '@deepseek-ai/dsh-decision'
import type { FailoverCandidateProbe, FailoverDecision } from './types.ts'

const EFFICIENCY_RUBRIC = [
  'wasteful-verbose-burn',
  'moderately-high-tokens',
  'compact-efficient',
  'ultra-lean-optimal',
] as const

const VALIDITY_RUBRIC = [
  'broken-or-hallucinated',
  'questionable-syntax',
  'sound-implementation',
  'production-grade-idiomatic',
] as const

const LATENCY_RUBRIC = [
  'severe-latency-penalty',
  'material-latency-penalty',
  'minor-latency-penalty',
  'negligible-latency-penalty',
] as const

/**
 * Arbitrate between parallel candidate model probes and select one exact route.
 * @param decisionService - Active typed-decision service.
 * @param params - Failed route context and completed canary probes.
 * @param signal - Optional cancellation signal.
 * @returns Selected provider/model route and its calibrated takeover metrics.
 */
export async function arbitrateModelFailover(
  decisionService: DecisionService,
  params: {
    readonly failedModel: string
    readonly failureReason: string
    readonly currentTask: string
    readonly candidateProbes: readonly FailoverCandidateProbe[]
  },
  signal?: AbortSignal,
): Promise<FailoverDecision> {
  const healthyProbes = params.candidateProbes.filter(
    probe => probe.error === undefined && probe.probeOutput.length > 0,
  )
  if (healthyProbes.length === 0) {
    throw new Error(`All candidate model probes failed during failover arbitration for ${params.failedModel}`)
  }

  const singleCandidate = healthyProbes[0]
  if (healthyProbes.length === 1 && singleCandidate !== undefined) {
    return {
      recommendedProvider: singleCandidate.provider,
      recommendedModel: singleCandidate.model,
      confidence: 1,
      tokenEfficiencyScore: 2,
      codeValidityScore: 2,
      latencyPenaltyScore: 3,
      latencyMs: 0,
      providerId: 'heuristic-single-survivor',
    }
  }

  const choices = new Map<string, FailoverCandidateProbe>()
  const criteria: Record<string, string> = {}
  const questions: Record<string, DecisionQuestion> = {}
  healthyProbes.forEach((probe, index) => {
    const id = `worker_${String(index + 1)}`
    choices.set(id, probe)
    criteria[id] = `${probe.provider}/${probe.model}: latency=${String(probe.latencyMs)}ms, tokens=${String(probe.tokenCount)}, sample=${probe.probeOutput.slice(0, 100).replace(/\n/g, ' ')}`
    questions[`code_validity_${id}`] = {
      type: 'score',
      instructions: `Rate the code validity of ${id} (${probe.provider}/${probe.model}) for the active task.`,
      criteria: VALIDITY_RUBRIC,
    }
    questions[`token_efficiency_${id}`] = {
      type: 'score',
      instructions: `Rate the output value per token of ${id} (${probe.provider}/${probe.model}).`,
      criteria: EFFICIENCY_RUBRIC,
    }
    questions[`latency_penalty_${id}`] = {
      type: 'score',
      instructions: `Rate how small the latency penalty is for ${id} (${probe.provider}/${probe.model}); higher is better.`,
      criteria: LATENCY_RUBRIC,
    }
  })
  questions.best_provider = {
    type: 'choice',
    instructions: 'Which healthy worker route should DSH reconnect to? Prioritize valid code, task fitness, token efficiency, and low latency.',
    criteria,
  }

  const state = [
    `Failed Model: ${params.failedModel}`,
    `Failure Reason: ${params.failureReason}`,
    `Active Task Goal: ${params.currentTask}`,
    'Parallel Candidate Probe Results:',
    ...[...choices].map(([id, probe]) =>
      `[${id}: ${probe.provider}/${probe.model}] Latency: ${String(probe.latencyMs)}ms | Tokens: ${String(probe.tokenCount)}\nOutput: ${probe.probeOutput.slice(0, 400)}`),
  ].join('\n')

  const started = performance.now()
  const result = await decisionService.evaluate({ state, questions }, signal)
  const latencyMs = Math.round(performance.now() - started)
  const routeAnswer = result.answers.best_provider
  if (routeAnswer?.type !== 'choice') {
    throw new Error('Decision provider returned unexpected answer types for failover arbitration')
  }
  const selected = choices.get(routeAnswer.choice)
  if (selected === undefined) {
    throw new Error(`Decision provider selected unknown failover worker ${JSON.stringify(routeAnswer.choice)}`)
  }

  const validityAnswer = result.answers[`code_validity_${routeAnswer.choice}`]
  const efficiencyAnswer = result.answers[`token_efficiency_${routeAnswer.choice}`]
  const latencyAnswer = result.answers[`latency_penalty_${routeAnswer.choice}`]
  if (validityAnswer?.type !== 'score' || efficiencyAnswer?.type !== 'score'
    || latencyAnswer?.type !== 'score') {
    throw new Error('Decision provider returned unexpected per-worker scores for failover arbitration')
  }

  return {
    recommendedProvider: selected.provider,
    recommendedModel: selected.model,
    confidence: routeAnswer.probabilities[routeAnswer.choice] ?? 0,
    tokenEfficiencyScore: efficiencyAnswer.score,
    codeValidityScore: validityAnswer.score,
    latencyPenaltyScore: latencyAnswer.score,
    latencyMs,
    providerId: result.providerId,
  }
}
