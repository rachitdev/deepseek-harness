/**
 * `LayaDecisionProvider`: a `DecisionProvider` backed by Laya's System 1 decision model.
 * Executes on-demand against an auto-terminating local worker process to ensure zero idle memory.
 * Translates questions and answers between the DSH decision seam and Laya wire vocabulary.
 * @module @deepseek-ai/dsh-decision-laya/provider
 */

import { spawn } from 'node:child_process'
import { existsSync } from 'node:fs'
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

/** The Laya-backed decision provider with on-demand activation. */
export class LayaDecisionProvider implements DecisionProvider {
  readonly id = LAYA_PROVIDER_ID
  private startLock: Promise<void> | undefined

  constructor(private readonly options: LayaDecisionProviderOptions) {}

  available(): boolean {
    return (
      (existsSync(this.options.pythonPath) && existsSync(this.options.workerScriptPath))
      || URL.canParse(this.options.baseURL)
    )
  }

  /**
   * Ensure the local worker process is running and healthy.
   * If down, starts it on demand and polls `/health` until ready.
   * @param signal - optional cancellation for the startup wait.
   */
  async ensureWorker(signal?: AbortSignal): Promise<void> {
    if (await this.isHealthy()) {
      return
    }

    if (this.startLock !== undefined) {
      await this.startLock
      return
    }

    this.startLock = (async () => {
      try {
        if (await this.isHealthy()) return

        if (!existsSync(this.options.pythonPath) || !existsSync(this.options.workerScriptPath)) {
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
          },
        )
        child.unref()

        const deadline = Date.now() + this.options.startupTimeoutMs
        while (Date.now() < deadline) {
          if (signal?.aborted) {
            throw new DecisionError('Worker startup aborted', 'DECISION_ABORTED')
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

    await this.startLock
  }

  private async isHealthy(): Promise<boolean> {
    try {
      const res = await fetch(`${this.options.baseURL}/health`, {
        signal: AbortSignal.timeout(this.options.healthTimeoutMs),
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
      throw new DecisionError(message, 'DECISION_PROVIDER_ERROR')
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

function assertNever(value: never): never {
  throw new DecisionError(
    `unhandled decision question type: ${JSON.stringify(value)}`,
    'DECISION_PROVIDER_ERROR',
  )
}

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && error.name === 'AbortError'
}
