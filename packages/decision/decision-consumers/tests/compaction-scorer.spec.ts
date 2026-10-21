import type { DecisionAnswer, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { describe, expect, it, vi } from 'vitest'
import { scoreCompactionItems } from '../src/compaction-scorer.ts'
import { createMockDecisionService } from './helpers.ts'

/** Answer every question in a batch uniformly, so a test asserts batching rather than policy. */
function answerAll(request: DecisionRequest, verdict: 'keep' | 'drop'): DecisionResult {
  const answers: Record<string, DecisionAnswer> = {}
  for (const key of Object.keys(request.questions)) {
    answers[key] = key.startsWith('val_')
      ? { type: 'score', score: verdict === 'keep' ? 3 : 0.1, probabilities: { '0': 1 } }
      : { type: 'boolean', probability: verdict === 'keep' ? 0.99 : 0.01 }
  }
  return { providerId: 'jev', answers }
}

describe('scoreCompactionItems', () => {
  it('returns empty plan immediately for empty items array', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not be called')))
    const plan = await scoreCompactionItems(mock, [], 'goal')
    expect(plan.keepIds).toHaveLength(0)
    expect(plan.dropIds).toHaveLength(0)
    expect(plan.summarizeIds).toHaveLength(0)
    expect(plan.retainedBytes).toBe(0)
    expect(plan.prunedBytes).toBe(0)
    expect(plan.providerId).toBe('none')
  })

  it('partitions items into keep, summarize, and drop sets with byte accounting', async () => {
    const items = [
      { id: 'i1', role: 'user', summary: 'Do not touch production DB', bytes: 100 },
      { id: 'i2', role: 'tool', summary: 'Ran ls -la on tmp directory', bytes: 500 },
      { id: 'i3', role: 'assistant', summary: 'Architectural analysis and file plan', bytes: 1000 },
    ]

    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          val_i1: { type: 'score', score: 3.0, probabilities: { '0': 0, '1': 0, '2': 0, '3': 1 } },
          must_keep_i1: { type: 'boolean', probability: 0.99 },
          val_i2: { type: 'score', score: 0.2, probabilities: { '0': 0.9, '1': 0.1, '2': 0, '3': 0 } },
          must_keep_i2: { type: 'boolean', probability: 0.01 },
          val_i3: { type: 'score', score: 1.5, probabilities: { '0': 0, '1': 0.6, '2': 0.4, '3': 0 } },
          must_keep_i3: { type: 'boolean', probability: 0.4 },
        },
      }),
    )

    const plan = await scoreCompactionItems(mock, items, 'Implement feature X')

    expect(plan.keepIds).toEqual(['i1'])
    expect(plan.dropIds).toEqual(['i2'])
    expect(plan.summarizeIds).toEqual(['i3'])
    expect(plan.retainedBytes).toBe(100 + 200) // i1 (100) + i3 summarized (20% of 1000 = 200)
    expect(plan.prunedBytes).toBe(500 + 800)   // i2 (500) + i3 pruned (80% of 1000 = 800)
  })

  it('fails on unexpected provider answer types', async () => {
    const items = [{ id: 'i1', role: 'tool', summary: 'test', bytes: 10 }]
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          val_i1: { type: 'choice', choice: 'a', probabilities: {} },
          must_keep_i1: { type: 'boolean', probability: 0.5 },
        },
      }),
    )

    await expect(scoreCompactionItems(mock, items, 'goal')).rejects.toThrow(/malformed answer/)
  })

  it('splits a transcript too large for one request into batches covering every item once', async () => {
    const items = Array.from({ length: 400 }, (_, i) => ({
      id: `t${String(i)}`,
      role: 'tool',
      summary: `Read src/module_${String(i)}.ts producing ${'x'.repeat(400)}`,
      bytes: 4200,
    }))

    const seenQuestionCounts: number[] = []
    const mock = createMockDecisionService((request: DecisionRequest) => {
      seenQuestionCounts.push(Object.keys(request.questions).length)
      return Promise.resolve(answerAll(request, 'drop'))
    })

    const plan = await scoreCompactionItems(mock, items, 'refactor the dispatch layer')

    expect(seenQuestionCounts.length).toBeGreaterThan(1)
    expect(seenQuestionCounts.reduce((a, b) => a + b, 0)).toBe(items.length * 2)
    expect(plan.dropIds).toHaveLength(items.length)
  })

  it('resends identical state to every batch so each item is judged against the whole transcript', async () => {
    const items = Array.from({ length: 300 }, (_, i) => ({
      id: `t${String(i)}`,
      role: 'tool',
      summary: `Item ${String(i)} ${'y'.repeat(300)}`,
      bytes: 1000,
    }))

    const states: string[] = []
    const mock = createMockDecisionService((request: DecisionRequest) => {
      states.push(request.state)
      return Promise.resolve(answerAll(request, 'keep'))
    })

    await scoreCompactionItems(mock, items, 'goal')

    expect(states.length).toBeGreaterThan(1)
    expect(new Set(states).size).toBe(1)
    for (const item of items) {
      expect(states[0]).toContain(`[ID: ${item.id} |`)
    }
  })

  it('truncates summaries in stages to fit the state budget while keeping every item present', async () => {
    const items = Array.from({ length: 50 }, (_, i) => ({
      id: `t${String(i)}`,
      role: 'tool',
      summary: 'z'.repeat(5000),
      bytes: 5000,
    }))

    let observedState = ''
    const mock = createMockDecisionService((request: DecisionRequest) => {
      observedState = request.state
      return Promise.resolve(answerAll(request, 'drop'))
    })

    await scoreCompactionItems(mock, items, 'goal', { maxStateTokens: 4000 })

    expect(observedState).toContain('…')
    expect(Math.ceil(observedState.length / 4)).toBeLessThanOrEqual(4000)
    expect(observedState).toContain('[ID: t49 |')
  })

  it('reports a state leaving no question budget instead of failing at the provider', async () => {
    const items = Array.from({ length: 20 }, (_, i) => ({
      id: `t${String(i)}`,
      role: 'tool',
      summary: 'q'.repeat(2000),
      bytes: 2000,
    }))
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not be called')))

    await expect(
      scoreCompactionItems(mock, items, 'goal', { maxStateTokens: 100_000, maxRequestTokens: 60 }),
    ).rejects.toThrow(/no room for questions/)
  })

  it('forwards cancellation signal to every batch', async () => {
    const items = [{ id: 'i1', role: 'tool', summary: 'test', bytes: 10 }]
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          val_i1: { type: 'score' as const, score: 0, probabilities: { '0': 1 } },
          must_keep_i1: { type: 'boolean' as const, probability: 0 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await scoreCompactionItems(mock, items, 'goal', {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
