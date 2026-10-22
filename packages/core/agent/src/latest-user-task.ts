/**
 * Latest user-authored task text, shared by the consumers that need a prompt-sized
 * digest of what the user last asked for.
 *
 * Both the model-failover probe and the skill adjudicator need the same thing: the
 * newest message the human actually typed, truncated to a caller-owned budget, with
 * a caller-owned fallback for a session that has no user text yet. The two callers
 * previously carried identical copies that differed only in that fallback string.
 *
 * @module @deepseek-ai/dsh-agent/latest-user-task
 */

import type { Agent } from './types.ts'

/**
 * Read the newest user-authored message text from a Session.
 *
 * @param agent - the Agent whose Session is scanned newest-first.
 * @param maxChars - maximum characters returned; the text is sliced, never reflowed.
 * @param fallback - returned when the Session holds no non-empty user text.
 * @returns the truncated newest user text, or `fallback`.
 */
export function latestUserTaskText(agent: Agent, maxChars: number, fallback: string): string {
  const messages = agent.session.deriveMessages()
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index]
    if (message?.role !== 'user' || message.source.kind !== 'user') continue
    const text = message.content
      .filter(block => block.type === 'text')
      .map(block => block.text)
      .join('\n')
      .trim()
    if (text.length > 0) return text.slice(0, maxChars)
  }
  return fallback
}
