import { afterEach, describe, expect, it, vi } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, { type DecisionRequest } from '@deepseek-ai/dsh-decision'
import * as jevPlugin from '@deepseek-ai/dsh-decision-jev'
import type { JevAnswer } from '@deepseek-ai/dsh-decision-jev/src/types.ts'
import {
  JEV_DEFAULT_BASE_URL,
  JEV_DEFAULT_TIMEOUT_MS,
  JEV_EVALUATION_PATH,
  JevDecisionProvider,
  fromJevAnswer,
  toJevQuestion,
} from '@deepseek-ai/dsh-decision-jev'

const request: DecisionRequest = {
  state: 'search results page',
  questions: {
    element: { type: 'choice', instructions: 'Which element?', criteria: { a: 'first', b: 'second' } },
    severity: { type: 'score', instructions: 'How bad?', criteria: ['fine', 'bad', 'critical'] },
    done: { type: 'boolean', instructions: 'Is the task complete?' },
  },
}

const fullAnswers = {
  element: { type: 'choice', choice: 'a', probabilities: { a: 0.9, b: 0.1 } },
  severity: { type: 'score', score: 1.5, probabilities: { '0': 0.1, '1': 0.4, '2': 0.5 } },
  done: { type: 'noul', noul: 0.2 },
} satisfies Record<string, JevAnswer>

/** Build a provider whose key and endpoint are already resolved. */
function makeProvider(overrides: Partial<ConstructorParameters<typeof JevDecisionProvider>[0]> = {}) {
  return new JevDecisionProvider({
    resolveApiKey: () => Promise.resolve('key'),
    hasApiKey: true,
    baseURL: JEV_DEFAULT_BASE_URL,
    model: 'jev-latest',
    ...overrides,
  })
}

/**
 * The request the provider sent through the stubbed `fetch`.
 * @param spy - the scripted `fetch` installed for this test.
 * @returns the requested URL and the init the provider passed with it.
 */
function firstRequest(spy: ReturnType<typeof stubFetch>): { url: string; init: RequestInit } {
  const [call] = spy.mock.calls
  const [input, init] = call ?? []
  if (init === undefined) throw new Error('the provider never sent a request')
  const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input?.url
  if (url === undefined) throw new Error('the provider sent a request without a URL')
  return { url, init }
}

/** Install a scripted `fetch`, returning the spy so a test can assert the request it sent. */
function stubFetch(response: Response | Error) {
  const spy = vi.fn<typeof fetch>(() => response instanceof Error ? Promise.reject(response) : Promise.resolve(response))
  vi.stubGlobal('fetch', spy)
  return spy
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('Jev question translation', () => {
  it('sends a choice option map under criteria', () => {
    expect(toJevQuestion({ type: 'choice', instructions: 'i', criteria: { a: 'A', b: 'B' } }))
      .toEqual({ type: 'choice', instructions: 'i', criteria: { a: 'A', b: 'B' } })
  })

  it('sends score levels as an ordered array under criteria', () => {
    expect(toJevQuestion({ type: 'score', instructions: 'i', criteria: ['low', 'high'] }))
      .toEqual({ type: 'score', instructions: 'i', criteria: ['low', 'high'] })
  })

  it('translates a boolean question to a TypeSafe noul', () => {
    expect(toJevQuestion({ type: 'boolean', instructions: 'i' }))
      .toEqual({ type: 'noul', instructions: 'i' })
  })

  it('refuses an unknown question type', () => {
    expect(() => toJevQuestion({ type: 'ranking', instructions: 'i' } as never))
      .toThrow(expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error)
  })
})

describe('Jev answer translation', () => {
  it('maps each complete answer arm', () => {
    expect(fromJevAnswer('k', fullAnswers.element)).toEqual({ type: 'choice', choice: 'a', probabilities: { a: 0.9, b: 0.1 } })
    expect(fromJevAnswer('k', fullAnswers.severity)).toMatchObject({ type: 'score', score: 1.5 })
    expect(fromJevAnswer('k', fullAnswers.done)).toEqual({ type: 'boolean', probability: 0.2 })
  })

  it.each([
    ['choice missing its distribution', { type: 'choice' as const, choice: 'a' }],
    ['choice missing its pick', { type: 'choice' as const, probabilities: { a: 1 } }],
    ['score missing its distribution', { type: 'score' as const, score: 1 }],
    ['score missing its position', { type: 'score' as const, probabilities: { '0': 1 } }],
    ['noul missing its probability', { type: 'noul' as const }],
    ['an unknown answer type', { type: 'ranking' } as never],
  ])('refuses to fabricate a value for %s', (_label, answer) => {
    expect(() => fromJevAnswer('k', answer))
      .toThrow(expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error)
  })
})

describe('JevDecisionProvider availability', () => {
  it.each([
    ['no key', { hasApiKey: false }],
    ['an unparseable endpoint', { baseURL: 'not a url' }],
    ['an empty model id', { model: '' }],
  ])('is unavailable with %s', (_label, overrides) => {
    expect(makeProvider(overrides).available()).toBe(false)
  })

  it('is available when key, endpoint, and model are all present', () => {
    expect(makeProvider().available()).toBe(true)
  })
})

describe('JevDecisionProvider evaluation', () => {
  it('posts to the direct System One endpoint and maps every answer back', async () => {
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    const result = await makeProvider().evaluate(request)

    const { url, init } = firstRequest(spy)
    expect(url).toBe(`${JEV_DEFAULT_BASE_URL}${JEV_EVALUATION_PATH}`)
    expect(init.redirect).toBe('error')
    expect((init.headers as Record<string, string>)['authorization']).toBe('Bearer key')
    expect((init.headers as Record<string, string>)['ai-model-id']).toBeUndefined()
    expect(JSON.parse(init.body as string) as Record<string, unknown>).toEqual({
      model: 'jev-latest',
      state: request.state,
      questions: expect.any(Object) as object,
    })
    expect(result).toMatchObject({
      providerId: 'jev',
      answers: { element: { choice: 'a' }, severity: { score: 1.5 }, done: { probability: 0.2 } },
    })
  })

  it('fails when no key resolves', async () => {
    stubFetch(jsonResponse({ answers: fullAnswers }))
    await expect(makeProvider({ resolveApiKey: () => Promise.resolve(undefined) }).evaluate(request))
      .rejects.toThrow(expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error)
  })

  it('reports a rate limit distinctly so a consumer can fall back', async () => {
    stubFetch(jsonResponse({ error: { message: 'Free tier requests on this model are rate-limited.' } }, 429))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_RATE_LIMITED' }) as Error,
    )
  })

  it('surfaces the TypeSafe error message on a non-rate-limit failure', async () => {
    stubFetch(jsonResponse({ error: { message: 'invalid_api_key' } }, 401))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(/invalid_api_key/)
  })

  it('marks an unusable key as a request rejection so no fallback repeats it', async () => {
    stubFetch(jsonResponse({ error: { message: 'invalid_api_key' } }, 401))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
  })

  it('marks a state above the endpoint token ceiling as a request rejection', async () => {
    stubFetch(jsonResponse({ error: { message: 'max_tokens_exceeded' } }, 400))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
  })

  it('keeps a server-side failure a provider error so a fallback still runs', async () => {
    stubFetch(jsonResponse({ error: { message: 'upstream unavailable' } }, 503))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('falls back to the status code when the error body is not JSON', async () => {
    stubFetch(new Response('TypeSafe down', { status: 502 }))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(/HTTP 502/)
  })

  it('reports an unparseable success body as a provider error', async () => {
    stubFetch(new Response('not json', { status: 200, headers: { 'content-type': 'application/json' } }))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('rejects a response carrying no answers at all', async () => {
    stubFetch(jsonResponse({}))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(/carried no answers/)
  })

  it('rejects a response missing one of the questions asked', async () => {
    stubFetch(jsonResponse({ answers: { element: fullAnswers.element } }))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(/no answer for question "severity"/)
  })

  it('reports a transport failure as a provider error', async () => {
    stubFetch(new TypeError('connection refused'))
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it.each([
    ['during the request', () => stubFetch(new DOMException('aborted', 'AbortError'))],
    ['while reading an error body', () => {
      const response = new Response('x', { status: 500 })
      vi.spyOn(response, 'json').mockRejectedValue(new DOMException('aborted', 'AbortError'))
      return stubFetch(response)
    }],
    ['while reading a success body', () => {
      const response = new Response('x', { status: 200 })
      vi.spyOn(response, 'json').mockRejectedValue(new DOMException('aborted', 'AbortError'))
      return stubFetch(response)
    }],
  ])('reports cancellation %s as an abort, not a provider error', async (_label, arrange) => {
    arrange()
    await expect(makeProvider().evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error,
    )
  })

  it('forwards a cancellation signal to fetch', async () => {
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    const controller = new AbortController()
    await makeProvider().evaluate(request, controller.signal)
    const { init } = firstRequest(spy)
    expect(init.signal?.aborted).toBe(false)
    controller.abort()
    expect(init.signal?.aborted).toBe(true)
  })

  /** Install a `fetch` that never settles until its signal aborts, like a stalled upstream. */
  function stubHangingFetch() {
    vi.stubGlobal('fetch', vi.fn((_url: string, init: RequestInit) => new Promise((_resolve, reject) => {
      const signal = init.signal
      if (signal?.aborted === true) {
        reject(signal.reason as Error)
        return
      }
      signal?.addEventListener('abort', () => { reject(signal.reason as Error) })
    })))
  }

  it('fails a call still pending at the deadline as a provider error, so the fallback runs', async () => {
    stubHangingFetch()
    const started = performance.now()
    await expect(makeProvider({ timeoutMs: 30 }).evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR', message: 'Jev did not answer within 30 ms' }) as Error,
    )
    expect(performance.now() - started).toBeLessThan(1_000)
  })

  it('keeps a caller cancellation an abort even when a deadline is configured', async () => {
    stubHangingFetch()
    const controller = new AbortController()
    const pending = makeProvider({ timeoutMs: 5_000 }).evaluate(request, controller.signal)
    controller.abort()
    await expect(pending).rejects.toThrow(expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error)
  })

  it('defaults the deadline to three seconds', () => {
    expect(JEV_DEFAULT_TIMEOUT_MS).toBe(3_000)
  })
})

describe('decision-jev plugin', () => {
  it('registers on the decision seam and is removed when the fiber disposes', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', 'env-key')
    stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      const fiber = await ctx.plugin(jevPlugin, {})
      await expect(ctx.decision.evaluate(request)).resolves.toMatchObject({ providerId: 'jev' })
      await fiber.dispose()
      await expect(ctx.decision.evaluate(request)).rejects.toThrow(
        expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
      )
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('registers an unavailable provider when no key is reachable', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', '')
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      await ctx.plugin(jevPlugin, {})
      await expect(ctx.decision.evaluate(request)).rejects.toThrow(
        expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
      )
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('uses a literal key from config ahead of the environment', async () => {
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    await ctx.plugin(jevPlugin, { apiKey: 'literal-key' })
    await ctx.decision.evaluate(request)
    const { init } = firstRequest(spy)
    expect((init.headers as Record<string, string>)['authorization']).toBe('Bearer literal-key')
  })
})

describe('decision-jev credential and endpoint resolution', () => {
  it('resolves the key through the credentials seam when one is mounted', async () => {
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    const ctx = new Context()
    ctx.provide('credentials', {
      resolve: (ref: string) => Promise.resolve({ ref, value: 'sealed-key' }),
    } as never)
    await ctx.plugin(DecisionRuntime, {})
    await ctx.plugin(jevPlugin, {})
    await ctx.decision.evaluate(request)
    const { init } = firstRequest(spy)
    expect((init.headers as Record<string, string>)['authorization']).toBe('Bearer sealed-key')
  })

  it('treats an empty literal key as absent and falls back to the environment', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', 'env-key')
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      await ctx.plugin(jevPlugin, { apiKey: '' })
      await ctx.decision.evaluate(request)
      const { init } = firstRequest(spy)
      expect((init.headers as Record<string, string>)['authorization']).toBe('Bearer env-key')
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('reports no key when the credentials seam resolves nothing', async () => {
    stubFetch(jsonResponse({ answers: fullAnswers }))
    vi.stubEnv('TYPESAFE_API_KEY', 'present-so-available')
    try {
      const ctx = new Context()
      ctx.provide('credentials', { resolve: () => Promise.resolve(undefined) } as never)
      await ctx.plugin(DecisionRuntime, {})
      await ctx.plugin(jevPlugin, {})
      await expect(ctx.decision.evaluate(request)).rejects.toThrow(/no API key/)
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('takes the endpoint from the environment when config omits it', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', 'env-key')
    vi.stubEnv('TYPESAFE_BASE_URL', 'https://typesafe.example')
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      await ctx.plugin(jevPlugin, {})
      await ctx.decision.evaluate(request)
      const { url } = firstRequest(spy)
      expect(url).toBe(`https://typesafe.example${JEV_EVALUATION_PATH}`)
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('honors an explicitly configured endpoint, model, and key reference', async () => {
    vi.stubEnv('CUSTOM_TYPESAFE_KEY', 'ref-key')
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      await ctx.plugin(jevPlugin, {
        apiKeyEnv: 'CUSTOM_TYPESAFE_KEY',
        baseURL: 'https://pinned.example',
        model: 'jev-pinned',
      })
      await ctx.decision.evaluate(request)
      const { url, init } = firstRequest(spy)
      expect(url).toBe(`https://pinned.example${JEV_EVALUATION_PATH}`)
      expect(JSON.parse(init.body as string)).toMatchObject({ model: 'jev-pinned' })
    } finally {
      vi.unstubAllEnvs()
    }
  })
})

describe('decision-jev defaults without schema filling', () => {
  // The Loader applies `Config`'s defaults, but `apply` is also reachable directly (a composition
  // that constructs the plugin itself). These cover the in-code fallbacks that stand in for the
  // schema so a direct caller never gets an undefined endpoint, model, or credential reference.
  it('falls back to the packaged endpoint, model, and key reference', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', 'env-key')
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      jevPlugin.apply(ctx, {})
      await ctx.decision.evaluate(request)
      const { url, init } = firstRequest(spy)
      expect(url).toBe(`${JEV_DEFAULT_BASE_URL}${JEV_EVALUATION_PATH}`)
      expect(JSON.parse(init.body as string)).toMatchObject({ model: 'jev-latest' })
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('lets TYPESAFE_MODEL win after schema resolution when model is unset', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', 'env-key')
    vi.stubEnv('TYPESAFE_MODEL', 'jev-dynamic-latest')
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const resolved = jevPlugin.Config({})
      expect(resolved.model).toBeUndefined()
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      jevPlugin.apply(ctx, resolved)
      await ctx.decision.evaluate(request)
      const body = spy.mock.calls[0]?.[1]?.body
      expect(typeof body).toBe('string')
      if (typeof body !== 'string') return
      expect(JSON.parse(body)).toMatchObject({ model: 'jev-dynamic-latest' })
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('respects TYPESAFE_MODEL environment override when model is not explicitly configured', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', 'env-key')
    vi.stubEnv('TYPESAFE_MODEL', 'jev-dynamic-latest')
    const spy = stubFetch(jsonResponse({ answers: fullAnswers }))
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      jevPlugin.apply(ctx, {})
      await ctx.decision.evaluate(request)
      const { init } = firstRequest(spy)
      expect(JSON.parse(init.body as string)).toMatchObject({ model: 'jev-dynamic-latest' })
    } finally {
      vi.unstubAllEnvs()
    }
  })

  it('reports no key when neither a literal, a seam, nor the environment supplies one', async () => {
    vi.stubEnv('TYPESAFE_API_KEY', '')
    try {
      const ctx = new Context()
      await ctx.plugin(DecisionRuntime, {})
      jevPlugin.apply(ctx, {})
      await expect(ctx.decision.evaluate(request)).rejects.toThrow(
        expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
      )
    } finally {
      vi.unstubAllEnvs()
    }
  })
})

describe('decision-jev key presence edge cases', () => {
  it('treats an entirely absent environment variable as no key', async () => {
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    jevPlugin.apply(ctx, { apiKeyEnv: 'DSH_DECISION_JEV_ABSENT_KEY_FOR_TEST' })
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
    )
  })

  it('fails loudly when the key disappears between the availability check and the call', async () => {
    // `available()` samples synchronously while `resolveApiKey` reads per call, so a key revoked in
    // between must surface as a loud provider error rather than an unauthenticated request.
    stubFetch(jsonResponse({ answers: fullAnswers }))
    vi.stubEnv('TYPESAFE_API_KEY', 'present-at-mount')
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    jevPlugin.apply(ctx, {})
    vi.unstubAllEnvs()
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(/no API key/)
  })
})
