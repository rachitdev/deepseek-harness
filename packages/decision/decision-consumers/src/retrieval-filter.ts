/**
 * `RetrievalFilter`: filters retrieved knowledge items, memories, or search results before injecting
 * them into model prompt context. Uses `ctx.decision` to score relevance and evaluate keep
 * probability per chunk in parallel, dropping irrelevant fragments and saving context budget.
 * @module @deepseek-ai/dsh-decision-consumers/retrieval-filter
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { FilteredRetrievalResult, RetrievalCandidate, ScoredRetrievalItem } from './types.ts'

/** Options configuring retrieval filtering thresholds. */
export interface RetrievalFilterOptions {
  /** Minimum relevance score (0..3) to retain a chunk; defaults to 1.5. */
  readonly minRelevanceScore?: number
  /** Minimum keep probability to retain a chunk; defaults to 0.50. */
  readonly minKeepProbability?: number
}

/**
 * Filter an array of candidate retrieval chunks against a query.
 * @param decisionService - the active decision seam service.
 * @param query - the active user query or task intent.
 * @param candidates - retrieved document/memory chunks.
 * @param options - filtering thresholds.
 * @param signal - optional cancellation signal.
 * @returns partitioned retained and dropped chunks with relevance scores.
 */
export async function filterRetrievalCandidates(
  decisionService: DecisionService,
  query: string,
  candidates: readonly RetrievalCandidate[],
  options: RetrievalFilterOptions = {},
  signal?: AbortSignal,
): Promise<FilteredRetrievalResult> {
  const minScore = options.minRelevanceScore ?? 1.5
  const minKeepProb = options.minKeepProbability ?? 0.5

  if (candidates.length === 0) {
    return { retained: [], dropped: [], latencyMs: 0, providerId: 'none' }
  }

  // Construct questions for each candidate chunk.
  // One decision request answers all chunks in parallel!
  const questions: Record<string, { type: 'score' | 'boolean'; instructions: string; criteria?: readonly string[] }> = {}
  for (const candidate of candidates) {
    questions[`rel_${candidate.id}`] = {
      type: 'score',
      instructions: `How relevant is chunk "${candidate.id}" to the target query?`,
      criteria: ['irrelevant-noise', 'tangentially-related', 'moderately-relevant', 'directly-answers-query'],
    }
    questions[`keep_${candidate.id}`] = {
      type: 'boolean',
      instructions: `Should chunk "${candidate.id}" be included in the context window for this query?`,
    }
  }

  const state = [
    `Target Query: ${query}`,
    'Candidate Chunks:',
    ...candidates.map(c => `[ID: ${c.id}] ${c.title !== undefined ? `(${c.title}) ` : ''}${c.content.slice(0, 400)}`),
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: questions as Parameters<typeof decisionService.evaluate>[0]['questions'],
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const scored: ScoredRetrievalItem[] = []
  for (const candidate of candidates) {
    const relAns = result.answers[`rel_${candidate.id}`]
    const keepAns = result.answers[`keep_${candidate.id}`]

    if (relAns?.type !== 'score' || keepAns?.type !== 'boolean') {
      throw new Error(`Decision provider returned malformed answer for retrieval candidate ${candidate.id}`)
    }

    const retained = relAns.score >= minScore || keepAns.probability >= minKeepProb
    scored.push({
      id: candidate.id,
      relevanceScore: relAns.score,
      keepProbability: keepAns.probability,
      retained,
    })
  }

  const retained = scored.filter(s => s.retained)
  const dropped = scored.filter(s => !s.retained)

  return {
    retained,
    dropped,
    latencyMs,
    providerId: result.providerId,
  }
}
