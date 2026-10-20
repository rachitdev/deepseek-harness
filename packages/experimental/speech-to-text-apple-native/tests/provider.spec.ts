/** Apple provider routes recordings through an isolated local MCP call and removes temporary audio. */
import { access, readFile } from 'node:fs/promises'
import { Context } from '@deepseek-ai/cordis'
import SpeechToText from '@deepseek-ai/dsh-experimental-speech-to-text'
import { afterEach, expect, it, vi } from 'vitest'
import { apply, audioFileExtension, Config } from '../src/index.ts'

const state = vi.hoisted(() => ({
  path: '',
  locale: '',
  audio: Buffer.alloc(0),
  closed: 0,
  transportClosed: 0,
  connects: 0,
  failure: false,
  connectFailure: false,
  waitForAbort: false,
  rawOutput: '',
}))

vi.mock('@modelcontextprotocol/client', () => ({
  Client: class {
    onclose?: () => void
    async connect(): Promise<void> {
      state.connects++
      if (state.connectFailure) throw new Error('connection rejected')
    }
    async close(): Promise<void> {
      state.closed++
      this.onclose?.()
    }
    async callTool(request: { arguments: { path: string; locale: string } }, options: { signal: AbortSignal }) {
      state.path = request.arguments.path
      state.locale = request.arguments.locale
      state.audio = await readFile(state.path)
      if (state.waitForAbort) {
        await new Promise<void>((_resolve, reject) => {
          options.signal.addEventListener('abort', () => {
            reject(options.signal.reason instanceof Error ? options.signal.reason : new Error(String(options.signal.reason)))
          }, { once: true })
        })
      }
      return {
        isError: state.failure,
        content: [{
          type: 'text',
          text: state.failure
            ? 'unsupported_locale: not installed'
            : (state.rawOutput || JSON.stringify({ text: 'hello', audio_seconds: 1.5 })),
        }],
      }
    }
  },
}))

vi.mock('@modelcontextprotocol/client/stdio', () => ({
  StdioClientTransport: class {
    readonly dummy = true
    async close(): Promise<void> {
      state.transportClosed++
    }
  },
}))

afterEach(() => {
  state.path = ''
  state.locale = ''
  state.audio = Buffer.alloc(0)
  state.closed = 0
  state.transportClosed = 0
  state.connects = 0
  state.failure = false
  state.connectFailure = false
  state.waitForAbort = false
  state.rawOutput = ''
})

it('maps audio media types to matching file extensions', () => {
  expect(audioFileExtension(undefined)).toBe('wav')
  expect(audioFileExtension('audio/wav')).toBe('wav')
  expect(audioFileExtension('audio/x-m4a; codecs=aac')).toBe('m4a')
  expect(audioFileExtension('audio/mp4')).toBe('m4a')
  expect(audioFileExtension('audio/mpeg')).toBe('mp3')
  expect(audioFileExtension('audio/caf')).toBe('caf')
  expect(audioFileExtension('audio/unknown')).toBe('wav')
})

it.skipIf(process.platform !== 'darwin')('transcribes only the selected locale, keeps a warm client, cleans up audio, and unregisters on unload', async () => {
  const ctx = new Context()
  try {
    await ctx.plugin(SpeechToText, { defaultProvider: 'apple-native', language: 'en_IN' })
    const fiber = ctx.plugin({ inject: ['speechToText'], apply: (inner: Context) => apply(inner, Config({
      command: process.execPath, providerId: 'apple-native', locales: ['en_IN', 'en_US'], timeoutMs: 120_000,
    })) })
    await fiber
    const speech = ctx.get('speechToText')!
    expect(speech.snapshot().providers).toMatchObject([{ id: 'apple-native', location: 'host-local', preparation: { phase: 'ready' } }])
    const audio = new Uint8Array([1, 2, 3])

    // First call connects cold.
    expect(await speech.transcribe(speech.resolve({ audio, mimeType: 'audio/mp4' }), new AbortController().signal))
      .toMatchObject({ text: 'hello', audioSeconds: 1.5 })
    expect(state.path.endsWith('.m4a')).toBe(true)
    expect(state.audio).toEqual(Buffer.from(audio))
    expect(state.locale).toBe('en_IN')
    expect(state.connects).toBe(1)
    expect(state.closed).toBe(0)
    await expect(access(state.path)).rejects.toThrow()

    // Second call reuses the warm client (no reconnect, no close).
    expect(await speech.transcribe(speech.resolve({ audio }), new AbortController().signal))
      .toMatchObject({ text: 'hello', audioSeconds: 1.5 })
    expect(state.path.endsWith('.wav')).toBe(true)
    expect(state.connects).toBe(1)
    expect(state.closed).toBe(0)
    await expect(access(state.path)).rejects.toThrow()

    state.failure = true
    await expect(speech.transcribe(speech.resolve({ audio }), new AbortController().signal)).rejects.toThrow('unsupported_locale')
    await expect(access(state.path)).rejects.toThrow()
    state.failure = false

    state.rawOutput = 'invalid-json'
    await expect(speech.transcribe(speech.resolve({ audio }), new AbortController().signal)).rejects.toThrow()
    await expect(access(state.path)).rejects.toThrow()

    state.rawOutput = JSON.stringify({ text: 'hello' })
    await expect(speech.transcribe(speech.resolve({ audio }), new AbortController().signal)).rejects.toThrow('invalid transcript')
    await expect(access(state.path)).rejects.toThrow()

    state.rawOutput = ''
    state.waitForAbort = true
    state.audio = Buffer.alloc(0)
    const abort = new AbortController()
    const pending = speech.transcribe(speech.resolve({ audio }), abort.signal)
    await vi.waitFor(() => { expect(state.audio).toEqual(Buffer.from(audio)) })
    abort.abort(new Error('Recording cancelled'))
    await expect(pending).rejects.toThrow('Recording cancelled')
    expect(state.closed).toBe(1)
    await expect(access(state.path)).rejects.toThrow()

    // Disposal cleans up the remaining warm client/transport.
    await fiber.dispose()
    expect(state.closed).toBeGreaterThanOrEqual(1)
    expect(state.transportClosed).toBeGreaterThanOrEqual(1)
    expect(speech.listProviders()).toEqual([])
  } finally { await ctx.fiber.dispose() }
})

it.skipIf(process.platform !== 'darwin')('closes the transport and does not leak when connect rejects', async () => {
  const ctx = new Context()
  try {
    await ctx.plugin(SpeechToText, { defaultProvider: 'apple-native', language: 'en_IN' })
    const fiber = await ctx.plugin({ inject: ['speechToText'], apply: (inner: Context) => apply(inner, Config({
      command: process.execPath, providerId: 'apple-native', locales: ['en_IN'], timeoutMs: 120_000,
    })) })
    const speech = ctx.get('speechToText')!
    state.connectFailure = true
    await expect(speech.transcribe(speech.resolve({ audio: new Uint8Array([1]) }), new AbortController().signal))
      .rejects.toThrow('connection rejected')
    expect(state.transportClosed).toBe(1)
    await fiber.dispose()
  } finally { await ctx.fiber.dispose() }
})

it('does not advertise an uninstalled executable', async () => {
  const ctx = new Context()
  try {
    await ctx.plugin(SpeechToText, { defaultProvider: 'apple-native', language: 'en_IN' })
    await ctx.plugin({ inject: ['speechToText'], apply: (inner: Context) => apply(inner, Config({
      command: '/nonexistent/apple-native-mcp', providerId: 'apple-native', locales: ['en_IN'], timeoutMs: 120_000,
    })) })
    expect(ctx.get('speechToText')!.listProviders()).toEqual([])
  } finally { await ctx.fiber.dispose() }
})
