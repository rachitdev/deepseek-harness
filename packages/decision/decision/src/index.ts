/**
 * Service Definition for the decision capability seam (`ctx.decision`): a provider registry and
 * provider-selecting execution for typed, calibrated decisions. Duplicate ids are rejected. At
 * execution time a configured provider must exist and be usable; without one, exactly one usable
 * provider is required, so selection never depends on registration order.
 * @module @deepseek-ai/dsh-decision
 */

import { Context, Service } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import type {
  DecisionProvider,
  DecisionQuestion,
  DecisionRequest,
  DecisionResult,
} from './types.ts'
import { DecisionError } from './types.ts'

export {
  DecisionError,
} from './types.ts'
export type {
  DecisionAnswer,
  DecisionBooleanAnswer,
  DecisionBooleanQuestion,
  DecisionChoiceAnswer,
  DecisionChoiceQuestion,
  DecisionProvider,
  DecisionQuestion,
  DecisionRequest,
  DecisionResult,
  DecisionScoreAnswer,
  DecisionScoreQuestion,
} from './types.ts'
export * from './wire-question.ts'

declare module '@deepseek-ai/cordis' {
  interface Context {
    decision: DecisionRuntime
  }
}

/** Service interface alias for `DecisionRuntime`. */
export type DecisionService = DecisionRuntime

/** Lowest number of options a choice can decide between. */
const CHOICE_MIN_OPTIONS = 2

/** Rubric bounds a score question must stay inside; a provider cannot calibrate beyond them. */
const SCORE_MIN_LEVELS = 2
const SCORE_MAX_LEVELS = 10

/**
 * Config for the decision seam. `provider` pins which registered provider wins; omitting it
 * auto-selects when exactly one usable provider is registered. Operational overrides feed this same
 * field rather than introduce a hidden priority chain.
 */
export interface DecisionRuntimeConfig {
  /** Explicit provider id. Omitted = auto-select when exactly one usable. */
  readonly provider?: string
  /** Fallback provider id triggered on-demand if the primary provider fails to respond. */
  readonly fallbackProvider?: string
}

/**
 * The decision service. Registered as `ctx.decision` (one instance per context).
 *
 * Selection semantics (resolved at execution time, never order-dependent):
 * - A configured id that is registered and `available()` → that provider.
 * - A configured id not registered → `DECISION_PROVIDER_CONFIGURED_MISSING`, unless `fallbackProvider` is registered and usable.
 * - A configured id registered but unavailable → `DECISION_PROVIDER_CONFIGURED_UNAVAILABLE`,
 *   unless `fallbackProvider` is registered and usable.
 * - A configured fallback id is invoked only after the primary cannot answer, never on a successful primary call.
 * - No id configured, exactly one registered usable non-fallback provider → that provider.
 * - No id configured, multiple usable non-fallback providers → `DECISION_PROVIDER_AMBIGUOUS`.
 * - No id configured, no usable provider → `DECISION_PROVIDER_UNAVAILABLE`.
 */
export class DecisionRuntime extends Service {
  /**
   * Provider selection config. Operational env overrides feed the SAME fields:
   * `$DSH_DECISION_PROVIDER` is equivalent to `provider`, and `$DSH_DECISION_FALLBACK_PROVIDER`
   * is equivalent to `fallbackProvider`. Neither is a hidden priority chain.
   */
  static Config: z<DecisionRuntimeConfig> = z.object({
    provider: z.string(),
    fallbackProvider: z.string(),
  })

  private providers = new Map<string, DecisionProvider>()
  private readonly providerId: string | undefined
  private readonly fallbackProviderId: string | undefined

  constructor(ctx: Context, config: DecisionRuntimeConfig = {}) {
    super(ctx, 'decision')
    this.providerId = config.provider ?? process.env.DSH_DECISION_PROVIDER
    this.fallbackProviderId = config.fallbackProvider ?? process.env.DSH_DECISION_FALLBACK_PROVIDER
  }

  /**
   * Register a decision provider.
   * @param provider - the backend to register; its `id` must be unique within the seam.
   * @returns a disposer that removes this registration.
   */
  registerProvider(provider: DecisionProvider): () => void {
    if (this.providers.has(provider.id)) {
      throw new DecisionError(
        `decision provider "${provider.id}" is already registered`,
        'DECISION_DUPLICATE_PROVIDER',
      )
    }
    this.providers.set(provider.id, provider)
    return () => {
      if (this.providers.get(provider.id) === provider) this.providers.delete(provider.id)
    }
  }

  /**
   * Answer every question in one request through the selected provider, or trigger the fallback
   * provider on-demand if the primary fails to respond.
   * @param request - the state and the questions asked about it.
   * @param signal - optional cancellation for the provider call.
   * @returns the answers, keyed identically to `request.questions`.
   */
  async evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult> {
    validateRequest(request)

    const fallbackId = this.fallbackProviderId
    const fallback = fallbackId !== undefined ? this.providers.get(fallbackId) : undefined

    let provider: DecisionProvider
    try {
      provider = selectProvider(this.providerId, this.providers, fallbackId)
    } catch (selectionError: unknown) {
      if (fallback !== undefined && fallback.available()) {
        return await fallback.evaluate(request, signal)
      }
      throw selectionError
    }

    if (fallback === undefined || !fallback.available() || provider.id === fallback.id) {
      return await provider.evaluate(request, signal)
    }

    try {
      return await provider.evaluate(request, signal)
    } catch (primaryError: unknown) {
      if (signal?.aborted || isAbortError(primaryError)) {
        throw primaryError
      }
      if (isRequestRejection(primaryError)) {
        throw primaryError
      }
      return await fallback.evaluate(request, signal)
    }
  }
}

/**
 * Reject a request a provider could not answer meaningfully. Validation lives here rather than in
 * each provider so every backend rejects the same malformed request identically, and so a consumer
 * building questions from runtime data (page elements, configured model tiers) fails at the seam
 * with one code instead of inside a vendor's error vocabulary.
 * @param request - the request to check.
 */
function validateRequest(request: DecisionRequest): void {
  const entries = Object.entries(request.questions)
  if (entries.length === 0) {
    throw new DecisionError('a decision request must carry at least one question', 'DECISION_REQUEST_INVALID')
  }
  for (const [key, question] of entries) validateQuestion(key, question)
}

/**
 * Check one question against the bounds its arm can be calibrated within.
 * @param key - the question's key in the request, used to locate the failure.
 * @param question - the question to check.
 */
function validateQuestion(key: string, question: DecisionQuestion): void {
  switch (question.type) {
    case 'choice': {
      const options = Object.keys(question.criteria).length
      if (options < CHOICE_MIN_OPTIONS) {
        throw new DecisionError(
          `choice question "${key}" needs at least ${String(CHOICE_MIN_OPTIONS)} options, got ${String(options)}`,
          'DECISION_REQUEST_INVALID',
        )
      }
      return
    }
    case 'score': {
      const levels = question.criteria.length
      if (levels < SCORE_MIN_LEVELS || levels > SCORE_MAX_LEVELS) {
        throw new DecisionError(
          `score question "${key}" needs ${String(SCORE_MIN_LEVELS)}-${String(SCORE_MAX_LEVELS)} levels, got ${String(levels)}`,
          'DECISION_REQUEST_INVALID',
        )
      }
      return
    }
    case 'boolean':
      return
    default:
      return assertNever(question)
  }
}

/**
 * Resolve which provider serves this call.
 * @param configuredId - the pinned provider id, when one is configured.
 * @param providers - every registered provider.
 * @returns the provider that must serve the call.
 */
function selectProvider(
  configuredId: string | undefined,
  providers: ReadonlyMap<string, DecisionProvider>,
  fallbackId?: string,
): DecisionProvider {
  if (configuredId !== undefined) {
    const provider = providers.get(configuredId)
    if (provider === undefined) {
      throw new DecisionError(
        `configured decision provider "${configuredId}" is not registered`,
        'DECISION_PROVIDER_CONFIGURED_MISSING',
      )
    }
    if (!provider.available()) {
      throw new DecisionError(
        `configured decision provider "${configuredId}" is registered but unavailable`,
        'DECISION_PROVIDER_CONFIGURED_UNAVAILABLE',
      )
    }
    return provider
  }
  const usable = [...providers.values()].filter(provider => provider.available() && provider.id !== fallbackId)
  const [single] = usable
  if (single === undefined) {
    throw new DecisionError('no usable decision provider is registered', 'DECISION_PROVIDER_UNAVAILABLE')
  }
  if (usable.length > 1) {
    const ids = usable.map(provider => provider.id).join(', ')
    throw new DecisionError(
      `multiple usable decision providers are registered (${ids}); configure one explicitly`,
      'DECISION_PROVIDER_AMBIGUOUS',
    )
  }
  return single
}

/**
 * True when the primary rejected the request itself rather than failing to serve it. A fallback
 * receives the identical request, so it would reach the identical rejection after paying its own
 * startup cost; the rejection reaches the consumer instead.
 * @param error - the error the primary provider threw.
 * @returns whether the failure is a property of the request.
 */
function isRequestRejection(error: unknown): boolean {
  return error instanceof DecisionError && error.code === 'DECISION_REQUEST_REJECTED'
}

/** True for an aborted decision request, surfaced as DECISION_ABORTED or DOMException. */
function isAbortError(error: unknown): boolean {
  if (error instanceof DecisionError && error.code === 'DECISION_ABORTED') return true
  return error instanceof DOMException && error.name === 'AbortError'
}

/**
 * Exhaustiveness guard for the closed question union.
 * @param value - the arm TypeScript proved unreachable.
 * @returns never; always throws.
 */
function assertNever(value: never): never {
  throw new DecisionError(
    `unhandled decision question type: ${JSON.stringify(value)}`,
    'DECISION_REQUEST_INVALID',
  )
}

export default DecisionRuntime
