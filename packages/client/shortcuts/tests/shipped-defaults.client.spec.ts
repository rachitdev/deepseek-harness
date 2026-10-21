/**
 * Every default a shipped client plugin declares must be admitted by the platform it names.
 *
 * `ShortcutRegistry.register` validates a command's defaults for every runtime and platform and
 * rejects the whole registration on the first inadmissible one, so a single bad default removes the
 * command everywhere instead of failing where it was declared. Only a check over the declarations
 * themselves catches that: this scan reads each shipping plugin's source, because no package exports
 * its command table.
 */
import { readFile } from 'node:fs/promises'
import { glob } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { bindingIssue, normalizeBinding } from '../src/protocol.ts'
import type { ShortcutModifier, ShortcutPlatform, ShortcutRuntime } from '../src/protocol.ts'

const REPO_ROOT = fileURLToPath(new URL('../../../..', import.meta.url))
/**
 * Plugins whose defaults are computed from a helper call rather than written as literals.
 *
 * A scan cannot read those, so each is listed with the spec that registers the real command table
 * through {@link ShortcutRegistry} — where the admission check runs against the computed values.
 */
const COMPUTED_DEFAULTS: Readonly<Record<string, { spec: string; registers: RegExp }>> = {
  'packages/client/ui-workspace/src/client/shortcuts.ts': {
    spec: 'packages/client/ui-workspace/tests/shortcuts.client.spec.ts',
    // The spec must mount the real plugin and read the registration back, so the computed
    // defaults are admitted by ShortcutRegistry rather than merely mentioned.
    registers: /registers six commands with Desktop and Web defaults/u,
  },
}
const CLIENT_SOURCES = 'packages/client/*/src/**/*.ts'
const PLATFORM_PROFILE = /'(web|desktop):(macos|windows|linux)':\s*\{([^}]*)\}/gu
const CODE_FIELD = /code:\s*'([A-Za-z0-9]+)'/u
const MODIFIERS_FIELD = /modifiers:\s*\[([^\]]*)\]/u

/** One default binding exactly as a plugin source file declares it. */
interface DeclaredDefault {
  readonly runtime: ShortcutRuntime
  readonly platform: ShortcutPlatform
  readonly code: string
  readonly modifiers: readonly ShortcutModifier[]
}

/**
 * Read one source file's declared defaults.
 * @param source - the plugin source text.
 * @returns every `'runtime:platform': { code, modifiers }` entry it declares.
 */
function declaredDefaults(source: string): DeclaredDefault[] {
  return [...source.matchAll(PLATFORM_PROFILE)].flatMap((match) => {
    const [, runtime, platform, body] = match
    const code = CODE_FIELD.exec(body ?? '')?.[1]
    const modifiers = MODIFIERS_FIELD.exec(body ?? '')?.[1]
    if (code === undefined) return []
    return [{
      runtime: runtime as ShortcutRuntime,
      platform: platform as ShortcutPlatform,
      code,
      modifiers: modifiers === undefined
        ? []
        : [...modifiers.matchAll(/'([^']+)'/gu)].map(entry => entry[1] as ShortcutModifier),
    }]
  })
}

describe('shipped shortcut defaults', () => {
  it('declares only bindings the owning platform admits', async () => {
    const plugins: string[] = []
    for await (const file of glob(CLIENT_SOURCES, { cwd: REPO_ROOT })) {
      if (!file.includes('/tests/')) plugins.push(file)
    }
    const registrations: string[] = []
    const violations: string[] = []
    let declared = 0
    for (const file of plugins) {
      const source = await readFile(join(REPO_ROOT, file), 'utf8')
      if (!source.includes('shortcuts.register(')) continue
      registrations.push(file)
      const entries = declaredDefaults(source)
      declared += entries.length
      for (const entry of entries) {
        const binding = normalizeBinding({ code: entry.code, modifiers: entry.modifiers }, entry.platform)
        const issue = bindingIssue(binding, entry.runtime, entry.platform)
        if (issue !== null) violations.push(`${file}: ${entry.runtime}:${entry.platform} ${entry.code} → ${issue}`)
      }
    }
    // A declaration form this scan cannot read would silently shrink the check, so each registering
    // plugin must contribute at least one entry or name the spec that registers it for real.
    const undeclared: string[] = []
    for (const file of registrations) {
      if (declaredDefaults(await readFile(join(REPO_ROOT, file), 'utf8')).length > 0) continue
      const declared = COMPUTED_DEFAULTS[file]
      if (declared === undefined) {
        undeclared.push(file)
        continue
      }
      const registration = await readFile(join(REPO_ROOT, declared.spec), 'utf8')
      expect(registration, `${declared.spec} must register the computed defaults`)
        .toMatch(declared.registers)
    }
    expect(undeclared, 'plugins registering shortcuts whose defaults this scan cannot read').toEqual([])
    expect(Object.keys(COMPUTED_DEFAULTS).filter(file => !registrations.includes(file)),
      'allowlisted plugins that no longer register shortcuts').toEqual([])
    expect(declared).toBeGreaterThan(registrations.length)
    expect(violations).toEqual([])
  })
})
