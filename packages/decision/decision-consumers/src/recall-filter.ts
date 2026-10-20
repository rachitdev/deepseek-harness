/**
 * Live recall filter: screens plugin-injected memory recall before it enters a model step. Each
 * recalled `<memory>` item is judged for usefulness to the current user request and, optionally,
 * for being a stored prompt-injection attempt; low-relevance items are omitted and flagged items are
 * replaced by a stub that keeps only their uri. Any screening failure leaves the recall unchanged.
 * @module @deepseek-ai/dsh-decision-consumers/recall-filter
 */

import type { Context } from '@deepseek-ai/cordis'
import type { Agent, PreStepDecision } from '@deepseek-ai/dsh-agent'
import { createUserMessage, type Message, type UserMessage } from '@deepseek-ai/dsh-llm'
import z from '@deepseek-ai/schemastery'
import { parseRecallBlock, renderRecallBlock, type ParsedRecallBlock, type RecallItemRendering } from './recall-block.ts'
import { screenRecallItems } from './recall-screen.ts'
import type { RecallItemJudgement } from './types.ts'

/** Live recall-filter policy. */
export interface RecallFilterConfig {
  /** Whether recall messages are screened. */
  readonly enabled?: boolean
  /** `source.plugin` of the recall messages to screen. */
  readonly sourcePlugin?: string
  /** `source.form` of the recall messages to screen. */
  readonly sourceForm?: string
  /** Items scoring below this position on the 0..3 usefulness rubric are omitted. */
  readonly minRelevance?: number
  /** Whether each item is also screened as a possible stored prompt injection. */
  readonly screenInjection?: boolean
  /** Injection probability at or above which an item is withheld. */
  readonly injectionThreshold?: number
  /** Whether an item the decision provider refused to evaluate is withheld rather than kept. */
  readonly withholdUnscreenable?: boolean
  /** Maximum items judged in one decision request. */
  readonly batchSize?: number
  /** Maximum characters of each item body sent to the decision provider. */
  readonly maxItemChars?: number
  /** Maximum user-request characters sent to the decision provider. */
  readonly maxQueryChars?: number
  /** Maximum characters of the previous assistant reply sent as conversation context; 0 omits it. */
  readonly maxContextChars?: number
  /** Deadline for the whole screening pass; on expiry the recall passes unchanged. */
  readonly timeoutMs?: number
}

/** Runtime schema for {@link RecallFilterConfig}. */
export const RecallFilterConfig: z<RecallFilterConfig> = z.object({
  enabled: z.boolean().default(false),
  sourcePlugin: z.string().min(1).default('openviking-memory'),
  sourceForm: z.string().min(1).default('recall'),
  minRelevance: z.number().min(0).max(3).default(0.5),
  screenInjection: z.boolean().default(true),
  injectionThreshold: z.number().min(0).max(1).default(0.7),
  withholdUnscreenable: z.boolean().default(true),
  batchSize: z.number().step(1).min(1).default(12),
  maxItemChars: z.number().step(1).min(1).default(1_500),
  maxQueryChars: z.number().step(1).min(1).default(2_000),
  maxContextChars: z.number().step(1).min(0).default(1_500),
  timeoutMs: z.number().step(1).min(1).default(3_000),
})

type ResolvedRecallFilterConfig = Required<Omit<RecallFilterConfig, 'enabled'>>

/** Marker placed on a withheld item's stub. */
export const WITHHELD_INJECTION_MARKER = 'withheld: flagged as possible prompt injection'
/** Marker placed on an item the decision provider refused to evaluate. */
export const WITHHELD_UNSCREENABLE_MARKER = 'withheld: could not be screened'

/**
 * Model-facing note appended when low-relevance items were omitted.
 * @param count - number of omitted items.
 * @returns the note line.
 */
export function omittedItemsNote(count: number): string {
  return `(${String(count)} lower-relevance recalled item${count === 1 ? '' : 's'} omitted; search memory if more context is needed.)`
}

/** Per-message screening counts, for the log line. */
interface RecallScreenCounts {
  kept: number
  omitted: number
  withheld: number
}

function soleText(message: UserMessage): string | undefined {
  const [block, ...rest] = message.content
  return block?.type === 'text' && rest.length === 0 ? block.text : undefined
}

function textOf(content: Message['content']): string {
  return content.flatMap(block => block.type === 'text' ? [block.text] : []).join('\n\n')
}

function previousReplyText(agent: Agent, maxChars: number): string {
  if (maxChars === 0) return ''
  const reply = agent.session.deriveMessages().findLast(message => message.role === 'assistant')
  return reply === undefined ? '' : textOf(reply.content).trim().slice(-maxChars)
}

function requestText(messages: readonly UserMessage[], maxChars: number): string {
  return messages
    .filter(message => message.source.kind === 'user')
    .map(message => textOf(message.content))
    .filter(text => text.length > 0)
    .join('\n\n')
    .trim()
    .slice(0, maxChars)
}

function renderingFor(
  judgement: RecallItemJudgement,
  config: ResolvedRecallFilterConfig,
  counts: RecallScreenCounts,
): RecallItemRendering {
  if (judgement.kind === 'rejected') {
    if (!config.withholdUnscreenable) {
      counts.kept += 1
      return { kind: 'keep' }
    }
    counts.withheld += 1
    return { kind: 'stub', marker: WITHHELD_UNSCREENABLE_MARKER }
  }
  if (judgement.injectionProbability !== undefined && judgement.injectionProbability >= config.injectionThreshold) {
    counts.withheld += 1
    return { kind: 'stub', marker: WITHHELD_INJECTION_MARKER }
  }
  if (judgement.relevance < config.minRelevance) {
    counts.omitted += 1
    return { kind: 'drop' }
  }
  counts.kept += 1
  return { kind: 'keep' }
}

interface RecallTarget {
  readonly index: number
  readonly message: UserMessage
  readonly block: ParsedRecallBlock
}

/**
 * Install the recall filter on `agent/pre-step`.
 *
 * The filter must observe recall appended by other `prepend` listeners after their own `next()`,
 * so it has to run outermost. Plugins apply concurrently, so load order cannot place it there;
 * instead its listener is (re)registered with `prepend` on every `agent/created`, after every
 * boot-time registration, and again after any hot reload that happened between sessions.
 * @param ctx - plugin context carrying the decision service.
 * @param raw - recall-filter configuration; omitted fields take the schema defaults.
 */
export function installRecallFilter(ctx: Context, raw: RecallFilterConfig): void {
  // The schema supplies every default, so validation is also the explicit resolve step.
  const config = RecallFilterConfig(raw) as ResolvedRecallFilterConfig

  const screen = async (agent: Agent, decision: PreStepDecision, signal: AbortSignal): Promise<PreStepDecision> => {
    if (decision.kind !== 'enter') return decision
    const targets: RecallTarget[] = []
    decision.messages.forEach((message, index) => {
      // The producer is an external package outside MessageSourceMap; its source carries
      // `kind: plugin:<name>`, `plugin: <name>`, and a ContextForm `form`.
      const source = message.source as Readonly<Record<string, unknown>>
      if (source['kind'] !== `plugin:${config.sourcePlugin}` || source['plugin'] !== config.sourcePlugin || source['form'] !== config.sourceForm) return
      const text = soleText(message)
      const block = text === undefined ? undefined : parseRecallBlock(text)
      if (block !== undefined && block.items.length > 0) targets.push({ index, message, block })
    })
    if (targets.length === 0) return decision
    const request = requestText(decision.messages, config.maxQueryChars)
    if (request.length === 0) return decision
    const query = { request, previousReply: previousReplyText(agent, config.maxContextChars) }

    const deadline = AbortSignal.any([signal, AbortSignal.timeout(config.timeoutMs)])
    const started = performance.now()
    let screened: Awaited<ReturnType<typeof screenRecallItems>>
    try {
      screened = await screenRecallItems(
        ctx.decision,
        query,
        targets.flatMap(target => target.block.items),
        config,
        deadline,
      )
    } catch (error: unknown) {
      signal.throwIfAborted()
      ctx.logger.warn('recall filter: screening failed for agent %s; recall passes unscreened: %o', agent.id, error)
      return decision
    }

    const counts: RecallScreenCounts = { kept: 0, omitted: 0, withheld: 0 }
    const replacements = new Map<number, UserMessage>()
    let offset = 0
    for (const target of targets) {
      const judgements = screened.judgements.slice(offset, offset + target.block.items.length)
      offset += target.block.items.length
      const before = counts.omitted
      const renderings = judgements.map(judgement => renderingFor(judgement, config, counts))
      renderings.forEach((rendering, index) => {
        if (rendering.kind !== 'stub') return
        // Withheld memory stays stored; the uri lets an operator inspect or delete it.
        // oxlint-disable-next-line typescript/no-non-null-assertion -- one judgement per rendering
        ctx.logger.warn('recall filter: agent %s withheld %s (%s)', agent.id, judgements[index]!.uri, rendering.marker)
      })
      const omitted = counts.omitted - before
      const text = renderRecallBlock(target.block, renderings, omitted > 0 ? omittedItemsNote(omitted) : undefined)
      replacements.set(target.index, createUserMessage({ content: [{ type: 'text', text }], source: target.message.source }))
    }
    ctx.logger.info(
      'recall filter: agent %s kept %d, omitted %d, withheld %d in %dms via %s',
      agent.id,
      counts.kept,
      counts.omitted,
      counts.withheld,
      Math.round(performance.now() - started),
      screened.providerIds.join(',') || 'none',
    )
    return {
      ...decision,
      messages: decision.messages.map((message, index) => replacements.get(index) ?? message),
    }
  }

  let unregister: (() => boolean) | undefined
  const register = (): void => {
    unregister?.()
    unregister = ctx.on('agent/pre-step', async ({ agent, signal }, next) => {
      return screen(agent, await next(), signal)
    }, { prepend: true })
  }
  register()
  ctx.on('agent/created', () => {
    register()
    return undefined
  })
}
