import { describe, expect, it, vi } from 'vitest'
import { selectSkill } from '../src/skill-selector.ts'
import type { DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { createMockDecisionService } from './helpers.ts'

const SKILLS = [
  { name: 'repository-review', description: 'Inspect code and report file-backed findings.' },
  { name: 'database-migrations', description: 'Review schema changes and migration safety.' },
]

describe('selectSkill', () => {
  it('answers locally without a provider call on an empty catalog', async () => {
    const spy = vi.fn()
    const mock = createMockDecisionService(() => {
      spy()
      return Promise.resolve<DecisionResult>({ providerId: 'jev', answers: {} })
    })

    const selection = await selectSkill(mock, { userRequest: 'Summarize this file', availableSkills: [] })

    expect(selection.action).toEqual({ type: 'direct_answer' })
    expect(selection.providerId).toBe('')
    expect(spy).not.toHaveBeenCalled()
  })

  it('loads the chosen skill when confident and low-risk', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          needs_skill: { type: 'boolean', probability: 0.9 },
          skill: { type: 'choice', choice: 'database-migrations', probabilities: { 'repository-review': 0.1, 'database-migrations': 0.85, none: 0.05 } },
          risk: { type: 'score', score: 0.4, probabilities: { '0': 0.8, '1': 0.2, '2': 0 } },
        },
      }),
    )

    const selection = await selectSkill(mock, { userRequest: 'Review the payments migration', availableSkills: SKILLS })

    expect(selection.action).toEqual({ type: 'load_skill', skill: 'database-migrations' })
    expect(selection.confidence).toBe(0.85)
    expect(selection.riskScore).toBe(0.4)
  })

  it('answers directly when the turn does not need a skill', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          needs_skill: { type: 'boolean', probability: 0.2 },
          skill: { type: 'choice', choice: 'none', probabilities: { 'repository-review': 0.1, 'database-migrations': 0.1, none: 0.8 } },
          risk: { type: 'score', score: 0, probabilities: { '0': 1, '1': 0, '2': 0 } },
        },
      }),
    )

    const selection = await selectSkill(mock, { userRequest: 'What is 2 + 2?', availableSkills: SKILLS })

    expect(selection.action).toEqual({ type: 'direct_answer' })
  })

  it('clarifies when the provider picks no skill', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          needs_skill: { type: 'boolean', probability: 0.9 },
          skill: { type: 'choice', choice: 'none', probabilities: { 'repository-review': 0.2, 'database-migrations': 0.2, none: 0.6 } },
          risk: { type: 'score', score: 0.5, probabilities: { '0': 0.6, '1': 0.4, '2': 0 } },
        },
      }),
    )

    const selection = await selectSkill(mock, { userRequest: 'Rewrite our docs', availableSkills: SKILLS })

    expect(selection.action.type).toBe('clarify')
  })

  it('clarifies when the chosen skill is below the confidence threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          needs_skill: { type: 'boolean', probability: 0.9 },
          skill: { type: 'choice', choice: 'repository-review', probabilities: { 'repository-review': 0.5, 'database-migrations': 0.3, none: 0.2 } },
          risk: { type: 'score', score: 0.5, probabilities: { '0': 0.6, '1': 0.4, '2': 0 } },
        },
      }),
    )

    const selection = await selectSkill(mock, { userRequest: 'Review the payments migration', availableSkills: SKILLS })

    expect(selection.action.type).toBe('clarify')
  })

  it('requires approval when the risk rubric reaches the approval threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          needs_skill: { type: 'boolean', probability: 0.9 },
          skill: { type: 'choice', choice: 'database-migrations', probabilities: { 'repository-review': 0.05, 'database-migrations': 0.9, none: 0.05 } },
          risk: { type: 'score', score: 1.6, probabilities: { '0': 0, '1': 0.4, '2': 0.6 } },
        },
      }),
    )

    const selection = await selectSkill(mock, { userRequest: 'Migrate production tables', availableSkills: SKILLS })

    expect(selection.action).toEqual({ type: 'load_skill_with_approval', skill: 'database-migrations' })
  })

  it('honors custom thresholds', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          needs_skill: { type: 'boolean', probability: 0.9 },
          skill: { type: 'choice', choice: 'repository-review', probabilities: { 'repository-review': 0.6, 'database-migrations': 0.2, none: 0.2 } },
          risk: { type: 'score', score: 0.4, probabilities: { '0': 0.8, '1': 0.2, '2': 0 } },
        },
      }),
    )

    const selection = await selectSkill(
      mock,
      { userRequest: 'Review the payments migration', availableSkills: SKILLS },
      { minSkillConfidence: 0.55 },
    )

    expect(selection.action.type).toBe('load_skill')
  })

  it('fails on unexpected provider answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          needs_skill: { type: 'choice', choice: 'yes', probabilities: {} },
          skill: { type: 'choice', choice: 'none', probabilities: {} },
          risk: { type: 'score', score: 0, probabilities: {} },
        },
      }),
    )

    await expect(
      selectSkill(mock, { userRequest: 'Review the payments migration', availableSkills: SKILLS }),
    ).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards the cancellation signal', async () => {
    const spy = vi.fn((_request: DecisionRequest, _signal?: AbortSignal) =>
      Promise.resolve<DecisionResult>({
        providerId: 'test',
        answers: {
          needs_skill: { type: 'boolean' as const, probability: 0.9 },
          skill: { type: 'choice' as const, choice: 'database-migrations', probabilities: { 'repository-review': 0.05, 'database-migrations': 0.9, none: 0.05 } },
          risk: { type: 'score' as const, score: 0, probabilities: { '0': 1, '1': 0, '2': 0 } },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await selectSkill(mock, { userRequest: 'Review the payments migration', availableSkills: SKILLS }, {}, controller.signal)

    expect(typeof spy.mock.calls[0]?.[0].state).toBe('string')
    expect(spy.mock.calls[0]?.[1]).toBe(controller.signal)
  })
})
