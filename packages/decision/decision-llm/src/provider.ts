/**
 * `LlmDecisionProvider`: a `DecisionProvider` that answers the decision seam by constraining an
 * existing `LlmService` model to a typed JSON reply. It keeps every Consumer working without a
 * decision-model credential, so the seam is never a single-vendor dependency.
 * @module @deepseek-ai/dsh-decision-llm/provider
 */

import type { Context } from '@deepseek-ai/cordis'
import { DecisionError } from '@deepseek-ai/dsh-decision'
import type { DecisionProvider, DecisionRequest, DecisionResult } from '@deepseek-ai/dsh-decision'
import { BlockAssembler } from '@deepseek-ai/dsh-llm'
import { DECISION_SYSTEM_PROMPT, parseDecisionResponse, renderDecisionPrompt } from './prompt.ts'


/** Stable id this provider registers under. */
export const LLM_PROVIDER_ID = 'llm'

/** Resolved provider options (the plugin's `apply` supplies config defaults). */
export interface LlmDecisionProviderOptions {
  /** Registered LLM provider route serving decisions. */
  provider: string
  /** Exact model id on that route. */
  model: string
  /** Upper bound on the JSON reply; a decision answer is small by construction. */
  maxTokens: number
}

/**
 * The LLM-backed decision provider. Answers are parsed and validated against the questions asked,
 * so a model that invents an option or an out-of-range probability fails loudly rather than
 * reaching a consumer's confidence gate as if it were calibrated.
 */
export class LlmDecisionProvider implements DecisionProvider {
  readonly id = LLM_PROVIDER_ID

  constructor(
    private readonly ctx: Context,
    private readonly options: LlmDecisionProviderOptions,
  ) {}

  available(): boolean {
    return this.ctx.get('llm') !== undefined
      && this.options.provider.length > 0
      && this.options.model.length > 0
  }

  async evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult> {
    const llm = this.ctx.get('llm')
    if (llm === undefined) {
      throw new DecisionError('decision-llm requires the llm service', 'DECISION_PROVIDER_ERROR')
    }

    const assembler = new BlockAssembler()
    for await (
      const chunk of llm.stream({
        provider: this.options.provider,
        model: this.options.model,
        system: DECISION_SYSTEM_PROMPT,
        // Request-only input: the decision prompt is built and discarded here, never written to a Session.
        messages: [{ role: 'user', content: [{ type: 'text', text: renderDecisionPrompt(request) }] }],
        maxTokens: this.options.maxTokens,
        ...signal !== undefined ? { signal } : {},
      })
    ) {
      assembler.push(chunk)
    }

    const text = assembler.blocks()
      .filter((block): block is Extract<ReturnType<BlockAssembler['blocks']>[number], { type: 'text' }> =>
        block.type === 'text')
      .map(block => block.text)
      .join('')
    if (text.trim().length === 0) {
      throw new DecisionError('decision model produced no text', 'DECISION_PROVIDER_ERROR')
    }

    let answers
    try {
      answers = parseDecisionResponse(text, request)
    } catch (error: unknown) {
      if (error instanceof DecisionError) throw error
      throw new DecisionError(
        `decision model returned unparseable JSON: ${String(error)}`,
        'DECISION_PROVIDER_ERROR',
        { cause: error },
      )
    }
    return { answers, providerId: this.id }
  }
}
