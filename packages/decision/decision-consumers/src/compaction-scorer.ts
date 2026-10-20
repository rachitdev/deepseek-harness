/**
 * `CompactionScorer`: scores conversation and tool transcript items during compaction using
 * `ctx.decision`. Prioritizes durable findings, decisions, and constraints while identifying
 * ephemeral tool outputs to prune or summarize, preserving context budget without loss of continuity.
 * @module @deepseek-ai/dsh-decision-consumers/compaction-scorer
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { CompactionPlan, CompactionTranscriptItem } from './types.ts'

/** Options configuring compaction scoring thresholds and request sizing. */
export interface CompactionScorerOptions {
  /** Score (0..3) above which an item is retained verbatim; defaults to 2.0. */
  readonly keepScoreThreshold?: number
  /** Score (0..3) below which an item is completely dropped; defaults to 0.8. */
  readonly dropScoreThreshold?: number
  /**
   * Estimated token ceiling for the shared state block; defaults to 25000. Item summaries are
   * truncated in stages until the rendered state fits, so a transcript larger than one provider
   * request still produces a plan.
   */
  readonly maxStateTokens?: number
  /**
   * Estimated token ceiling for state plus one batch of questions; defaults to 30000, below the
   * 32768 the System One endpoint currently enforces. Questions are split across concurrent
   * requests to stay under it.
   */
  readonly maxRequestTokens?: number
}

/**
 * Characters per token used to size requests. The decision seam carries no tokenizer, and the
 * ceiling being approached belongs to the provider, so this estimate is deliberately conservative:
 * over-estimating costs one extra request, while under-estimating costs the whole plan.
 */
const CHARS_PER_TOKEN = 4

/** Summary truncation widths applied in order until the rendered state fits `maxStateTokens`. */
const TRUNCATION_STAGES: readonly (number | undefined)[] = [undefined, 1000, 200, 60]

/** Estimate the token cost of a rendered string. */
function estimateTokens(text: string): number {
  return Math.ceil(text.length / CHARS_PER_TOKEN)
}

/** Render one transcript item, optionally truncating its summary to `limit` characters. */
function renderItem(item: CompactionTranscriptItem, limit: number | undefined): string {
  const summary =
    limit !== undefined && item.summary.length > limit ? `${item.summary.slice(0, limit)}…` : item.summary
  return `[ID: ${item.id} | ${item.role} | ${String(item.bytes)}B]: ${summary}`
}

/** Render the shared state block at one truncation stage. */
function renderState(items: readonly CompactionTranscriptItem[], goal: string, limit: number | undefined): string {
  return [`Session Goal: ${goal}`, 'Transcript Items to Evaluate:', ...items.map(i => renderItem(i, limit))].join('\n')
}

/**
 * Render the state at the widest truncation stage that fits the budget. Every item stays present at
 * every stage: an item missing from the state would be judged against a transcript that does not
 * mention it, so detail is reduced rather than items dropped.
 * @param items - transcript items to render.
 * @param goal - the active conversation or task goal.
 * @param maxStateTokens - estimated token ceiling for the rendered state.
 * @returns the rendered state block.
 */
function fitState(items: readonly CompactionTranscriptItem[], goal: string, maxStateTokens: number): string {
  let rendered = renderState(items, goal, TRUNCATION_STAGES[0])
  for (const limit of TRUNCATION_STAGES) {
    rendered = renderState(items, goal, limit)
    if (estimateTokens(rendered) <= maxStateTokens) {
      return rendered
    }
  }
  return rendered
}

/**
 * Score transcript items and generate a compaction retention plan.
 * @param decisionService - the active decision seam service.
 * @param items - transcript items considered for compaction.
 * @param goal - the active conversation or task goal.
 * @param options - threshold options.
 * @param signal - optional cancellation signal.
 * @returns partitioned keep, drop, and summarize IDs with byte metrics.
 */
export async function scoreCompactionItems(
  decisionService: DecisionService,
  items: readonly CompactionTranscriptItem[],
  goal: string,
  options: CompactionScorerOptions = {},
  signal?: AbortSignal,
): Promise<CompactionPlan> {
  const keepThreshold = options.keepScoreThreshold ?? 2.0
  const dropThreshold = options.dropScoreThreshold ?? 0.8

  if (items.length === 0) {
    return {
      keepIds: [],
      dropIds: [],
      summarizeIds: [],
      retainedBytes: 0,
      prunedBytes: 0,
      latencyMs: 0,
      providerId: 'none',
    }
  }

  const state = fitState(items, goal, options.maxStateTokens ?? 25_000)
  const maxRequestTokens = options.maxRequestTokens ?? 30_000

  const questionsFor = (
    item: CompactionTranscriptItem,
  ): Record<string, { type: 'score' | 'boolean'; instructions: string; criteria?: readonly string[] }> => ({
    [`val_${item.id}`]: {
      type: 'score',
      instructions: `What is the retention value of item "${item.id}" (${item.role}) for ongoing work?`,
      criteria: [
        'ephemeral-intermediate-noise',
        'helpful-context-can-summarize',
        'key-finding-or-artifact',
        'user-constraint-or-unalterable-decision',
      ],
    },
    [`must_keep_${item.id}`]: {
      type: 'boolean',
      instructions: `Must item "${item.id}" be kept verbatim to prevent broken state or lost user direction?`,
    },
  })

  // The state is resent with every batch, so only the question budget is divisible. A provider
  // answers the questions in one request concurrently, making batch count — not question count —
  // the thing worth minimizing.
  const stateTokens = estimateTokens(state)
  const perItemTokens = Math.max(
    1,
    ...items.map(item => estimateTokens(JSON.stringify(questionsFor(item)))),
  )
  const questionBudget = maxRequestTokens - stateTokens
  if (questionBudget < perItemTokens) {
    throw new Error(
      `Compaction state alone consumes ~${String(stateTokens)} tokens of the ~${String(maxRequestTokens)}-token request budget, leaving no room for questions. Reduce the transcript or raise maxRequestTokens.`,
    )
  }
  const batchSize = Math.max(1, Math.min(items.length, Math.floor(questionBudget / perItemTokens)))
  const batches: CompactionTranscriptItem[][] = []
  for (let i = 0; i < items.length; i += batchSize) {
    batches.push(items.slice(i, i + batchSize))
  }

  const t0 = performance.now()
  const settled = await Promise.all(
    batches.map(async (batch) => {
      const questions: Record<string, { type: 'score' | 'boolean'; instructions: string; criteria?: readonly string[] }> =
        {}
      for (const item of batch) {
        Object.assign(questions, questionsFor(item))
      }
      return await decisionService.evaluate(
        {
          state,
          questions: questions as Parameters<typeof decisionService.evaluate>[0]['questions'],
        },
        signal,
      )
    }),
  )
  const latencyMs = Math.round(performance.now() - t0)

  const first = settled[0]
  if (first === undefined) {
    throw new Error('Compaction scoring produced no decision requests for a non-empty transcript')
  }
  const result = {
    answers: Object.assign({}, ...settled.map(r => r.answers)) as typeof first.answers,
    providerId: first.providerId,
  }

  const keepIds: string[] = []
  const dropIds: string[] = []
  const summarizeIds: string[] = []

  let retainedBytes = 0
  let prunedBytes = 0

  for (const item of items) {
    const valAns = result.answers[`val_${item.id}`]
    const keepAns = result.answers[`must_keep_${item.id}`]

    if (valAns?.type !== 'score' || keepAns?.type !== 'boolean') {
      throw new Error(`Decision provider returned malformed answer for compaction item ${item.id}`)
    }

    if (valAns.score >= keepThreshold || keepAns.probability >= 0.7) {
      keepIds.push(item.id)
      retainedBytes += item.bytes
    } else if (valAns.score <= dropThreshold && keepAns.probability < 0.2) {
      dropIds.push(item.id)
      prunedBytes += item.bytes
    } else {
      summarizeIds.push(item.id)
      // Summarization retains ~20% of original bytes
      retainedBytes += Math.round(item.bytes * 0.2)
      prunedBytes += Math.round(item.bytes * 0.8)
    }
  }

  return {
    keepIds,
    dropIds,
    summarizeIds,
    retainedBytes,
    prunedBytes,
    latencyMs,
    providerId: result.providerId,
  }
}
