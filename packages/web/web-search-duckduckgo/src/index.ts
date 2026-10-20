/**
 * `@deepseek-ai/dsh-web-search-duckduckgo`: registers a DuckDuckGo-backed
 * `WebSearchProvider` with `ctx.web`. A function/namespace plugin (NOT a
 * default-export service): a search provider does not own the `ctx.web` key —
 * it registers INTO the seam's provider registry, exactly as
 * `@deepseek-ai/dsh-web-search-exa` registers an adapter into the same seam.
 * The key is owned by `@deepseek-ai/dsh-web`.
 *
 * DuckDuckGo's Lite endpoint needs no API key, so this provider has no
 * credential configuration at all.
 *
 * @module @deepseek-ai/dsh-web-search-duckduckgo
 */

import type { Context } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import type {} from '@deepseek-ai/dsh-web'
import {
  DuckDuckGoSearchProvider,
  DUCKDUCKGO_DEFAULT_BASE_URL,
} from './provider.ts'

export {
  DUCKDUCKGO_DEFAULT_BASE_URL,
  DUCKDUCKGO_PROVIDER_ID,
  DuckDuckGoSearchProvider,
} from './provider.ts'
export type { DuckDuckGoSearchProviderOptions } from './provider.ts'

/** Cordis plugin name used by loader diagnostics. */
export const name = 'web-search-duckduckgo'

/** The web seam this provider registers into. */
export const inject = ['web']

/** Plugin config (all optional — `apply` fills constant defaults). */
export interface Config {
  /** Endpoint base serving the Lite HTML page. Defaults to the public endpoint. */
  baseURL?: string
  /** Region filter sent as the `kl` parameter (e.g. `us-en`). Omitted sends none. */
  region?: string
}

export const Config: z<Config> = z.object({
  baseURL: z.string(),
  region: z.string(),
})

/**
 * Register the DuckDuckGo search provider with `ctx.web`.
 * @param ctx - plugin context; the registration disposes with this fiber.
 * @param config - optional base URL and region.
 */
export function apply(ctx: Context, config: Config): void {
  ctx.effect(() => ctx.web.registerSearchProvider(new DuckDuckGoSearchProvider({
    baseURL: config.baseURL ?? DUCKDUCKGO_DEFAULT_BASE_URL,
    ...config.region !== undefined ? { region: config.region } : {},
  })))
}
