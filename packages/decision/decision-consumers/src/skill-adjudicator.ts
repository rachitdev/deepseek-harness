import type { DecisionQuestion, DecisionResult, DecisionService } from '@deepseek-ai/dsh-decision'
import type { SkillCandidate } from './types.ts'

const DEFAULT_RELEVANCE_THRESHOLD = 0.65
const DEFAULT_PHASE_CONFIDENCE = 0.55

/** Input for DSH/JEV skill adjudication over a caller-bounded candidate set. */
export interface SkillAdjudicationRequest {
  /** User task whose skills are being selected. */
  readonly request: string
  /** Skill names proposed by DSH before JEV evaluation. */
  readonly dshProposed: readonly string[]
  /** Bounded catalog available to both passes. */
  readonly candidates: readonly SkillCandidate[]
}

/** Thresholds for expanding and accepting the second-pass skill plan. */
export interface SkillAdjudicationOptions {
  /** Relevance probability that admits a skill to final comparison. Defaults to 0.65. */
  readonly relevanceThreshold?: number
  /** Winning phase probability required for a companion. Defaults to 0.55. */
  readonly minPhaseConfidence?: number
}

/** DSH-owned execution plan produced from one or two JEV calls. */
export interface SkillAdjudication {
  readonly primary: string
  readonly parallel: readonly string[]
  readonly later: readonly string[]
  readonly highProbability: readonly string[]
  readonly relevance: Readonly<Record<string, number>>
  readonly disagreement: boolean
  readonly adjudicated: boolean
  readonly providerId: string
}

/**
 * Compare DSH's proposal with JEV relevance, then classify competing skills by execution phase.
 * The caller owns candidate shortlisting, failure fallback, instruction loading, and durable logging.
 *
 * @param decisionService - configured decision service.
 * @param request - task, DSH proposal, and bounded skill candidates.
 * @param options - relevance and phase-confidence thresholds.
 * @param signal - cancellation signal forwarded to every decision call.
 * @returns the primary, parallel-now, and later-depth skill plan.
 */
export async function adjudicateSkillSelection(
  decisionService: DecisionService,
  request: SkillAdjudicationRequest,
  options: SkillAdjudicationOptions = {},
  signal?: AbortSignal,
): Promise<SkillAdjudication> {
  const relevanceThreshold = options.relevanceThreshold ?? DEFAULT_RELEVANCE_THRESHOLD
  const minPhaseConfidence = options.minPhaseConfidence ?? DEFAULT_PHASE_CONFIDENCE
  assertProbability('relevanceThreshold', relevanceThreshold)
  assertProbability('minPhaseConfidence', minPhaseConfidence)
  const candidates = uniqueCandidates(request.candidates)
  if (candidates.length === 0) throw new Error('skill adjudication requires at least one candidate')
  const names = new Set(candidates.map(candidate => candidate.name))
  const proposed = [...new Set(request.dshProposed)]
  const [firstProposal] = proposed
  if (firstProposal === undefined || proposed.some(name => !names.has(name))) {
    throw new Error('skill adjudication requires every DSH proposal in the candidate set')
  }
  if (candidates.length === 1) {
    return {
      primary: firstProposal,
      parallel: [],
      later: [],
      highProbability: [firstProposal],
      relevance: { [firstProposal]: 1 },
      disagreement: false,
      adjudicated: false,
      providerId: 'local',
    }
  }

  const criteria = Object.fromEntries(candidates.map(candidate => [candidate.name, candidate.description]))
  const firstQuestions: Record<string, DecisionQuestion> = {
    primary: {
      type: 'choice',
      instructions: 'Which single skill must own the first phase of this request?',
      criteria,
    },
  }
  for (const candidate of candidates) {
    firstQuestions[`relevant:${candidate.name}`] = {
      type: 'boolean',
      instructions: `Is ${candidate.name} materially relevant now, in parallel, or in a later phase?`,
    }
  }
  const state = renderState(request.request, proposed, candidates)
  const first = await decisionService.evaluate({ state, questions: firstQuestions }, signal)
  const firstPrimary = choice(first, 'primary')
  const relevanceByName = new Map(candidates.map(candidate => [
    candidate.name,
    probability(first, `relevant:${candidate.name}`),
  ]))
  const relevance = Object.fromEntries(relevanceByName)
  const highProbability = candidates
    .filter(candidate => (relevanceByName.get(candidate.name) ?? 0) >= relevanceThreshold)
    .map(candidate => candidate.name)
  const disagreement = !proposed.includes(firstPrimary)
  const comparison = [...new Set([...proposed, firstPrimary, ...highProbability])]
  if (!disagreement && comparison.length === 1) {
    return {
      primary: firstPrimary,
      parallel: [],
      later: [],
      highProbability,
      relevance,
      disagreement,
      adjudicated: false,
      providerId: first.providerId,
    }
  }

  const comparisonCriteria = Object.fromEntries(comparison.map(name => [name, criteria[name] ?? '']))
  const secondQuestions: Record<string, DecisionQuestion> = {
    primary: {
      type: 'choice',
      instructions: 'After comparing the DSH proposal and JEV candidates, which skill owns the first phase?',
      criteria: comparisonCriteria,
    },
  }
  for (const name of comparison) {
    secondQuestions[`phase:${name}`] = {
      type: 'choice',
      instructions: `When should ${name} be used for this request?`,
      criteria: {
        primary_now: 'Own the first phase now.',
        parallel_now: 'Run with the primary skill now because the work is independent or complementary.',
        later_depth: 'Load after an earlier phase produces information this skill needs.',
        not_needed: 'Do not load this skill for this request.',
      },
    }
  }
  const second = await decisionService.evaluate({
    state: `${state}\nFirst-pass JEV primary: ${firstPrimary}\nHigh-probability skills: ${highProbability.join(', ') || 'none'}`,
    questions: secondQuestions,
  }, signal)
  const primary = choice(second, 'primary')
  const parallel: string[] = []
  const later: string[] = []
  for (const name of comparison) {
    if (name === primary) continue
    const phaseAnswer = choiceAnswer(second, `phase:${name}`)
    const confidence = phaseAnswer.probabilities[phaseAnswer.choice] ?? 0
    if (confidence < minPhaseConfidence) continue
    if (phaseAnswer.choice === 'parallel_now' || phaseAnswer.choice === 'primary_now') parallel.push(name)
    if (phaseAnswer.choice === 'later_depth') later.push(name)
  }
  return {
    primary,
    parallel,
    later,
    highProbability,
    relevance,
    disagreement,
    adjudicated: true,
    providerId: second.providerId,
  }
}

function renderState(request: string, proposed: readonly string[], candidates: readonly SkillCandidate[]): string {
  return [
    `User request: ${request}`,
    `DSH proposed skills: ${proposed.join(', ')}`,
    'Candidate skills:',
    ...candidates.map(candidate => `- ${candidate.name}: ${candidate.description}`),
  ].join('\n')
}

function uniqueCandidates(candidates: readonly SkillCandidate[]): SkillCandidate[] {
  const seen = new Set<string>()
  return candidates.filter((candidate) => {
    if (seen.has(candidate.name)) return false
    seen.add(candidate.name)
    return true
  })
}

function choice(result: DecisionResult, key: string): string {
  return choiceAnswer(result, key).choice
}

function choiceAnswer(result: DecisionResult, key: string): Extract<DecisionResult['answers'][string], { type: 'choice' }> {
  const answer = result.answers[key]
  if (answer?.type !== 'choice') throw new Error(`skill adjudication expected choice answer "${key}"`)
  return answer
}

function probability(result: DecisionResult, key: string): number {
  const answer = result.answers[key]
  if (answer?.type !== 'boolean') throw new Error(`skill adjudication expected boolean answer "${key}"`)
  return answer.probability
}

function assertProbability(name: string, value: number): void {
  if (!Number.isFinite(value) || value < 0 || value > 1) {
    throw new Error(`skill adjudication ${name} must be between 0 and 1`)
  }
}
