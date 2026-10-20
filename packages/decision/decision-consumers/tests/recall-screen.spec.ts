import { describe, expect, it } from 'vitest'
import { DecisionError, type DecisionRequest, type DecisionResult } from '@deepseek-ai/dsh-decision'
import { screenRecallItems, type RecallScreenOptions } from '../src/recall-screen.ts'
import { createMockDecisionService } from './helpers.ts'

const Q = { request: 'q', previousReply: '' }
const OPTIONS: RecallScreenOptions = { batchSize: 2, maxItemChars: 10, screenInjection: true }

function items(count: number): Array<{ uri: string; content: string }> {
  return Array.from({ length: count }, (_value, index) => ({ uri: `viking://m${String(index)}`, content: `body ${String(index)} padded beyond the bound` }))
}

function answerAll(request: DecisionRequest, relevance = 2, injection = 0.05): DecisionResult {
  return {
    providerId: 'mock',
    answers: Object.fromEntries(Object.entries(request.questions).map(([key, question]) => [
      key,
      question.type === 'score'
        ? { type: 'score', score: relevance, probabilities: {} }
        : { type: 'boolean', probability: injection },
    ])),
  }
}

describe('screenRecallItems', () => {
  it('batches items, bounds item bodies, and returns judgements in input order', async () => {
    const requests: DecisionRequest[] = []
    const service = createMockDecisionService((request) => {
      requests.push(request)
      return Promise.resolve(answerAll(request))
    })

    const result = await screenRecallItems(service, { request: 'the request', previousReply: '' }, items(5), OPTIONS)

    expect(requests).toHaveLength(3)
    expect(Object.keys(requests[0]?.questions ?? {})).toEqual(['rel_0', 'inj_0', 'rel_1', 'inj_1'])
    expect(requests[0]?.state.startsWith('Current user request:\nthe request')).toBe(true)
    expect(requests[0]?.state).toContain('[m1] uri=viking://m1\nbody 1 pad')
    expect(requests[0]?.state).not.toContain('beyond')
    expect(result.judgements.map(judgement => judgement.uri)).toEqual(items(5).map(item => item.uri))
    expect(result.judgements[0]).toEqual({ uri: 'viking://m0', kind: 'judged', relevance: 2, injectionProbability: 0.05 })
    expect(result.providerIds).toEqual(['mock'])
  })

  it('omits the injection question when injection screening is off', async () => {
    const requests: DecisionRequest[] = []
    const service = createMockDecisionService((request) => {
      requests.push(request)
      return Promise.resolve(answerAll(request))
    })

    const result = await screenRecallItems(service, Q, items(1), { ...OPTIONS, screenInjection: false })

    expect(Object.keys(requests[0]?.questions ?? {})).toEqual(['rel_0'])
    expect(result.judgements[0]).toEqual({ uri: 'viking://m0', kind: 'judged', relevance: 2 })
  })

  it('places the previous reply before the request as conversation context', async () => {
    const requests: DecisionRequest[] = []
    const service = createMockDecisionService((request) => {
      requests.push(request)
      return Promise.resolve(answerAll(request))
    })

    await screenRecallItems(service, { request: 'go ahead', previousReply: 'Shall I sign the binary?' }, items(1), OPTIONS)

    expect(requests[0]?.state.startsWith('Previous assistant reply (conversation context):\nShall I sign the binary?\n\nCurrent user request:\ngo ahead')).toBe(true)
  })

  it('marks an empty body as uri-only', async () => {
    const requests: DecisionRequest[] = []
    const service = createMockDecisionService((request) => {
      requests.push(request)
      return Promise.resolve(answerAll(request))
    })

    await screenRecallItems(service, Q, [{ uri: 'viking://dir', content: '' }], OPTIONS)

    expect(requests[0]?.state).toContain('[m0] uri=viking://dir\n(uri only)')
  })

  it('splits a rejected batch so only the refused item goes unjudged', async () => {
    const service = createMockDecisionService((request) => {
      if (request.state.includes('HOSTILE')) {
        return Promise.reject(new DecisionError('forbidden', 'DECISION_REQUEST_REJECTED'))
      }
      return Promise.resolve(answerAll(request))
    })
    const input = [
      { uri: 'viking://ok', content: 'fine' },
      { uri: 'viking://bad', content: 'HOSTILE' },
    ]

    const result = await screenRecallItems(service, Q, input, OPTIONS)

    expect(result.judgements).toEqual([
      { uri: 'viking://ok', kind: 'judged', relevance: 2, injectionProbability: 0.05 },
      { uri: 'viking://bad', kind: 'rejected' },
    ])
  })

  it('propagates provider failures other than request rejection', async () => {
    const service = createMockDecisionService(() => Promise.reject(new DecisionError('down', 'DECISION_PROVIDER_ERROR')))

    await expect(screenRecallItems(service, Q, items(3), OPTIONS)).rejects.toThrow('down')
  })

  it('rejects a malformed answer', async () => {
    const service = createMockDecisionService(() => Promise.resolve({ providerId: 'mock', answers: {} }))

    await expect(screenRecallItems(service, Q, items(1), OPTIONS)).rejects.toThrow(/malformed answer for recall item viking:\/\/m0/)
  })
})
