import { describe, expect, it, vi } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, {
  DecisionError,
  type DecisionQuestion,
  type DecisionRequest,
} from '@deepseek-ai/dsh-decision'
import * as layaPlugin from '../src/index.ts'
import {
  fromLayaAnswer,
  LAYA_PROVIDER_ID,
  LayaDecisionProvider,
  toLayaQuestion,
} from '../src/provider.ts'

const request: DecisionRequest = {
  state: 'Worker failure on primary node',
  questions: {
    action: {
      type: 'choice',
      instructions: 'Pick action',
      criteria: { restart: 'restart worker', drop: 'drop job' },
    },
    severity: {
      type: 'score',
      instructions: 'Severity score',
      criteria: ['low', 'medium', 'high'],
    },
    critical: {
      type: 'boolean',
      instructions: 'Is it critical?',
    },
  },
}

const fullAnswers = {
  action: {
    type: 'choice' as const,
    choice: 'restart',
    probabilities: { restart: 0.9, drop: 0.1 },
  },
  severity: {
    type: 'score' as const,
    score: 1.8,
    probabilities: { '0': 0.1, '1': 0.1, '2': 0.8 },
  },
  critical: {
    type: 'noul' as const,
    noul: 0.85,
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

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { 'content-type': 'application/json' },
    ...init,
  })
}

describe('toLayaQuestion', () => {
  it('translates choice questions to criteria dict', () => {
    const q: DecisionQuestion = {
      type: 'choice',
      instructions: 'Choose',
      criteria: { a: 'A', b: 'B' },
    }
    expect(toLayaQuestion(q)).toEqual({
      type: 'choice',
      instructions: 'Choose',
      criteria: { a: 'A', b: 'B' },
    })
  })

  it('translates score questions to level criteria', () => {
    const q: DecisionQuestion = {
      type: 'score',
      instructions: 'Score',
      criteria: ['low', 'high'],
    }
    expect(toLayaQuestion(q)).toEqual({
      type: 'score',
      instructions: 'Score',
      criteria: ['low', 'high'],
    })
  })

  it('translates boolean questions to noul', () => {
    const q: DecisionQuestion = {
      type: 'boolean',
      instructions: 'Is true?',
    }
    expect(toLayaQuestion(q)).toEqual({
      type: 'noul',
      instructions: 'Is true?',
    })
  })
})

describe('fromLayaAnswer', () => {
  it('maps choice answers with probabilities', () => {
    const answer = fromLayaAnswer('act', {
      type: 'choice',
      choice: 'restart',
      probabilities: { restart: 0.9, drop: 0.1 },
    })
    expect(answer).toEqual({
      type: 'choice',
      choice: 'restart',
      probabilities: { restart: 0.9, drop: 0.1 },
    })
  })

  it('rejects choice answers without distribution', () => {
    expect(() => fromLayaAnswer('act', { type: 'choice', choice: 'restart' }))
      .toThrow(DecisionError)
  })

  it('maps score answers with probabilities', () => {
    const answer = fromLayaAnswer('sev', {
      type: 'score',
      score: 1.5,
      probabilities: { '0': 0.2, '1': 0.8 },
    })
    expect(answer).toEqual({
      type: 'score',
      score: 1.5,
      probabilities: { '0': 0.2, '1': 0.8 },
    })
  })

  it('maps noul answers to boolean probability', () => {
    const answer = fromLayaAnswer('crit', {
      type: 'noul',
      noul: 0.92,
    })
    expect(answer).toEqual({
      type: 'boolean',
      probability: 0.92,
    })
  })

  it('rejects unknown answer types', () => {
    expect(() => fromLayaAnswer('bad', { type: 'unknown' } as unknown as Parameters<typeof fromLayaAnswer>[1]))
      .toThrow(DecisionError)
  })
})

describe('LayaDecisionProvider evaluate', () => {
  it('queries health and evaluates questions successfully when healthy', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) {
        return jsonResponse({ status: 'ok' })
      }
      if (req.url.endsWith('/v1/systemone')) {
        return jsonResponse({ answers: fullAnswers })
      }
      return new Response('Not Found', { status: 404 })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/tmp/fake-python',
      workerScriptPath: '/tmp/fake-worker.py',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
      modelSubfolder: 'typed-decisions',
    })

    const result = await provider.evaluate(request)
    expect(result.providerId).toBe(LAYA_PROVIDER_ID)
    expect(result.answers.action).toEqual({
      type: 'choice',
      choice: 'restart',
      probabilities: { restart: 0.9, drop: 0.1 },
    })
    expect(result.answers.critical).toEqual({
      type: 'boolean',
      probability: 0.85,
    })
  })

  it('surfaces worker HTTP errors', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'Out of memory' } }, { status: 500 })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/tmp/fake-python',
      workerScriptPath: '/tmp/fake-worker.py',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
      modelSubfolder: 'typed-decisions',
    })

    await expect(provider.evaluate(request)).rejects.toThrow(/Out of memory/)
  })

  it('surfaces abort signal cancellation as DECISION_ABORTED', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      const error = new DOMException('The operation was aborted.', 'AbortError')
      throw error
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/tmp/fake-python',
      workerScriptPath: '/tmp/fake-worker.py',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
      modelSubfolder: 'typed-decisions',
    })

    const controller = new AbortController()
    controller.abort()
    await expect(provider.evaluate(request, controller.signal)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error,
    )
  })
})

describe('decision-laya plugin integration', () => {
  it('registers provider with decision service and disposes cleanly', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ answers: fullAnswers })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    const fiber = await ctx.plugin(layaPlugin, {
      baseURL: 'http://127.0.0.1:58085',
    })

    const res = await ctx.decision.evaluate(request)
    expect(res.providerId).toBe(LAYA_PROVIDER_ID)

    await fiber.dispose()
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(DecisionError)
  })
})
