/**
 * Real-process e2e test for Laya local on-demand activation and JEV-to-Laya failover.
 *
 * Runs only when the local python environment, worker script, and model cache exist.
 * Verifies that:
 * 1. A cold LayaDecisionProvider spawns the local Python worker process on-demand.
 * 2. It waits for the model to finish loading and answers typed questions.
 * 3. Answers match the decision seam schema and return calibrated probabilities.
 * 4. Under DecisionRuntime with JEV configured as primary and Laya as fallback:
 *    When JEV is unreachable, the runtime automatically fails over to Laya without throwing.
 */

import { existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'
import { afterAll, describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, { type DecisionRequest } from '@deepseek-ai/dsh-decision'
import { JevDecisionProvider } from '@deepseek-ai/dsh-decision-jev'
import {
  DEFAULT_HUB_CACHE_DIR,
  hasCachedModel,
  LAYA_DEFAULT_MODEL,
  LAYA_DEFAULT_MODEL_SUBFOLDER,
  LAYA_PROVIDER_ID,
  LayaDecisionProvider,
} from '../src/index.ts'

const TEST_PORT = 58092
const TEST_BASE_URL = `http://127.0.0.1:${String(TEST_PORT)}`
const PYTHON_PATH = join(homedir(), '.dsh', 'laya-env', 'bin', 'python')
const WORKER_SCRIPT = join(homedir(), '.dsh', 'bin', 'laya-worker.py')

const isRunnable =
  existsSync(PYTHON_PATH)
  && existsSync(WORKER_SCRIPT)
  && hasCachedModel(LAYA_DEFAULT_MODEL, LAYA_DEFAULT_MODEL_SUBFOLDER, DEFAULT_HUB_CACHE_DIR)

const testRequest: DecisionRequest = {
  state: 'Page: search results for "usb cable". Elements: 1 USB-C Cable 2m link, 2 Adapter link, 3 Sort button. Task: open the 2m USB-C cable page.',
  questions: {
    element: {
      type: 'choice',
      instructions: 'Which element should be clicked next?',
      criteria: { '1': 'USB-C Cable 2m link', '2': 'Adapter link', '3': 'Sort button' },
    },
    risk: {
      type: 'score',
      instructions: 'How risky is this action?',
      criteria: ['safe', 'caution', 'destructive'],
    },
    done: {
      type: 'boolean',
      instructions: 'Is the task already complete?',
    },
  },
}

afterAll(async () => {
  try {
    await fetch(`${TEST_BASE_URL}/shutdown`, { method: 'POST', signal: AbortSignal.timeout(2000) })
  } catch {
    // Worker may already be stopped or never started.
  }
})

describe.skipIf(!isRunnable)('decision-laya live e2e (local worker process)', () => {
  it('spawns local worker on-demand, loads checkpoint, and answers typed questions', async () => {
    const provider = new LayaDecisionProvider({
      pythonPath: PYTHON_PATH,
      workerScriptPath: WORKER_SCRIPT,
      host: '127.0.0.1',
      port: TEST_PORT,
      idleTimeoutSeconds: 30,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: TEST_BASE_URL,
      model: LAYA_DEFAULT_MODEL,
      modelSubfolder: LAYA_DEFAULT_MODEL_SUBFOLDER,
      hubCacheDir: DEFAULT_HUB_CACHE_DIR,
    })

    expect(provider.available()).toBe(true)

    const result = await provider.evaluate(testRequest)

    expect(result.providerId).toBe(LAYA_PROVIDER_ID)
    const element = result.answers.element
    expect(element).toBeDefined()
    if (element === undefined || element.type !== 'choice') throw new Error('element answer lost its arm')
    expect(element.choice).toBe('1')
    expect(element.probabilities['1']).toBeGreaterThan(0.5)

    const risk = result.answers.risk
    expect(risk).toBeDefined()
    if (risk === undefined || risk.type !== 'score') throw new Error('risk answer lost its arm')
    expect(risk.score).toBeGreaterThanOrEqual(0)
    expect(risk.score).toBeLessThanOrEqual(2)

    const done = result.answers.done
    expect(done).toBeDefined()
    if (done === undefined || done.type !== 'boolean') throw new Error('done answer lost its arm')
    expect(done.probability).toBeGreaterThanOrEqual(0)
    expect(done.probability).toBeLessThanOrEqual(1)
  }, 90_000)

  it('fails over from unreachable JEV to warm Laya worker through DecisionRuntime', async () => {
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, { provider: 'jev', fallbackProvider: 'laya' })

    const deadJev = new JevDecisionProvider({
      resolveApiKey: () => Promise.resolve('mock-key'),
      hasApiKey: true,
      baseURL: 'http://127.0.0.1:58199', // unreachable high port; real connection refused
      model: 'jev-latest',
    })

    const laya = new LayaDecisionProvider({
      pythonPath: PYTHON_PATH,
      workerScriptPath: WORKER_SCRIPT,
      host: '127.0.0.1',
      port: TEST_PORT,
      idleTimeoutSeconds: 30,
      startupTimeoutMs: 60000,
      healthTimeoutMs: 1000,
      startupPollIntervalMs: 500,
      baseURL: TEST_BASE_URL,
      model: LAYA_DEFAULT_MODEL,
      modelSubfolder: LAYA_DEFAULT_MODEL_SUBFOLDER,
      hubCacheDir: DEFAULT_HUB_CACHE_DIR,
    })

    ctx.decision.registerProvider(deadJev)
    ctx.decision.registerProvider(laya)

    // JEV fails -> Laya handles the evaluation seamlessly!
    const result = await ctx.decision.evaluate(testRequest)

    expect(result.providerId).toBe(LAYA_PROVIDER_ID)
    expect(result.answers.element).toMatchObject({ type: 'choice', choice: '1' })
  }, 30_000)
})
