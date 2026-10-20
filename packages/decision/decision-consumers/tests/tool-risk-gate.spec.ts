import { describe, expect, it, vi } from 'vitest'
import { assessToolRisk } from '../src/tool-risk-gate.ts'
import { createMockDecisionService } from './helpers.ts'

describe('assessToolRisk', () => {
  it('handles missing choice probability gracefully', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          risk_score: { type: 'score', score: 0.1, probabilities: { '0': 1 } },
          requires_human: { type: 'boolean', probability: 0 },
          category: { type: 'choice', choice: 'read', probabilities: {} },
        },
      }),
    )

    const assessment = await assessToolRisk(mock, {
      toolName: 'read',
      toolArgs: {},
    })
    expect(assessment.confidence).toBe(0)
  })

  it('allows safe read-only actions', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          risk_score: { type: 'score', score: 0.1, probabilities: { '0': 0.9, '1': 0.1, '2': 0, '3': 0 } },
          requires_human: { type: 'boolean', probability: 0.05 },
          category: { type: 'choice', choice: 'read', probabilities: { read: 0.95, local_edit: 0.05, destructive_edit: 0, external_action: 0 } },
        },
      }),
    )

    const assessment = await assessToolRisk(mock, {
      toolName: 'read',
      toolArgs: { file_path: 'src/index.ts' },
    })

    expect(assessment.recommendedAction).toBe('allow')
    expect(assessment.riskCategory).toBe('read')
    expect(assessment.riskLevel).toBe(0.1)
    expect(assessment.requiresConfirmationProbability).toBe(0.05)
  })

  it('prompts on mutations exceeding maxAutoApproveRisk', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          risk_score: { type: 'score', score: 1.8, probabilities: { '0': 0, '1': 0.3, '2': 0.7, '3': 0 } },
          requires_human: { type: 'boolean', probability: 0.65 },
          category: { type: 'choice', choice: 'destructive_edit', probabilities: { read: 0, local_edit: 0.2, destructive_edit: 0.8, external_action: 0 } },
        },
      }),
    )

    const assessment = await assessToolRisk(mock, {
      toolName: 'bash',
      toolArgs: { command: 'rm -rf dist/' },
    })

    expect(assessment.recommendedAction).toBe('prompt')
    expect(assessment.riskCategory).toBe('destructive_edit')
  })

  it('denies system-critical or unrecoverable actions', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          risk_score: { type: 'score', score: 2.9, probabilities: { '0': 0, '1': 0, '2': 0.1, '3': 0.9 } },
          requires_human: { type: 'boolean', probability: 0.99 },
          category: { type: 'choice', choice: 'destructive_edit', probabilities: { read: 0, local_edit: 0, destructive_edit: 1, external_action: 0 } },
        },
      }),
    )

    const assessment = await assessToolRisk(mock, {
      toolName: 'bash',
      toolArgs: { command: 'git push origin main --force' },
    })

    expect(assessment.recommendedAction).toBe('deny')
  })

  it('fails on unexpected provider answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          risk_score: { type: 'choice', choice: 'high', probabilities: {} },
          requires_human: { type: 'boolean', probability: 0.5 },
          category: { type: 'choice', choice: 'read', probabilities: {} },
        },
      }),
    )

    await expect(assessToolRisk(mock, {
      toolName: 'read',
      toolArgs: {},
    })).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards cancellation signal', async () => {
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          risk_score: { type: 'score' as const, score: 0, probabilities: { '0': 1, '1': 0, '2': 0, '3': 0 } },
          requires_human: { type: 'boolean' as const, probability: 0 },
          category: { type: 'choice' as const, choice: 'read', probabilities: { read: 1 } },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await assessToolRisk(mock, { toolName: 'read', toolArgs: {} }, {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
