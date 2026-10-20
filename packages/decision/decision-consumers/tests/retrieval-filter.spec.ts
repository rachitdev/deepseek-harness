import { describe, expect, it, vi } from 'vitest'
import { filterRetrievalCandidates } from '../src/retrieval-filter.ts'
import { createMockDecisionService } from './helpers.ts'

describe('filterRetrievalCandidates', () => {
  it('returns empty result immediately for empty candidates array', async () => {
    const mock = createMockDecisionService(() => Promise.reject(new Error('should not be called')))
    const result = await filterRetrievalCandidates(mock, 'query', [])
    expect(result.retained).toHaveLength(0)
    expect(result.dropped).toHaveLength(0)
    expect(result.providerId).toBe('none')
  })

  it('filters noise and retains relevant chunks in parallel', async () => {
    const candidates = [
      { id: 'c1', content: 'Exact answer to the query with code sample', title: 'Doc 1' },
      { id: 'c2', content: 'Unrelated changelog notes from last year', title: 'Doc 2' },
    ]

    const mock = createMockDecisionService((request) => {
      expect(request.questions['rel_c1']).toBeDefined()
      expect(request.questions['keep_c1']).toBeDefined()
      expect(request.questions['rel_c2']).toBeDefined()
      expect(request.questions['keep_c2']).toBeDefined()
      return Promise.resolve({
        providerId: 'jev',
        answers: {
          rel_c1: { type: 'score', score: 2.8, probabilities: { '0': 0, '1': 0, '2': 0.2, '3': 0.8 } },
          keep_c1: { type: 'boolean', probability: 0.95 },
          rel_c2: { type: 'score', score: 0.2, probabilities: { '0': 0.8, '1': 0.2, '2': 0, '3': 0 } },
          keep_c2: { type: 'boolean', probability: 0.05 },
        },
      })
    })

    const result = await filterRetrievalCandidates(mock, 'How to install', candidates)

    expect(result.retained).toHaveLength(1)
    expect(result.retained[0]?.id).toBe('c1')
    expect(result.retained[0]?.relevanceScore).toBe(2.8)
    expect(result.dropped).toHaveLength(1)
    expect(result.dropped[0]?.id).toBe('c2')
  })

  it('fails on unexpected provider answer types', async () => {
    const candidates = [{ id: 'c1', content: 'text' }]
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          rel_c1: { type: 'boolean', probability: 0.5 },
          keep_c1: { type: 'boolean', probability: 0.5 },
        },
      }),
    )

    await expect(filterRetrievalCandidates(mock, 'q', candidates)).rejects.toThrow(/malformed answer/)
  })

  it('forwards cancellation signal', async () => {
    const candidates = [{ id: 'c1', content: 'text' }]
    const spy = vi.fn(() =>
      Promise.resolve({
        providerId: 'test',
        answers: {
          rel_c1: { type: 'score' as const, score: 0, probabilities: { '0': 1 } },
          keep_c1: { type: 'boolean' as const, probability: 0 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await filterRetrievalCandidates(mock, 'q', candidates, {}, controller.signal)
    expect(spy).toHaveBeenCalledWith(expect.anything(), controller.signal)
  })
})
