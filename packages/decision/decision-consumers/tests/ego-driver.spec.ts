import { describe, expect, it, vi } from 'vitest'
import { decideEgoStep, DEFAULT_EGO_CONFIDENCE_THRESHOLD } from '../src/ego-driver.ts'
import { createMockDecisionService } from './helpers.ts'

describe('decideEgoStep', () => {
  const candidates = [
    { id: '1', label: 'USB Cable 2m', description: 'Product link' },
    { id: '2', label: 'Next Page', description: 'Pagination' },
  ]

  it('supports candidates without description and missing choice probability', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          element: { type: 'choice', choice: 'plain', probabilities: {} },
          task_complete: { type: 'boolean', probability: 0 },
          page_blocked: { type: 'boolean', probability: 0 },
          risk_level: { type: 'score', score: 0, probabilities: { '0': 1 } },
        },
      }),
    )

    const decision = await decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Search results',
      candidates: [
        { id: 'plain', label: 'Plain link' },
        { id: 'other', label: 'Other link' },
      ],
    })

    expect(decision.confidence).toBe(0)
    expect(decision.action.type).toBe('escalate')
  })

  it('rejects candidate lists with fewer than two options', async () => {
    const mock = createMockDecisionService(() => Promise.resolve({ answers: {}, providerId: 'test' }))
    await expect(decideEgoStep(mock, {
      taskGoal: 'find cable',
      pageState: 'results page',
      candidates: [{ id: '1', label: 'Only option' }],
    })).rejects.toThrow(/at least two candidate elements/)
  })

  it('clicks the winning element when confidence meets threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          element: { type: 'choice', choice: '1', probabilities: { '1': 0.95, '2': 0.05 } },
          task_complete: { type: 'boolean', probability: 0.1 },
          page_blocked: { type: 'boolean', probability: 0.02 },
          risk_level: { type: 'score', score: 0.2, probabilities: { '0': 0.8, '1': 0.2, '2': 0 } },
        },
      }),
    )

    const decision = await decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Search results for cable',
      candidates,
    })

    expect(decision.action).toEqual({ type: 'click', elementId: '1' })
    expect(decision.confidence).toBe(0.95)
    expect(decision.taskComplete).toBe(false)
    expect(decision.pageBlocked).toBe(false)
    expect(decision.providerId).toBe('jev')
  })

  it('escalates when winning element confidence is below threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          element: { type: 'choice', choice: '1', probabilities: { '1': 0.55, '2': 0.45 } },
          task_complete: { type: 'boolean', probability: 0.1 },
          page_blocked: { type: 'boolean', probability: 0.05 },
          risk_level: { type: 'score', score: 0.1, probabilities: { '0': 0.9, '1': 0.1, '2': 0 } },
        },
      }),
    )

    const decision = await decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Ambiguous results',
      candidates,
    }, { confidenceThreshold: DEFAULT_EGO_CONFIDENCE_THRESHOLD })

    expect(decision.action.type).toBe('escalate')
    if (decision.action.type === 'escalate') {
      expect(decision.action.reason).toContain('Low decision confidence')
    }
  })

  it('escalates immediately when page is blocked by error or captcha', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          element: { type: 'choice', choice: '1', probabilities: { '1': 0.9, '2': 0.1 } },
          task_complete: { type: 'boolean', probability: 0.0 },
          page_blocked: { type: 'boolean', probability: 0.92 },
          risk_level: { type: 'score', score: 0.1, probabilities: { '0': 0.9, '1': 0.1, '2': 0 } },
        },
      }),
    )

    const decision = await decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Cloudflare captcha screen',
      candidates,
    })

    expect(decision.action).toEqual({
      type: 'escalate',
      reason: 'Page appears blocked (confidence 0.92)',
    })
    expect(decision.pageBlocked).toBe(true)
  })

  it('completes the task when task_complete probability meets threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          element: { type: 'choice', choice: '1', probabilities: { '1': 0.8, '2': 0.2 } },
          task_complete: { type: 'boolean', probability: 0.95 },
          page_blocked: { type: 'boolean', probability: 0.01 },
          risk_level: { type: 'score', score: 0.0, probabilities: { '0': 1, '1': 0, '2': 0 } },
        },
      }),
    )

    const decision = await decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Product details page for 2m Braided USB-C Cable',
      candidates,
    })

    expect(decision.action).toEqual({ type: 'complete' })
    expect(decision.taskComplete).toBe(true)
  })

  it('fails loudly if provider returns unexpected answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          element: { type: 'boolean', probability: 0.5 },
          task_complete: { type: 'boolean', probability: 0.1 },
          page_blocked: { type: 'boolean', probability: 0.0 },
          risk_level: { type: 'score', score: 0, probabilities: { '0': 1, '1': 0, '2': 0 } },
        },
      }),
    )

    await expect(decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Search page',
      candidates,
    })).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards cancellation signal to evaluate', async () => {
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          element: { type: 'choice' as const, choice: '1', probabilities: { '1': 1, '2': 0 } },
          task_complete: { type: 'boolean' as const, probability: 0 },
          page_blocked: { type: 'boolean' as const, probability: 0 },
          risk_level: { type: 'score' as const, score: 0, probabilities: { '0': 1, '1': 0, '2': 0 } },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await decideEgoStep(mock, {
      taskGoal: 'Open 2m cable',
      pageState: 'Search page',
      candidates,
    }, {}, controller.signal)

    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
