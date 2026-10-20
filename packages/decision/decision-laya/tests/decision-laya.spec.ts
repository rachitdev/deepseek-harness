import { mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
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
  hasCachedModel,
  isLoopbackHost,
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

describe('LayaDecisionProvider availability', () => {
  it('is unavailable when python executable is missing', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: '/nonexistent/fake-python',
      workerScriptPath: '/bin/sh',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
      hubCacheDir: '/nonexistent',
    })
    expect(provider.available()).toBe(false)
  })

  it('is unavailable when worker script is missing', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: process.execPath,
      workerScriptPath: '/nonexistent/fake-worker.py',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
      hubCacheDir: '/nonexistent',
    })
    expect(provider.available()).toBe(false)
  })

  it('is unavailable when model checkpoint is not cached locally on loopback', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: process.execPath,
      workerScriptPath: process.execPath,
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/nonexistent',
      hubCacheDir: '/nonexistent',
    })
    expect(provider.available()).toBe(false)
  })

  it('is unavailable with an invalid baseURL', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: process.execPath,
      workerScriptPath: process.execPath,
      host: 'remote.example',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'not-a-url',
      model: 'convaiinnovations/laya',
    })
    expect(provider.available()).toBe(false)
  })

  it('is unavailable with an empty model id', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: process.execPath,
      workerScriptPath: process.execPath,
      host: 'remote.example',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://remote.example:58085',
      model: '',
    })
    expect(provider.available()).toBe(false)
  })

  it('is available when remote host is configured with valid URL and model', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: '/nonexistent',
      workerScriptPath: '/nonexistent',
      host: 'remote.example',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://remote.example:58085',
      model: 'convaiinnovations/laya',
    })
    expect(provider.available()).toBe(true)
  })

  it('honors available override option', () => {
    const provider = new LayaDecisionProvider({
      pythonPath: '/nonexistent',
      workerScriptPath: '/nonexistent',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
      available: () => true,
    })
    expect(provider.available()).toBe(true)
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

  it('surfaces worker HTTP 500 errors as DECISION_PROVIDER_ERROR', async () => {
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

    await expect(provider.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('surfaces worker HTTP 429 as DECISION_RATE_LIMITED', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'Rate limit exceeded' } }, { status: 429 })
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

    await expect(provider.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_RATE_LIMITED' }) as Error,
    )
  })

  it('surfaces worker HTTP 400 as DECISION_REQUEST_REJECTED', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'max_tokens_exceeded' } }, { status: 400 })
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

    await expect(provider.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
  })

  it('surfaces worker HTTP 401 as DECISION_REQUEST_REJECTED', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ error: { message: 'unauthorized' } }, { status: 401 })
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

    await expect(provider.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_REQUEST_REJECTED' }) as Error,
    )
  })

  it('rejects a response carrying no answers', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({})
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

    await expect(provider.evaluate(request)).rejects.toThrow(/carried no answers/)
  })

  it('rejects a response missing one of the questions asked', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ answers: { action: fullAnswers.action } })
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

    await expect(provider.evaluate(request)).rejects.toThrow(/no answer for question "severity"/)
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

  it('cancels quickly when abort signal fires while waiting for startup lock without poisoning other callers', async () => {
    let healthChecks = 0
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) {
        healthChecks++
        if (healthChecks < 3) return new Response('loading', { status: 503 })
        return jsonResponse({ status: 'ok' })
      }
      return new Response('Not Found', { status: 404 })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/bin/sh',
      workerScriptPath: '/bin/sh',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 10000,
      healthTimeoutMs: 100,
      startupPollIntervalMs: 50,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
    })

    const controller1 = new AbortController()
    const controller2 = new AbortController()

    const p1 = provider.ensureWorker(controller1.signal)
    const p2 = provider.ensureWorker(controller2.signal)

    // Aborting controller1 should abort p1 immediately
    controller1.abort()
    await expect(p1).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error,
    )

    // But p2 continues waiting and succeeds once the worker becomes healthy
    await expect(p2).resolves.toBeUndefined()
  })

  it('throws DECISION_PROVIDER_ERROR when remote worker is unreachable without spawning local python', async () => {
    stubFetch(async () => {
      return new Response('gateway timeout', { status: 504 })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/nonexistent',
      workerScriptPath: '/nonexistent',
      host: 'remote.example',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 1000,
      healthTimeoutMs: 100,
      startupPollIntervalMs: 100,
      baseURL: 'http://remote.example:58085',
      model: 'convaiinnovations/laya',
    })

    await expect(provider.ensureWorker()).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('surfaces abort signal cancellation as DECISION_ABORTED when probing remote worker', async () => {
    stubFetch(async () => {
      await new Promise(resolve => setTimeout(resolve, 300))
      return jsonResponse({ status: 'ok' })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/nonexistent',
      workerScriptPath: '/nonexistent',
      host: 'remote.example',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 1000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 100,
      baseURL: 'http://remote.example:58085',
      model: 'convaiinnovations/laya',
    })

    const controller = new AbortController()
    const p = provider.ensureWorker(controller.signal)
    controller.abort()

    await expect(p).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_ABORTED' }) as Error,
    )
  })

  it('throws DECISION_PROVIDER_UNAVAILABLE when python executable is missing at startup', async () => {
    stubFetch(async () => {
      return new Response('Not Found', { status: 404 })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: '/nonexistent/fake-python',
      workerScriptPath: '/bin/sh',
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 1000,
      healthTimeoutMs: 100,
      startupPollIntervalMs: 100,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
    })

    await expect(provider.ensureWorker()).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
    )
  })

  it('throws DECISION_PROVIDER_ERROR when worker startup times out', async () => {
    stubFetch(async () => {
      return new Response('Loading', { status: 503 })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: process.execPath,
      workerScriptPath: process.execPath,
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 50,
      healthTimeoutMs: 20,
      startupPollIntervalMs: 20,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
    })

    await expect(provider.ensureWorker()).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('surfaces invalid JSON response as DECISION_PROVIDER_ERROR', async () => {
    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return new Response('<html>not json</html>', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    })

    const provider = new LayaDecisionProvider({
      pythonPath: process.execPath,
      workerScriptPath: process.execPath,
      host: '127.0.0.1',
      port: 58085,
      idleTimeoutSeconds: 300,
      startupTimeoutMs: 1000,
      healthTimeoutMs: 100,
      startupPollIntervalMs: 100,
      baseURL: 'http://127.0.0.1:58085',
      model: 'convaiinnovations/laya',
    })

    await expect(provider.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })
})

describe('isLoopbackHost helper', () => {
  it('correctly classifies loopback IPv4, IPv6, and localhost', () => {
    expect(isLoopbackHost('127.0.0.1')).toBe(true)
    expect(isLoopbackHost('127.0.0.2')).toBe(true)
    expect(isLoopbackHost('localhost')).toBe(true)
    expect(isLoopbackHost('::1')).toBe(true)
    expect(isLoopbackHost('[::1]')).toBe(true)
    expect(isLoopbackHost('0.0.0.0')).toBe(true)
  })

  it('rejects remote domains starting with 127 or non-loopback addresses', () => {
    expect(isLoopbackHost('127.worker.example')).toBe(false)
    expect(isLoopbackHost('remote.example')).toBe(false)
    expect(isLoopbackHost('192.168.1.1')).toBe(false)
  })
})

describe('hasCachedModel helper', () => {
  it('detects model path directly when directory exists and contains model files', () => {
    const fixtureDir = join(tmpdir(), `dsh-laya-test-model-${Math.random().toString(36).slice(2)}`)
    mkdirSync(fixtureDir, { recursive: true })
    writeFileSync(join(fixtureDir, 'model.safetensors'), 'mock weights')
    try {
      expect(hasCachedModel(fixtureDir)).toBe(true)
      expect(hasCachedModel(fixtureDir, 'nonexistent-subfolder-xyz')).toBe(false)
    } finally {
      rmSync(fixtureDir, { recursive: true, force: true })
    }
  })

  it('returns false when cache dir is missing or empty or contains no weights', () => {
    expect(hasCachedModel('org/model', undefined, undefined)).toBe(false)
    expect(hasCachedModel('org/model', undefined, '/nonexistent-path-abc')).toBe(false)
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
      available: true,
    })

    const res = await ctx.decision.evaluate(request)
    expect(res.providerId).toBe(LAYA_PROVIDER_ID)

    await fiber.dispose()
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(DecisionError)
  })

  it('reads configuration from environment variables when unset', async () => {
    vi.stubEnv('DSH_LAYA_HOST', '127.0.0.2')
    vi.stubEnv('DSH_LAYA_PORT', '58099')
    vi.stubEnv('DSH_LAYA_MODEL', 'custom/model')
    vi.stubEnv('DSH_LAYA_MODEL_SUBFOLDER', 'custom-sub')
    vi.stubEnv('DSH_LAYA_AVAILABLE', '1')

    stubFetch(async (req) => {
      if (req.url.endsWith('/health')) return jsonResponse({ status: 'ok' })
      return jsonResponse({ answers: fullAnswers })
    })

    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'laya' })
    await ctx.plugin(layaPlugin, {})

    const res = await ctx.decision.evaluate(request)
    expect(res.providerId).toBe(LAYA_PROVIDER_ID)
  })
})
