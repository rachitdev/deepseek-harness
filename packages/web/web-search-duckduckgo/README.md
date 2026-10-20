---
description: "The DuckDuckGo-backed search provider for ctx.web: a keyless Lite HTML backend that parses organic results into the seam's normalized WebSearchResult."
kind: "package-reference"
---

# @deepseek-ai/dsh-web-search-duckduckgo

English | [中文](README.zh.md)

## Summary

A [DuckDuckGo](https://duckduckgo.com)-backed `WebSearchProvider` for the [web capability seam](../web/README.md) (`ctx.web`). It fetches the free DuckDuckGo Lite HTML results page (`GET https://lite.duckduckgo.com/lite/?q=<query>`) and parses its organic-results table into the seam's normalized `WebSearchResult`. It needs no API key, so it works as a zero-cost backend and a fallback when credentialed providers are unconfigured. It registers a provider into `ctx.web` and neither owns `ctx.web` nor registers a model-facing tool; `@deepseek-ai/dsh-tool-web` owns that tool.

## Table of Contents

- [Summary](#summary)
- [Config](#config)
- [Mapping](#mapping)
- [Dev Note](#dev-note)
- [Model Experience](#model-experience)
- [Known Limitations and Deferred Work](#known-limitations-and-deferred-work)

-----

## Config

| Key | Default | Meaning |
|---|---|---|
| `baseURL` | `https://lite.duckduckgo.com/lite/` | Endpoint base serving the Lite HTML page; the query string is appended. An unparseable value makes the provider unavailable. |
| `region` | (unset) | Region filter sent as the `kl` parameter (e.g. `us-en`). Unset sends no region. Must be an `xx-yy` language-region tag when present. |

```yaml
- id: web-search-duckduckgo
  name: '@deepseek-ai/dsh-web-search-duckduckgo'
```

## Mapping

DuckDuckGo Lite serves every organic result on one page as `result-link` anchors plus a `result-snippet` anchor or `td.result-snippet` cell, so each result maps to a `WebSearchSource`: `url` ← the anchor href after unwrap of the `/l/?uddg=` redirect wrapper, `title` ← the link text, `snippet` ← the row's snippet text (a row without a snippet maps without one). A row with no link, no decodable http(s) destination URL, or a blank title has nothing portable to contribute and is dropped. DuckDuckGo returns no generated answer, so `content` is omitted, and Lite ranks server-side, so a request's `maxResults` is enforced by the seam's truncation rather than at this provider. Failures surface as `WebError`: network failure, HTTP errors, and oversized bodies as `WEB_PROVIDER_ERROR`; an aborted request as `WEB_ABORTED`; and DuckDuckGo's 202 anomaly (bot-throttle) page as `WEB_PROVIDER_ERROR` with a message that names throttling. HTTP redirects are rejected before the `Location` target is contacted.

## Dev Note

The provider scrapes an unofficial HTML layout rather than calling a documented API, which is the whole reason it can ship without a credential and also the reason it is not a production default. Keeping the parsing rules and the throttle-detection path covered by fixtures is what keeps a layout change visible as a test failure instead of a silent empty result set.

## Model Experience

Indirectly, through [`dsh-tool-web`](../tool-web/README.md), which retains this provider's URLs, titles, and snippets or its exact `DuckDuckGo search aborted`, `DuckDuckGo search request failed: <error>`, `DuckDuckGo API error (HTTP <status>)`, and `DuckDuckGo returned an unprocessable response body: <error>` failures under the consumer's error wrapper while page chrome and provider-private markup remain outside context.

#### KV Cache effect

No direct invalidation; the named consumer owns any request-prefix changes.

## Known Limitations and Deferred Work

- **Scraping contract** — the Lite HTML layout is unofficial and can change without notice; a layout change surfaces as empty results or parse drift, not an API error. The official paid APIs (Exa, Perplexity) stay the supported backends for production deployments.
- **No publication dates** — Lite exposes no per-result timestamp, so `publishedAt` is never mapped.
- **Bot throttling** — automated request patterns can receive the anomaly page; the provider reports it as a throttle error instead of retrying.
