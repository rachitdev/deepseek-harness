/**
 * REAL-composition tier: boot the package fixture through app/boot and Cordis
 * Loader, fail an active model, probe alternatives, judge them, and assert the
 * durable request header plus model-produced message use the selected route.
 */

import { readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { LOADER_SMOKE_TEST_TIMEOUT_MS, runLoaderSmoke } from '@deepseek-ai/dsh-loader-smoke'

const driver = fileURLToPath(new URL('./fixtures/loader/driver.ts', import.meta.url))
const configPath = fileURLToPath(new URL('./fixtures/loader/cordis.yml', import.meta.url))
const repoTsconfig = fileURLToPath(new URL('../../../../tsconfig.json', import.meta.url))

interface FailoverLoaderReport {
  requests: Array<{ model: string; agentLoop: boolean }>
  judgeCalls: number
  retryEvents: number
  requestHeader: { provider: string; model: string }
  finalMessage: {
    role: string
    source: { kind: string; provider: string; model: string }
    content: Array<{ type: string; text: string }>
  } | null
}

describe('model failover through a real Loader composition', () => {
  it('switches the failed request route before the same step is retried', async () => {
    let report: FailoverLoaderReport | undefined
    const { stderr } = await runLoaderSmoke({
      label: 'decision model failover Loader smoke',
      tempDirPrefix: 'decision-failover-loader-',
      binScript: driver,
      libBinScript: driver,
      configPath,
      tsconfigPath: repoTsconfig,
      inspect: async (cwd) => {
        report = JSON.parse(await readFile(join(cwd, 'failover-loader-report.json'), 'utf8')) as FailoverLoaderReport
      },
    })

    expect(stderr).not.toContain('UNHANDLED')
    expect(report).toEqual({
      requests: [
        { model: 'active', agentLoop: true },
        { model: 'candidate-a', agentLoop: false },
        { model: 'candidate-b', agentLoop: false },
        { model: 'candidate-b', agentLoop: true },
      ],
      judgeCalls: 1,
      retryEvents: 0,
      requestHeader: { provider: 'fixture', model: 'candidate-b' },
      finalMessage: {
        role: 'assistant',
        source: { kind: 'model', provider: 'fixture', model: 'candidate-b' },
        content: [{ type: 'text', text: 'export const loaderFailover = true' }],
      },
    })
  }, LOADER_SMOKE_TEST_TIMEOUT_MS)
})
