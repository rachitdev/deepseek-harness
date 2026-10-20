import { describe, expect, it, vi } from 'vitest'
import { routeTurnModel } from '../src/model-router.ts'
import { createMockDecisionService } from './helpers.ts'

describe('routeTurnModel', () => {
  it('handles missing choice probability gracefully', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          tier: { type: 'choice', choice: 'fast', probabilities: {} },
          complexity: { type: 'score', score: 0, probabilities: { '0': 1 } },
          deep_reasoning: { type: 'boolean', probability: 0 },
        },
      }),
    )

    const decision = await routeTurnModel(mock, { userMessage: 'test' })
    expect(decision.confidence).toBe(0)
  })

  it('routes routine lookup tasks to the fast tier', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          tier: { type: 'choice', choice: 'fast', probabilities: { fast: 0.92, frontier: 0.08 } },
          complexity: { type: 'score', score: 0.3, probabilities: { '0': 0.8, '1': 0.2, '2': 0, '3': 0 } },
          deep_reasoning: { type: 'boolean', probability: 0.05 },
        },
      }),
    )

    const decision = await routeTurnModel(mock, {
      userMessage: 'Where is the main entry point defined?',
    })

    expect(decision.selectedTier).toBe('fast')
    expect(decision.recommendedModel).toBe('deepseek-chat')
    expect(decision.complexityScore).toBe(0.3)
    expect(decision.needsDeepReasoning).toBe(false)
  })

  it('routes architectural and complex reasoning to the frontier tier', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          tier: { type: 'choice', choice: 'frontier', probabilities: { fast: 0.1, frontier: 0.9 } },
          complexity: { type: 'score', score: 2.7, probabilities: { '0': 0, '1': 0.1, '2': 0.2, '3': 0.7 } },
          deep_reasoning: { type: 'boolean', probability: 0.88 },
        },
      }),
    )

    const decision = await routeTurnModel(mock, {
      userMessage: 'Redesign the distributed consensus protocol to tolerate partition skew',
    })

    expect(decision.selectedTier).toBe('frontier')
    expect(decision.recommendedModel).toBe('deepseek-reasoner')
    expect(decision.complexityScore).toBe(2.7)
    expect(decision.needsDeepReasoning).toBe(true)
  })

  it('honors custom model selections and complexity thresholds', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          tier: { type: 'choice', choice: 'fast', probabilities: { fast: 0.7, frontier: 0.3 } },
          complexity: { type: 'score', score: 1.5, probabilities: { '0': 0, '1': 0.6, '2': 0.4, '3': 0 } },
          deep_reasoning: { type: 'boolean', probability: 0.2 },
        },
      }),
    )

    const decision = await routeTurnModel(
      mock,
      { userMessage: 'Refactor helper functions' },
      { fastModel: 'mini-fast', frontierModel: 'max-reasoner', complexityThreshold: 1.4 },
    )

    // Since complexity (1.5) >= threshold (1.4), it upgrades to frontier tier
    expect(decision.selectedTier).toBe('frontier')
    expect(decision.recommendedModel).toBe('max-reasoner')
  })

  it('fails on unexpected provider answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          tier: { type: 'score', score: 1, probabilities: {} },
          complexity: { type: 'score', score: 1, probabilities: {} },
          deep_reasoning: { type: 'boolean', probability: 0.1 },
        },
      }),
    )

    await expect(routeTurnModel(mock, { userMessage: 'test' })).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards cancellation signal', async () => {
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          tier: { type: 'choice' as const, choice: 'fast', probabilities: { fast: 1 } },
          complexity: { type: 'score' as const, score: 0, probabilities: { '0': 1 } },
          deep_reasoning: { type: 'boolean' as const, probability: 0 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await routeTurnModel(mock, { userMessage: 'test' }, {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
