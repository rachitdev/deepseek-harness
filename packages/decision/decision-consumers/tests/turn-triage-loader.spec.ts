/**
 * REAL-composition tier: boot the enforced turn-triage fixture through app/boot and the Cordis
 * Loader, run one turn, and assert the model request and the durable request header use the
 * tier the decision provider chose.
 */

import { readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { LOADER_SMOKE_TEST_TIMEOUT_MS, runLoaderSmoke } from '@deepseek-ai/dsh-loader-smoke'

const driver = fileURLToPath(new URL('./fixtures/triage-loader/driver.ts', import.meta.url))
const configPath = fileURLToPath(new URL('./fixtures/triage-loader/cordis.yml', import.meta.url))
const repoTsconfig = fileURLToPath(new URL('../../../../tsconfig.json', import.meta.url))

describe('turn triage through a real Loader composition', () => {
  it('routes the turn to the chosen tier and logs the routed model', async () => {
    let report: unknown
    const { stderr } = await runLoaderSmoke({
      label: 'decision turn triage Loader smoke',
      tempDirPrefix: 'decision-turn-triage-loader-',
      binScript: driver,
      libBinScript: driver,
      configPath,
      tsconfigPath: repoTsconfig,
      inspect: async (cwd) => {
        report = JSON.parse(await readFile(join(cwd, 'turn-triage-loader-report.json'), 'utf8'))
      },
    })

    expect(stderr).not.toContain('UNHANDLED')
    expect(report).toEqual({
      decisionCalls: 1,
      requestedModels: ['frontier-model'],
      loggedHeader: { provider: 'fixture', model: 'frontier-model' },
    })
  }, LOADER_SMOKE_TEST_TIMEOUT_MS)
})
