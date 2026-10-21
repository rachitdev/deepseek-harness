import { describe, expect, it, vi } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, {
  DecisionError,
  type DecisionProvider,
  type DecisionRequest,
  type DecisionResult,
} from '@deepseek-ai/dsh-decision'

const available = true
const unavailable = false

/** A scripted provider for contract tests. */
function makeProvider(
  id: string,
  usable: boolean,
  evaluate: (request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult> = () =>
    Promise.resolve({ answers: {}, providerId: id }),
): DecisionProvider {
  return { id, available: () => usable, evaluate }
}

/** Mount a DecisionRuntime on a fresh root context with the given config. */
async function mountDecision(
  config: ConstructorParameters<typeof DecisionRuntime>[1] = {},
): Promise<{ ctx: Context; decision: DecisionRuntime }> {
  const ctx = new Context()
  await ctx.plugin(DecisionRuntime, config)
  return { ctx, decision: ctx.decision }
}

/** A minimal valid request, so selection tests are not also validation tests. */
const booleanRequest: DecisionRequest = {
  state: 'the deploy failed twice',
  questions: { urgent: { type: 'boolean', instructions: 'Is this urgent?' } },
}

describe('DecisionRuntime registration', () => {
  it('registers a provider and removes it via the returned disposer', async () => {
    const { decision } = await mountDecision()
    const dispose = decision.registerProvider(makeProvider('one', available))
    await expect(decision.evaluate(booleanRequest)).resolves.toMatchObject({ providerId: 'one' })
    dispose()
    await expect(decision.evaluate(booleanRequest)).rejects.toThrow(DecisionError)
  })

  it('rejects a duplicate provider id', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('dup', available))
    expect(() => decision.registerProvider(makeProvider('dup', available)))
      .toThrow(expect.objectContaining({ code: 'DECISION_DUPLICATE_PROVIDER' }) as Error)
  })

  it('leaves a replacement registration intact when an older disposer runs', async () => {
    const { decision } = await mountDecision()
    const dispose = decision.registerProvider(makeProvider('same', available))
    dispose()
    const replacement = makeProvider('same', available, () =>
      Promise.resolve({ answers: {}, providerId: 'replacement' }))
    decision.registerProvider(replacement)
    dispose()
    await expect(decision.evaluate(booleanRequest)).resolves.toMatchObject({ providerId: 'replacement' })
  })
})

describe('DecisionRuntime provider selection', () => {
  it('auto-selects the only usable provider', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('only', available))
    decision.registerProvider(makeProvider('down', unavailable))
    await expect(decision.evaluate(booleanRequest)).resolves.toMatchObject({ providerId: 'only' })
  })

  it('uses the configured provider rather than registration order', async () => {
    const { decision } = await mountDecision({ provider: 'second' })
    decision.registerProvider(makeProvider('first', available))
    decision.registerProvider(makeProvider('second', available))
    await expect(decision.evaluate(booleanRequest)).resolves.toMatchObject({ providerId: 'second' })
  })

  it('fails when the configured provider is not registered', async () => {
    const { decision } = await mountDecision({ provider: 'absent' })
    decision.registerProvider(makeProvider('present', available))
    await expect(decision.evaluate(booleanRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_CONFIGURED_MISSING' }) as Error,
    )
  })

  it('fails when the configured provider is registered but unavailable', async () => {
    const { decision } = await mountDecision({ provider: 'down' })
    decision.registerProvider(makeProvider('down', unavailable))
    await expect(decision.evaluate(booleanRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_CONFIGURED_UNAVAILABLE' }) as Error,
    )
  })

  it('refuses to guess between multiple usable providers', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('a', available))
    decision.registerProvider(makeProvider('b', available))
    await expect(decision.evaluate(booleanRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_AMBIGUOUS' }) as Error,
    )
  })

  it('fails when no usable provider is registered', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('down', unavailable))
    await expect(decision.evaluate(booleanRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
    )
  })

  it('reads the provider id from the environment when config omits it', async () => {
    vi.stubEnv('DSH_DECISION_PROVIDER', 'from-env')
    try {
      const { decision } = await mountDecision()
      decision.registerProvider(makeProvider('from-env', available))
      decision.registerProvider(makeProvider('other', available))
      await expect(decision.evaluate(booleanRequest)).resolves.toMatchObject({ providerId: 'from-env' })
    } finally {
      vi.unstubAllEnvs()
    }
  })
})

describe('DecisionRuntime request validation', () => {
  it('rejects a request carrying no questions', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    await expect(decision.evaluate({ state: 's', questions: {} })).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_INVALID' }) as Error,
    )
  })

  it('rejects a choice with fewer than two options', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    await expect(decision.evaluate({
      state: 's',
      questions: { pick: { type: 'choice', instructions: 'i', criteria: { only: 'the only option' } } },
    })).rejects.toThrow(expect.objectContaining({ code: 'DECISION_REQUEST_INVALID' }) as Error)
  })

  it('accepts a choice with two options', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    await expect(decision.evaluate({
      state: 's',
      questions: { pick: { type: 'choice', instructions: 'i', criteria: { a: 'first', b: 'second' } } },
    })).resolves.toMatchObject({ providerId: 'p' })
  })

  it('rejects a score rubric with too few levels', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    await expect(decision.evaluate({
      state: 's',
      questions: { rate: { type: 'score', instructions: 'i', criteria: ['only'] } },
    })).rejects.toThrow(expect.objectContaining({ code: 'DECISION_REQUEST_INVALID' }) as Error)
  })

  it('rejects a score rubric with too many levels', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    await expect(decision.evaluate({
      state: 's',
      questions: {
        rate: {
          type: 'score',
          instructions: 'i',
          criteria: Array.from({ length: 11 }, (_unused, index) => `level ${String(index)}`),
        },
      },
    })).rejects.toThrow(expect.objectContaining({ code: 'DECISION_REQUEST_INVALID' }) as Error)
  })

  it('accepts a score rubric at both bounds', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    for (const length of [2, 10]) {
      await expect(decision.evaluate({
        state: 's',
        questions: {
          rate: {
            type: 'score',
            instructions: 'i',
            criteria: Array.from({ length }, (_unused, index) => `level ${String(index)}`),
          },
        },
      })).resolves.toMatchObject({ providerId: 'p' })
    }
  })

  it('rejects an unknown question type', async () => {
    const { decision } = await mountDecision()
    decision.registerProvider(makeProvider('p', available))
    const request = { state: 's', questions: { odd: { type: 'ranking', instructions: 'i' } } } as const
    // @ts-expect-error -- a request built from runtime data can carry a question type the union does not declare.
    await expect(decision.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_INVALID' }) as Error,
    )
  })

  it('validates before selecting a provider, so a bad request fails the same way with none registered', async () => {
    const { decision } = await mountDecision()
    await expect(decision.evaluate({ state: 's', questions: {} })).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_INVALID' }) as Error,
    )
  })
})

describe('DecisionRuntime execution', () => {
  it('passes the request and cancellation signal through to the provider', async () => {
    const { decision } = await mountDecision()
    const evaluate = vi.fn<(request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult>>(
      () => Promise.resolve({ answers: {}, providerId: 'spy' }),
    )
    decision.registerProvider(makeProvider('spy', available, evaluate))
    const controller = new AbortController()
    await decision.evaluate(booleanRequest, controller.signal)
    expect(evaluate).toHaveBeenCalledWith(booleanRequest, controller.signal)
  })

  it('surfaces a provider failure unchanged', async () => {
    const { decision } = await mountDecision()
    const failure = new DecisionError('upstream is down', 'DECISION_PROVIDER_ERROR')
    decision.registerProvider(makeProvider('bad', available, () => Promise.reject(failure)))
    await expect(decision.evaluate(booleanRequest)).rejects.toBe(failure)
  })
})

describe('DecisionRuntime fallback provider', () => {
  it('does not invoke fallback when primary succeeds', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    const fallbackFn = vi.fn()
    decision.registerProvider(makeProvider('primary', available, () => Promise.resolve({ answers: {}, providerId: 'primary' })))
    decision.registerProvider(makeProvider('fallback', available, fallbackFn))
    const result = await decision.evaluate(booleanRequest)
    expect(result.providerId).toBe('primary')
    expect(fallbackFn).not.toHaveBeenCalled()
  })

  it('triggers fallback on-demand when primary throws DECISION_RATE_LIMITED', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    decision.registerProvider(makeProvider('primary', available, () => Promise.reject(new DecisionError('Rate limit', 'DECISION_RATE_LIMITED'))))
    decision.registerProvider(makeProvider('fallback', available, () => Promise.resolve({ answers: {}, providerId: 'fallback' })))
    const result = await decision.evaluate(booleanRequest)
    expect(result.providerId).toBe('fallback')
  })

  it('triggers fallback on-demand when primary is unavailable', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    decision.registerProvider(makeProvider('primary', unavailable))
    decision.registerProvider(makeProvider('fallback', available, () => Promise.resolve({ answers: {}, providerId: 'fallback' })))
    const result = await decision.evaluate(booleanRequest)
    expect(result.providerId).toBe('fallback')
  })

  it('does not trigger fallback when request is aborted', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    const fallbackFn = vi.fn()
    const controller = new AbortController()
    controller.abort()
    decision.registerProvider(makeProvider('primary', available, () => Promise.reject(new DecisionError('aborted', 'DECISION_ABORTED'))))
    decision.registerProvider(makeProvider('fallback', available, fallbackFn))
    await expect(decision.evaluate(booleanRequest, controller.signal)).rejects.toThrow(DecisionError)
    expect(fallbackFn).not.toHaveBeenCalled()
  })

  it('throws primary error when fallback is also unavailable', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    const primaryErr = new DecisionError('Network error', 'DECISION_PROVIDER_ERROR')
    decision.registerProvider(makeProvider('primary', available, () => Promise.reject(primaryErr)))
    decision.registerProvider(makeProvider('fallback', unavailable))
    await expect(decision.evaluate(booleanRequest)).rejects.toBe(primaryErr)
  })

  it('does not trigger fallback when the primary rejected the request itself', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    const fallbackFn = vi.fn()
    const rejection = new DecisionError('max_tokens_exceeded', 'DECISION_REQUEST_REJECTED')
    decision.registerProvider(makeProvider('primary', available, () => Promise.reject(rejection)))
    decision.registerProvider(makeProvider('fallback', available, fallbackFn))
    await expect(decision.evaluate(booleanRequest)).rejects.toBe(rejection)
    expect(fallbackFn).not.toHaveBeenCalled()
  })

  it('still triggers fallback when the primary failed to serve an otherwise valid request', async () => {
    const { decision } = await mountDecision({ provider: 'primary', fallbackProvider: 'fallback' })
    decision.registerProvider(makeProvider('primary', available, () => Promise.reject(new DecisionError('502 upstream', 'DECISION_PROVIDER_ERROR'))))
    decision.registerProvider(makeProvider('fallback', available, () => Promise.resolve({ answers: {}, providerId: 'fallback' })))
    const result = await decision.evaluate(booleanRequest)
    expect(result.providerId).toBe('fallback')
  })
})
