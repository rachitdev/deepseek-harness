/**
 * Service Provider plugin registering {@link JevDecisionProvider} on the decision seam
 * (`ctx.decision`). The TypeSafe key is a credential *reference* resolved per call, so a rotated key
 * reaches the next decision without a plugin restart.
 * @module @deepseek-ai/dsh-decision-jev
 */

import type { Context } from '@deepseek-ai/cordis'
import { credentialRef } from '@deepseek-ai/dsh-credentials'
import { launchEnvironmentOf } from '@deepseek-ai/dsh-launch-environment'
import z from '@deepseek-ai/schemastery'
import {
  JEV_DEFAULT_BASE_URL,
  JEV_DEFAULT_MODEL,
  JevDecisionProvider,
} from './provider.ts'

export {
  JEV_DEFAULT_BASE_URL,
  JEV_DEFAULT_MODEL,
  JEV_EVALUATION_PATH,
  JEV_PROVIDER_ID,
  JevDecisionProvider,
  fromJevAnswer,
  toJevQuestion,
} from './provider.ts'
export type { JevDecisionProviderOptions } from './provider.ts'

/** Environment variable naming the direct TypeSafe credential by default. */
const DEFAULT_API_KEY_ENV = 'TYPESAFE_API_KEY'

/** Environment variable naming the direct TypeSafe endpoint. */
const BASE_URL_ENV = 'TYPESAFE_BASE_URL'

/** Environment variable naming the direct TypeSafe model id override (defaults to JEV_DEFAULT_MODEL 'jev-latest'). */
const MODEL_ENV = 'TYPESAFE_MODEL'

/** Cordis plugin name. */
export const name = 'decision-jev'

/** The decision seam must exist before a provider can register on it. */
export const inject = ['decision']

/** Plugin config; every deployment-varying value is a validated field, not a constant. */
export interface Config {
  /** Literal TypeSafe key. Prefer `apiKeyEnv`; a literal is for throwaway local runs. */
  apiKey?: string
  /** Environment variable naming the TypeSafe key. */
  apiKeyEnv?: string
  /** TypeSafe API endpoint base. */
  baseURL?: string
  /** Direct TypeSafe model id. */
  model?: string
}

export const Config: z<Config> = z.object({
  apiKey: z.string().role('secret'),
  apiKeyEnv: z.string().role('credential-ref').default(DEFAULT_API_KEY_ENV),
  // No schema default: `$TYPESAFE_BASE_URL` must be able to win when no section pins the endpoint,
  // and a schema default would fill the field first and make that fallback unreachable.
  baseURL: z.string(),
  // Declared here rather than only at the use site: a configuration surface renders the resolved
  // section, so a default the schema does not carry reads there as no value at all.
  model: z.string().default(JEV_DEFAULT_MODEL),
})

/**
 * Register the Jev decision provider for this context's lifetime.
 * @param ctx - plugin context carrying the decision, credential, and environment planes.
 * @param config - the currently authoritative section.
 */
export function apply(ctx: Context, config: Config = {}): void {
  const apiKeyEnv = credentialRef(config.apiKeyEnv ?? DEFAULT_API_KEY_ENV)
  const literalApiKey = config.apiKey !== undefined && config.apiKey.length > 0
    ? config.apiKey
    : undefined

  const resolveApiKey = async (): Promise<string | undefined> => {
    if (literalApiKey !== undefined) return literalApiKey
    const credentials = ctx.get('credentials')
    if (credentials !== undefined) return (await credentials.resolve(apiKeyEnv))?.value
    // Without the seam the environment is the whole credential plane.
    const ambient = launchEnvironmentOf(ctx).get(apiKeyEnv)
    return ambient !== undefined && ambient.value.length > 0 ? ambient.value : undefined
  }

  const provider = new JevDecisionProvider({
    resolveApiKey,
    // `available()` must not perform async work, so key presence is sampled synchronously. A
    // mounted credentials seam counts as a possible key: it resolves asynchronously, and treating
    // its absence-from-the-environment as "no key" would hide a provider whose key is sealed.
    // The authoritative read still happens per call in `resolveApiKey`, which fails loudly when
    // nothing resolves.
    hasApiKey: literalApiKey !== undefined
      || ctx.get('credentials') !== undefined
      || (launchEnvironmentOf(ctx).get(apiKeyEnv)?.value.length ?? 0) > 0,
    baseURL: config.baseURL
      ?? launchEnvironmentOf(ctx).get(BASE_URL_ENV)?.value
      ?? JEV_DEFAULT_BASE_URL,
    model: config.model
      ?? launchEnvironmentOf(ctx).get(MODEL_ENV)?.value
      ?? JEV_DEFAULT_MODEL,
  })

  ctx.effect(() => ctx.decision.registerProvider(provider))
}
