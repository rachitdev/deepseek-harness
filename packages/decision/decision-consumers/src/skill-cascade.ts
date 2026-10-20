/**
 * `SkillCascade`: two-stage progressive-disclosure skill selection. Stage one picks a skill
 * category from a caller-grouped catalog and judges whether any skill is needed; stage two picks
 * the exact skill inside the winning category. A stage with a single option is resolved locally
 * without a decision call.
 * @module @deepseek-ai/dsh-decision-consumers/skill-cascade
 */

import type { DecisionChoiceAnswer, DecisionResult, DecisionService } from '@deepseek-ai/dsh-decision'
import type { SkillCandidate } from './types.ts'

/** One caller-owned group of catalog skills judged as a single stage-one option. */
export interface SkillCascadeCategory {
  /** Category id returned verbatim as the stage-one choice. */
  readonly name: string
  /** Routing description the decision provider judges the category against. */
  readonly description: string
  /** Non-empty skills belonging to this category. */
  readonly skills: readonly SkillCandidate[]
}

/** Input for one cascade over a grouped catalog. */
export interface SkillCascadeRequest {
  /** User task whose skill is being selected. */
  readonly request: string
  /** Non-empty categories with unique names; every category holds at least one skill. */
  readonly categories: readonly SkillCascadeCategory[]
}

/** Stage-one and stage-two verdicts of one cascade. */
export interface SkillCascade {
  /** Winning category name. */
  readonly category: string
  /** Probability of the winning category; `1` when it was the only category. */
  readonly categoryConfidence: number
  /** Probability that the request needs any skill; `1` when stage one made no call. */
  readonly needsSkillProbability: number
  /** Winning skill name inside {@link category}. */
  readonly skill: string
  /** Probability of the winning skill within its category; `1` when it was the only skill. */
  readonly skillConfidence: number
  /** Decision calls made: 0, 1, or 2. */
  readonly calls: number
  /** Provider id per decision call, in call order. */
  readonly providerIds: readonly string[]
}

/**
 * Select a skill through a category-first cascade.
 * @param decisionService - configured decision service.
 * @param request - task and grouped catalog.
 * @param signal - cancellation signal forwarded to every decision call.
 * @returns the category and skill verdicts with their probabilities.
 */
export async function cascadeSkillSelection(
  decisionService: DecisionService,
  request: SkillCascadeRequest,
  signal?: AbortSignal,
): Promise<SkillCascade> {
  const categories = request.categories
  const names = new Set<string>()
  for (const category of categories) {
    if (category.skills.length === 0) throw new Error(`skill cascade category "${category.name}" has no skills`)
    if (names.has(category.name)) throw new Error(`skill cascade category "${category.name}" is duplicated`)
    names.add(category.name)
  }

  const [first] = categories
  if (first === undefined) throw new Error('skill cascade requires at least one category')
  const providerIds: string[] = []
  let chosen: SkillCascadeCategory = first
  let categoryConfidence = 1
  let needsSkillProbability = 1
  if (categories.length > 1) {
    const result = await decisionService.evaluate({
      state: [
        `User request: ${request.request}`,
        'Skill categories:',
        ...categories.map(category => `- ${category.name}: ${category.description} (skills: ${category.skills.map(skill => skill.name).join(', ')})`),
      ].join('\n'),
      questions: {
        category: {
          type: 'choice',
          instructions: 'Which skill category best matches the work this request asks for?',
          criteria: Object.fromEntries(categories.map(category => [category.name, category.description])),
        },
        needs_skill: {
          type: 'boolean',
          instructions: 'Does this request need specialised skill instructions rather than a direct answer?',
        },
      },
    }, signal)
    providerIds.push(result.providerId)
    const answer = choiceAnswer(result, 'category')
    const found = categories.find(category => category.name === answer.choice)
    if (found === undefined) throw new Error(`skill cascade provider chose unknown category "${answer.choice}"`)
    chosen = found
    categoryConfidence = winningProbability(answer)
    const needs = result.answers['needs_skill']
    if (needs?.type !== 'boolean') throw new Error('skill cascade expected boolean answer "needs_skill"')
    needsSkillProbability = needs.probability
  }

  const [firstSkill] = chosen.skills as readonly [SkillCandidate, ...SkillCandidate[]]
  let skill = firstSkill.name
  let skillConfidence = 1
  if (chosen.skills.length > 1) {
    const result = await decisionService.evaluate({
      state: [
        `User request: ${request.request}`,
        `Skill category: ${chosen.name}: ${chosen.description}`,
      ].join('\n'),
      questions: {
        skill: {
          type: 'choice',
          instructions: 'Which single skill should be loaded first for this request?',
          criteria: Object.fromEntries(chosen.skills.map(candidate => [candidate.name, candidate.description])),
        },
      },
    }, signal)
    providerIds.push(result.providerId)
    const answer = choiceAnswer(result, 'skill')
    if (!chosen.skills.some(candidate => candidate.name === answer.choice)) {
      throw new Error(`skill cascade provider chose unknown skill "${answer.choice}"`)
    }
    skill = answer.choice
    skillConfidence = winningProbability(answer)
  }

  return {
    category: chosen.name,
    categoryConfidence,
    needsSkillProbability,
    skill,
    skillConfidence,
    calls: providerIds.length,
    providerIds,
  }
}

/** A provider-reported choice without a probability for its own winner counts as zero confidence. */
function winningProbability(answer: DecisionChoiceAnswer): number {
  return answer.probabilities[answer.choice] ?? 0
}

function choiceAnswer(result: DecisionResult, key: string): DecisionChoiceAnswer {
  const answer = result.answers[key]
  if (answer?.type !== 'choice') throw new Error(`skill cascade expected choice answer "${key}"`)
  return answer
}
