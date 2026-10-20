/**
 * Wire types for the DuckDuckGo Lite search page (`GET https://lite.duckduckgo.com/lite/`).
 * Types only — no runtime code. The endpoint serves one flat HTML table of
 * organic results; the provider parses it with regexes rather than a DOM, so
 * these interfaces name only the fields that mapping consumes.
 *
 * @module @deepseek-ai/dsh-web-search-duckduckgo/types
 */

/** One anchor of DuckDuckGo's results table. */
export interface DuckDuckGoAnchor {
  /** `href` attribute value; a redirect wrapper for result links. */
  href: string
  /** Element class attribute (`result-link`, `result-snippet`, ...). */
  class: string
  /** Raw inner HTML; entity- and tag-decoded before use. */
  html: string
}

/** One parsed organic row of the DuckDuckGo Lite results page. */
export interface DuckDuckGoRow {
  /** The result link anchor, when the row carries one. */
  link?: DuckDuckGoAnchor
  /** The snippet anchor, when present. */
  snippet?: DuckDuckGoAnchor
}

/** Parsed fields of one DuckDuckGo Lite result. */
export interface DuckDuckGoResult {
  /** Destination URL after unwrap of the `/l/?uddg=` redirect wrapper. */
  url: string
  /** Result title text. */
  title: string
  /** Result snippet text; absent when the page supplies none. */
  snippet?: string
}
