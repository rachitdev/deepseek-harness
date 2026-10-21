/**
 * Parity test suite between JEV and Laya decision providers.
 *
 * Verifies that:
 * 1. Wire question translation is symmetric across all question types (choice, score, boolean -> noul).
 * 2. Wire answer translation is symmetric across all answer arms (choice, score, noul -> boolean).
 * 3. Error classification is identical:
 *    - HTTP 429 -> DECISION_RATE_LIMITED (triggers failover)
 *    - HTTP 400/401 -> DECISION_REQUEST_REJECTED (does NOT trigger failover)
 *    - HTTP 500/503 / network errors -> DECISION_PROVIDER_ERROR (triggers failover)
 *    - AbortController / AbortSignal -> DECISION_ABORTED (never misreported as provider failure)
 * 4. Response structure enforcement is identical (missing answers, missing question keys).
 * 5. End-to-end failover integration under DecisionRuntime works seamlessly:
 *    - When JEV succeeds, Laya is never invoked.
 *    - When JEV fails with 5xx / 429 / transport error, Laya automatically handles the decision.
 *    - When JEV rejects the request itself (4xx), Laya is not invoked to prevent futile model loading.
 */

import { afterEach, describe, expect, it, vi } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, {
  type DecisionQuestion,
  type DecisionRequest,
} from '@deepseek-ai/dsh-decision'
import {
  fromJevAnswer,
  JEV_DEFAULT_BASE_URL,
  JEV_PROVIDER_ID,
  JevDecisionProvider,
  toJevQuestion,
} from '@deepseek-ai/dsh-decision-jev'
import {
  fromLayaAnswer,
  LAYA_PROVIDER_ID,
  LayaDecisionProvider,
  toLayaQuestion,
} from '../src/provider.ts'

const sampleRequest: DecisionRequest = {
  state: 'Search results: item 1 is matching cable, item 2 is unrelated adapter',
  questions: {
    target: {
      type: 'choice',
      instructions: 'Which item to click?',
      criteria: { '1': 'matching cable', '2': 'unrelated adapter' },
    },
    risk: {
      type: 'score',
      instructions: 'Action risk level',
      criteria: ['safe', 'moderate', 'dangerous'],
    },
    completed: {
      type: 'boolean',
      instructions: 'Is the task complete?',
    },
  },
}

const mockWireAnswers = {
  target: {
    type: 'choice' as const,
    choice: '1',
    probabilities: { '1': 0.88, '2': 0.12 },
  },
  risk: {
    type: 'score' as const,
    score: 0.3,
    probabilities: { '0': 0.75, '1': 0.2, '2': 0.05 },
  },
  completed: {
    type: 'noul' as const,
    noul: 0.15,
  },
}

function stubFetch(handler: (req: Request) => Response | Promise<Response>): ReturnType<typeof vi.fn> {
  const spy = vi.fn((input: RequestInfo | URL, init?: RequestInit) => {
    const req = input instanceof Request ? input : new Request(input, init)
    return Promise.resolve(handler(req))
  })
  vi.stubGlobal('fetch', spy)
  return spy
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

function makeJevProvider(overrides: Partial<ConstructorParameters<typeof JevDecisionProvider>[0]> = {}) {
  return new JevDecisionProvider({
    resolveApiKey: () => Promise.resolve('test-key'),
    hasApiKey: true,
    baseURL: JEV_DEFAULT_BASE_URL,
    model: 'jev-latest',
    ...overrides,
  })
}

function makeLayaProvider(overrides: Partial<ConstructorParameters<typeof LayaDecisionProvider>[0]> = {}) {
  return new LayaDecisionProvider({
    pythonPath: process.execPath,
    workerScriptPath: process.execPath,
    host: '127.0.0.1',
    port: 58085,
    idleTimeoutSeconds: 300,
    startupTimeoutMs: 10000,
    healthTimeoutMs: 1000,
    startupPollIntervalMs: 200,
    baseURL: 'http://127.0.0.1:58085',
    model: 'convaiinnovations/laya',
    modelSubfolder: 'typed-decisions',
    available: () => true,
    ...overrides,
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('JEV and Laya: Question Translation Parity', () => {
  it('translates choice questions identically to wire criteria dict', () => {
    const q: DecisionQuestion = {
      type: 'choice',
      instructions: 'Pick element',
      criteria: { optA: 'Option A', optB: 'Option B' },
    }
    const jevWire = toJevQuestion(q)
    const layaWire = toLayaQuestion(q)

    expect(jevWire).toEqual(layaWire)
    expect(jevWire).toEqual({
      type: 'choice',
      instructions: 'Pick element',
      criteria: { optA: 'Option A', optB: 'Option B' },
    })
  })

  it('translates score questions identically to wire criteria list', () => {
    const q: DecisionQuestion = {
      type: 'score',
      instructions: 'Rate difficulty',
      criteria: ['easy', 'medium', 'hard'],
    }
    const jevWire = toJevQuestion(q)
    const layaWire = toLayaQuestion(q)

    expect(jevWire).toEqual(layaWire)
    expect(jevWire).toEqual({
      type: 'score',
      instructions: 'Rate difficulty',
      criteria: ['easy', 'medium', 'hard'],
    })
  })

  it('translates boolean questions identically to noul wire vocabulary', () => {
    const q: DecisionQuestion = {
      type: 'boolean',
      instructions: 'Is valid?',
    }
    const jevWire = toJevQuestion(q)
    const layaWire = toLayaQuestion(q)

    expect(jevWire).toEqual(layaWire)
    expect(jevWire).toEqual({
      type: 'noul',
      instructions: 'Is valid?',
    })
  })

  it('both refuse unhandled question types with DECISION_PROVIDER_ERROR', () => {
    const invalidQ = { type: 'custom_type', instructions: 'Invalid' } as const

    // @ts-expect-error -- a question built from runtime data can carry a type the union does not declare.
    expect(() => toJevQuestion(invalidQ)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    // @ts-expect-error -- a question built from runtime data can carry a type the union does not declare.
    expect(() => toLayaQuestion(invalidQ)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })
})

describe('JEV and Laya: Answer Translation Parity', () => {
  it('maps choice answers with probabilities identically into seam types', () => {
    const jevAnswer = fromJevAnswer('k', mockWireAnswers.target)
    const layaAnswer = fromLayaAnswer('k', mockWireAnswers.target)

    expect(jevAnswer).toEqual(layaAnswer)
    expect(jevAnswer).toEqual({
      type: 'choice',
      choice: '1',
      probabilities: { '1': 0.88, '2': 0.12 },
    })
  })

  it('maps score answers with continuous rubric position identically into seam types', () => {
    const jevAnswer = fromJevAnswer('k', mockWireAnswers.risk)
    const layaAnswer = fromLayaAnswer('k', mockWireAnswers.risk)

    expect(jevAnswer).toEqual(layaAnswer)
    expect(jevAnswer).toEqual({
      type: 'score',
      score: 0.3,
      probabilities: { '0': 0.75, '1': 0.2, '2': 0.05 },
    })
  })

  it('maps noul wire answers identically to boolean seam answers', () => {
    const jevAnswer = fromJevAnswer('k', mockWireAnswers.completed)
    const layaAnswer = fromLayaAnswer('k', mockWireAnswers.completed)

    expect(jevAnswer).toEqual(layaAnswer)
    expect(jevAnswer).toEqual({
      type: 'boolean',
      probability: 0.15,
    })
  })

  it('both refuse malformed choice answers lacking distribution', () => {
    const malformed = { type: 'choice' as const, choice: '1' }

    expect(() => fromJevAnswer('k', malformed)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    expect(() => fromLayaAnswer('k', malformed)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('both refuse malformed score answers lacking distribution', () => {
    const malformed = { type: 'score' as const, score: 1.0 }

    expect(() => fromJevAnswer('k', malformed)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    expect(() => fromLayaAnswer('k', malformed)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('both refuse malformed noul answers lacking probability', () => {
    const malformed = { type: 'noul' as const }

    expect(() => fromJevAnswer('k', malformed)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    expect(() => fromLayaAnswer('k', malformed)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('both refuse unknown answer types with DECISION_PROVIDER_ERROR', () => {
    const unknownAnswer = { type: 'unsupported' } as never

    expect(() => fromJevAnswer('k', unknownAnswer)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    expect(() => fromLayaAnswer('k', unknownAnswer)).toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })
})

describe('JEV and Laya: Error Code & Failover Semantics Parity', () => {
  it('both classify HTTP 429 as DECISION_RATE_LIMITED so failover triggers', async () => {
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'Too many requests' } }, 429)
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_RATE_LIMITED' }) as Error,
    )
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_RATE_LIMITED' }) as Error,
    )
  })

  it('both classify HTTP 400 and 401 as DECISION_REQUEST_REJECTED so failover is suppressed', async () => {
    // 401: Unauthorized / Invalid key
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'Unauthorized / invalid key' } }, 401)
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )

    // 400: Bad request / max tokens exceeded
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'max_tokens_exceeded' } }, 400)
    })

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
  })

  it('both classify HTTP 500 and 503 as DECISION_PROVIDER_ERROR so failover triggers', async () => {
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'Internal service unavailable' } }, 503)
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('both classify transport/network drop as DECISION_PROVIDER_ERROR', async () => {
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      throw new TypeError('Failed to fetch: network unreachable')
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('both surface cancellation as DECISION_ABORTED without misreporting as provider failure', async () => {
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      throw new DOMException('Operation aborted', 'AbortError')
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    const controller = new AbortController()
    controller.abort()

    await expect(jev.evaluate(sampleRequest, controller.signal)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error,
    )
    await expect(laya.evaluate(sampleRequest, controller.signal)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error,
    )
  })
})

describe('JEV and Laya: Response Integrity Enforcement Parity', () => {
  it('both reject responses carrying no answers field', async () => {
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ model: 'test-model' })
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(/carried no answers/)
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(/carried no answers/)
  })

  it('both reject responses missing one of the requested questions', async () => {
    stubFetch((req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({
        answers: {
          target: mockWireAnswers.target,
          risk: mockWireAnswers.risk,
          // 'completed' question is intentionally omitted
        },
      })
    })

    const jev = makeJevProvider()
    const laya = makeLayaProvider()

    await expect(jev.evaluate(sampleRequest)).rejects.toThrow(/no answer for question "completed"/)
    await expect(laya.evaluate(sampleRequest)).rejects.toThrow(/no answer for question "completed"/)
  })
})

describe('End-to-End DecisionRuntime Failover: JEV with Laya fallback', () => {
  it('serves normally from JEV when JEV is healthy, leaving Laya untouched', async () => {
    const layaSpy = vi.fn()
    stubFetch((req) => {
      if (req.url.includes('typesafe')) {
        return jsonResponse({ answers: mockWireAnswers })
      }
      layaSpy()
      return jsonResponse({ status: 'ok' })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })
    ctx.decision.registerProvider(makeJevProvider())
    ctx.decision.registerProvider(makeLayaProvider())

    const result = await ctx.decision.evaluate(sampleRequest)

    expect(result.providerId).toBe(JEV_PROVIDER_ID)
    expect(result.answers.target).toMatchObject({ type: 'choice', choice: '1' })
    expect(layaSpy).not.toHaveBeenCalled()
  })

  it.each([500, 503])('automatically falls back to Laya when JEV suffers a %d outage', async (status) => {
    stubFetch((req) => {
      if (req.url.includes('typesafe')) {
        return jsonResponse({ error: { message: `TypeSafe outage (${status})` } }, status)
      }
      if (req.url.endsWith('/health')) {
        return jsonResponse({ status: 'ok' })
      }
      if (req.url.includes('58085') || req.url.endsWith('/v1/systemone')) {
        return jsonResponse({ answers: mockWireAnswers })
      }
      return new Response('Not found', { status: 404 })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })
    ctx.decision.registerProvider(makeJevProvider())
    ctx.decision.registerProvider(makeLayaProvider())

    const result = await ctx.decision.evaluate(sampleRequest)

    // Seamless failover occurred! Result came from Laya!
    expect(result.providerId).toBe(LAYA_PROVIDER_ID)
    expect(result.answers.target).toMatchObject({ type: 'choice', choice: '1' })
    expect(result.answers.risk).toMatchObject({ type: 'score', score: 0.3 })
    expect(result.answers.completed).toMatchObject({ type: 'boolean', probability: 0.15 })
  })

  it('automatically falls back to Laya when JEV suffers a network transport failure (connection drop)', async () => {
    stubFetch((req) => {
      if (req.url.includes('typesafe')) {
        throw new TypeError('fetch failed: ECONNRESET')
      }
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      if (req.url.endsWith('/v1/systemone')) return jsonResponse({ answers: mockWireAnswers })
      return new Response('Not found', { status: 404 })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })
    ctx.decision.registerProvider(makeJevProvider())
    ctx.decision.registerProvider(makeLayaProvider())

    const result = await ctx.decision.evaluate(sampleRequest)

    expect(result.providerId).toBe(LAYA_PROVIDER_ID)
    expect(result.answers.target).toMatchObject({ type: 'choice', choice: '1' })
  })

  it('automatically falls back to Laya when JEV is rate-limited (HTTP 429)', async () => {
    stubFetch((req) => {
      if (req.url.includes('typesafe')) {
        return jsonResponse({ error: { message: 'Rate limited' } }, 429)
      }
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      if (req.url.endsWith('/v1/systemone')) return jsonResponse({ answers: mockWireAnswers })
      return new Response('Not found', { status: 404 })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })
    ctx.decision.registerProvider(makeJevProvider())
    ctx.decision.registerProvider(makeLayaProvider())

    const result = await ctx.decision.evaluate(sampleRequest)

    expect(result.providerId).toBe(LAYA_PROVIDER_ID)
    expect(result.answers.target).toMatchObject({ type: 'choice', choice: '1' })
  })

  it('does NOT fall back to Laya when JEV rejects the request itself (HTTP 401)', async () => {
    const layaSpy = vi.fn()
    stubFetch((req) => {
      if (req.url.includes('typesafe')) {
        return jsonResponse({ error: { message: 'invalid_api_key' } }, 401)
      }
      layaSpy()
      return jsonResponse({ status: 'ok' })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })
    ctx.decision.registerProvider(makeJevProvider())
    ctx.decision.registerProvider(makeLayaProvider())

    // Must throw DECISION_REQUEST_REJECTED without invoking Laya fallback
    await expect(ctx.decision.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
    expect(layaSpy).not.toHaveBeenCalled()
  })

  it('does NOT fall back to Laya when JEV rejects due to max_tokens_exceeded (HTTP 400)', async () => {
    const layaSpy = vi.fn()
    stubFetch((req) => {
      if (req.url.includes('typesafe')) {
        return jsonResponse({ error: { message: 'max_tokens_exceeded' } }, 400)
      }
      layaSpy()
      return jsonResponse({ status: 'ok' })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })
    ctx.decision.registerProvider(makeJevProvider())
    ctx.decision.registerProvider(makeLayaProvider())

    await expect(ctx.decision.evaluate(sampleRequest)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
    expect(layaSpy).not.toHaveBeenCalled()
  })
})
