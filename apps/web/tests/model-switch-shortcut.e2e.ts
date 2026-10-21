// Web e2e scenario: the model switch keyboard shortcut (Cmd/Ctrl+Shift+M by
// default) opens the same /model popup the slash menu opens, for the focused
// conversation, from the composer or from the page. A pick switches the
// composer's model without consuming the typed draft, and Escape returns the
// keyboard to the composer. Zero model calls: the switch is settings/llm-domain
// traffic only. Runs on Chromium and WebKit (Safari's engine).
import { fileURLToPath } from 'node:url'
import type { Browser, Page } from 'playwright'
import { chromium, webkit } from 'playwright'
import { afterAll, beforeAll, describe, expect, it, onTestFailed } from 'vitest'
import { launchWebScaffold, watchConsole, webSnapshotMode, type WebScaffold } from './scaffold.ts'
import { ZH_BROWSER_LOCALE, connectFreshWorkspaceZh, saveFailureShot } from './support.ts'

/** Starts the shared Agent default on this scenario's own route. */
const OVERLAY = fileURLToPath(new URL('./default-model.overlay.yml', import.meta.url))
const MODE = webSnapshotMode()

describe.skipIf(MODE === 'record').each([
  { name: 'Chromium', engine: chromium },
  { name: 'WebKit', engine: webkit },
])('web e2e: the model switch shortcut opens /model ($name)', ({ name, engine }) => {
  let scaffold: WebScaffold
  let browser: Browser
  let page: Page
  let tripwire: ReturnType<typeof watchConsole>

  beforeAll(async () => {
    scaffold = await launchWebScaffold({ extraOverlayPath: OVERLAY })
    await scaffold.ctx.settings.update('llm-pi-ai', {
      providers: {
        'origin-gateway': {
          displayName: 'Origin Gateway',
          api: 'openai-completions',
          baseURL: 'https://gateway.origin.example/v1',
          models: [{ id: 'origin-large', name: 'Origin Large' }],
        },
        'acme-gateway': {
          displayName: 'Acme Gateway',
          api: 'openai-completions',
          baseURL: 'https://gateway.acme.example/v1',
          models: [{ id: 'acme-large', name: 'Acme Large' }, { id: 'acme-small', name: 'Acme Small' }],
        },
      },
    })
    browser = await engine.launch()
    page = await browser.newPage({ viewport: { width: 1680, height: 1000 }, locale: ZH_BROWSER_LOCALE })
    tripwire = watchConsole(page)
    await page.goto(scaffold.authenticatedUrl, { waitUntil: 'load' })
    await page.waitForSelector('[class*="frame"]', { timeout: 30_000 })
    await connectFreshWorkspaceZh(page, scaffold.workspaceCwd)
  }, 120_000)

  afterAll(async () => {
    await browser?.close()
    await scaffold?.close()
  })

  const popup = () => page.locator('[aria-label="/model 选项"]')
  const trigger = () => page.getByRole('button', { name: /^选择模型/ })

  it('opens from the composer, switches the model, keeps the draft, and returns focus', async () => {
    onTestFailed(() => saveFailureShot(page, `web-e2e-model-switch-shortcut-${name}`))
    await page.locator('[data-conversation-session]').first().waitFor()
    const composer = page.locator('[data-composer-input]').first()
    await expect.poll(() => trigger().textContent()).toContain('Origin Large')
    await composer.click()
    await page.keyboard.type('keep this draft')
    await page.keyboard.press('ControlOrMeta+Shift+KeyM')
    const search = page.getByRole('textbox', { name: '筛选选项', exact: true })
    await search.waitFor()
    expect(await search.getAttribute('placeholder')).toBe('搜索模型…')
    expect(await search.evaluate(input => input === input.ownerDocument.activeElement)).toBe(true)
    await search.fill('acme small')
    await page.getByRole('option', { name: /Acme Small/ }).click()
    await popup().waitFor({ state: 'detached' })
    await expect.poll(() => trigger().textContent()).toContain('Acme Small')
    await expect.poll(() => composer.textContent()).toBe('keep this draft')
    await expect.poll(() => composer.evaluate(node => node.contains(node.ownerDocument.activeElement))).toBe(true)
    await composer.fill('')
  })

  it('opens from the page outside the composer and closes with Escape back to the composer', async () => {
    onTestFailed(() => saveFailureShot(page, `web-e2e-model-switch-shortcut-page-${name}`))
    await page.locator('body').click({ position: { x: 5, y: 5 } })
    await page.keyboard.press('ControlOrMeta+Shift+KeyM')
    await popup().waitFor()
    await page.getByRole('option', { name: /Acme Small/ }).waitFor()
    await page.keyboard.press('Escape')
    await popup().waitFor({ state: 'detached' })
    const composer = page.locator('[data-composer-input]').first()
    await expect.poll(() => composer.evaluate(node => node.contains(node.ownerDocument.activeElement))).toBe(true)
    expect(tripwire.pageErrors).toEqual([])
  })
})
