import { describe, expect, it, vi } from 'vitest'
import type { DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { adjudicateSkillSelection } from '../src/skill-adjudicator.ts'
import { createMockDecisionService } from './helpers.ts'

const candidates = [
  { name: 'architecture', description: 'Design the technical architecture.' },
  { name: 'diagram-design', description: 'Render a validated SVG diagram.' },
  { name: 'code-review', description: 'Review code for defects.' },
]

describe('adjudicateSkillSelection', () => {
  it('keeps a sole DSH/JEV agreement after one decision call', async () => {
    const evaluate = vi.fn(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: {
        primary: { type: 'choice', choice: 'code-review', probabilities: { architecture: 0.02, 'diagram-design': 0.01, 'code-review': 0.97 } },
        'relevant:architecture': { type: 'boolean', probability: 0.02 },
        'relevant:diagram-design': { type: 'boolean', probability: 0.01 },
        'relevant:code-review': { type: 'boolean', probability: 0.98 },
      },
    }))

    const result = await adjudicateSkillSelection(createMockDecisionService(evaluate), {
      request: 'Review this pull request.',
      dshProposed: ['code-review'],
      candidates,
    })

    expect(result).toMatchObject({ primary: 'code-review', parallel: [], later: [], adjudicated: false, disagreement: false })
    expect(evaluate).toHaveBeenCalledTimes(1)
  })

  it('compares a disagreement and separates parallel from later skills', async () => {
    const evaluate = vi.fn<(request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult>>()
      .mockResolvedValueOnce({
        providerId: 'jev',
        answers: {
          primary: { type: 'choice', choice: 'architecture', probabilities: { architecture: 0.8, 'diagram-design': 0.18, 'code-review': 0.02 } },
          'relevant:architecture': { type: 'boolean', probability: 0.95 },
          'relevant:diagram-design': { type: 'boolean', probability: 0.9 },
          'relevant:code-review': { type: 'boolean', probability: 0.1 },
        },
      })
      .mockResolvedValueOnce({
        providerId: 'jev',
        answers: {
          primary: { type: 'choice', choice: 'architecture', probabilities: { architecture: 0.9, 'diagram-design': 0.1 } },
          'phase:architecture': { type: 'choice', choice: 'primary_now', probabilities: { primary_now: 0.95, parallel_now: 0.02, later_depth: 0.02, not_needed: 0.01 } },
          'phase:diagram-design': { type: 'choice', choice: 'later_depth', probabilities: { primary_now: 0.02, parallel_now: 0.08, later_depth: 0.88, not_needed: 0.02 } },
        },
      })

    const result = await adjudicateSkillSelection(createMockDecisionService(evaluate), {
      request: 'Design an architecture and then make its SVG diagram.',
      dshProposed: ['diagram-design'],
      candidates,
    })

    expect(result).toMatchObject({
      primary: 'architecture',
      parallel: [],
      later: ['diagram-design'],
      highProbability: ['architecture', 'diagram-design'],
      adjudicated: true,
      disagreement: true,
    })
    expect(evaluate).toHaveBeenCalledTimes(2)
    expect(evaluate.mock.calls[1]?.[0].questions).toHaveProperty('phase:diagram-design')
  })

  it('rejects a DSH proposal outside the bounded candidate set', async () => {
    await expect(adjudicateSkillSelection(createMockDecisionService(() => Promise.resolve({ providerId: 'jev', answers: {} })), {
      request: 'Review this.',
      dshProposed: ['missing'],
      candidates,
    })).rejects.toThrow('every DSH proposal')
  })
})
