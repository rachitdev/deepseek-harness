/** Deterministic substitutes for the turn-triage Loader smoke: an LLM adapter and a decision provider. */

import { Context, Service } from '@deepseek-ai/cordis'
import type {} from '@deepseek-ai/dsh-agent'
import type { DecisionProvider, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { LlmAdapter, type GenerateOptions, type StreamChunk } from '@deepseek-ai/dsh-llm'

class FixtureAdapter extends LlmAdapter {
  readonly models: string[] = []

  async * stream(options: GenerateOptions): AsyncIterable<StreamChunk> {
    this.models.push(options.model)
    yield { type: 'block-start', index: 0, blockType: 'text' }
    yield { type: 'text-delta', index: 0, text: 'done' }
    yield { type: 'block-end', index: 0, block: { type: 'text', text: 'done' } }
    yield { type: 'finish', reason: { kind: 'stop' } }
  }
}

class FixtureDecisionProvider implements DecisionProvider {
  readonly id = 'fixture-jev'
  calls = 0

  available(): boolean {
    return true
  }

  evaluate(request: DecisionRequest): Promise<DecisionResult> {
    this.calls += 1
    const choice = request.state.includes('Refactor') ? 'frontier' : 'fast'
    return Promise.resolve({ providerId: this.id, answers: { tier: { type: 'choice', choice, probabilities: { [choice]: 0.9 } } } })
  }
}

declare module '@deepseek-ai/cordis' {
  interface Context {
    /** Deterministic observations exported to the Loader smoke driver. */
    triageFixture: TriageFixtureRuntime
  }
}

/** Registers the scripted adapter and decision provider. */
export default class TriageFixtureRuntime extends Service {
  static inject = ['llm', 'decision']
  readonly adapter = new FixtureAdapter()
  readonly provider = new FixtureDecisionProvider()

  constructor(ctx: Context) {
    super(ctx, 'triageFixture')
    ctx.effect(() => ctx.llm.registerAdapter(['fixture'], this.adapter))
    ctx.effect(() => ctx.decision.registerProvider(this.provider))
  }
}
