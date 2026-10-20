import { describe, expect, it, vi } from 'vitest'
import { detectLoopStall } from '../src/loop-stall-detector.ts'
import { createMockDecisionService } from './helpers.ts'

describe('detectLoopStall', () => {
  it('returns progressing immediately for single action sequences', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not be called')))
    const verdict = await detectLoopStall(mock, [{ tool: 'read', inputSummary: 'a', resultSummary: 'b' }], 'goal')
    expect(verdict.stalled).toBe(false)
    expect(verdict.suggestedIntervention).toBe('continue')
  })

  it('detects progressing workflows without stall', async () => {
    const actions = [
      { tool: 'read', inputSummary: 'file1.ts', resultSummary: 'found interface' },
      { tool: 'edit', inputSummary: 'file1.ts: replace foo with bar', resultSummary: 'edited' },
      { tool: 'bash', inputSummary: 'pnpm test', resultSummary: 'all 5 passed' },
    ]

    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          loop_state: { type: 'choice', choice: 'progressing', probabilities: { progressing: 0.95, repeating_error: 0.05, oscillating: 0, exhausted_leads: 0 } },
          is_stalled: { type: 'boolean', probability: 0.02 },
          severity: { type: 'score', score: 0.1, probabilities: { '0': 0.9, '1': 0.1, '2': 0, '3': 0 } },
        },
      }),
    )

    const verdict = await detectLoopStall(mock, actions, 'Fix test')
    expect(verdict.stalled).toBe(false)
    expect(verdict.loopState).toBe('progressing')
    expect(verdict.suggestedIntervention).toBe('continue')
  })

  it('detects repetitive errors and warns or interrupts', async () => {
    const actions = [
      { tool: 'bash', inputSummary: 'cargo build', resultSummary: 'error[E0425]: cannot find value `x`', error: true },
      { tool: 'bash', inputSummary: 'cargo build', resultSummary: 'error[E0425]: cannot find value `x`', error: true },
      { tool: 'bash', inputSummary: 'cargo build', resultSummary: 'error[E0425]: cannot find value `x`', error: true },
    ]

    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          loop_state: { type: 'choice', choice: 'repeating_error', probabilities: { progressing: 0, repeating_error: 0.98, oscillating: 0.02, exhausted_leads: 0 } },
          is_stalled: { type: 'boolean', probability: 0.92 },
          severity: { type: 'score', score: 2.8, probabilities: { '0': 0, '1': 0, '2': 0.2, '3': 0.8 } },
        },
      }),
    )

    const verdict = await detectLoopStall(mock, actions, 'Fix build')
    expect(verdict.stalled).toBe(true)
    expect(verdict.loopState).toBe('repeating_error')
    expect(verdict.suggestedIntervention).toBe('interrupt')
  })

  it('suggests warn intervention for moderate friction below interrupt threshold', async () => {
    const actions = [
      { tool: 'bash', inputSummary: 'cargo check', resultSummary: 'warning: unused variable', error: false },
      { tool: 'bash', inputSummary: 'cargo check', resultSummary: 'warning: unused variable', error: false },
    ]

    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          loop_state: { type: 'choice', choice: 'repeating_error', probabilities: { progressing: 0.1, repeating_error: 0.9, oscillating: 0, exhausted_leads: 0 } },
          is_stalled: { type: 'boolean', probability: 0.70 },
          severity: { type: 'score', score: 1.8, probabilities: { '0': 0.1, '1': 0.8, '2': 0.1, '3': 0 } },
        },
      }),
    )

    const verdict = await detectLoopStall(mock, actions, 'Clean warnings')
    expect(verdict.stalled).toBe(true)
    expect(verdict.suggestedIntervention).toBe('warn')
  })

  it('fails on unexpected provider answer types', async () => {
    const actions = [
      { tool: 'read', inputSummary: 'a', resultSummary: 'b' },
      { tool: 'read', inputSummary: 'c', resultSummary: 'd' },
    ]
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          loop_state: { type: 'boolean', probability: 0.5 },
          is_stalled: { type: 'boolean', probability: 0.5 },
          severity: { type: 'score', score: 0, probabilities: {} },
        },
      }),
    )

    await expect(detectLoopStall(mock, actions, 'goal')).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards cancellation signal', async () => {
    const actions = [
      { tool: 'read', inputSummary: 'a', resultSummary: 'b' },
      { tool: 'read', inputSummary: 'c', resultSummary: 'd' },
    ]
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          loop_state: { type: 'choice' as const, choice: 'progressing', probabilities: { progressing: 1 } },
          is_stalled: { type: 'boolean' as const, probability: 0 },
          severity: { type: 'score' as const, score: 0, probabilities: { '0': 1 } },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await detectLoopStall(mock, actions, 'goal', {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
