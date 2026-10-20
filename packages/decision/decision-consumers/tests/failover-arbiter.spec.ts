import { describe, expect, it, vi } from 'vitest'
import type { DecisionAnswer, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { arbitrateModelFailover } from '../src/failover-arbiter.ts'
import type { FailoverCandidateProbe } from '../src/types.ts'
import { createMockDecisionService } from './helpers.ts'

function probe(model: string, overrides: Partial<FailoverCandidateProbe> = {}): FailoverCandidateProbe {
  return {
    provider: 'omniroute',
    model,
    latencyMs: 500,
    tokenCount: 100,
    probeOutput: `output from ${model}`,
    ...overrides,
  }
}

function resultFor(
  request: DecisionRequest,
  choice = 'worker_1',
  confidence = 0.93,
  overrides: Readonly<Record<string, DecisionAnswer>> = {},
): DecisionResult {
  const answers: Record<string, DecisionAnswer> = {}
  for (const [key, question] of Object.entries(request.questions)) {
    switch (question.type) {
      case 'choice':
        answers[key] = {
          type: 'choice',
          choice,
          probabilities: { [choice]: confidence },
        }
        break
      case 'score':
        answers[key] = {
          type: 'score',
          score: key.startsWith('latency_penalty_') ? 2.5 : 3,
          probabilities: { '3': 1 },
        }
        break
      case 'boolean':
        throw new Error(`takeover arbitration must not ask the boolean question "${key}"`)
    }
  }
  return { providerId: 'jev', answers: { ...answers, ...overrides } }
}

describe('arbitrateModelFailover', () => {
  it('throws when all candidate probes failed', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not evaluate')))

    await expect(arbitrateModelFailover(mock, {
      failedModel: 'auto/best-coding',
      failureReason: 'HTTP 503 Service Unavailable',
      currentTask: 'Implement LRU Cache',
      candidateProbes: [
        probe('model-a', { probeOutput: '', error: 'HTTP 429' }),
        probe('model-b', { probeOutput: '', error: 'Timeout' }),
      ],
    })).rejects.toThrow(/All candidate model probes failed/)
  })

  it('returns the single healthy exact route without calling the decision service', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not evaluate')))

    const decision = await arbitrateModelFailover(mock, {
      failedModel: 'auto/best-coding',
      failureReason: 'HTTP 429 Quota Exceeded',
      currentTask: 'Implement LRU Cache',
      candidateProbes: [
        probe('claude/claude-sonnet-5'),
        probe('cx/gpt-5.6-sol-ultra', { probeOutput: '', error: 'HTTP 429' }),
      ],
    })

    expect(decision).toMatchObject({
      recommendedProvider: 'omniroute',
      recommendedModel: 'claude/claude-sonnet-5',
      confidence: 1,
      codeValidityScore: 2,
      latencyPenaltyScore: 3,
      providerId: 'heuristic-single-survivor',
    })
  })

  it('selects the exact token-efficient worker route and returns its scores', async () => {
    const mock = createMockDecisionService(request => Promise.resolve(resultFor(request)))

    const decision = await arbitrateModelFailover(mock, {
      failedModel: 'auto/best-coding',
      failureReason: 'HTTP 502 Bad Gateway',
      currentTask: 'Implement lock-free queue in Rust',
      candidateProbes: [
        probe('claude/claude-sonnet-5', {
          latencyMs: 1_200,
          tokenCount: 210,
          probeOutput: 'pub struct LockFreeQueue<T> { ... }',
        }),
        probe('cx/gpt-5.6-sol-ultra', {
          provider: 'other-route',
          latencyMs: 8_500,
          tokenCount: 4_800,
          probeOutput: 'Thinking Process: 1. Let us analyze the atomic ordering...',
        }),
      ],
    })

    expect(decision).toMatchObject({
      recommendedProvider: 'omniroute',
      recommendedModel: 'claude/claude-sonnet-5',
      confidence: 0.93,
      tokenEfficiencyScore: 3,
      codeValidityScore: 3,
      latencyPenaltyScore: 2.5,
      providerId: 'jev',
    })
  })

  it('asks only a route choice plus graded per-worker scores', async () => {
    const spy = vi.fn((request: DecisionRequest) => Promise.resolve(resultFor(request)))
    const mock = createMockDecisionService(spy)

    await arbitrateModelFailover(mock, {
      failedModel: 'test',
      failureReason: 'fail',
      currentTask: 'task',
      candidateProbes: [probe('m1'), probe('m2')],
    })

    const request = spy.mock.calls[0]?.[0]
    expect(Object.values(request?.questions ?? {}).filter(question => question.type === 'boolean'))
      .toHaveLength(0)
    expect(request?.questions.best_provider?.type).toBe('choice')
    expect(Object.keys(request?.questions ?? {}).sort()).toEqual([
      'best_provider',
      'code_validity_worker_1',
      'code_validity_worker_2',
      'latency_penalty_worker_1',
      'latency_penalty_worker_2',
      'token_efficiency_worker_1',
      'token_efficiency_worker_2',
    ])
  })

  it('handles a missing worker probability as zero confidence', async () => {
    const mock = createMockDecisionService(request => Promise.resolve(resultFor(request, 'worker_1', 0, {
      best_provider: { type: 'choice', choice: 'worker_1', probabilities: {} },
    })))

    const decision = await arbitrateModelFailover(mock, {
      failedModel: 'test',
      failureReason: 'timeout',
      currentTask: 'task',
      candidateProbes: [probe('union-alpha'), probe('gemini-flash')],
    })

    expect(decision.confidence).toBe(0)
    expect(decision.recommendedModel).toBe('union-alpha')
  })

  it('fails on unexpected route-answer types', async () => {
    const mock = createMockDecisionService(request => Promise.resolve(resultFor(request, 'worker_1', 1, {
      best_provider: { type: 'boolean', probability: 0.5 },
    })))

    await expect(arbitrateModelFailover(mock, {
      failedModel: 'test',
      failureReason: 'fail',
      currentTask: 'task',
      candidateProbes: [probe('m1'), probe('m2')],
    })).rejects.toThrow(/unexpected answer types/)
  })

  it('fails when the provider selects an unknown worker id', async () => {
    const mock = createMockDecisionService(request => Promise.resolve(resultFor(request, 'worker_99')))

    await expect(arbitrateModelFailover(mock, {
      failedModel: 'test',
      failureReason: 'fail',
      currentTask: 'task',
      candidateProbes: [probe('m1'), probe('m2')],
    })).rejects.toThrow(/unknown failover worker/)
  })

  it('fails when the selected worker score has an unexpected type', async () => {
    const mock = createMockDecisionService(request => Promise.resolve(resultFor(request, 'worker_1', 1, {
      code_validity_worker_1: { type: 'boolean', probability: 1 },
    })))

    await expect(arbitrateModelFailover(mock, {
      failedModel: 'test',
      failureReason: 'fail',
      currentTask: 'task',
      candidateProbes: [probe('m1'), probe('m2')],
    })).rejects.toThrow(/unexpected per-worker scores/)
  })

  it('forwards cancellation to the decision provider', async () => {
    const spy = vi.fn((request: DecisionRequest) => Promise.resolve(resultFor(request)))
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await arbitrateModelFailover(mock, {
      failedModel: 'test',
      failureReason: 'fail',
      currentTask: 'task',
      candidateProbes: [probe('m1'), probe('m2')],
    }, controller.signal)

    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
