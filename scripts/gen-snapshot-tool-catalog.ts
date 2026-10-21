/**
 * Generate `docs/tool-catalog.json` from the recorded-session header pins.
 *
 * A pin stores tool names and one schema digest per capability class; the
 * schema bodies live here, deduplicated by digest. `--check` re-derives every
 * pin from the committed catalog without booting a suite.
 */

import { spawnSync } from 'node:child_process'
import { mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  emptyToolCatalog,
  formatToolCatalog,
  mergeToolCatalog,
  parseToolCatalog,
  parseToolPins,
  schemaDigest,
  type ToolCatalog,
} from '@deepseek-ai/dsh-session-snapshot'

const repoRoot = fileURLToPath(new URL('../', import.meta.url))
const catalogPath = join(repoRoot, 'docs/tool-catalog.json')
const partsDir = join(repoRoot, '.tmp/tool-catalog-parts')
/** Suites the pin-refresh run harvests itself. */
const suites = ['snapshots/acp', 'snapshots/sdk', 'snapshots/session'] as const

/**
 * Directories that own committed pins. The Web lane writes its fragments from
 * its own refresh run, so this generator validates its pins without harvesting
 * them.
 */
const pinSuites = [...suites, 'snapshots/web'] as const

/** Run the pin-refresh suites that emit one catalog fragment per header class. */
function harvestPins(): void {
  const result = spawnSync(
    'pnpm',
    ['exec', 'vitest', 'run', '--config', 'vitest.snapshot.config.ts', ...suites],
    {
      cwd: repoRoot,
      stdio: 'inherit',
      env: { ...process.env, DSH_SNAPSHOT: 'repin', DSH_SNAPSHOT_CATALOG_PARTS: partsDir },
    },
  )
  if (result.status !== 0) throw new Error(`acp-snapshot: the pin-refresh run failed with status ${String(result.status)}`)
}

/** Merge every harvested fragment into one catalog. */
async function mergeParts(): Promise<ToolCatalog> {
  if (!existsSync(partsDir)) throw new Error(`acp-snapshot: the pin-refresh run wrote no catalog fragments to ${partsDir}`)
  const fragments = (await readdir(partsDir)).filter(name => name.endsWith('.json')).sort()
  if (fragments.length === 0) throw new Error(`acp-snapshot: ${partsDir} holds no catalog fragment`)
  let catalog = emptyToolCatalog()
  for (const fragment of fragments) {
    catalog = mergeToolCatalog(catalog, parseToolCatalog(await readFile(join(partsDir, fragment), 'utf8')))
  }
  return catalog
}

/**
 * Keep the committed catalog entry of every pinned class this host could not
 * harvest, such as a `pwsh`-only scenario on a host without `pwsh`. A pinned
 * class absent from both the harvest and the committed catalog fails.
 */
async function carryUnharvested(harvested: ToolCatalog): Promise<ToolCatalog> {
  const committed = existsSync(catalogPath)
    ? parseToolCatalog(await readFile(catalogPath, 'utf8'))
    : emptyToolCatalog()
  let catalog = harvested
  for (const file of await pinFiles()) {
    const pins = parseToolPins(await readFile(file, 'utf8'))
    if (catalog.classes[pins.class] !== undefined) continue
    const entry = committed.classes[pins.class]
    if (entry === undefined) {
      throw new Error(`${file}: class "${pins.class}" was not harvested on this host and is absent from ${catalogPath}`)
    }
    const carried = emptyToolCatalog()
    carried.classes[pins.class] = entry
    for (const revision of entry.revisions) {
      for (const tool of revision.tools) {
        const schema = committed.schemas[tool.digest]
        if (schema === undefined) throw new Error(`${catalogPath}: class "${pins.class}" references unknown digest ${tool.digest}`)
        carried.schemas[tool.digest] = schema
      }
    }
    catalog = mergeToolCatalog(catalog, carried)
    process.stdout.write(`${pins.class}: not harvested on this host; kept the committed entry\n`)
  }
  return catalog
}

/** Every committed pin file of the recorded-session suites. */
async function pinFiles(): Promise<string[]> {
  const files: string[] = []
  for (const suite of pinSuites) {
    for (const entry of await readdir(join(repoRoot, suite), { withFileTypes: true })) {
      if (!entry.isDirectory()) continue
      const candidate = join(repoRoot, suite, entry.name, 'tool-pins.expected.json')
      if (existsSync(candidate)) files.push(candidate)
    }
  }
  return files.sort()
}

/**
 * Re-derive every pin from the committed catalog: each class must exist, agree
 * on capability, name every pinned tool, carry a resolvable body per digest,
 * and hold nothing the pins do not claim.
 */
async function check(): Promise<void> {
  const committed = await readFile(catalogPath, 'utf8')
  const catalog = parseToolCatalog(committed)
  if (formatToolCatalog(catalog) !== committed) {
    throw new Error(`${catalogPath} is not in canonical form; run this generator without --check`)
  }
  const files = await pinFiles()
  if (files.length === 0) throw new Error('acp-snapshot: no committed tool pins found')
  const claimed = new Map<string, Set<string>>()
  for (const file of files) {
    const pins = parseToolPins(await readFile(file, 'utf8'))
    const entry = catalog.classes[pins.class]
    if (entry === undefined) throw new Error(`${file}: the catalog does not carry class "${pins.class}"`)
    if (entry.capability !== pins.capability) {
      throw new Error(`${file}: class "${pins.class}" is catalogued as "${entry.capability}" but pins "${pins.capability}"`)
    }
    if (entry.revisions.length !== pins.sets.length) {
      throw new Error(
        `${file}: class "${pins.class}" is catalogued with ${entry.revisions.length} header revision(s) but the pin `
        + `carries ${pins.sets.length}; regenerate docs/tool-catalog.json`,
      )
    }
    const owners = claimed.get(pins.class) ?? new Set<string>()
    claimed.set(pins.class, owners)
    for (const [index, set] of pins.sets.entries()) {
      const revision = entry.revisions[index]
      if (revision === undefined) throw new Error(`${file}: class "${pins.class}" does not catalogue revision ${index + 1}`)
      const names = new Set(revision.tools.map(tool => tool.name))
      const missing = set.tools.filter(name => !names.has(name))
      if (missing.length > 0) {
        throw new Error(`${file}: revision ${index + 1} pins ${missing.join(', ')} which class "${pins.class}" does not catalogue`)
      }
      for (const tool of revision.tools) {
        const schema = catalog.schemas[tool.digest]
        if (schema === undefined) throw new Error(`${file}: class "${pins.class}" references unknown digest ${tool.digest}`)
        if (schemaDigest(schema.schema) !== tool.digest) {
          throw new Error(`${file}: catalogued body for ${schema.name} does not hash to ${tool.digest}`)
        }
        owners.add(tool.name)
      }
    }
  }
  for (const [name, entry] of Object.entries(catalog.classes)) {
    const owners = claimed.get(name)
    if (owners === undefined) throw new Error(`${catalogPath}: class "${name}" is not claimed by any committed pin`)
    const extra = entry.revisions
      .flatMap(revision => revision.tools.map(tool => tool.name))
      .filter(name => !owners.has(name))
    if (extra.length > 0) throw new Error(`${catalogPath}: class "${name}" carries unpinned tools ${extra.join(', ')}`)
  }
  process.stdout.write(`${catalogPath}: ${Object.keys(catalog.classes).length} classes, ${Object.keys(catalog.schemas).length} schemas\n`)
}

async function main(): Promise<void> {
  if (process.argv.includes('--check')) {
    await check()
    return
  }
  if (!process.argv.includes('--no-harvest')) {
    await rm(partsDir, { recursive: true, force: true })
    await mkdir(partsDir, { recursive: true })
    harvestPins()
  }
  const catalog = await carryUnharvested(await mergeParts())
  await writeFile(catalogPath, formatToolCatalog(catalog))
  process.stdout.write(
    `${catalogPath}: ${Object.keys(catalog.classes).length} classes, ${Object.keys(catalog.schemas).length} schemas\n`,
  )
}

await main()
