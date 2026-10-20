/**
 * Service Provider plugin registering {@link LayaDecisionProvider} on the decision seam
 * (`ctx.decision`). Manages on-demand activation of the local Laya decision model with automatic
 * idle reclamation.
 * @module @deepseek-ai/dsh-decision-laya
 */

import { existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import type { Context } from '@deepseek-ai/cordis'
import { launchEnvironmentOf } from '@deepseek-ai/dsh-launch-environment'
import z from '@deepseek-ai/schemastery'
import {
  LAYA_DEFAULT_HEALTH_TIMEOUT_MS,
  LAYA_DEFAULT_HOST,
  LAYA_DEFAULT_IDLE_TIMEOUT_SECONDS,
  LAYA_DEFAULT_MODEL,
  LAYA_DEFAULT_MODEL_SUBFOLDER,
  LAYA_DEFAULT_PORT,
  LAYA_DEFAULT_STARTUP_POLL_INTERVAL_MS,
  LAYA_DEFAULT_STARTUP_TIMEOUT_MS,
  LayaDecisionProvider,
} from './provider.ts'

export {
  LAYA_DEFAULT_HEALTH_TIMEOUT_MS,
  LAYA_DEFAULT_HOST,
  LAYA_DEFAULT_IDLE_TIMEOUT_SECONDS,
  LAYA_DEFAULT_MODEL,
  LAYA_DEFAULT_MODEL_SUBFOLDER,
  LAYA_DEFAULT_PORT,
  LAYA_DEFAULT_STARTUP_POLL_INTERVAL_MS,
  LAYA_DEFAULT_STARTUP_TIMEOUT_MS,
  LAYA_PROVIDER_ID,
  LayaDecisionProvider,
  fromLayaAnswer,
  hasCachedModel,
  isExecutableAvailable,
  isLoopbackBaseURL,
  isLoopbackHost,
  toLayaQuestion,
} from './provider.ts'
export type { LayaDecisionProviderOptions } from './types.ts'

/** Cordis plugin name. */
export const name = 'decision-laya'

/** The decision seam must exist before a provider can register on it. */
export const inject = ['decision']

/** Host-local installation paths used when the profile does not override them. */
const DEFAULT_PYTHON_ENV = join(homedir(), '.dsh', 'laya-env', 'bin', 'python')
const DEFAULT_BIN_WORKER = join(homedir(), '.dsh', 'bin', 'laya-worker.py')

/** Default Hugging Face cache root where checkpoints reside. */
export const DEFAULT_HUB_CACHE_DIR = process.env.HF_HUB_CACHE
  ?? join(process.env.HF_HOME ?? join(homedir(), '.cache', 'huggingface'), 'hub')

/** Plugin configuration schema. */
export interface Config {
  /** Python interpreter executable path. */
  pythonPath?: string
  /** Path to worker.py. */
  workerScriptPath?: string
  /** Loopback host address. */
  host?: string
  /** Local port to serve on. */
  port?: number
  /** Idle timeout in seconds before the worker process auto-terminates. */
  idleTimeoutSeconds?: number
  /** Maximum milliseconds to wait for cold model loading. */
  startupTimeoutMs?: number
  /** Timeout in milliseconds for one worker health probe. */
  healthTimeoutMs?: number
  /** Delay in milliseconds between worker readiness probes. */
  startupPollIntervalMs?: number
  /** Base URL for the worker HTTP API. */
  baseURL?: string
  /** HuggingFace repo or local snapshot path. */
  model?: string
  /** Checkpoint subfolder within the HuggingFace repository. */
  modelSubfolder?: string
  /** Hugging Face hub cache directory where the checkpoint is stored. */
  hubCacheDir?: string
  /** Explicit availability override. */
  available?: boolean
}

export const Config: z<Config> = z.object({
  pythonPath: z.string(),
  workerScriptPath: z.string(),
  host: z.string().default(LAYA_DEFAULT_HOST),
  port: z.number().default(LAYA_DEFAULT_PORT),
  idleTimeoutSeconds: z.number().default(LAYA_DEFAULT_IDLE_TIMEOUT_SECONDS),
  startupTimeoutMs: z.number().default(LAYA_DEFAULT_STARTUP_TIMEOUT_MS),
  healthTimeoutMs: z.number().default(LAYA_DEFAULT_HEALTH_TIMEOUT_MS),
  startupPollIntervalMs: z.number().default(LAYA_DEFAULT_STARTUP_POLL_INTERVAL_MS),
  baseURL: z.string(),
  model: z.string().default(LAYA_DEFAULT_MODEL),
  modelSubfolder: z.string().default(LAYA_DEFAULT_MODEL_SUBFOLDER),
  hubCacheDir: z.string(),
  available: z.boolean(),
})

/**
 * Register the Laya decision provider.
 * @param ctx - Cordis context carrying decision and launch environment planes.
 * @param config - resolved plugin configuration.
 */
export function apply(ctx: Context, config: Config = {}): void {
  const env = launchEnvironmentOf(ctx)

  const host = config.host ?? env.get('DSH_LAYA_HOST')?.value ?? LAYA_DEFAULT_HOST
  const port = config.port ?? (env.get('DSH_LAYA_PORT')?.value ? Number(env.get('DSH_LAYA_PORT')?.value) : LAYA_DEFAULT_PORT)
  const idleTimeoutSeconds = config.idleTimeoutSeconds ?? LAYA_DEFAULT_IDLE_TIMEOUT_SECONDS
  const startupTimeoutMs = config.startupTimeoutMs ?? LAYA_DEFAULT_STARTUP_TIMEOUT_MS
  const healthTimeoutMs = config.healthTimeoutMs ?? LAYA_DEFAULT_HEALTH_TIMEOUT_MS
  const startupPollIntervalMs = config.startupPollIntervalMs ?? LAYA_DEFAULT_STARTUP_POLL_INTERVAL_MS
  const baseURL = config.baseURL ?? env.get('DSH_LAYA_BASE_URL')?.value ?? `http://${host}:${String(port)}`
  const model = config.model ?? env.get('DSH_LAYA_MODEL')?.value ?? LAYA_DEFAULT_MODEL
  const modelSubfolder = config.modelSubfolder
    ?? env.get('DSH_LAYA_MODEL_SUBFOLDER')?.value
    ?? LAYA_DEFAULT_MODEL_SUBFOLDER

  const pythonPath = config.pythonPath
    ?? env.get('DSH_LAYA_PYTHON')?.value
    ?? (existsSync(DEFAULT_PYTHON_ENV) ? DEFAULT_PYTHON_ENV : 'python3')

  const bundledWorker = fileURLToPath(new URL('worker.py', import.meta.url))
  const defaultWorker = existsSync(DEFAULT_BIN_WORKER)
    ? DEFAULT_BIN_WORKER
    : existsSync(bundledWorker)
      ? bundledWorker
      : DEFAULT_BIN_WORKER

  const workerScriptPath = config.workerScriptPath
    ?? env.get('DSH_LAYA_WORKER_PATH')?.value
    ?? defaultWorker

  const hfHome = env.get('HF_HOME')?.value
  const hubCacheDir = config.hubCacheDir
    ?? env.get('DSH_LAYA_HUB_CACHE')?.value
    ?? env.get('HF_HUB_CACHE')?.value
    ?? (hfHome ? join(hfHome, 'hub') : undefined)
    ?? DEFAULT_HUB_CACHE_DIR

  const availableEnv = env.get('DSH_LAYA_AVAILABLE')?.value.toLowerCase()
  const configAvailable = config.available
  const available = configAvailable !== undefined
    ? () => configAvailable
    : availableEnv === '1' || availableEnv === 'true'
      ? () => true
      : availableEnv === '0' || availableEnv === 'false'
        ? () => false
        : undefined

  const provider = new LayaDecisionProvider({
    pythonPath,
    workerScriptPath,
    host,
    port,
    idleTimeoutSeconds,
    startupTimeoutMs,
    healthTimeoutMs,
    startupPollIntervalMs,
    baseURL,
    model,
    modelSubfolder,
    hubCacheDir,
    ...available !== undefined ? { available } : {},
  })

  ctx.effect(() => ctx.decision.registerProvider(provider))
}
