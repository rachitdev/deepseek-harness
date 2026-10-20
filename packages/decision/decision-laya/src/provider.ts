/**
 * `LayaDecisionProvider`: a `DecisionProvider` backed by Laya's System 1 decision model.
 * Executes on-demand against an auto-terminating local worker process to ensure zero idle memory.
 * Translates questions and answers between the DSH decision seam and Laya wire vocabulary.
 * @module @deepseek-ai/dsh-decision-laya/provider
 */

import { spawn } from 'node:child_process'
import { existsSync, readdirSync } from 'node:fs'
import { isIP } from 'node:net'
import { isAbsolute, join } from 'node:path'
import { DecisionError } from '@deepseek-ai/dsh-decision'
import type {
  DecisionAnswer,
  DecisionProvider,
  DecisionQuestion,
  DecisionRequest,
  DecisionResult,
} from '@deepseek-ai/dsh-decision'
import type {
  LayaAnswer,
  LayaDecisionProviderOptions,
  LayaQuestion,
  LayaResponse,
} from './types.ts'

/** Stable id this provider registers under. */
export const LAYA_PROVIDER_ID = 'laya'

/** Default local worker port. */
export const LAYA_DEFAULT_PORT = 58085

/** Default loopback host. */
export const LAYA_DEFAULT_HOST = '127.0.0.1'

/** Default idle timeout in seconds (5 minutes). */
export const LAYA_DEFAULT_IDLE_TIMEOUT_SECONDS = 300

/** Default HuggingFace model repo. */
export const LAYA_DEFAULT_MODEL = 'convaiinnovations/laya'

/** Checkpoint specialized for the typed decisions exposed by `ctx.decision`. */
export const LAYA_DEFAULT_MODEL_SUBFOLDER = 'typed-decisions'

/** Default maximum wait for a cold worker to load the model. */
export const LAYA_DEFAULT_STARTUP_TIMEOUT_MS = 60000

/** Default timeout for one local health probe. */
export const LAYA_DEFAULT_HEALTH_TIMEOUT_MS = 1000

/** Default delay between worker readiness probes. */
export const LAYA_DEFAULT_STARTUP_POLL_INTERVAL_MS = 500

/** Attribution header sent on every request. */
const USER_AGENT = 'deepseek-harness/0.0.1'

/**
 * Translate one seam question onto Laya's wire vocabulary.
 * @param question - the seam question to translate.
 * @returns the question formatted for Laya.
 */
export function toLayaQuestion(question: DecisionQuestion): LayaQuestion {
  switch (question.type) {
    case 'choice':
      return { type: 'choice', instructions: question.instructions, criteria: question.criteria }
    case 'score':
      return { type: 'score', instructions: question.instructions, criteria: question.criteria }
    case 'boolean':
      return { type: 'noul', instructions: question.instructions }
    default:
      return assertNever(question)
  }
}

/**
 * Map one Laya answer back onto the seam's answer union.
 * @param key - question key.
 * @param answer - the answer as returned by Laya.
 * @returns the answer in seam vocabulary.
 */
export function fromLayaAnswer(key: string, answer: LayaAnswer): DecisionAnswer {
  switch (answer.type) {
    case 'choice': {
      if (answer.choice === undefined || answer.probabilities === undefined) {
        throw new DecisionError(`Laya choice answer "${key}" is missing its distribution`, 'DECISION_PROVIDER_ERROR')
      }
      return { type: 'choice', choice: answer.choice, probabilities: answer.probabilities }
    }
    case 'score': {
      if (answer.score === undefined || answer.probabilities === undefined) {
        throw new DecisionError(`Laya score answer "${key}" is missing its distribution`, 'DECISION_PROVIDER_ERROR')
      }
      return { type: 'score', score: answer.score, probabilities: answer.probabilities }
    }
    case 'noul': {
      if (answer.noul === undefined) {
        throw new DecisionError(`Laya noul answer "${key}" is missing its probability`, 'DECISION_PROVIDER_ERROR')
      }
      return { type: 'boolean', probability: answer.noul }
    }
    default:
      throw new DecisionError(
        `Laya returned an unknown answer type for "${key}": ${JSON.stringify(answer.type)}`,
        'DECISION_PROVIDER_ERROR',
      )
  }
}

/**
 * Determine whether a hostname is loopback or local.
 *
 * @param host - hostname or IP literal to inspect.
 * @returns true when the host resolves to a local loopback interface.
 */
export function isLoopbackHost(host: string): boolean {
  const h = host.replace(/^\[|\]$/g, '')
  if (h === 'localhost' || h === '::1' || h === '0.0.0.0') return true
  if (isIP(h) === 4) {
    return h.startsWith('127.')
  }
  return false
}

/**
 * Determine whether a base URL points to a loopback address.
 *
 * @param baseURL - candidate HTTP(S) base URL.
 * @param fallbackHost - host fallback when `baseURL` cannot be parsed.
 * @returns true when the target host is loopback.
 */
export function isLoopbackBaseURL(baseURL: string, fallbackHost?: string): boolean {
  try {
    const parsed = new URL(baseURL)
    return isLoopbackHost(parsed.hostname)
  } catch {
    return fallbackHost !== undefined ? isLoopbackHost(fallbackHost) : false
  }
}

/**
 * Check if an executable path or command is valid to launch.
 *
 * @param executablePath - filesystem path or bare command name.
 * @returns true when the path exists on disk or names a non-empty command.
 */
export function isExecutableAvailable(executablePath: string): boolean {
  if (executablePath.includes('/') || executablePath.includes('\\')) {
    return existsSync(executablePath)
  }
  return executablePath.trim().length > 0
}

/**
 * Check whether a Hugging Face repository snapshot or local model folder exists on disk.
 *
 * @param model - Hugging Face model id or local filesystem path.
 * @param subfolder - optional checkpoint subfolder inside the model directory.
 * @param hubCacheDir - local Hugging Face hub cache root.
 * @returns true when weights or config artifacts are present on disk.
 */
export function hasCachedModel(model: string, subfolder?: string, hubCacheDir?: string): boolean {
  const isExplicitPath = isAbsolute(model) || model.startsWith('./') || model.startsWith('../')
  if (isExplicitPath) {
    const target = subfolder === undefined ? model : join(model, subfolder)
    return existsSync(target) && hasModelFiles(target)
  }
  if (!hubCacheDir || !existsSync(hubCacheDir)) return false
  const repoSlug = `models--${model.replace(/\//g, '--')}`
  const snapshotsDir = join(hubCacheDir, repoSlug, 'snapshots')
  if (!existsSync(snapshotsDir)) return false
  try {
    const entries = readdirSync(snapshotsDir)
    for (const entry of entries) {
      const target = subfolder === undefined ? join(snapshotsDir, entry) : join(snapshotsDir, entry, subfolder)
      if (existsSync(target) && hasModelFiles(target)) return true
    }
  } catch {
    return false
  }
  return false
}

function hasModelFiles(dir: string): boolean {
  try {
    const files = readdirSync(dir)
    return files.some(f => f.endsWith('.safetensors') || f.endsWith('.bin') || f === 'rl_agent_config.json')
  } catch {
    return false
  }
}

/** The Laya-backed decision provider with on-demand activation. */
export class LayaDecisionProvider implements DecisionProvider {
  readonly id = LAYA_PROVIDER_ID
  private startLock: Promise<void> | undefined

  constructor(private readonly options: LayaDecisionProviderOptions) {}

  available(): boolean {
    if (this.options.available !== undefined) {
      return this.options.available()
    }
    if (!URL.canParse(this.options.baseURL) || this.options.model.length === 0) {
      return false
    }
    if (!isLoopbackBaseURL(this.options.baseURL, this.options.host)) {
      return true
    }
    return (
      isExecutableAvailable(this.options.pythonPath)
      && existsSync(this.options.workerScriptPath)
      && hasCachedModel(this.options.model, this.options.modelSubfolder, this.options.hubCacheDir)
    )
  }

  /**
   * Ensure the local worker process is running and healthy.
   * If down, starts it on demand and polls `/health` until ready.
   * @param signal - optional cancellation for the startup wait.
   */
  async ensureWorker(signal?: AbortSignal): Promise<void> {
    if (signal?.aborted) {
      throw new DecisionError('Worker startup aborted', 'DECISION_ABORTED')
    }

    if (!isLoopbackBaseURL(this.options.baseURL, this.options.host)) {
      const healthy = await this.isHealthy(signal)
      if (signal?.aborted) {
        throw new DecisionError('Worker startup aborted', 'DECISION_ABORTED')
      }
      if (!healthy) {
        throw new DecisionError(
          `Remote Laya worker at ${this.options.baseURL} is not reachable`,
          'DECISION_PROVIDER_ERROR',
        )
      }
      return
    }

    if (await this.isHealthy(signal)) {
      return
    }

    if (this.startLock !== undefined) {
      await raceWithAbort(this.startLock, signal)
      return
    }

    this.startLock = (async () => {
      try {
        if (await this.isHealthy()) return

        if (!isExecutableAvailable(this.options.pythonPath) || !existsSync(this.options.workerScriptPath)) {
          throw new DecisionError(
            `Cannot start Laya worker: python (${this.options.pythonPath}) or worker script (${this.options.workerScriptPath}) missing`,
            'DECISION_PROVIDER_UNAVAILABLE',
          )
        }

        const child = spawn(
          this.options.pythonPath,
          [
            this.options.workerScriptPath,
            '--host',
            this.options.host,
            '--port',
            String(this.options.port),
            '--idle-timeout',
            String(this.options.idleTimeoutSeconds),
            '--model',
            this.options.model,
            ...this.options.modelSubfolder === undefined
              ? []
              : ['--model-subfolder', this.options.modelSubfolder],
          ],
          {
            stdio: 'ignore',
            detached: true,
            env: {
              ...process.env,
              USE_TF: '0',
              ...(this.options.hubCacheDir ? { HF_HUB_CACHE: this.options.hubCacheDir } : {}),
            },
          },
        )
        let spawnError: Error | undefined
        child.once('error', (err) => {
          spawnError = err
        })
        child.unref()

        const deadline = Date.now() + this.options.startupTimeoutMs
        while (Date.now() < deadline) {
          if (spawnError !== undefined) {
            throw new DecisionError(
              `Failed to launch Laya worker: ${spawnError.message}`,
              'DECISION_PROVIDER_UNAVAILABLE',
            )
          }
          if (await this.isHealthy()) {
            return
          }
          await new Promise(resolve => setTimeout(resolve, this.options.startupPollIntervalMs))
        }

        throw new DecisionError(
          `Timed out waiting for Laya worker to initialize at ${this.options.baseURL}`,
          'DECISION_PROVIDER_ERROR',
        )
      } finally {
        this.startLock = undefined
      }
    })()

    await raceWithAbort(this.startLock, signal)
  }

  private async isHealthy(signal?: AbortSignal): Promise<boolean> {
    if (signal?.aborted) return false
    try {
      const probeSignal = signal !== undefined
        ? AbortSignal.any([signal, AbortSignal.timeout(this.options.healthTimeoutMs)])
        : AbortSignal.timeout(this.options.healthTimeoutMs)
      const res = await fetch(`${this.options.baseURL}/health`, {
        signal: probeSignal,
        redirect: 'error',
      })
      return res.ok
    } catch {
      return false
    }
  }

  async evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult> {
    await this.ensureWorker(signal)

    const questions = Object.fromEntries(
      Object.entries(request.questions).map(([key, question]) => [key, toLayaQuestion(question)]),
    )

    let response: Response
    try {
      response = await fetch(`${this.options.baseURL}/v1/systemone`, {
        method: 'POST',
        redirect: 'error',
        headers: {
          'content-type': 'application/json',
          'accept': 'application/json',
          'user-agent': USER_AGENT,
        },
        body: JSON.stringify({ state: request.state, questions }),
        ...signal !== undefined ? { signal } : {},
      })
    } catch (error: unknown) {
      if (isAbortError(error)) throw new DecisionError('Laya evaluation aborted', 'DECISION_ABORTED', { cause: error })
      throw new DecisionError(`Laya evaluation request failed: ${String(error)}`, 'DECISION_PROVIDER_ERROR', { cause: error })
    }

    if (!response.ok) {
      const status = response.status
      let message = `Laya worker error (HTTP ${String(status)})`
      try {
        const parsed = await response.json() as LayaResponse
        const detail = parsed.error?.message
        if (detail !== undefined && detail.length > 0) message = detail
      } catch (error: unknown) {
        if (isAbortError(error)) throw new DecisionError('Laya evaluation aborted', 'DECISION_ABORTED', { cause: error })
      }
      const code =
        status === 429
          ? 'DECISION_RATE_LIMITED'
          : status >= 400 && status < 500
            ? 'DECISION_REQUEST_REJECTED'
            : 'DECISION_PROVIDER_ERROR'
      throw new DecisionError(message, code)
    }

    let payload: LayaResponse
    try {
      payload = await response.json() as LayaResponse
    } catch (error: unknown) {
      if (isAbortError(error)) throw new DecisionError('Laya evaluation aborted', 'DECISION_ABORTED', { cause: error })
      throw new DecisionError(
        `Laya returned an unprocessable response body: ${String(error)}`,
        'DECISION_PROVIDER_ERROR',
        { cause: error },
      )
    }

    const returned = payload.answers
    if (returned === undefined) {
      throw new DecisionError('Laya response carried no answers', 'DECISION_PROVIDER_ERROR')
    }
    const answers: Record<string, DecisionAnswer> = {}
    for (const key of Object.keys(request.questions)) {
      const answer = returned[key]
      if (answer === undefined) {
        throw new DecisionError(`Laya returned no answer for question "${key}"`, 'DECISION_PROVIDER_ERROR')
      }
      answers[key] = fromLayaAnswer(key, answer)
    }
    return { answers, providerId: this.id }
  }
}

function raceWithAbort<T>(promise: Promise<T>, signal?: AbortSignal): Promise<T> {
  if (!signal) return promise
  if (signal.aborted) return Promise.reject(new DecisionError('Worker startup aborted', 'DECISION_ABORTED'))
  return new Promise((resolve, reject) => {
    const onAbort = () => {
      signal.removeEventListener('abort', onAbort)
      reject(new DecisionError('Worker startup aborted', 'DECISION_ABORTED'))
    }
    signal.addEventListener('abort', onAbort, { once: true })
    promise.then(
      (res) => {
        signal.removeEventListener('abort', onAbort)
        resolve(res)
      },
      (err: unknown) => {
        signal.removeEventListener('abort', onAbort)
        reject(err instanceof Error ? err : new DecisionError(String(err), 'DECISION_PROVIDER_ERROR'))
      },
    )
  })
}

function assertNever(value: never): never {
  throw new DecisionError(
    `unhandled decision question type: ${JSON.stringify(value)}`,
    'DECISION_PROVIDER_ERROR',
  )
}

function isAbortError(error: unknown): boolean {
  if (error instanceof DecisionError && error.code === 'DECISION_ABORTED') return true
  return error instanceof DOMException && error.name === 'AbortError'
}
