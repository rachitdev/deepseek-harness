import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import AgentRegistry from '@deepseek-ai/dsh-agent'
import DecisionRuntime from '@deepseek-ai/dsh-decision'
import LlmRuntime from '@deepseek-ai/dsh-llm'
import SessionStore from '@deepseek-ai/dsh-session'
import SessionProjectionRegistry from '@deepseek-ai/dsh-session-projection'
import * as consumersPlugin from '../src/index.ts'

async function contextWithDependencies(): Promise<Context> {
  const ctx = new Context()
  await ctx.plugin(LlmRuntime)
  await ctx.plugin(SessionStore)
  await ctx.plugin(SessionProjectionRegistry)
  await ctx.plugin(AgentRegistry)
  await ctx.plugin(DecisionRuntime, {})
  return ctx
}

describe('decision-consumers plugin', () => {
  it('registers on a context carrying its injected services and disposes cleanly', async () => {
    const ctx = await contextWithDependencies()
    const fiber = await ctx.plugin(consumersPlugin, {})
    expect(fiber).toBeDefined()
    await fiber.dispose()
    await ctx.fiber.dispose()
  })

  it('rejects enabled failover without a fallback route', async () => {
    const ctx = await contextWithDependencies()
    await expect(ctx.plugin(consumersPlugin, {
      failover: { enabled: true, candidates: [] },
    })).rejects.toThrow(/requires at least one fallback candidate route/)
    await ctx.fiber.dispose()
  })

  it('rejects duplicate fallback routes', async () => {
    const ctx = await contextWithDependencies()
    await expect(ctx.plugin(consumersPlugin, {
      failover: {
        enabled: true,
        candidates: [
          { provider: 'mock', model: 'same' },
          { provider: 'mock', model: 'same' },
        ],
      },
    })).rejects.toThrow(/must be unique/)
    await ctx.fiber.dispose()
  })
})
