/**
 * Service Provider plugin registering {@link LlmDecisionProvider} on the decision seam
 * (`ctx.decision`). It needs no decision-model credential, so every Consumer works on a plain
 * harness install and the seam never depends on one vendor.
 * @module @deepseek-ai/dsh-decision-llm
 */

import type { Context } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import { LlmDecisionProvider } from './provider.ts'

export { LLM_PROVIDER_ID, LlmDecisionProvider } from './provider.ts'
export type { LlmDecisionProviderOptions } from './provider.ts'
export {
  DECISION_SYSTEM_PROMPT,
  parseDecisionResponse,
  renderDecisionPrompt,
} from './prompt.ts'

/** Upper bound on the JSON reply; a decision answer is small by construction. */
const DEFAULT_MAX_TOKENS = 1024

/** Cordis plugin name. */
export const name = 'decision-llm'

/** The decision seam must exist before a provider can register on it; `llm` serves the calls. */
export const inject = ['decision', 'llm']

/** Plugin config; the route is deployment-varying, so it is required rather than guessed. */
export interface Config {
  /** Registered LLM provider route serving decisions. */
  provider: string
  /** Exact model id on that route. A cheap, fast model is the point of this provider. */
  model: string
  /** Upper bound on the JSON reply. */
  maxTokens?: number
}

export const Config: z<Config> = z.object({
  // No default: which route and model serve decisions varies per deployment, and a guessed route
  // would fail at call time rather than at load.
  provider: z.string().required(),
  model: z.string().required(),
  maxTokens: z.number().step(1).min(1).default(DEFAULT_MAX_TOKENS),
})

/**
 * Register the LLM-backed decision provider for this context's lifetime.
 * @param ctx - plugin context carrying the decision and llm planes.
 * @param config - the currently authoritative section.
 */
export function apply(ctx: Context, config: Config): void {
  const provider = new LlmDecisionProvider(ctx, {
    provider: config.provider,
    model: config.model,
    maxTokens: config.maxTokens ?? DEFAULT_MAX_TOKENS,
  })
  ctx.effect(() => ctx.decision.registerProvider(provider))
}
