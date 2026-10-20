/**
 * Vocabulary for the decision capability seam (`ctx.decision`). A decision model answers typed
 * questions about one state and returns calibrated probabilities; it emits no prose, so the seam
 * carries no messages, streaming, tool calls, or output-token controls.
 * @module @deepseek-ai/dsh-decision/types
 */

import { HarnessError } from '@deepseek-ai/dsh-llm'

/**
 * One question put to a decision provider. A CLOSED discriminated union owned by `dsh-decision`:
 * providers translate every arm and consumers `switch` on `type` ending in `assertNever`, so a new
 * arm is a coordinated change across known packages rather than a plugin extension.
 */
export type DecisionQuestion =
  | DecisionChoiceQuestion
  | DecisionScoreQuestion
  | DecisionBooleanQuestion

/**
 * Pick one option from a named set. `criteria` maps each option id to the description the model
 * judges against; option ids are the consumer's own vocabulary and come back verbatim in the answer.
 */
export interface DecisionChoiceQuestion {
  readonly type: 'choice'
  /** What the model is being asked to decide. */
  readonly instructions: string
  /** Option id to its description. At least two entries; a one-option choice has no decision to make. */
  readonly criteria: Readonly<Record<string, string>>
}

/**
 * Place the state on an ordered rubric. `criteria` lists level descriptions from lowest to highest;
 * the answer may land between levels, so a score is continuous rather than an index.
 */
export interface DecisionScoreQuestion {
  readonly type: 'score'
  /** What the model is being asked to rate. */
  readonly instructions: string
  /** Ordered level descriptions, lowest first. Between 2 and 10 entries. */
  readonly criteria: readonly string[]
}

/** Judge whether one statement holds. The answer is the probability that it is true. */
export interface DecisionBooleanQuestion {
  readonly type: 'boolean'
  /** The statement whose truth is being judged. */
  readonly instructions: string
}

/**
 * One answer from a decision provider, keyed identically to its question. A CLOSED discriminated
 * union: the `type` always matches the question's `type`, so a consumer that asked a `choice` may
 * narrow on that arm.
 */
export type DecisionAnswer =
  | DecisionChoiceAnswer
  | DecisionScoreAnswer
  | DecisionBooleanAnswer

/** The selected option plus the full distribution that produced it. */
export interface DecisionChoiceAnswer {
  readonly type: 'choice'
  /** The winning option id, always a key of the question's `criteria`. */
  readonly choice: string
  /** Probability per option id; keys match the question's `criteria`, values sum to approximately 1. */
  readonly probabilities: Readonly<Record<string, number>>
}

/** A position on the rubric plus the distribution over levels. */
export interface DecisionScoreAnswer {
  readonly type: 'score'
  /** Continuous position, `0` to `criteria.length - 1`; may fall between levels. */
  readonly score: number
  /** Probability per level index, keyed by its stringified index. */
  readonly probabilities: Readonly<Record<string, number>>
}

/** The probability that the statement holds. */
export interface DecisionBooleanAnswer {
  readonly type: 'boolean'
  /** Probability in `[0, 1]` that the statement is true. */
  readonly probability: number
}

/**
 * What a decision provider is asked to evaluate: one state, and every question asked about it.
 * Questions are evaluated together because a provider prices and paces one call, not one question —
 * an extra question costs tokens but little additional time.
 */
export interface DecisionRequest {
  /** The situation being judged, as plain text the provider reads verbatim. */
  readonly state: string
  /** Questions keyed by a consumer-chosen name; answers come back under the same keys. */
  readonly questions: Readonly<Record<string, DecisionQuestion>>
}

/** The answers to one {@link DecisionRequest}, keyed identically to its `questions`. */
export interface DecisionResult {
  /** One answer per question, under the request's own keys. */
  readonly answers: Readonly<Record<string, DecisionAnswer>>
  /** Id of the provider that answered, for session-log attribution. */
  readonly providerId: string
}

/**
 * A decision-capable backend. Registered with `ctx.decision.registerProvider`. `id` is a stable
 * string, unique within the seam.
 */
export interface DecisionProvider {
  readonly id: string
  /** Cheap local usability check; must not make network calls. */
  available(): boolean
  /** Answer every question in one request; honor `signal` for cancellation. */
  evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult>
}

/**
 * Typed decision error with a machine-routable, open-string `code` and chained `cause`. Consumers
 * must tolerate provider-specific codes. Shared codes cover unavailable, missing, unusable,
 * ambiguous, or duplicate providers, malformed requests, and provider failure.
 */
export class DecisionError extends HarnessError {}
