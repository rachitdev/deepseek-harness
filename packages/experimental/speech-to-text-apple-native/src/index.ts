/** macOS-only speech provider backed by the local Apple Native MCP server. */
import { access, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { constants } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { performance } from 'node:perf_hooks'
import type { Client } from '@modelcontextprotocol/client'
import type { StdioClientTransport } from '@modelcontextprotocol/client/stdio'
import type { Context } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import type {} from '@deepseek-ai/dsh-experimental-speech-to-text'
import type { SpeechProviderId, Transcript } from '@deepseek-ai/dsh-experimental-speech-to-text/types'
import { scrubbedParentEnv } from '@deepseek-ai/dsh-subprocess'

/** Local executable and accepted recognition locales; no model download is performed by DSH. */
export interface Config {
  /** Path to the apple-native-mcp executable on the host machine. */
  command: string
  /** Provider identifier registered with the speechToText service. */
  providerId: string
  /** BCP-47 locale identifiers supported by this on-device provider. */
  locales: string[]
  /** Maximum duration in milliseconds to await recognition completion. */
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

const MIME_EXTENSIONS: Readonly<Record<string, string>> = {
  'audio/wav': 'wav',
  'audio/wave': 'wav',
  'audio/x-wav': 'wav',
  'audio/mp4': 'm4a',
  'audio/x-m4a': 'm4a',
  'audio/m4a': 'm4a',
  'audio/aac': 'm4a',
  'audio/mpeg': 'mp3',
  'audio/mp3': 'mp3',
  'audio/aiff': 'aiff',
  'audio/x-aiff': 'aiff',
  'audio/x-caf': 'caf',
  'audio/caf': 'caf',
  'audio/webm': 'webm',
  'audio/ogg': 'ogg',
  'audio/flac': 'flac',
  'audio/x-flac': 'flac',
}

/**
 * Resolve the file extension expected by Apple SpeechAnalyzer from an optional MIME type.
 * @param mimeType - optional incoming audio media type (parameters stripped).
 * @returns normalized file extension without a leading dot; defaults to `wav`.
 */
export function audioFileExtension(mimeType: string | undefined): string {
  const base = mimeType?.split(';', 1)[0]?.trim().toLowerCase() ?? ''
  return MIME_EXTENSIONS[base] ?? 'wav'
}

interface WarmSession {
  readonly client: Client
  readonly transport: StdioClientTransport
}

/**
 * Register only when this macOS Host has the configured local executable.
 * @param ctx - plugin context providing `speechToText`.
 * @param config - local executable path, provider id, locales, and timeout.
 */
export async function apply(ctx: Context, config: Config): Promise<void> {
  if (process.platform !== 'darwin') return
  try { await access(config.command, constants.X_OK) } catch (_error) {
    // The optional native installation may not exist on another user's Mac.
    return
  }
  const controller = new AbortController()
  let activeSession: WarmSession | undefined
  let connecting: Promise<WarmSession> | undefined

  const closeSession = async (session: WarmSession | undefined): Promise<void> => {
    if (!session) return
    if (activeSession === session) activeSession = undefined
    try {
      await session.client.close()
    } catch (_closeError) {
      // Fall back to closing the underlying transport directly.
    }
    try {
      await (session.transport as { close?: () => Promise<void> }).close?.()
    } catch (_transportError) {
      // Ignore transport teardown errors during cleanup.
    }
  }

  const acquireSession = async (signal: AbortSignal): Promise<WarmSession> => {
    if (activeSession) return activeSession
    if (connecting) return await connecting
    const current = (async (): Promise<WarmSession> => {
      // The stdio transport is CommonJS and reaches the Node-only `global` through `cross-spawn`, so the
      // client loads with the first session: a browser Worker mounts this plugin to list its provider
      // without spawning the local server, and `apply` never gets here without the executable.
      let modules: [typeof import('@modelcontextprotocol/client'), typeof import('@modelcontextprotocol/client/stdio')]
      try {
        modules = await Promise.all([import('@modelcontextprotocol/client'), import('@modelcontextprotocol/client/stdio')])
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error)
        throw new Error(`Apple on-device speech could not load the local MCP client to start recognition: ${reason}`, { cause: error })
      }
      const [clientModule, transportModule] = modules
      const client = new clientModule.Client({ name: 'dsh-apple-speech', version: '1.0.0' })
      const transport = new transportModule.StdioClientTransport({ command: config.command, args: [], env: scrubbedParentEnv() })
      const session: WarmSession = { client, transport }
      client.onclose = () => {
        if (activeSession === session) activeSession = undefined
      }
      try {
        await client.connect(transport, { signal, timeout: config.timeoutMs })
        if (signal.aborted) {
          await closeSession(session)
          signal.throwIfAborted()
        }
        activeSession = session
        return session
      } catch (error) {
        await closeSession(session)
        throw error
      }
    })()
    connecting = current
    try {
      return await current
    } finally {
      if (connecting === current) connecting = undefined
    }
  }

  ctx.effect(() => async () => {
    controller.abort(new Error('Apple speech provider unloaded'))
    const current = activeSession
    activeSession = undefined
    await closeSession(current)
  })
  ctx.effect(() => ctx.speechToText.register({
    info: { id: config.providerId as SpeechProviderId, name: 'Apple on-device speech', location: 'host-local', languages: config.locales },
    transcribe: async (input, signal): Promise<Transcript> => {
      const combined = AbortSignal.any([signal, controller.signal])
      combined.throwIfAborted()
      const directory = await mkdtemp(join(tmpdir(), 'dsh-apple-speech-'))
      const file = join(directory, `recording.${audioFileExtension(input.mimeType)}`)
      let session: WarmSession | undefined
      try {
        await writeFile(file, input.audio, { mode: 0o600, flag: 'wx' })
        combined.throwIfAborted()
        session = await acquireSession(combined)
        combined.throwIfAborted()
        const start = performance.now()
        const result = await session.client.callTool({ name: 'transcribe_file', arguments: { path: file, locale: input.language } },
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
      } catch (error) {
        if (combined.aborted) await closeSession(session)
        throw error
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    },
  }))
}
