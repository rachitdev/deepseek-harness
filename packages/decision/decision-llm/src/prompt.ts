/**
 * Prompt construction and response parsing for the LLM-backed decision provider. A text model has
 * no native typed-answer channel, so this module states the answer format, then parses and
 * validates what comes back against the questions that were asked.
 * @module @deepseek-ai/dsh-decision-llm/prompt
 */

import { DecisionError } from '@deepseek-ai/dsh-decision'
import type { DecisionAnswer, DecisionQuestion, DecisionRequest } from '@deepseek-ai/dsh-decision'

/** Instruction block telling the model exactly what JSON to emit. */
export const DECISION_SYSTEM_PROMPT = [
  'You answer typed questions about a situation and return only JSON.',
  'Respond with a single JSON object whose keys are exactly the question keys given to you.',
  'For a "choice" question, answer {"choice": "<option id>", "probabilities": {"<option id>": <0..1>, ...}}',
  'covering every option id, summing to 1.',
  'For a "score" question, answer {"score": <number>, "probabilities": {"<level index>": <0..1>, ...}}',
  'where the score runs from 0 to the last level index and may fall between levels.',
  'For a "boolean" question, answer {"probability": <0..1>} giving the probability the statement is true.',
  'Report genuine uncertainty in the probabilities; do not round them to 0 or 1 unless the state is decisive.',
  'Output no prose, no explanation, and no code fences.',
].join('\n')

/**
 * Render one request as the user-turn text the model judges.
 * @param request - the state and questions to describe.
 * @returns the rendered prompt.
 */
export function renderDecisionPrompt(request: DecisionRequest): string {
  const lines: string[] = ['# Situation', request.state, '', '# Questions']
  for (const [key, question] of Object.entries(request.questions)) {
    lines.push('', `## ${key}`, `type: ${question.type}`, `question: ${question.instructions}`)
    switch (question.type) {
      case 'choice':
        lines.push('options:')
        for (const [id, description] of Object.entries(question.criteria)) {
          lines.push(`- ${id}: ${description}`)
        }
        break
      case 'score':
        lines.push('levels (lowest first):')
        question.criteria.forEach((level, index) => lines.push(`- ${String(index)}: ${level}`))
        break
      case 'boolean':
        break
      default:
        return assertNever(question)
    }
  }
  return lines.join('\n')
}

/**
 * Parse a model response into validated answers for exactly the questions asked. A model can emit
 * anything, so this is a model-output boundary: every field is checked rather than trusted.
 * @param text - the raw model response.
 * @param request - the request whose questions must all be answered.
 * @returns one validated answer per question key.
 */
export function parseDecisionResponse(
  text: string,
  request: DecisionRequest,
): Record<string, DecisionAnswer> {
  const parsed: unknown = JSON.parse(stripCodeFence(text))
  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    throw new DecisionError('decision model did not return a JSON object', 'DECISION_PROVIDER_ERROR')
  }
  const record = parsed as Record<string, unknown>
  const answers: Record<string, DecisionAnswer> = {}
  for (const [key, question] of Object.entries(request.questions)) {
    const raw = record[key]
    if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) {
      throw new DecisionError(`decision model returned no answer object for "${key}"`, 'DECISION_PROVIDER_ERROR')
    }
    answers[key] = parseAnswer(key, question, raw as Record<string, unknown>)
  }
  return answers
}

/**
 * Validate one answer against the question that produced it.
 * @param key - the question key, used to locate a malformed answer.
 * @param question - the question asked.
 * @param raw - the parsed answer object.
 * @returns the validated answer.
 */
function parseAnswer(
  key: string,
  question: DecisionQuestion,
  raw: Record<string, unknown>,
): DecisionAnswer {
  switch (question.type) {
    case 'choice': {
      const choice = raw['choice']
      if (typeof choice !== 'string' || !(choice in question.criteria)) {
        throw new DecisionError(
          `decision model chose "${String(choice)}" for "${key}", which is not one of its options`,
          'DECISION_PROVIDER_ERROR',
        )
      }
      return {
        type: 'choice',
        choice,
        probabilities: parseProbabilities(key, raw['probabilities'], Object.keys(question.criteria)),
      }
    }
    case 'score': {
      const score = raw['score']
      const highest = question.criteria.length - 1
      if (typeof score !== 'number' || !Number.isFinite(score) || score < 0 || score > highest) {
        throw new DecisionError(
          `decision model scored "${key}" at ${String(score)}, outside 0..${String(highest)}`,
          'DECISION_PROVIDER_ERROR',
        )
      }
      const levelKeys = question.criteria.map((_level, index) => String(index))
      return { type: 'score', score, probabilities: parseProbabilities(key, raw['probabilities'], levelKeys) }
    }
    case 'boolean': {
      const probability = raw['probability']
      if (!isProbability(probability)) {
        throw new DecisionError(
          `decision model returned probability ${String(probability)} for "${key}", outside 0..1`,
          'DECISION_PROVIDER_ERROR',
        )
      }
      return { type: 'boolean', probability }
    }
    default:
      return assertNever(question)
  }
}

/**
 * Validate a distribution covering exactly the expected keys.
 * @param key - the question key, used to locate a malformed distribution.
 * @param raw - the parsed `probabilities` value.
 * @param expected - every key the distribution must carry.
 * @returns the validated distribution.
 */
function parseProbabilities(
  key: string,
  raw: unknown,
  expected: readonly string[],
): Record<string, number> {
  if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) {
    throw new DecisionError(`decision model returned no probability map for "${key}"`, 'DECISION_PROVIDER_ERROR')
  }
  const record = raw as Record<string, unknown>
  const probabilities: Record<string, number> = {}
  for (const name of expected) {
    const value = record[name]
    if (!isProbability(value)) {
      throw new DecisionError(
        `decision model gave "${key}" entry "${name}" the value ${String(value)}, outside 0..1`,
        'DECISION_PROVIDER_ERROR',
      )
    }
    probabilities[name] = value
  }
  return probabilities
}

/**
 * Whether a parsed value is a usable probability.
 * @param value - the parsed value.
 * @returns true when it is a finite number within `[0, 1]`.
 */
function isProbability(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 && value <= 1
}

/**
 * Remove a Markdown code fence a model wrapped its JSON in.
 * @param text - the raw response.
 * @returns the text with any surrounding fence removed.
 */
function stripCodeFence(text: string): string {
  const trimmed = text.trim()
  if (!trimmed.startsWith('```')) return trimmed
  const withoutOpen = trimmed.replace(/^```[a-zA-Z]*\n?/, '')
  return withoutOpen.endsWith('```') ? withoutOpen.slice(0, -3).trim() : withoutOpen.trim()
}

/**
 * Exhaustiveness guard for the closed question union.
 * @param value - the arm TypeScript proved unreachable.
 * @returns never; always throws.
 */
function assertNever(value: never): never {
  throw new DecisionError(
    `unhandled decision question type: ${JSON.stringify(value)}`,
    'DECISION_PROVIDER_ERROR',
  )
}
