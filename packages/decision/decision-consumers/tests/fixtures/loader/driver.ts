#!/usr/bin/env node
/** Boot the real failover composition, run one turn, and persist observations. */

import { writeFile } from 'node:fs/promises'
import { boot, resolveConfigPath } from '@deepseek-ai/dsh-app-boot'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import { SessionId } from '@deepseek-ai/dsh-session'
import type {} from './fixture-runtime.ts'

const configPath = process.argv[2]
if (configPath === undefined) throw new Error('failover Loader driver requires a config path')

const ctx = await boot('decision-failover-loader-smoke', resolveConfigPath(configPath, undefined))
try {
  const agent = await ctx.agentLoop.create(SessionId('loader-failover'), {
    provider: 'fixture',
    model: 'active',
  })
  agent.followup(createUserMessage({
    content: [{ type: 'text', text: 'Produce the loader failover marker.' }],
    source: { kind: 'user' },
  }))
  await agent.whenIdle()

  const finalMessage = agent.session.deriveMessages().at(-1)
  await writeFile('./failover-loader-report.json', JSON.stringify({
    requests: ctx.failoverFixture.adapter.requests,
    judgeCalls: ctx.failoverFixture.provider.calls,
    retryEvents: agent.session.snapshotEvents().filter(event => event.type === 'llm/retry').length,
    requestHeader: agent.session.requestHeader()?.config,
    finalMessage: finalMessage === undefined
      ? null
      : { role: finalMessage.role, source: finalMessage.source, content: finalMessage.content },
  }))
} finally {
  await ctx.fiber.dispose()
}
