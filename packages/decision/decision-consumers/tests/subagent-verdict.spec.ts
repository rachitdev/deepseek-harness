import { describe, expect, it, vi } from 'vitest'
import { evaluateSubagent } from '../src/subagent-verdict.ts'
import { createMockDecisionService } from './helpers.ts'

describe('evaluateSubagent', () => {
  it('handles missing choice probability gracefully', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          verdict: { type: 'choice', choice: 'accept', probabilities: {} },
          quality: { type: 'score', score: 2.5, probabilities: { '2': 1 } },
          criteria_met: { type: 'boolean', probability: 0.9 },
        },
      }),
    )

    const evaluation = await evaluateSubagent(mock, {
      brief: 'test',
      acceptanceCriteria: ['done'],
      subagentOutput: 'done',
    })
    expect(evaluation.confidence).toBe(0)
  })

  it('accepts high quality output meeting all criteria', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          verdict: { type: 'choice', choice: 'accept', probabilities: { accept: 0.94, retry: 0.06, escalate: 0 } },
          quality: { type: 'score', score: 2.9, probabilities: { '0': 0, '1': 0, '2': 0.1, '3': 0.9 } },
          criteria_met: { type: 'boolean', probability: 0.98 },
        },
      }),
    )

    const evaluation = await evaluateSubagent(mock, {
      brief: 'Implement LRU cache',
      acceptanceCriteria: ['O(1) get', 'O(1) put', 'Capacity eviction'],
      subagentOutput: 'Clean doubly-linked list implementation with map and unit tests passing',
    })

    expect(evaluation.verdict).toBe('accept')
    expect(evaluation.qualityScore).toBe(2.9)
    expect(evaluation.criteriaMet).toBe(true)
    expect(evaluation.feedbackRecommendation).toBeUndefined()
  })

  it('requests retry with targeted feedback when quality is acceptable but criteria incomplete', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          verdict: { type: 'choice', choice: 'retry', probabilities: { accept: 0.1, retry: 0.85, escalate: 0.05 } },
          quality: { type: 'score', score: 1.8, probabilities: { '0': 0, '1': 0.3, '2': 0.7, '3': 0 } },
          criteria_met: { type: 'boolean', probability: 0.5 },
        },
      }),
    )

    const evaluation = await evaluateSubagent(mock, {
      brief: 'Implement LRU cache',
      acceptanceCriteria: ['O(1) get', 'O(1) put', 'Capacity eviction'],
      subagentOutput: 'Implemented get and put, eviction logic missing test',
    })

    expect(evaluation.verdict).toBe('retry')
    expect(evaluation.feedbackRecommendation).toContain('Request focused correction')
  })

  it('escalates on fundamental failure or unusable output', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          verdict: { type: 'choice', choice: 'escalate', probabilities: { accept: 0, retry: 0.1, escalate: 0.9 } },
          quality: { type: 'score', score: 0.4, probabilities: { '0': 0.8, '1': 0.2, '2': 0, '3': 0 } },
          criteria_met: { type: 'boolean', probability: 0.05 },
        },
      }),
    )

    const evaluation = await evaluateSubagent(mock, {
      brief: 'Implement LRU cache',
      acceptanceCriteria: ['O(1) get'],
      subagentOutput: 'Crash error: TypeError: cannot read property of undefined',
    })

    expect(evaluation.verdict).toBe('escalate')
  })

  it('fails on unexpected provider answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          verdict: { type: 'score', score: 1, probabilities: {} },
          quality: { type: 'score', score: 1, probabilities: {} },
          criteria_met: { type: 'boolean', probability: 0.5 },
        },
      }),
    )

    await expect(evaluateSubagent(mock, {
      brief: 'b',
      acceptanceCriteria: [],
      subagentOutput: 'o',
    })).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards cancellation signal', async () => {
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          verdict: { type: 'choice' as const, choice: 'accept', probabilities: { accept: 1 } },
          quality: { type: 'score' as const, score: 2, probabilities: { '2': 1 } },
          criteria_met: { type: 'boolean' as const, probability: 1 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await evaluateSubagent(mock, {
      brief: 'b',
      acceptanceCriteria: [],
      subagentOutput: 'o',
    }, {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
