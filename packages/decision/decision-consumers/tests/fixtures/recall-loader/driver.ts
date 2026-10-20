#!/usr/bin/env node
/** Boot the real recall-filter composition, run one turn, and persist the logged recall. */

import { writeFile } from 'node:fs/promises'
import { boot, resolveConfigPath } from '@deepseek-ai/dsh-app-boot'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import { SessionId } from '@deepseek-ai/dsh-session'
import type {} from './fixture-runtime.ts'

const configPath = process.argv[2]
if (configPath === undefined) throw new Error('recall-filter Loader driver requires a config path')

const ctx = await boot('decision-recall-filter-loader-smoke', resolveConfigPath(configPath, undefined))
try {
  const agent = await ctx.agentLoop.create(SessionId('loader-recall-filter'), { provider: 'fixture', model: 'm' })
  agent.followup(createUserMessage({
    content: [{ type: 'text', text: 'Which language should the new DSH utility use?' }],
    source: { kind: 'user' },
  }))
  await agent.whenIdle()

  await writeFile('./recall-filter-loader-report.json', JSON.stringify({
    llmRequests: ctx.recallFixture.adapter.requests,
    decisionCalls: ctx.recallFixture.provider.calls,
    recall: agent.session.deriveMessages()
      .filter(message => (message.source as Readonly<Record<string, unknown>>)['kind'] === 'plugin:openviking-memory')
      .flatMap(message => message.content.flatMap(block => block.type === 'text' ? [block.text] : [])),
  }))
} finally {
  await ctx.fiber.dispose()
}
