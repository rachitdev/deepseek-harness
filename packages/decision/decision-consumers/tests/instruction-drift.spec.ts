import type { DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { describe, expect, it, vi } from 'vitest'
import { assessInstructionDrift } from '../src/instruction-drift.ts'
import { createMockDecisionService } from './helpers.ts'

/** Build a decision result carrying one boolean drift probability. */
function driftResult(probability: number): DecisionResult {
  return {
    providerId: 'jev',
    answers: { drift: { type: 'boolean', probability } },
  }
}

const EVIDENCE = {
  instructions: '- Tests live at package level under tests/, not src/__tests__/.',
  workSummary: 'Agent created src/__tests__/scorer.test.ts inside the package source directory.',
}

describe('assessInstructionDrift', () => {
  it('reports drift above the threshold with a positive margin', async () => {
    const mock = createMockDecisionService(() => Promise.resolve(driftResult(0.94)))
    const verdict = await assessInstructionDrift(mock, EVIDENCE)
    expect(verdict.drifted).toBe(true)
    expect(verdict.driftProbability).toBe(0.94)
    expect(verdict.thresholdMargin).toBeCloseTo(0.04)
    expect(verdict.providerId).toBe('jev')
  })

  it('does not report conforming work, and reports how far below the threshold it fell', async () => {
    const mock = createMockDecisionService(() => Promise.resolve(driftResult(0.03)))
    const verdict = await assessInstructionDrift(mock, EVIDENCE)
    expect(verdict.drifted).toBe(false)
    expect(verdict.thresholdMargin).toBeCloseTo(-0.87)
  })

  it('does not report work an instruction exemption permits, at the calibrated default', async () => {
    // Measured against TypeSafe Jev: hardcoding a security invariant that AGENTS.md explicitly
    // exempts scores ~0.90 in isolation and 0.82 in context. The default threshold keeps that
    // band below the reporting line; lowering it to foreman's 0.80 reports conforming work.
    const mock = createMockDecisionService(() => Promise.resolve(driftResult(0.82)))
    const verdict = await assessInstructionDrift(mock, {
      instructions: '- No hardcoded tunables. Protocol constants and security invariants stay fixed.',
      workSummary: 'Agent hardcoded the minimum PBKDF2 iteration count as a fixed constant.',
    })
    expect(verdict.drifted).toBe(false)
  })

  it('decides locally when no instructions bind the work', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not be called')))
    const verdict = await assessInstructionDrift(mock, { instructions: '   ', workSummary: 'anything' })
    expect(verdict.drifted).toBe(false)
    expect(verdict.providerId).toBe('none')
    expect(verdict.latencyMs).toBe(0)
  })

  it('bounds instruction and work text sent as state', async () => {
    let observed = ''
    const mock = createMockDecisionService((request: DecisionRequest) => {
      observed = request.state
      return Promise.resolve(driftResult(0.1))
    })
    await assessInstructionDrift(
      mock,
      { instructions: 'i'.repeat(50_000), workSummary: 'w'.repeat(50_000) },
      { maxInstructionChars: 100, maxWorkChars: 50 },
    )
    expect(observed).toContain('…[truncated]')
    expect(observed.length).toBeLessThan(1000)
  })

  it('fails on a malformed provider answer rather than assuming no drift', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve<DecisionResult>({
        providerId: 'bad',
        answers: { drift: { type: 'score', score: 2, probabilities: {} } },
      }),
    )
    await expect(assessInstructionDrift(mock, EVIDENCE)).rejects.toThrow(/malformed answer/)
  })

  it('forwards cancellation signal', async () => {
    const spy = vi.fn(() => Promise.resolve(driftResult(0.1)))
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()
    await assessInstructionDrift(mock, EVIDENCE, {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
