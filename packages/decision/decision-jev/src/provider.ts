/**
 * `JevDecisionProvider`: a `DecisionProvider` backed by Jev through TypeSafe's direct System One
 * endpoint. Jev returns typed answers with calibrated probabilities and emits no prose, so this
 * provider translates the seam's boolean arm to TypeSafe's `noul` vocabulary and maps every answer
 * straight back.
 * @module @deepseek-ai/dsh-decision-jev/provider
 */

import { DecisionError, toSingleAnswerWireQuestion } from '@deepseek-ai/dsh-decision'
import type {
  DecisionAnswer,
  DecisionProvider,
  DecisionQuestion,
  DecisionRequest,
  DecisionResult,
} from '@deepseek-ai/dsh-decision'
import type { JevAnswer, JevQuestion, JevResponse } from './types.ts'

/** Stable id this provider registers under. */
export const JEV_PROVIDER_ID = 'jev'

/** Default TypeSafe API endpoint; the System One path is appended. */
export const JEV_DEFAULT_BASE_URL = 'https://api.typesafe.ai'

/** Path TypeSafe serves System One evaluations on. Not chat-completions shaped. */
export const JEV_EVALUATION_PATH = '/v1/systemone'

/** Default direct TypeSafe model id for Jev. */
export const JEV_DEFAULT_MODEL = 'jev-latest'

/**
 * Default ceiling on one awaited Jev call. Jev answers in well under a second, so a call still
 * pending after this long is treated as failed and the configured fallback provider takes over.
 */
export const JEV_DEFAULT_TIMEOUT_MS = 3_000

/** Attribution header sent on every request. Bump with the package version. */
const USER_AGENT = 'deepseek-harness/0.0.1'

/** Resolved provider options (the plugin's `apply` supplies env-var and constant defaults). */
export interface JevDecisionProviderOptions {
  /** Resolve the TypeSafe API key for the next call; `undefined` means unavailable. */
  resolveApiKey: () => Promise<string | undefined>
  /** Whether a key is reachable at all, for the local `available()` check. */
  hasApiKey: boolean
  /** Endpoint base; {@link JEV_EVALUATION_PATH} is appended. */
  baseURL: string
  /** Direct TypeSafe model id. */
  model: string
  /** Ceiling on one call, in milliseconds; defaults to {@link JEV_DEFAULT_TIMEOUT_MS}. */
  timeoutMs?: number
}

/**
 * Translate one seam question onto TypeSafe's wire vocabulary. Both `choice` and `score` send their
 * levels under `criteria`; the seam's `boolean` arm becomes TypeSafe's `noul` arm.
 * @param question - the seam question to translate.
 * @returns the question as the System One endpoint accepts it.
 */
export function toJevQuestion(question: DecisionQuestion): JevQuestion {
  return toSingleAnswerWireQuestion(question)
}

/**
 * Map one TypeSafe answer back onto the seam's answer union, checking that the returned arm carries
 * its required fields. An answer missing its payload is a provider error rather than a defaulted
 * value: a fabricated probability would be indistinguishable from a calibrated one at every
 * consumer confidence threshold.
 * @param key - the question key, used to locate a malformed answer.
 * @param answer - the answer as returned by TypeSafe.
 * @returns the answer in seam vocabulary.
 */
export function fromJevAnswer(key: string, answer: JevAnswer): DecisionAnswer {
  switch (answer.type) {
    case 'choice': {
      if (answer.choice === undefined || answer.probabilities === undefined) {
        throw new DecisionError(`Jev choice answer "${key}" is missing its distribution`, 'DECISION_PROVIDER_ERROR')
      }
      return { type: 'choice', choice: answer.choice, probabilities: answer.probabilities }
    }
    case 'score': {
      if (answer.score === undefined || answer.probabilities === undefined) {
        throw new DecisionError(`Jev score answer "${key}" is missing its distribution`, 'DECISION_PROVIDER_ERROR')
      }
      return { type: 'score', score: answer.score, probabilities: answer.probabilities }
    }
    case 'noul': {
      if (answer.noul === undefined) {
        throw new DecisionError(`Jev noul answer "${key}" is missing its probability`, 'DECISION_PROVIDER_ERROR')
      }
      return { type: 'boolean', probability: answer.noul }
    }
    default:
      throw new DecisionError(
        `Jev returned an unknown answer type for "${key}": ${JSON.stringify(answer.type)}`,
        'DECISION_PROVIDER_ERROR',
      )
  }
}

/** The Jev-backed decision provider; HTTP redirects fail as `DECISION_PROVIDER_ERROR`. */
export class JevDecisionProvider implements DecisionProvider {
  readonly id = JEV_PROVIDER_ID

  constructor(private readonly options: JevDecisionProviderOptions) {}

  available(): boolean {
    return this.options.hasApiKey
      && URL.canParse(this.options.baseURL)
      && this.options.model.length > 0
  }

  async evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult> {
    const apiKey = await this.options.resolveApiKey()
    if (apiKey === undefined || apiKey.length === 0) {
      throw new DecisionError('Jev provider has no API key to authenticate with', 'DECISION_PROVIDER_ERROR')
    }

    const questions = Object.fromEntries(
      Object.entries(request.questions).map(([key, question]) => [key, toJevQuestion(question)]),
    )

    // The deadline covers the body read as well as the request: a stalled stream is as slow as a
    // stalled connection. Expiry is a provider failure, not a caller abort, so the fallback runs.
    const timeoutMs = this.options.timeoutMs ?? JEV_DEFAULT_TIMEOUT_MS
    const deadline = AbortSignal.timeout(timeoutMs)
    const combined = signal !== undefined ? AbortSignal.any([signal, deadline]) : deadline
    const abortError = (error: unknown): DecisionError => signal?.aborted === true || !deadline.aborted
      ? new DecisionError('Jev evaluation aborted', 'DECISION_ABORTED', { cause: error })
      : new DecisionError(`Jev did not answer within ${String(timeoutMs)} ms`, 'DECISION_PROVIDER_ERROR', { cause: error })

    let response: Response
    try {
      response = await fetch(`${this.options.baseURL}${JEV_EVALUATION_PATH}`, {
        method: 'POST',
        redirect: 'error',
        headers: {
          'authorization': `Bearer ${apiKey}`,
          'content-type': 'application/json',
          'accept': 'application/json',
          'user-agent': USER_AGENT,
        },
        body: JSON.stringify({ model: this.options.model, state: request.state, questions }),
        signal: combined,
      })
    } catch (error: unknown) {
      if (isAbortError(error)) throw abortError(error)
      throw new DecisionError(`Jev evaluation request failed: ${String(error)}`, 'DECISION_PROVIDER_ERROR', { cause: error })
    }

    if (!response.ok) {
      const status = response.status
      let message = `TypeSafe API error (HTTP ${String(status)})`
      try {
        const parsed = await response.json() as JevResponse
        const detail = parsed.error?.message
        if (detail !== undefined && detail.length > 0) message = detail
      } catch (error: unknown) {
        // An abort mid-body must surface as DECISION_ABORTED: cancellation is not a provider error.
        if (isAbortError(error)) throw abortError(error)
        // Otherwise the HTTP status already sits in `message`; a non-JSON error body can only cost
        // a richer message, never the real error.
      }
      // TypeSafe rate-limits traffic per account; surfacing it distinctly lets a consumer fall back
      // to the LLM-backed provider rather than treat the decision as unanswerable.
      //
      // A 4xx other than 429 rejects this request itself — an unusable key (401), a malformed body,
      // or a state above the endpoint's token ceiling (400 `max_tokens_exceeded`). Retrying the
      // identical request on another provider cannot change that answer, and the configured
      // fallback loads a local model to reach the same failure, so the code marks the request
      // rather than the provider as the cause.
      const code =
        status === 429
          ? 'DECISION_RATE_LIMITED'
          : status >= 400 && status < 500
            ? 'DECISION_REQUEST_REJECTED'
            : 'DECISION_PROVIDER_ERROR'
      throw new DecisionError(message, code)
    }

    let payload: JevResponse
    try {
      payload = await response.json() as JevResponse
    } catch (error: unknown) {
      if (isAbortError(error)) throw abortError(error)
      throw new DecisionError(
        `Jev returned an unprocessable response body: ${String(error)}`,
        'DECISION_PROVIDER_ERROR',
        { cause: error },
      )
    }

    const returned = payload.answers
    if (returned === undefined) {
      throw new DecisionError('Jev response carried no answers', 'DECISION_PROVIDER_ERROR')
    }
    const answers: Record<string, DecisionAnswer> = {}
    for (const key of Object.keys(request.questions)) {
      const answer = returned[key]
      if (answer === undefined) {
        throw new DecisionError(`Jev returned no answer for question "${key}"`, 'DECISION_PROVIDER_ERROR')
      }
      answers[key] = fromJevAnswer(key, answer)
    }
    return { answers, providerId: this.id }
  }
}

/** True for a fetch/`AbortSignal` abort, surfaced as `DECISION_ABORTED`. */
function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && (error.name === 'AbortError' || error.name === 'TimeoutError')
}
