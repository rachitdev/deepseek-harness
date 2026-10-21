/**
 * The delegation-rejection notice: one report per tool under the
 * delegation-sourced `never` pin, silence under any other policy, and the
 * delivery channel a parent's status and teardown select. The draining-parent
 * channel is a teardown race — the manager disposes a resident child before it
 * can ask again — so the decision is pinned here and the delivery path itself
 * is covered end-to-end by the queued notice and by settlement notices.
 */

import { afterEach, describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import type { Agent } from '@deepseek-ai/dsh-agent'
import AgentLoop from '@deepseek-ai/dsh-agent-loop'
import { mountAgentLoopTestDependencies } from '@deepseek-ai/dsh-agent-loop-testkit'
import ApprovalService, { ApprovalRequestId } from '@deepseek-ai/dsh-user-approval'
import { SessionId } from '@deepseek-ai/dsh-session'
import { observeDelegatedApprovalRejections, PendingApprovalAsks } from '../src/child-agent.ts'
import { runtimeNoticeChannel } from '../src/manager.ts'

const contexts: Context[] = []

afterEach(async () => {
  for (const ctx of contexts.splice(0).reverse()) await ctx.fiber.dispose()
})

/** Mount the approval capability and one idle Agent to observe. */
async function setup(): Promise<{ child: Agent }> {
  const ctx = new Context()
  contexts.push(ctx)
  await mountAgentLoopTestDependencies(ctx)
  await ctx.plugin(ApprovalService)
  await ctx.plugin(AgentLoop, { agents: [] })
  const child = await ctx.agentLoop.create(
    SessionId('delegated-child'),
    { provider: 'mock', model: 'mock' },
    {},
  )
  return { child }
}

/** Record one ask and its terminal decision for a tool. */
function decide(child: Agent, id: string, toolName: string, outcome: 'rejected' | 'allowed-once' | 'unavailable'): void {
  child.session.append('approval/asked', { id: ApprovalRequestId(id), toolName })
  child.session.append('approval/decided', { id: ApprovalRequestId(id), outcome })
}

describe('delegation rejection notice', () => {
  it('reports one rejection per tool name within one residency', async () => {
    const { child } = await setup()
    const reported = new Set<string>()
    const delivered: string[] = []
    observeDelegatedApprovalRejections(child, { pinned: true, reported }, (message) => {
      delivered.push(message.source.kind === 'subagent-delegation' ? message.source.toolName : 'unexpected')
    })

    decide(child, 'first', 'write', 'rejected')
    decide(child, 'second', 'write', 'rejected')
    decide(child, 'third', 'bash', 'rejected')

    expect(delivered).toEqual(['write', 'bash'])
    expect([...reported]).toEqual(['write', 'bash'])
  })

  it('stays silent unless the delegation-sourced pin is the latest policy', async () => {
    const { child } = await setup()
    const delivered: string[] = []
    observeDelegatedApprovalRejections(child, { pinned: false, reported: new Set() }, (message) => {
      delivered.push(message.source.kind === 'subagent-delegation' ? message.source.toolName : 'unexpected')
    })

    // A `never` the child chose itself is not a delegation pin.
    child.session.append('approval/policy', { policy: 'never' })
    decide(child, 'own-never', 'write', 'rejected')
    expect(delivered).toEqual([])

    child.session.append('approval/policy', { policy: 'never', source: 'delegation' })
    decide(child, 'delegated', 'write', 'rejected')
    expect(delivered).toEqual(['write'])

    // A later answerable policy takes the reporting duty with it.
    child.session.append('approval/policy', { policy: 'ask' })
    decide(child, 'answerable', 'bash', 'rejected')
    expect(delivered).toEqual(['write'])
  })

  it('does not attribute a policy a cold resume only replays', async () => {
    const { child } = await setup()
    // A resumed child replays its log, including a `never` it may have set
    // itself; nothing in that history proves the delegation pinned it.
    child.session.append('approval/policy', { policy: 'never' })
    const delivered: string[] = []
    observeDelegatedApprovalRejections(child, { pinned: false, reported: new Set() }, (message) => {
      delivered.push(message.source.kind === 'subagent-delegation' ? message.source.toolName : 'unexpected')
    })

    decide(child, 'replayed-never', 'write', 'rejected')

    expect(delivered).toEqual([])
  })

  it('frees every ask at its terminal decision and remembers only reported rejections', async () => {
    const { child } = await setup()
    const delivered: string[] = []
    observeDelegatedApprovalRejections(child, { pinned: true, reported: new Set() }, (message) => {
      delivered.push(message.source.kind === 'subagent-delegation' ? message.source.toolName : 'unexpected')
    })

    // Non-rejecting outcomes must not retain their ask, and must not silence a
    // later rejection of the same tool.
    decide(child, 'granted', 'write', 'allowed-once')
    decide(child, 'unavailable', 'write', 'unavailable')
    expect(delivered).toEqual([])

    decide(child, 'rejected', 'write', 'rejected')
    expect(delivered).toEqual(['write'])
  })

  it('leaves an already reported tool to its earlier notice', async () => {
    const { child } = await setup()
    const delivered: string[] = []
    observeDelegatedApprovalRejections(
      child,
      { pinned: true, reported: new Set(['write']) },
      (message) => { delivered.push(message.source.kind === 'subagent-delegation' ? message.source.toolName : 'unexpected') },
    )

    decide(child, 'again', 'write', 'rejected')
    decide(child, 'new-tool', 'bash', 'rejected')

    expect(delivered).toEqual(['bash'])
  })

  it('forgets each ask at its decision, whatever the outcome', () => {
    const asks = new PendingApprovalAsks()

    asks.asked('granted', 'write')
    asks.asked('rejected', 'bash')
    asks.asked('unavailable', 'edit')
    expect(asks.pending).toBe(3)

    expect(asks.decide('granted')).toBe('write')
    expect(asks.decide('rejected')).toBe('bash')
    expect(asks.decide('unavailable')).toBe('edit')
    expect(asks.pending).toBe(0)

    // An unpaired decision adds nothing to remember.
    expect(asks.decide('granted')).toBeUndefined()
    expect(asks.pending).toBe(0)
  })

  it('selects the parent channel from its status and teardown', () => {
    expect(runtimeNoticeChannel('idle', false)).toBe('queue')
    expect(runtimeNoticeChannel('running', false)).toBe('steer')
    expect(runtimeNoticeChannel('idle', true)).toBe('inject')
    expect(runtimeNoticeChannel('running', true)).toBe('inject')
  })
})
