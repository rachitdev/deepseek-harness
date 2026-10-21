import { afterEach, describe, expect, it, vi, type Mock } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import WebRuntime from '@deepseek-ai/dsh-web'
import { DuckDuckGoSearchProvider, DUCKDUCKGO_DEFAULT_BASE_URL, DUCKDUCKGO_PROVIDER_ID } from '@deepseek-ai/dsh-web-search-duckduckgo'
import * as duckduckgoPlugin from '@deepseek-ai/dsh-web-search-duckduckgo'
import { decodeEntities, mapLiteResponse, mapResults, parseLiteRows, unwrapResultUrl } from '../src/provider.ts'

const options = { baseURL: 'https://lite.ddg.test/lite/' }

/** A minimal Lite-shaped page: one link row plus one snippet row per result. */
function litePage(entries: ReadonlyArray<{ title: string; url: string; snippet?: string; quote?: "'" | '"' }>): string {
  return entries.map(({ title, url, snippet, quote = '"' }) => {
    const target = `//duckduckgo.com/l/?uddg=${encodeURIComponent(url)}&rut=x`
    const link = `<a rel="nofollow" href=${quote}${target}${quote} class=${quote}result-link${quote}>${title}</a>`
    const row = `<td>${link}</td>`
    return snippet === undefined ? row : `${row}<td><a href=${quote}${target}${quote} class=${quote}result-snippet${quote}> ${snippet} </a></td>`
  }).join('\n')
}

function htmlResponse(body: string, init: ResponseInit = {}): Response {
  return new Response(body, { status: 200, headers: { 'content-type': 'text/html' }, ...init })
}

/**
 * The single call the provider made to the stubbed `fetch`.
 * @param mock - the typed `fetch` stub installed for this test.
 * @returns the requested URL and the init the provider passed with it.
 */
function recordedFetch(mock: Mock<typeof fetch>): { url: URL; init: RequestInit | undefined } {
  const [call] = mock.mock.calls
  if (call === undefined) throw new Error('the provider never called fetch')
  return { url: new URL(String(call[0])), init: call[1] }
}

/** A response whose body read rejects, for the provider's body-failure paths. */
class FailingBodyResponse extends Response {
  readonly #failure: unknown

  /**
   * @param failure - the value the body read rejects with, such as an abort or a stream error.
   */
  constructor(failure: unknown) {
    super('')
    this.#failure = failure
  }

  /** @returns a promise that always rejects with the recorded failure. */
  override text(): Promise<string> {
    return Promise.reject(this.#failure)
  }
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('DuckDuckGo entity decoding', () => {
  it('decodes numeric and named entities', () => {
    expect(decodeEntities('&#x2F;&#x2E;&quot;&amp;&lt;&gt;&nbsp;')).toEqual(['/', '.', '"', '&', '<', '>', ' '].join(''))
  })

  it('leaves unknown text untouched', () => {
    expect(decodeEntities('plain text')).toBe('plain text')
  })
})

describe('DuckDuckGo URL unwrapping', () => {
  it('unwraps a redirect-wrapped result', () => {
    expect(unwrapResultUrl('//duckduckgo.com/l/?uddg=https%3A%2F%2Fa.test%2Fx&rut=abc')).toBe('https://a.test/x')
  })

  it('passes a direct http(s) href through', () => {
    expect(unwrapResultUrl('https://direct.test/page')).toBe('https://direct.test/page')
  })

  it('drops non-http schemes and unparseable hrefs', () => {
    expect(unwrapResultUrl('javascript:void(0)')).toBeUndefined()
    expect(unwrapResultUrl('mailto:a@b.test')).toBeUndefined()
  })

  it('drops a redirect wrapper whose uddg target is not http(s)', () => {
    expect(unwrapResultUrl(`//duckduckgo.com/l/?uddg=${encodeURIComponent('javascript:x')}`)).toBeUndefined()
    expect(unwrapResultUrl('//duckduckgo.com/l/?uddg=%22not%20a%20url')).toBeUndefined()
  })
})

describe('DuckDuckGo Lite page parsing', () => {
  it('parses both attribute quote styles and pairs snippets with their row', () => {
    const rows = parseLiteRows(litePage([
      { title: 'First', url: 'https://a.test', snippet: 'one', quote: "'" },
      { title: 'Second', url: 'https://b.test', quote: '"' },
    ]))
    expect(rows).toHaveLength(2)
    expect(rows[0]).toMatchObject({ link: { class: 'result-link' }, snippet: { class: 'result-snippet' } })
    expect(rows[1]?.snippet).toBeUndefined()
  })

  it('pairs a td.result-snippet cell with the preceding result link', () => {
    const html = '<td><a href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fa.test" class="result-link">Cell</a></td>'
      + '<td class="result-snippet">from the cell</td>'
    expect(mapResults(html)).toEqual([{ url: 'https://a.test/', title: 'Cell', snippet: 'from the cell' }])
  })

  it('ignores anchors that are not results', () => {
    const rows = parseLiteRows('<a href="//x.test">nav</a>' + litePage([{ title: 'Only', url: 'https://a.test' }]))
    expect(rows).toHaveLength(1)
    expect(mapResults(litePage([]))).toEqual([])
  })

  it('maps a full page to normalized sources', () => {
    const html = litePage([
      { title: 'A &amp; B', url: 'https://a.test/x', snippet: 'salient sentence' },
      { title: 'C', url: 'https://c.test' },
    ])
    expect(mapResults(html)).toEqual([
      { url: 'https://a.test/x', title: 'A & B', snippet: 'salient sentence' },
      { url: 'https://c.test/', title: 'C' },
    ])
  })

  it('drops rows without a decodable URL or with a blank title', () => {
    const html = '<a class="result-link"> </a>'
      + `<a class='result-link' href='//duckduckgo.com/l/?uddg=${encodeURIComponent(encodeURIComponent('javascript:x'))}&rut=x'>bad</a>`
      + litePage([{ title: 'Good', url: 'https://a.test' }])
    expect(mapLiteResponse(html)).toEqual({
      sources: [{ url: 'https://a.test/', title: 'Good' }],
      truncated: false,
    })
  })

  it('decodes entities in titles, snippets, and direct hrefs', () => {
    expect(mapResults(litePage([{ title: 'Q&amp;A', url: 'https://a.test/?x=1%26y=2', snippet: '&#x27;quoted&#39;' }]))).toEqual([{
      url: 'https://a.test/?x=1%26y=2',
      title: 'Q&A',
      snippet: "'quoted'",
    }])
    // A direct (unwrapped) href carries literal entities in its value.
    const direct = '<a class="result-link" href="https://b.test/p?a=1&amp;b=2">B</a>'
    expect(mapResults(direct)).toEqual([{ url: 'https://b.test/p?a=1&b=2', title: 'B' }])
  })

  it('omits content and reports untruncated', () => {
    const result = mapLiteResponse(litePage([{ title: 'T', url: 'https://a.test' }]))
    expect(result.truncated).toBe(false)
    expect(result.content).toBeUndefined()
  })
})

describe('DuckDuckGoSearchProvider availability', () => {
  it('is available with the default endpoint and no key at all', () => {
    expect(new DuckDuckGoSearchProvider({ baseURL: DUCKDUCKGO_DEFAULT_BASE_URL }).available()).toBe(true)
  })

  it('is misconfigured when the base URL is unparseable', () => {
    expect(new DuckDuckGoSearchProvider({ baseURL: 'not a url' }).available()).toBe(false)
  })

  it('is misconfigured when region is not an xx-yy tag', () => {
    expect(new DuckDuckGoSearchProvider({ ...options, region: 'us' }).available()).toBe(false)
    expect(new DuckDuckGoSearchProvider({ ...options, region: 'US-EN' }).available()).toBe(true)
  })
})

describe('DuckDuckGoSearchProvider request mapping', () => {
  it('sends a GET with the query parameter and attribution headers', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => htmlResponse(litePage([])))
    vi.stubGlobal('fetch', fetchMock)

    await new DuckDuckGoSearchProvider(options).search({ query: 'hello world' })

    expect(fetchMock).toHaveBeenCalledOnce()
    const { url, init } = recordedFetch(fetchMock)
    expect(url.toString()).toBe('https://lite.ddg.test/lite/?q=hello+world')
    expect(init).toMatchObject({ method: 'GET', redirect: 'error' })
    expect(init?.headers).toMatchObject({ accept: 'text/html', 'user-agent': 'deepseek-harness/0.0.1' })
  })

  it('appends the region filter when configured', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => htmlResponse(litePage([])))
    vi.stubGlobal('fetch', fetchMock)
    await new DuckDuckGoSearchProvider({ ...options, region: 'de-de' }).search({ query: 'q' })
    expect(recordedFetch(fetchMock).url.searchParams.get('kl')).toBe('de-de')
  })

  it('forwards the abort signal', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => htmlResponse(litePage([])))
    vi.stubGlobal('fetch', fetchMock)
    const controller = new AbortController()
    await new DuckDuckGoSearchProvider(options).search({ query: 'q' }, controller.signal)
    expect(recordedFetch(fetchMock).init?.signal).toBe(controller.signal)
  })
})

describe('DuckDuckGoSearchProvider error handling', () => {
  it('maps an HTTP error to WEB_PROVIDER_ERROR', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => htmlResponse('', { status: 503 })))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ code: 'WEB_PROVIDER_ERROR', message: 'DuckDuckGo API error (HTTP 503)' }))
  })

  it('names bot throttling on the 202 anomaly page', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => htmlResponse('anomaly detection fired', { status: 202 })))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({
        code: 'WEB_PROVIDER_ERROR',
        message: 'DuckDuckGo is throttling automated searches (anomaly page); retry later',
      }))
  })

  it('keeps the status-line message on a 202 without the anomaly marker', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => htmlResponse('empty', { status: 202 })))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ message: 'DuckDuckGo API error (HTTP 202)' }))
  })

  it('maps a network failure to WEB_PROVIDER_ERROR', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.reject(new TypeError('connection refused'))))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ code: 'WEB_PROVIDER_ERROR' }))
  })

  it('maps an abort to WEB_ABORTED', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.reject(new DOMException('aborted', 'AbortError'))))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ code: 'WEB_ABORTED' }))
  })

  it('surfaces an abort during body read as WEB_ABORTED', async () => {
    vi.stubGlobal('fetch', vi.fn<typeof fetch>(async () => new FailingBodyResponse(new DOMException('aborted', 'AbortError'))))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ code: 'WEB_ABORTED' }))
  })

  it('maps a failing body read to WEB_PROVIDER_ERROR', async () => {
    vi.stubGlobal('fetch', vi.fn<typeof fetch>(async () => new FailingBodyResponse(new TypeError('stream broken'))))
    const error: unknown = await new DuckDuckGoSearchProvider(options).search({ query: 'q' })
      .then(() => undefined, (e: unknown) => e)
    expect(error).toMatchObject({ code: 'WEB_PROVIDER_ERROR' })
    expect((error as Error).message).toContain('unprocessable response body')
  })

  it('rejects an oversized body instead of parsing it', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => htmlResponse('x'.repeat(1_000_001))))
    await expect(new DuckDuckGoSearchProvider(options).search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ code: 'WEB_PROVIDER_ERROR', message: expect.stringContaining('oversized response body') }))
  })
})

describe('web-search-duckduckgo plugin registration', () => {
  it('registers the provider into ctx.web (HMR-safe)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => htmlResponse(litePage([{ title: 'R', url: 'https://r.test', snippet: 's' }]))))
    const ctx = new Context()
    await ctx.plugin(WebRuntime, { searchProvider: DUCKDUCKGO_PROVIDER_ID })
    const fiber = await ctx.plugin(duckduckgoPlugin, {})
    await expect(ctx.web.search({ query: 'q' })).resolves.toEqual({
      sources: [{ url: 'https://r.test/', title: 'R', snippet: 's' }],
      truncated: false,
    })
    await fiber.dispose()
    await expect(ctx.web.search({ query: 'q' }))
      .rejects.toThrow(expect.objectContaining({ code: 'WEB_PROVIDER_CONFIGURED_MISSING' }))
  })

  it('has no default export (namespace plugin export shape)', () => {
    expect('default' in duckduckgoPlugin).toBe(false)
  })

  it('threads baseURL and region config into the request', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => htmlResponse(litePage([])))
    vi.stubGlobal('fetch', fetchMock)
    const ctx = new Context()
    await ctx.plugin(WebRuntime, { searchProvider: DUCKDUCKGO_PROVIDER_ID })
    const fiber = await ctx.plugin(duckduckgoPlugin, { baseURL: options.baseURL, region: 'us-en' })
    await ctx.web.search({ query: 'q' })
    expect(recordedFetch(fetchMock).url.toString()).toBe('https://lite.ddg.test/lite/?q=q&kl=us-en')
    await fiber.dispose()
  })

  it('falls back to the public Lite endpoint when config omits baseURL', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => htmlResponse(litePage([])))
    vi.stubGlobal('fetch', fetchMock)
    const ctx = new Context()
    await ctx.plugin(WebRuntime, { searchProvider: DUCKDUCKGO_PROVIDER_ID })
    const fiber = await ctx.plugin(duckduckgoPlugin, {})
    await ctx.web.search({ query: 'q' })
    expect(recordedFetch(fetchMock).url.toString()).toBe('https://lite.duckduckgo.com/lite/?q=q')
    await fiber.dispose()
  })
})
