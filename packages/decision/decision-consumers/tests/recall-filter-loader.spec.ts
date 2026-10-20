/**
 * REAL-composition tier: boot the recall-filter fixture through app/boot and the Cordis Loader
 * with the memory plugin listed after decision-consumers, run one turn, and assert the durable
 * session log holds the screened recall.
 */

import { readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { LOADER_SMOKE_TEST_TIMEOUT_MS, runLoaderSmoke } from '@deepseek-ai/dsh-loader-smoke'
import { omittedItemsNote, WITHHELD_INJECTION_MARKER } from '../src/index.ts'

const driver = fileURLToPath(new URL('./fixtures/recall-loader/driver.ts', import.meta.url))
const configPath = fileURLToPath(new URL('./fixtures/recall-loader/cordis.yml', import.meta.url))
const repoTsconfig = fileURLToPath(new URL('../../../../tsconfig.json', import.meta.url))

describe('recall filter through a real Loader composition', () => {
  it('screens recall from a memory plugin mounted after decision-consumers', async () => {
    let report: unknown
    const { stderr } = await runLoaderSmoke({
      label: 'decision recall filter Loader smoke',
      tempDirPrefix: 'decision-recall-filter-loader-',
      binScript: driver,
      libBinScript: driver,
      configPath,
      tsconfigPath: repoTsconfig,
      inspect: async (cwd) => {
        report = JSON.parse(await readFile(join(cwd, 'recall-filter-loader-report.json'), 'utf8'))
      },
    })

    expect(stderr).not.toContain('UNHANDLED')
    expect(report).toEqual({
      llmRequests: 1,
      decisionCalls: 2,
      recall: [[
        '<openviking-context>',
        '<memory uri="viking://useful">Prefers Rust for DSH utilities.</memory>',
        `<memory uri="viking://attack" screened="${WITHHELD_INJECTION_MARKER}" />`,
        omittedItemsNote(1),
        '</openviking-context>',
      ].join('\n')],
    })
  }, LOADER_SMOKE_TEST_TIMEOUT_MS)
})
