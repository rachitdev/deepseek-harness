/**
 * Text extraction shared by the live pre-step consumers: the direct user request of a step and the
 * tail of the previous assistant reply, both bounded before they reach a decision provider.
 * @module @deepseek-ai/dsh-decision-consumers/turn-text
 */

import type { Agent } from '@deepseek-ai/dsh-agent'
import type { Message, UserMessage } from '@deepseek-ai/dsh-llm'

/**
 * Join the text blocks of one message's content.
 * @param content - message content blocks.
 * @returns text blocks joined by blank lines; non-text blocks are skipped.
 */
export function textOf(content: Message['content']): string {
  return content.flatMap(block => block.type === 'text' ? [block.text] : []).join('\n\n')
}

/**
 * Tail of the agent's most recent assistant reply, used to resolve short follow-ups such as "go ahead".
 * @param agent - agent whose session log is read.
 * @param maxChars - maximum characters kept from the end of the reply; 0 returns an empty string.
 * @returns the trimmed reply tail, or an empty string when there is no assistant reply.
 */
export function previousReplyText(agent: Agent, maxChars: number): string {
  if (maxChars === 0) return ''
  const reply = agent.session.deriveMessages().findLast(message => message.role === 'assistant')
  return reply === undefined ? '' : textOf(reply.content).trim().slice(-maxChars)
}

/**
 * Direct user text of a step's messages; plugin-injected and harness messages are excluded.
 * @param messages - the step's user-role messages.
 * @param maxChars - maximum characters kept from the start of the joined text.
 * @returns the trimmed request text, or an empty string when the step carries no direct user text.
 */
export function requestText(messages: readonly UserMessage[], maxChars: number): string {
  return messages
    .filter(message => message.source.kind === 'user')
    .map(message => textOf(message.content))
    .filter(text => text.length > 0)
    .join('\n\n')
    .trim()
    .slice(0, maxChars)
}
