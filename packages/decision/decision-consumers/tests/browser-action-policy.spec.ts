import type { DecisionChoiceAnswer, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { describe, expect, it, vi } from 'vitest'
import { decideBrowserAction } from '../src/browser-action-policy.ts'
import type { BrowserActionPolicyRequest } from '../src/types.ts'
import { createMockDecisionService } from './helpers.ts'

const REQUEST: BrowserActionPolicyRequest = {
  goal: 'Search for flights from Zurich to London',
  pageUrl: 'https://example.test/flights',
  pageTitle: 'Flights',
  pageText: 'Book your trip',
  controls: [
    { id: '1', role: 'combobox', label: 'Where from?', value: 'San Francisco', operations: ['CLICK', 'TYPE_TEXT'] },
    { id: '2', role: 'combobox', label: 'Where to?', value: '', operations: ['CLICK', 'TYPE_TEXT'] },
    { id: '3', role: 'button', label: 'Search', operations: ['CLICK'] },
    { id: '4', role: 'combobox', label: 'Currency', value: 'USD', operations: ['SELECT'] },
  ],
  history: [],
}

/** Build a choice answer whose winning option carries `probability`. */
function choice(winner: string, probability: number, others: readonly string[] = []): DecisionChoiceAnswer {
  const probabilities: Record<string, number> = { [winner]: probability }
  const rest = others.length > 0 ? (1 - probability) / others.length : 0
  for (const other of others) probabilities[other] = rest
  return { type: 'choice', choice: winner, probabilities }
}

describe('decideBrowserAction', () => {
  it('asks one target head per offered operation in a single request', async () => {
    let observed: DecisionRequest | undefined
    const mock = createMockDecisionService((request: DecisionRequest) => {
      observed = request
      return Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: {
          operation: choice('CLICK', 0.9, ['TYPE_TEXT', 'SELECT', 'DONE']),
          click_target: choice('3', 0.88, ['1', '2']),
        },
      })
    })

    const decision = await decideBrowserAction(mock, REQUEST)

    const keys = Object.keys(observed?.questions ?? {})
    expect(keys).toContain('operation')
    expect(keys).toContain('click_target')
    expect(keys).toContain('type_text_target')
    expect(decision.operation).toBe('CLICK')
    expect(decision.targetId).toBe('3')
    expect(decision.executable).toBe(true)
  })

  it('reads only the target head matching the chosen operation', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: {
          operation: choice('TYPE_TEXT', 0.93, ['CLICK']),
          // A confident but irrelevant click head must not influence the result.
          click_target: choice('3', 0.99, ['1', '2']),
          type_text_target: choice('2', 0.81, ['1']),
        },
      }),
    )

    const decision = await decideBrowserAction(mock, REQUEST)

    expect(decision.operation).toBe('TYPE_TEXT')
    expect(decision.targetId).toBe('2')
    expect(decision.confidence).toBeCloseTo(0.81)
  })

  it('offers an operation only when a control supports it', async () => {
    let observed: DecisionRequest | undefined
    const mock = createMockDecisionService((request: DecisionRequest) => {
      observed = request
      return Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: { operation: choice('CLICK', 0.9, ['DONE']), click_target: choice('a', 0.9, ['b']) },
      })
    })

    await decideBrowserAction(mock, {
      ...REQUEST,
      controls: [
        { id: 'a', role: 'link', label: 'Home', operations: ['CLICK'] },
        { id: 'b', role: 'link', label: 'About', operations: ['CLICK'] },
      ],
    })

    const criteria = (observed?.questions['operation'] as { criteria: Record<string, string> }).criteria
    expect(Object.keys(criteria)).toContain('CLICK')
    // No control accepts text on this page, so the provider is never offered TYPE_TEXT.
    expect(Object.keys(criteria)).not.toContain('TYPE_TEXT')
    expect(Object.keys(observed?.questions ?? {})).not.toContain('type_text_target')
  })

  it('resolves a sole candidate from the operation head without a target question', async () => {
    let observed: DecisionRequest | undefined
    const mock = createMockDecisionService((request: DecisionRequest) => {
      observed = request
      return Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: { operation: choice('SELECT', 0.84, ['CLICK']) },
      })
    })

    const decision = await decideBrowserAction(mock, REQUEST)

    // One SELECT control exists, so a one-option choice question would be invalid for the seam.
    expect(Object.keys(observed?.questions ?? {})).not.toContain('select_target')
    expect(decision.operation).toBe('SELECT')
    expect(decision.targetId).toBe('4')
  })

  it('returns an untargeted operation with no target id', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: { operation: choice('DONE', 0.95, ['CLICK']) },
      }),
    )
    const decision = await decideBrowserAction(mock, REQUEST)
    expect(decision.operation).toBe('DONE')
    expect(decision.targetId).toBeUndefined()
    expect(decision.executable).toBe(true)
  })

  it('marks a low-confidence target non-executable rather than acting on a guess', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: {
          operation: choice('CLICK', 0.9, ['DONE']),
          click_target: choice('3', 0.41, ['1', '2']),
        },
      }),
    )
    const decision = await decideBrowserAction(mock, REQUEST)
    expect(decision.executable).toBe(false)
    expect(decision.targetId).toBe('3')
  })

  it('rejects an observation with no controls', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not be called')))
    await expect(decideBrowserAction(mock, { ...REQUEST, controls: [] })).rejects.toThrow(/at least one observed control/)
  })

  it('fails when the chosen operation has no target answer', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: { operation: choice('CLICK', 0.9, ['DONE']) },
      }),
    )
    await expect(decideBrowserAction(mock, REQUEST)).rejects.toThrow(/no target answer/)
  })

  it('forwards cancellation signal', async () => {
    const spy = vi.fn(() =>
      Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: { operation: choice('DONE', 0.9, ['CLICK']) },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()
    await decideBrowserAction(mock, REQUEST, {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
