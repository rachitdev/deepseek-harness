/**
 * `SkillSelector`: decides, per user turn, whether one of the available agent skills should be
 * loaded instead of answering directly, using `ctx.decision`. Keeps catalog routing out of the
 * conversation loop: the provider picks among known skill descriptions at decision latency, and
 * this consumer applies the catalog-empty, confidence, and risk gates in code.
 * @module @deepseek-ai/dsh-decision-consumers/skill-selector
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { SkillCandidate, SkillSelection } from './types.ts'

/**
 * Options configuring skill selection policy.
 */
export interface SkillSelectorOptions {
  /** Minimum probability that a skill is needed before one is loaded; defaults to 0.5. */
  readonly minNeedsSkillProbability?: number
  /** Minimum probability for the chosen skill option to be trusted; defaults to 0.65. */
  readonly minSkillConfidence?: number
  /** Risk score (0..2) at or above which the chosen skill needs explicit approval; defaults to 1.5. */
  readonly approvalRiskThreshold?: number
}

/**
 * Select the next step for a user turn against an available skill catalog.
 *
 * With an empty catalog the provider question would have a single answer option, so the consumer
 * decides locally without a provider call: `direct_answer`.
 *
 * @param decisionService - the active decision seam service.
 * @param request - the user turn and the skill catalog offered for this session.
 * @param options - routing thresholds.
 * @param signal - optional cancellation signal.
 * @returns the structured skill selection.
 */
export async function selectSkill(
  decisionService: DecisionService,
  request: {
    readonly userRequest: string
    readonly availableSkills: readonly SkillCandidate[]
    readonly repository?: { readonly language?: string; readonly changedFiles?: readonly string[] }
  },
  options: SkillSelectorOptions = {},
  signal?: AbortSignal,
): Promise<SkillSelection> {
  const minNeedsSkill = options.minNeedsSkillProbability ?? 0.5
  const minSkillConfidence = options.minSkillConfidence ?? 0.65
  const approvalRisk = options.approvalRiskThreshold ?? 1.5

  if (request.availableSkills.length === 0) {
    return {
      action: { type: 'direct_answer' },
      needsSkillProbability: 0,
      riskScore: 0,
      confidence: 1,
      latencyMs: 0,
      providerId: '',
    }
  }

  const skillCriteria: Record<string, string> = {}
  for (const candidate of request.availableSkills) {
    skillCriteria[candidate.name] = candidate.description
  }
  skillCriteria['none'] = 'No listed skill is a clear fit.'

  const state = [
    `User Request: ${request.userRequest}`,
    'Available Skills:',
    ...request.availableSkills.map(c => `- ${c.name}: ${c.description}`),
    request.repository === undefined
      ? ''
      : `Repository: language ${request.repository.language ?? 'unknown'}, changed files ${JSON.stringify(request.repository.changedFiles ?? [])}`,
  ]
    .filter(line => line !== '')
    .join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        needs_skill: {
          type: 'boolean',
          instructions:
            'Would one of the available skills listed in the state be required to handle this request well, rather than a short direct answer?',
        },
        skill: {
          type: 'choice',
          instructions: 'Which available skill best matches the user request and repository context?',
          criteria: skillCriteria,
        },
        risk: {
          type: 'score',
          instructions:
            'How risky would it be for the agent to make changes while acting on this request?',
          criteria: [
            'Read-only or easily reversible',
            'Changes application behavior but is testable',
            'Touches credentials, production data, permissions, or otherwise irreversible side effects',
          ],
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const answerTypesError = new Error(
    'Decision provider returned unexpected answer types for skill selection',
  )
  const needsAns = result.answers['needs_skill']
  if (needsAns?.type !== 'boolean') throw answerTypesError
  const skillAns = result.answers['skill']
  if (skillAns?.type !== 'choice') throw answerTypesError
  const riskAns = result.answers['risk']
  if (riskAns?.type !== 'score') throw answerTypesError

  const skillConfidence = skillAns.probabilities[skillAns.choice] ?? 0

  let action: SkillSelection['action']
  if (needsAns.probability < minNeedsSkill) {
    action = { type: 'direct_answer' }
  } else if (skillAns.choice === 'none' || skillConfidence < minSkillConfidence) {
    action = {
      type: 'clarify',
      reason:
        skillAns.choice === 'none'
          ? 'No listed skill is a clear fit.'
          : `Skill choice probability ${skillConfidence.toFixed(2)} is below the ${minSkillConfidence} threshold.`,
    }
  } else if (riskAns.score >= approvalRisk) {
    action = { type: 'load_skill_with_approval', skill: skillAns.choice }
  } else {
    action = { type: 'load_skill', skill: skillAns.choice }
  }

  return {
    action,
    needsSkillProbability: needsAns.probability,
    riskScore: riskAns.score,
    confidence: skillConfidence,
    latencyMs,
    providerId: result.providerId,
  }
}
