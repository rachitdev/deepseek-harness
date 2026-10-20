/**
 * Wire types for Laya's System 1 API and on-demand worker configuration.
 * @module @deepseek-ai/dsh-decision-laya/types
 */

/** One question as the Laya System 1 endpoint accepts it. */
export interface LayaQuestion {
  readonly type: 'choice' | 'score' | 'noul'
  readonly instructions: string
  /** Option map for `choice`, ordered level array for `score`, absent for `noul`. */
  readonly criteria?: Readonly<Record<string, string>> | readonly string[]
}

/** The request body posted to the Laya worker. */
export interface LayaRequest {
  readonly state: string
  readonly questions: Readonly<Record<string, LayaQuestion>>
}

/** One answer as the endpoint returns it; the arm is tagged by `type`. */
export interface LayaAnswer {
  readonly type: 'choice' | 'score' | 'noul'
  /** Winning option id on a `choice` answer. */
  readonly choice?: string
  /** Continuous rubric position on a `score` answer. */
  readonly score?: number
  /** Probability of truth on a `noul` answer. */
  readonly noul?: number
  /** Distribution over options or level indices on `choice` and `score` answers. */
  readonly probabilities?: Readonly<Record<string, number>>
  /** Calibrated confidence score. */
  readonly confidence?: number
  /** Action head output. */
  readonly action?: { readonly act_probability?: number }
}

/** The response body returned by the Laya worker. */
export interface LayaResponse {
  readonly model?: string
  readonly answers?: Readonly<Record<string, LayaAnswer>>
  readonly usage?: { readonly input_tokens?: number; readonly output_tokens?: number }
  readonly error?: { readonly message?: string; readonly type?: string }
}

/** Options configuring the on-demand Laya decision provider. */
export interface LayaDecisionProviderOptions {
  /** Python interpreter executable path to launch the worker under. */
  pythonPath: string
  /** Filesystem path to the worker Python script. */
  workerScriptPath: string
  /** Loopback host address the worker binds to. */
  host: string
  /** Local port the worker binds to. */
  port: number
  /** Automatic idle timeout in seconds before the worker terminates to reclaim RAM. */
  idleTimeoutSeconds: number
  /** Maximum milliseconds to wait for a cold worker to load the model. */
  startupTimeoutMs: number
  /** Timeout in milliseconds for each worker health probe. */
  healthTimeoutMs: number
  /** Delay in milliseconds between worker readiness probes. */
  startupPollIntervalMs: number
  /** Base URL of the worker HTTP server (defaults to http://<host>:<port>). */
  baseURL: string
  /** Model path or HuggingFace repo identifier. */
  model: string
  /** Optional checkpoint subfolder within the HuggingFace repository. */
  modelSubfolder?: string
}
