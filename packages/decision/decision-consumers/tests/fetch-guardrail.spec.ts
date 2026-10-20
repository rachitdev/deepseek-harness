import { describe, expect, it, vi } from 'vitest'
import { guardFetchedContent } from '../src/fetch-guardrail.ts'
import type { DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { createMockDecisionService } from './helpers.ts'

const CLEAN = {
  url: 'https://example.com/guide',
  title: 'Setup guide',
  content: 'Install the package with pnpm install and run the tests.',
}

describe('guardFetchedContent', () => {
  it('passes benign content', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          prompt_injection: { type: 'boolean', probability: 0.03 },
          credential_harvest: { type: 'boolean', probability: 0.05 },
        },
      }),
    )

    const verdict = await guardFetchedContent(mock, CLEAN)

    expect(verdict.verdict).toBe('pass')
    expect(verdict.injectionProbability).toBe(0.03)
    expect(verdict.thresholdMargin).toBeCloseTo(0.25, 5)
  })

  it('flags mid-band hazards for review', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          prompt_injection: { type: 'boolean', probability: 0.55 },
          credential_harvest: { type: 'boolean', probability: 0.2 },
        },
      }),
    )

    const verdict = await guardFetchedContent(mock, CLEAN)

    expect(verdict.verdict).toBe('review')
    expect(verdict.thresholdMargin).toBeCloseTo(0.25, 5)
  })

  it('blocks content above the hazard threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          prompt_injection: { type: 'boolean', probability: 0.92 },
          credential_harvest: { type: 'boolean', probability: 0.1 },
        },
      }),
    )

    const verdict = await guardFetchedContent(mock, CLEAN)

    expect(verdict.verdict).toBe('block')
    expect(verdict.thresholdMargin).toBeCloseTo(0.22, 5)
  })

  it('blocks on either hazard reaching the threshold', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          prompt_injection: { type: 'boolean', probability: 0.05 },
          credential_harvest: { type: 'boolean', probability: 0.85 },
        },
      }),
    )

    const verdict = await guardFetchedContent(mock, CLEAN)

    expect(verdict.verdict).toBe('block')
    expect(verdict.credentialHarvestProbability).toBe(0.85)
  })

  it('honors custom thresholds and clamps the margin to [0, 1]', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'jev',
        answers: {
          prompt_injection: { type: 'boolean', probability: 0.9 },
          credential_harvest: { type: 'boolean', probability: 0 },
        },
      }),
    )

    const verdict = await guardFetchedContent(
      mock,
      CLEAN,
      { blockThreshold: 0.6, reviewThreshold: 0.1 },
    )

    expect(verdict.verdict).toBe('block')
    expect(verdict.thresholdMargin).toBeCloseTo(0.3, 5)
  })

  it('bounds the content included in the provider state', async () => {
    const spy = vi.fn((_request: import('@deepseek-ai/dsh-decision').DecisionRequest, _signal?: AbortSignal) =>
      Promise.resolve<DecisionResult>({
        providerId: 'jev',
        answers: {
          prompt_injection: { type: 'boolean', probability: 0.02 },
          credential_harvest: { type: 'boolean', probability: 0.02 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const long = 'a'.repeat(20000)

    await guardFetchedContent(mock, { ...CLEAN, content: long }, { maxStateCharacters: 100 })

    const request = spy.mock.calls[0]![0]
    const state = request.state
    expect(state).toContain('URL: https://example.com/guide')
    expect(state).toContain('Title: Setup guide')
    expect(state.length).toBeLessThan(200)
  })

  it('fails on unexpected provider answer types', async () => {
    const mock = createMockDecisionService(() =>
      Promise.resolve({
        providerId: 'bad',
        answers: {
          prompt_injection: { type: 'score', score: 0.5, probabilities: {} },
          credential_harvest: { type: 'boolean', probability: 0.5 },
        },
      }),
    )

    await expect(guardFetchedContent(mock, CLEAN)).rejects.toThrow(/unexpected answer types/)
  })

  it('forwards the cancellation signal', async () => {
    const spy = vi.fn((_request: DecisionRequest, _signal?: AbortSignal) =>
      Promise.resolve<DecisionResult>({
        providerId: 'test',
        answers: {
          prompt_injection: { type: 'boolean' as const, probability: 0.02 },
          credential_harvest: { type: 'boolean' as const, probability: 0.02 },
        },
      }),
    )
    const mock = createMockDecisionService(spy)
    const controller = new AbortController()

    await guardFetchedContent(mock, CLEAN, {}, controller.signal)

    expect(typeof spy.mock.calls[0]?.[0].state).toBe('string')
    expect(spy.mock.calls[0]?.[1]).toBe(controller.signal)
  })
})
