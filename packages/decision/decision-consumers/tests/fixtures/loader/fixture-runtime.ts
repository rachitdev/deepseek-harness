/** Deterministic external-service substitutes for the real Loader failover smoke. */

import { Context, Service } from '@deepseek-ai/cordis'
import type { DecisionAnswer, DecisionProvider, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import {
  isAgentLoopRequest,
  LlmAdapter,
  resolveRetryPolicy,
  type GenerateOptions,
  type ResolvedRetryPolicy,
  type StreamChunk,
} from '@deepseek-ai/dsh-llm'

/** One observed fixture LLM call. */
export interface FixtureRequest {
  readonly model: string
  readonly agentLoop: boolean
}

class FixtureAdapter extends LlmAdapter {
  readonly requests: FixtureRequest[] = []
  private activeFailed = false
  private readonly policy = resolveRetryPolicy({
    mode: 'normal',
    maxRetries: 1,
    retryableCodes: ['RATE_LIMIT'],
    backoff: { initialDelayMs: 5_000, maxDelayMs: 5_000, jitterRatio: 0 },
  }, 'loader failover fixture')

  override providerRetryPolicy(_provider: string): ResolvedRetryPolicy {
    return this.policy
  }

  async * stream(options: GenerateOptions): AsyncIterable<StreamChunk> {
    const agentLoop = isAgentLoopRequest(options)
    this.requests.push({ model: options.model, agentLoop })
    if (agentLoop && options.model === 'active' && !this.activeFailed) {
      this.activeFailed = true
      yield {
        type: 'finish',
        reason: {
          kind: 'error',
          failure: { code: 'RATE_LIMIT', message: 'fixture active route exhausted', status: 429 },
        },
      }
      return
    }
    const text = options.model === 'candidate-b'
      ? 'export const loaderFailover = true'
      : 'const loaderFailover = false'
    yield { type: 'block-start', index: 0, blockType: 'text' }
    yield { type: 'text-delta', index: 0, text }
    yield { type: 'block-end', index: 0, block: { type: 'text', text } }
    yield { type: 'usage', usage: { inputTokens: 12, outputTokens: 6, totalTokens: 18 } }
    yield { type: 'finish', reason: { kind: 'stop' } }
  }
}

class FixtureDecisionProvider implements DecisionProvider {
  readonly id = 'fixture-jev'
  calls = 0

  available(): boolean {
    return true
  }

  evaluate(request: DecisionRequest, _signal?: AbortSignal): Promise<DecisionResult> {
    this.calls += 1
    const question = request.questions.best_provider
    if (question?.type !== 'choice') {
      return Promise.reject(new Error('fixture expected best_provider choice'))
    }
    const selected = Object.entries(question.criteria)
      .find(([, description]) => description.includes('/candidate-b: latency='))?.[0]
    if (selected === undefined) return Promise.reject(new Error('candidate-b was not offered'))
    return Promise.resolve({
      providerId: this.id,
      answers: Object.fromEntries(Object.entries(request.questions).map(([key, value]): [string, DecisionAnswer] => {
        switch (value.type) {
          case 'choice':
            return [key, {
              type: 'choice',
              choice: selected,
              probabilities: Object.fromEntries(Object.keys(value.criteria).map(worker => [
                worker,
                worker === selected ? 0.98 : 0.02,
              ])),
            }]
          case 'score':
            return [key, { type: 'score', score: 3, probabilities: { '3': 1 } }]
          case 'boolean':
            return [key, { type: 'boolean', probability: 0.99 }]
        }
      })),
    })
  }
}

declare module '@deepseek-ai/cordis' {
  interface Context {
    /** Deterministic observations exported to the Loader smoke driver. */
    failoverFixture: FailoverFixtureRuntime
  }
}

/** Registers the scripted adapter and decision provider used by the smoke. */
export default class FailoverFixtureRuntime extends Service {
  static inject = ['llm', 'decision']
  readonly adapter = new FixtureAdapter()
  readonly provider = new FixtureDecisionProvider()

  constructor(ctx: Context) {
    super(ctx, 'failoverFixture')
    ctx.effect(() => ctx.llm.registerAdapter(['fixture'], this.adapter))
    ctx.effect(() => ctx.decision.registerProvider(this.provider))
  }
}
