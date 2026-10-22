import { describe, expect, it } from 'vitest'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import { Session, SessionId } from '@deepseek-ai/dsh-session'
import { latestUserTaskText, type Agent } from '../src/index.ts'

const FALLBACK = 'fallback task'

function createAgent(id: string): Agent {
  return { session: Session.create(SessionId(id)) } as Agent
}

function appendUser(agent: Agent, text: string, source?: 'user' | 'notice'): void {
  agent.session.append('user/message', createUserMessage({
    content: [{ type: 'text', text }],
    source: source === 'notice'
      ? { kind: 'model-selection', form: 'notice', summary: 'plain → capable' }
      : { kind: 'user' },
  }), { surfaceOp: 'append' })
}

describe('latestUserTaskText', () => {
  it('returns the newest user-authored text', () => {
    const agent = createAgent('latest-user-task-newest')
    appendUser(agent, 'older request')
    appendUser(agent, 'newer request')

    expect(latestUserTaskText(agent, 4000, FALLBACK)).toBe('newer request')
  })

  it('truncates to the caller budget without reflowing', () => {
    const agent = createAgent('latest-user-task-truncate')
    appendUser(agent, 'abcdefghij')

    expect(latestUserTaskText(agent, 4, FALLBACK)).toBe('abcd')
  })

  it('falls back when the session holds no user message', () => {
    expect(latestUserTaskText(createAgent('latest-user-task-empty'), 4000, FALLBACK)).toBe(FALLBACK)
  })

  it('falls back when the newest user message carries no text', () => {
    const agent = createAgent('latest-user-task-blank')
    appendUser(agent, '   ')

    expect(latestUserTaskText(agent, 4000, FALLBACK)).toBe(FALLBACK)
  })

  it('ignores a message the user did not author', () => {
    const agent = createAgent('latest-user-task-sourced')
    appendUser(agent, 'runtime notice text', 'notice')

    expect(latestUserTaskText(agent, 4000, FALLBACK)).toBe(FALLBACK)
  })
})
