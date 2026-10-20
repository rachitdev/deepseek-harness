#!/usr/bin/env node
/** Boot the real turn-triage composition, run one turn, and persist the routed model. */

import { writeFile } from 'node:fs/promises'
import { boot, resolveConfigPath } from '@deepseek-ai/dsh-app-boot'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import { SessionId } from '@deepseek-ai/dsh-session'
import type {} from './fixture-runtime.ts'

const configPath = process.argv[2]
if (configPath === undefined) throw new Error('turn-triage Loader driver requires a config path')

const ctx = await boot('decision-turn-triage-loader-smoke', resolveConfigPath(configPath, undefined))
try {
  const agent = await ctx.agentLoop.create(SessionId('loader-turn-triage'), { provider: 'fixture', model: 'configured-model' })
  agent.followup(createUserMessage({
    content: [{ type: 'text', text: 'Refactor the session store across its three modules.' }],
    source: { kind: 'user' },
  }))
  await agent.whenIdle()

  await writeFile('./turn-triage-loader-report.json', JSON.stringify({
    decisionCalls: ctx.triageFixture.provider.calls,
    requestedModels: ctx.triageFixture.adapter.models,
    loggedHeader: agent.session.requestHeader()?.config,
  }))
} finally {
  await ctx.fiber.dispose()
}
