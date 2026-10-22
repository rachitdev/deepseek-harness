/**
 * The single-answer wire question shared by the providers that post one question
 * per key and read back a tagged answer.
 *
 * TypeSafe's System One endpoint and the Laya worker speak the same vocabulary:
 * `choice` and `score` carry `criteria`, and the seam's `boolean` arm becomes the
 * `noul` arm. Owning the translation here keeps the two providers from carrying
 * private copies of the same switch.
 *
 * @module @deepseek-ai/dsh-decision/wire-question
 */

import { DecisionError } from './types.ts'
import type { DecisionQuestion } from './types.ts'

/** One question as both single-answer wire protocols accept it. */
export interface SingleAnswerWireQuestion {
  readonly type: 'choice' | 'score' | 'noul'
  readonly instructions: string
  /** Option map for `choice`, ordered level array for `score`, absent for `noul`. */
  readonly criteria?: Readonly<Record<string, string>> | readonly string[]
}

/**
 * Translate one seam question onto the shared single-answer wire vocabulary.
 *
 * @param question - the seam question to translate.
 * @returns the question as a single-answer endpoint accepts it.
 */
export function toSingleAnswerWireQuestion(question: DecisionQuestion): SingleAnswerWireQuestion {
  switch (question.type) {
    case 'choice':
      return { type: 'choice', instructions: question.instructions, criteria: question.criteria }
    case 'score':
      return { type: 'score', instructions: question.instructions, criteria: question.criteria }
    case 'boolean':
      return { type: 'noul', instructions: question.instructions }
    default:
      return assertNever(question)
  }
}

/**
 * Exhaustiveness guard for the closed question union.
 *
 * @param value - the unreachable arm.
 * @returns never; always throws.
 */
function assertNever(value: never): never {
  throw new DecisionError(
    `unhandled decision question type: ${JSON.stringify(value)}`,
    'DECISION_PROVIDER_ERROR',
  )
}
