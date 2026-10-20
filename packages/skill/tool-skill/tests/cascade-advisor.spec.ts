import { describe, expect, it, vi } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import type { DecisionRequest, DecisionResult, DecisionService } from '@deepseek-ai/dsh-decision'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import type { Agent } from '@deepseek-ai/dsh-agent'
import type { SkillSummary } from '@deepseek-ai/dsh-skill'
import { createCascadeAdvisor, groupCatalog } from '../src/cascade-advisor.ts'

function summary(name: string): SkillSummary {
  return { name, description: `${name} description`, invocation: { modelInvocable: true, userInvocable: true }, source: 'runtime', provider: 'runtime' }
}

const ungrouped = { name: 'other', description: 'Other skills.' }

describe('groupCatalog', () => {
  it('keeps configured order, assigns a repeated skill to its first category, and drops empty categories', () => {
    const skills = ['a', 'b', 'c', 'd'].map(summary)
    const categories = groupCatalog(skills, {
      ungrouped,
      categories: [
        { name: 'first', description: 'First.', skills: ['b', 'a'] },
        { name: 'second', description: 'Second.', skills: ['a', 'c'] },
        { name: 'ghost', description: 'Missing.', skills: ['zzz'] },
      ],
    })
    expect(categories.map(category => [category.name, category.skills.map(skill => skill.name)])).toEqual([
      ['first', ['b', 'a']],
      ['second', ['c']],
      ['other', ['d']],
    ])
  })

  it('omits the ungrouped category when every live skill is grouped', () => {
    const categories = groupCatalog([summary('a')], { ungrouped, categories: [{ name: 'only', description: 'Only.', skills: ['a'] }] })
    expect(categories.map(category => category.name)).toEqual(['only'])
  })

  it('returns no categories for an empty catalog', () => {
    expect(groupCatalog([], { ungrouped, categories: [{ name: 'x', description: 'X.', skills: ['a'] }] })).toEqual([])
  })
})

describe('createCascadeAdvisor', () => {
  const agent = { id: 'agent-1' } as Agent
  const messages = [createUserMessage({ content: [{ type: 'text', text: 'Do the thing.' }], source: { kind: 'user' } })]
  const options = { categories: [], ungrouped, maxTaskChars: 5, timeoutMs: 1000 }

  function service(evaluate: (request: DecisionRequest) => Promise<DecisionResult>): DecisionService {
    return { evaluate, registerProvider: () => () => true } as Pick<DecisionService, 'evaluate' | 'registerProvider'> as DecisionService
  }

  it('makes no decision call and logs nothing for an empty catalog', async () => {
    const ctx = new Context()
    const info = vi.fn()
    ctx.logger.info = info
    const evaluate = vi.fn()
    const advisor = createCascadeAdvisor(ctx, service(evaluate), options)
    advisor.start(agent, 1, messages, () => Promise.resolve([]))
    await advisor.compare(agent, 'a')
    expect(evaluate).not.toHaveBeenCalled()
    expect(info).not.toHaveBeenCalled()
  })

  it('truncates the task, reports zero confidence for an unscored winner, and logs disagreement', async () => {
    const ctx = new Context()
    const info = vi.fn()
    ctx.logger.info = info
    const requests: DecisionRequest[] = []
    const advisor = createCascadeAdvisor(ctx, service((request) => {
      requests.push(request)
      return Promise.resolve({ providerId: 'jev', answers: { skill: { type: 'choice', choice: 'b', probabilities: {} } } })
    }), options)
    advisor.start(agent, 1, messages, () => Promise.resolve([summary('a'), summary('b')]))
    await advisor.compare(agent, 'a')
    expect(requests[0]!.state).toContain('User request: Do th\n')
    expect(info.mock.calls[0]).toContain('0.00')
    expect(info.mock.calls[1]?.slice(-1)).toEqual(['disagree'])
  })

  it('forgets a disposed agent so a later load compares against nothing', async () => {
    const ctx = new Context()
    const info = vi.fn()
    ctx.logger.info = info
    const advisor = createCascadeAdvisor(ctx, service(vi.fn()), options)
    advisor.start(agent, 1, messages, () => Promise.resolve([summary('a')]))
    advisor.forget(agent)
    await advisor.compare(agent, 'a')
    expect(info.mock.calls.some(args => String(args[0]).includes('model loaded'))).toBe(false)
  })
})
