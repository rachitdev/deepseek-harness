/**
 * Wire types for TypeSafe's direct System One API that serves Jev. TypeSafe calls the
 * probability-of-truth arm `noul`; this module owns that translation so the seam vocabulary never
 * leaks vendor naming.
 * @module @deepseek-ai/dsh-decision-jev/types
 */

import type { SingleAnswerWireQuestion } from '@deepseek-ai/dsh-decision'

/** One question as the System One endpoint accepts it; the shared single-answer wire shape. */
export type JevQuestion = SingleAnswerWireQuestion

/** The request body posted to the System One endpoint. */
export interface JevRequest {
  readonly model: string
  readonly state: string
  readonly questions: Readonly<Record<string, JevQuestion>>
}

/** One answer as the endpoint returns it; the arm is tagged by `type`. */
export interface JevAnswer {
  readonly type: 'choice' | 'score' | 'noul'
  /** Winning option id on a `choice` answer. */
  readonly choice?: string
  /** Continuous rubric position on a `score` answer. */
  readonly score?: number
  /** Probability of truth on a `noul` answer. */
  readonly noul?: number
  /** Distribution over options or level indices on `choice` and `score` answers. */
  readonly probabilities?: Readonly<Record<string, number>>
}

/** The response body returned by the System One endpoint. */
export interface JevResponse {
  readonly model?: string
  readonly answers?: Readonly<Record<string, JevAnswer>>
  readonly usage?: { readonly input_tokens?: number; readonly output_tokens?: number }
  readonly error?: { readonly message?: string; readonly type?: string }
}
