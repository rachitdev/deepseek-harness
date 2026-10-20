/** macOS-only speech provider backed by the local Apple Native MCP server. */
import { access, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { constants } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { performance } from 'node:perf_hooks'
import { Client } from '@modelcontextprotocol/client'
import { StdioClientTransport } from '@modelcontextprotocol/client/stdio'
import type { Context } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import type {} from '@deepseek-ai/dsh-experimental-speech-to-text'
import type { SpeechProviderId, Transcript } from '@deepseek-ai/dsh-experimental-speech-to-text/types'
import { scrubbedParentEnv } from '@deepseek-ai/dsh-subprocess'

/** Local executable and accepted recognition locales; no model download is performed by DSH. */
export interface Config {
  command: string
  providerId: string
  locales: string[]
  timeoutMs: number
}

export const name = 'experimental-speech-to-text-apple-native'
export const inject = ['speechToText']
export const Config: z<Config> = z.object({
  command: z.string().required(),
  providerId: z.string().min(1).default('apple-native'),
  locales: z.array(z.string().min(2)).min(1).required(),
  timeoutMs: z.number().min(1000).default(120_000),
})

/** Register only when this macOS Host has the configured local executable. */
export async function apply(ctx: Context, config: Config): Promise<void> {
  if (process.platform !== 'darwin') return
  try { await access(config.command, constants.X_OK) } catch (_error) {
    // The optional native installation may not exist on another user's Mac.
    return
  }
  const controller = new AbortController()
  ctx.effect(() => () => { controller.abort(new Error('Apple speech provider unloaded')) })
  ctx.effect(() => ctx.speechToText.register({
    info: { id: config.providerId as SpeechProviderId, name: 'Apple on-device speech', location: 'host-local', languages: config.locales },
    transcribe: async (input, signal): Promise<Transcript> => {
      const combined = AbortSignal.any([signal, controller.signal])
      combined.throwIfAborted()
      const directory = await mkdtemp(join(tmpdir(), 'dsh-apple-speech-'))
      const file = join(directory, 'recording.wav')
      let client: Client | undefined
      try {
        await writeFile(file, input.audio, { mode: 0o600, flag: 'wx' })
        combined.throwIfAborted()
        client = new Client({ name: 'dsh-apple-speech', version: '1.0.0' })
        const transport = new StdioClientTransport({ command: config.command, args: [], env: scrubbedParentEnv() })
        await client.connect(transport, { signal: combined, timeout: config.timeoutMs })
        combined.throwIfAborted()
        const start = performance.now()
        const result = await client.callTool({ name: 'transcribe_file', arguments: { path: file, locale: input.language } },
          { signal: combined, timeout: config.timeoutMs })
        combined.throwIfAborted()
        if (result.isError) throw new Error(result.content.map(block => block.type === 'text' ? block.text : '').join(' '))
        const text = result.content.find(block => block.type === 'text')
        if (!text) throw new Error('Apple speech returned no transcript')
        const value: unknown = JSON.parse(text.text)
        if (typeof value !== 'object' || value === null || !('text' in value) || typeof value.text !== 'string'
          || !('audio_seconds' in value) || typeof value.audio_seconds !== 'number') {
          throw new Error('Apple speech returned an invalid transcript')
        }
        return { text: value.text, audioSeconds: value.audio_seconds, inferenceSeconds: (performance.now() - start) / 1000 }
      } finally {
        try { await client?.close() } finally { await rm(directory, { recursive: true, force: true }) }
      }
    },
  }))
}
