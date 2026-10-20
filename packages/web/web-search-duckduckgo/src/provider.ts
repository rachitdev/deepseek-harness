/**
 * `DuckDuckGoSearchProvider`: a `WebSearchProvider` backed by DuckDuckGo's
 * free Lite HTML endpoint (`GET https://lite.duckduckgo.com/lite/`). It needs
 * no API key, so the provider is usable out of the box. The flat results table
 * is parsed with regexes, redirect-wrapped links are unwrapped to their
 * destination URLs, and entries without a decodable URL are dropped. DuckDuckGo
 * returns no generated answer, so `content` is omitted.
 * @module @deepseek-ai/dsh-web-search-duckduckgo/provider
 */

import { WebError } from '@deepseek-ai/dsh-web'
import type {
  WebSearchProvider,
  WebSearchRequest,
  WebSearchResult,
  WebSearchSource,
} from '@deepseek-ai/dsh-web'
import type { DuckDuckGoAnchor, DuckDuckGoResult, DuckDuckGoRow } from './types.ts'

/** Stable id this provider registers under. */
export const DUCKDUCKGO_PROVIDER_ID = 'duckduckgo'

/**
 * Default DuckDuckGo Lite endpoint; the query string carries the search
 * parameters (`q`, and optionally `kl`).
 */
export const DUCKDUCKGO_DEFAULT_BASE_URL = 'https://lite.duckduckgo.com/lite/'

/** Attribution header sent on every request. Bump with the package version. */
const USER_AGENT = 'deepseek-harness/0.0.1'

/**
 * Upper bound on the decoded response body. A healthy results page stays far
 * below this; a larger body means an interstitial or an error page, not
 * results worth parsing.
 */
const MAX_BODY_CHARS = 1_000_000

/**
 * The response marker of an anomaly/challenge page. A 202 status plus this
 * marker means rate limiting, not an empty result set, and gets its own
 * message so operators can tell throttling apart from breakage.
 */
const ANOMALY_MARKER = 'anomaly'

/** Resolved provider options (the plugin's `apply` fills constant defaults). */
export interface DuckDuckGoSearchProviderOptions {
  /** Endpoint base serving the Lite HTML page. */
  baseURL: string
  /** Region filter sent as the `kl` parameter. Omitted sends no region. */
  region?: string
}

/**
 * Decode HTML entities in text content. Numeric forms are decoded by code
 * unit, so astral entities would need surrogate pairing — none appear in
 * practice, and an undecoded entity degrades to visible text rather than a
 * wrong URL.
 *
 * @param text - raw text with HTML entities.
 * @returns decoded plain text.
 */
export function decodeEntities(text: string): string {
  return text
    .replace(/&#x([0-9a-f]+);/giu, (_, hex: string) => String.fromCodePoint(Number.parseInt(hex, 16)))
    .replace(/&#(\d+);/gu, (_, dec: string) => String.fromCodePoint(Number.parseInt(dec, 10)))
    .replace(/&quot;/gu, '"')
    .replace(/&amp;/gu, '&')
    .replace(/&lt;/gu, '<')
    .replace(/&gt;/gu, '>')
    .replace(/&nbsp;/gu, ' ')
}

/** Collapse runs of whitespace into one space and trim the ends. */
function normalizeText(html: string): string {
  return decodeEntities(html.replace(/<[^>]*>/gu, '')).replace(/\s+/gu, ' ').trim()
}

/**
 * Unwrap one result href to its destination URL. DuckDuckGo wraps organic
 * results in `/l/?uddg=<encoded>` redirects; direct hrefs pass through.
 *
 * @param href - the anchor's href value.
 * @returns the absolute destination URL, or `undefined` for a protocol-relative,
 *   relative, or non-http(s) href that cannot name a citeable web page.
 */
export function unwrapResultUrl(href: string): string | undefined {
  let url: URL
  try {
    // Result hrefs are protocol-relative (`//duckduckgo.com/l/?uddg=...`).
    url = new URL(href, 'https://duckduckgo.com')
  } catch {
    return undefined
  }
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return undefined
  const uddg = url.pathname === '/l/' || url.pathname === '/l'
    ? url.searchParams.get('uddg')
    : null
  if (uddg === null) return url.toString()
  try {
    const target = new URL(uddg)
    if (target.protocol !== 'https:' && target.protocol !== 'http:') return undefined
    return target.toString()
  } catch {
    return undefined
  }
}

/** Class tokens from an HTML attribute string. Values never contain quotes. */
function classTokens(attrs: string): string[] {
  const klass = /\bclass\s*=\s*("([^"]*)"|'([^']*)')/iu.exec(attrs)
  const value = klass === null ? '' : klass[2] ?? klass[3] ?? ''
  return value.split(/\s+/u).filter(token => token.length > 0)
}

/**
 * Parse one DuckDuckGo Lite results page into rows. Link and snippet anchors
 * use either quote style. Live Lite pages also put the snippet in a
 * `td.result-snippet` cell rather than an anchor. Attribute values never
 * contain quotes or angle brackets, and inner HTML is matched lazily to the
 * first closing tag.
 *
 * @param html - the full response body.
 * @returns anchors grouped row-by-row in document order.
 */
export function parseLiteRows(html: string): DuckDuckGoRow[] {
  const anchors: DuckDuckGoAnchor[] = []
  const anchorPattern = /<a\s([^>]*?)>(.*?)<\/a>/gisu
  for (const match of html.matchAll(anchorPattern)) {
    const attrs = match[1] ?? ''
    const href = /\bhref\s*=\s*("([^"]*)"|'([^']*)')/iu.exec(attrs)
    if (href === null) continue
    anchors.push({
      href: decodeEntities(href[2] ?? href[3] ?? ''),
      class: classTokens(attrs).join(' '),
      html: match[2] ?? '',
    })
  }
  const rows: DuckDuckGoRow[] = []
  for (const anchor of anchors) {
    const tokens = anchor.class.split(/\s+/u)
    if (tokens.includes('result-link')) {
      rows.push({ link: anchor })
    } else if (tokens.includes('result-snippet')) {
      const last = rows.at(-1)
      if (last !== undefined && last.snippet === undefined) last.snippet = anchor
    }
  }
  const cellPattern = /<td\b([^>]*?)>(.*?)<\/td>/gisu
  let rowIndex = 0
  for (const match of html.matchAll(cellPattern)) {
    if (!classTokens(match[1] ?? '').includes('result-snippet')) continue
    while (rowIndex < rows.length && rows[rowIndex]?.snippet !== undefined) rowIndex += 1
    const row = rows[rowIndex]
    if (row === undefined) break
    row.snippet = { href: '', class: 'result-snippet', html: match[2] ?? '' }
    rowIndex += 1
  }
  return rows
}

/**
 * Map parsed rows to normalized sources. A row without a link, without a
 * decodable destination URL, or with a blank title has nothing portable to
 * contribute and is dropped.
 *
 * @param rows - rows from {@link parseLiteRows}.
 * @returns normalized sources in document order.
 */
export function mapLiteRows(rows: readonly DuckDuckGoRow[]): WebSearchSource[] {
  const sources: WebSearchSource[] = []
  for (const row of rows) {
    if (row.link === undefined) continue
    const url = unwrapResultUrl(row.link.href)
    if (url === undefined) continue
    const title = normalizeText(row.link.html)
    if (title.length === 0) continue
    const snippet = row.snippet === undefined ? undefined : normalizeText(row.snippet.html)
    sources.push({
      url,
      title,
      ...snippet !== undefined && snippet.length > 0 ? { snippet } : {},
    })
  }
  return sources
}

/**
 * Parse a DuckDuckGo Lite results page into normalized search results.
 *
 * @param html - the full response body.
 * @returns the normalized result; DuckDuckGo returns no generated answer, so
 *   `content` is omitted. The seam owns the final `maxResults` truncation, so
 *   `truncated` reports `false`.
 */
export function mapLiteResponse(html: string): WebSearchResult {
  return { sources: mapLiteRows(parseLiteRows(html)), truncated: false }
}

/**
 * Parse a results page into plain {@link DuckDuckGoResult}s. Test-facing view
 * over the same mapping the provider applies.
 *
 * @param html - the full response body.
 * @returns decoded result fields in document order.
 */
export function mapResults(html: string): DuckDuckGoResult[] {
  return mapLiteRows(parseLiteRows(html)).flatMap((source) => {
    if (source.title === undefined) return []
    return [{
      url: source.url,
      title: source.title,
      ...source.snippet !== undefined ? { snippet: source.snippet } : {},
    }]
  })
}

/** The DuckDuckGo-backed search provider; HTTP redirects fail as `WEB_PROVIDER_ERROR`. */
export class DuckDuckGoSearchProvider implements WebSearchProvider {
  readonly id = DUCKDUCKGO_PROVIDER_ID

  constructor(private readonly options: DuckDuckGoSearchProviderOptions) {}

  available(): boolean {
    return isValidBaseUrl(this.options.baseURL)
      && (this.options.region === undefined || isValidRegion(this.options.region))
  }

  async search(request: WebSearchRequest, signal?: AbortSignal): Promise<WebSearchResult> {
    // Lite ranks server-side and serves every organic result on one page;
    // `maxResults` is enforced by the seam's truncation, not here.
    void request.maxResults
    const target = new URL(this.options.baseURL)
    target.searchParams.set('q', request.query)
    if (this.options.region !== undefined && this.options.region.length > 0) {
      target.searchParams.set('kl', this.options.region)
    }
    let response: Response
    try {
      response = await fetch(target, {
        method: 'GET',
        redirect: 'error',
        headers: {
          'accept': 'text/html',
          'user-agent': USER_AGENT,
        },
        ...signal !== undefined ? { signal } : {},
      })
    } catch (error: unknown) {
      if (isAbortError(error)) throw new WebError('DuckDuckGo search aborted', 'WEB_ABORTED', { cause: error })
      throw new WebError(`DuckDuckGo search request failed: ${String(error)}`, 'WEB_PROVIDER_ERROR', { cause: error })
    }

    // DuckDuckGo serves results with exactly 200; any other status is a
    // failure even inside 2xx — the anomaly (bot-throttle) page arrives as
    // 202 with `ok === true`.
    if (response.status !== 200) {
      const body = await readBody(response)
      if (response.status === 202 && body.includes(ANOMALY_MARKER)) {
        throw new WebError('DuckDuckGo is throttling automated searches (anomaly page); retry later', 'WEB_PROVIDER_ERROR')
      }
      throw new WebError(`DuckDuckGo API error (HTTP ${String(response.status)})`, 'WEB_PROVIDER_ERROR')
    }

    const body = await readBody(response)
    return mapLiteResponse(body)
  }
}

/** Read the response body as capped text, mapping aborts to `WEB_ABORTED`. */
async function readBody(response: Response): Promise<string> {
  try {
    const text = await response.text()
    if (text.length > MAX_BODY_CHARS) {
      throw new WebError(`DuckDuckGo returned an oversized response body (${String(text.length)} characters)`, 'WEB_PROVIDER_ERROR')
    }
    return text
  } catch (error: unknown) {
    if (error instanceof WebError) throw error
    if (isAbortError(error)) throw new WebError('DuckDuckGo search aborted', 'WEB_ABORTED', { cause: error })
    throw new WebError(`DuckDuckGo returned an unprocessable response body: ${String(error)}`, 'WEB_PROVIDER_ERROR', { cause: error })
  }
}

/** True when `baseURL` parses as an absolute URL (a cheap local config check). */
function isValidBaseUrl(baseURL: string): boolean {
  return URL.canParse(baseURL)
}

/** True for a region tag that can be sent as `kl` (short alphanumeric-dash form). */
function isValidRegion(region: string): boolean {
  return /^[a-z]{2}-[a-z]{2}$/iu.test(region)
}

/** True for a fetch/`AbortSignal` abort, surfaced as `WEB_ABORTED`. */
function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && error.name === 'AbortError'
}
