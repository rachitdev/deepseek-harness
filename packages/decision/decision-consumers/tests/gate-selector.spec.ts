import { describe, expect, it, vi } from 'vitest'
import { selectVerificationGates, SUITE_COMMANDS } from '../src/gate-selector.ts'
import { createMockDecisionService } from './helpers.ts'

describe('selectVerificationGates', () => {
  it('handles missing choice probability gracefully', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          suite: { type: 'choice', choice: 'doc_only', probabilities: {} },
          blast_radius: { type: 'score', score: 0, probabilities: { '0': 1 } },
          needs_coverage: { type: 'boolean', probability: 0 },
        },
      }),
    )

    const selection = await selectVerificationGates(mock, {
      diffStat: 'README.md',
      changedFiles: ['README.md'],
    })

    expect(selection.confidence).toBe(0)
  })

  it('selects doc_only suite for markdown and comment changes', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          suite: { type: 'choice', choice: 'doc_only', probabilities: { doc_only: 0.98, lint_and_unit: 0.02, full_coverage: 0, full_with_e2e: 0 } },
          blast_radius: { type: 'score', score: 0.1, probabilities: { '0': 0.9, '1': 0.1, '2': 0, '3': 0 } },
          needs_coverage: { type: 'boolean', probability: 0.01 },
        },
      }),
    )

    const selection = await selectVerificationGates(mock, {
      diffStat: ' docs/subsystems/decision.md | 45 +++---\n 1 file changed, 25 insertions(+), 20 deletions(-)',
      changedFiles: ['docs/subsystems/decision.md'],
    })

    expect(selection.recommendedSuite).toBe('doc_only')
    expect(selection.commands).toEqual(SUITE_COMMANDS.doc_only)
    expect(selection.needsCoverage).toBe(false)
  })

  it('selects full_coverage suite for cross-package seam and runtime changes', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          suite: { type: 'choice', choice: 'full_coverage', probabilities: { doc_only: 0, lint_and_unit: 0.1, full_coverage: 0.85, full_with_e2e: 0.05 } },
          blast_radius: { type: 'score', score: 2.6, probabilities: { '0': 0, '1': 0.1, '2': 0.3, '3': 0.6 } },
          needs_coverage: { type: 'boolean', probability: 0.95 },
        },
      }),
    )

    const selection = await selectVerificationGates(mock, {
      diffStat: ' packages/decision/decision/src/index.ts | 120 +++---\n 1 file changed',
      changedFiles: ['packages/decision/decision/src/index.ts'],
    })

    expect(selection.recommendedSuite).toBe('full_coverage')
    expect(selection.commands).toEqual(SUITE_COMMANDS.full_coverage)
    expect(selection.needsCoverage).toBe(true)
  })

  it('upgrades suite to full_coverage if blast radius is high despite choice', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          suite: { type: 'choice', choice: 'lint_and_unit', probabilities: { doc_only: 0, lint_and_unit: 0.6, full_coverage: 0.4, full_with_e2e: 0 } },
          blast_radius: { type: 'score', score: 2.2, probabilities: { '0': 0, '1': 0.1, '2': 0.8, '3': 0.1 } },
          needs_coverage: { type: 'boolean', probability: 0.7 },
        },
      }),
    )

    const selection = await selectVerificationGates(mock, {
      diffStat: ' packages/core/agent-loop/src/index.ts | 40 +++---',
      changedFiles: ['packages/core/agent-loop/src/index.ts'],
    })

    expect(selection.recommendedSuite).toBe('full_coverage')
    expect(selection.needsCoverage).toBe(true)
  })

  it('fails on unexpected provider answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          suite: { type: 'score', score: 0, probabilities: {} },
          blast_radius: { type: 'score', score: 0, probabilities: {} },
          needs_coverage: { type: 'boolean', probability: 0 },
        },
      }),
    )

    await expect(selectVerificationGates(mock, {
      diffStat: '',
      changedFiles: [],
    })).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards cancellation signal', async () => {
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          suite: { type: 'choice' as const, choice: 'doc_only', probabilities: { doc_only: 1 } },
          blast_radius: { type: 'score' as const, score: 0, probabilities: { '0': 1 } },
          needs_coverage: { type: 'boolean' as const, probability: 0 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await selectVerificationGates(mock, { diffStat: '', changedFiles: [] }, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
