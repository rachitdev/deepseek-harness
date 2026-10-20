import { describe, expect, it, vi } from 'vitest'
import type { DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { cascadeSkillSelection, type SkillCascadeCategory } from '../src/skill-cascade.ts'
import { createMockDecisionService } from './helpers.ts'

const security: SkillCascadeCategory = {
  name: 'security',
  description: 'Security review, pentesting, and vulnerability fixes.',
  skills: [
    { name: 'penetration-testing-with-strix', description: 'Run a Strix pentest.' },
    { name: 'find-security-vulnerabilities-in-code', description: 'Audit source for vulnerabilities.' },
  ],
}
const design: SkillCascadeCategory = {
  name: 'design',
  description: 'Diagrams and visual design.',
  skills: [{ name: 'diagram-design', description: 'Render an SVG diagram.' }],
}

describe('cascadeSkillSelection', () => {
  it('asks category then skill and returns both verdicts', async () => {
    const requests: DecisionRequest[] = []
    const evaluate = vi.fn((request: DecisionRequest) => {
      requests.push(request)
      if (requests.length === 1) {
        return Promise.resolve<DecisionResult>({
          providerId: 'jev',
          answers: {
            category: { type: 'choice', choice: 'security', probabilities: { security: 0.9, design: 0.1 } },
            needs_skill: { type: 'boolean', probability: 0.8 },
          },
        })
      }
      return Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: {
          skill: {
            type: 'choice',
            choice: 'find-security-vulnerabilities-in-code',
            probabilities: { 'penetration-testing-with-strix': 0.3, 'find-security-vulnerabilities-in-code': 0.7 },
          },
        },
      })
    })

    const result = await cascadeSkillSelection(createMockDecisionService(evaluate), {
      request: 'Audit this repo for injection bugs.',
      categories: [security, design],
    })

    expect(result).toEqual({
      category: 'security',
      categoryConfidence: 0.9,
      needsSkillProbability: 0.8,
      skill: 'find-security-vulnerabilities-in-code',
      skillConfidence: 0.7,
      calls: 2,
      providerIds: ['jev', 'jev'],
    })
    expect(Object.keys(requests[0]!.questions)).toEqual(['category', 'needs_skill'])
    // Stage two sees only the winning category's skills, not the whole catalog.
    expect(requests[1]!.questions['skill']).toMatchObject({
      criteria: {
        'penetration-testing-with-strix': 'Run a Strix pentest.',
        'find-security-vulnerabilities-in-code': 'Audit source for vulnerabilities.',
      },
    })
    expect(requests[1]!.state).not.toContain('diagram-design')
  })

  it('skips the skill stage when the winning category holds one skill', async () => {
    const evaluate = vi.fn(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: {
        category: { type: 'choice', choice: 'design', probabilities: { security: 0.05, design: 0.95 } },
        needs_skill: { type: 'boolean', probability: 0.9 },
      },
    }))

    const result = await cascadeSkillSelection(createMockDecisionService(evaluate), {
      request: 'Draw the architecture.',
      categories: [security, design],
    })

    expect(result).toMatchObject({ category: 'design', skill: 'diagram-design', skillConfidence: 1, calls: 1 })
    expect(evaluate).toHaveBeenCalledTimes(1)
  })

  it('skips the category stage when only one category exists', async () => {
    const evaluate = vi.fn(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: {
        skill: { type: 'choice', choice: 'penetration-testing-with-strix', probabilities: { 'penetration-testing-with-strix': 0.6, 'find-security-vulnerabilities-in-code': 0.4 } },
      },
    }))

    const result = await cascadeSkillSelection(createMockDecisionService(evaluate), {
      request: 'Pentest the staging host.',
      categories: [security],
    })

    expect(result).toMatchObject({ category: 'security', categoryConfidence: 1, needsSkillProbability: 1, skill: 'penetration-testing-with-strix', calls: 1 })
  })

  it('makes no call for a single category with a single skill', async () => {
    const evaluate = vi.fn()
    const result = await cascadeSkillSelection(createMockDecisionService(evaluate), { request: 'x', categories: [design] })
    expect(result).toMatchObject({ skill: 'diagram-design', calls: 0, providerIds: [] })
    expect(evaluate).not.toHaveBeenCalled()
  })

  it('rejects empty, duplicate, and skill-less categories', async () => {
    const service = createMockDecisionService(vi.fn())
    await expect(cascadeSkillSelection(service, { request: 'x', categories: [] })).rejects.toThrow('at least one category')
    await expect(cascadeSkillSelection(service, { request: 'x', categories: [design, design] })).rejects.toThrow('duplicated')
    await expect(cascadeSkillSelection(service, { request: 'x', categories: [{ ...design, skills: [] }] })).rejects.toThrow('has no skills')
  })

  it('fails closed on unknown choices and wrong answer types', async () => {
    const unknownCategory = createMockDecisionService(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: {
        category: { type: 'choice', choice: 'ghost', probabilities: { ghost: 1 } },
        needs_skill: { type: 'boolean', probability: 1 },
      },
    }))
    await expect(cascadeSkillSelection(unknownCategory, { request: 'x', categories: [security, design] })).rejects.toThrow('unknown category "ghost"')

    const missingNeeds = createMockDecisionService(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: { category: { type: 'choice', choice: 'design', probabilities: { design: 1 } } },
    }))
    await expect(cascadeSkillSelection(missingNeeds, { request: 'x', categories: [security, design] })).rejects.toThrow('"needs_skill"')

    const unknownSkill = createMockDecisionService(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: { skill: { type: 'choice', choice: 'ghost', probabilities: { ghost: 1 } } },
    }))
    await expect(cascadeSkillSelection(unknownSkill, { request: 'x', categories: [security] })).rejects.toThrow('unknown skill "ghost"')

    const wrongType = createMockDecisionService(() => Promise.resolve<DecisionResult>({
      providerId: 'jev',
      answers: { skill: { type: 'boolean', probability: 1 } },
    }))
    await expect(cascadeSkillSelection(wrongType, { request: 'x', categories: [security] })).rejects.toThrow('expected choice answer "skill"')
  })

  it('forwards the cancellation signal to every call', async () => {
    const controller = new AbortController()
    const signals: (AbortSignal | undefined)[] = []
    const service = createMockDecisionService((request, signal) => {
      signals.push(signal)
      return Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: 'category' in request.questions
          ? { category: { type: 'choice', choice: 'security', probabilities: { security: 1 } }, needs_skill: { type: 'boolean', probability: 1 } }
          : { skill: { type: 'choice', choice: 'penetration-testing-with-strix', probabilities: { 'penetration-testing-with-strix': 1 } } },
      })
    })
    await cascadeSkillSelection(service, { request: 'x', categories: [security, design] }, controller.signal)
    expect(signals).toEqual([controller.signal, controller.signal])
  })
})
