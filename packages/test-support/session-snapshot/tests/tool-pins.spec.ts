import { describe, expect, it } from 'vitest'
import {
  assertToolPin,
  assertToolPins,
  emptyToolCatalog,
  formatToolPins,
  parseToolPins,
  registerCatalogClass,
  toolSetDigest,
  type SnapshotCapability,
  type ToolPins,
} from '../src/index.ts'

/** One emitted tool schema, with the parameter description under test. */
function grepSchema(description: string): Record<string, unknown> {
  return {
    name: 'grep',
    description: 'Search file contents.',
    parameters: {
      type: 'object',
      properties: { pattern: { type: 'string', description } },
      required: ['pattern'],
    },
  }
}

/** A tool schema that advertises the escalation pair, as a confining backend adds it. */
function escalatingGrepSchema(description: string): Record<string, unknown> {
  return {
    ...grepSchema(description),
    parameters: {
      type: 'object',
      properties: {
        pattern: { type: 'string', description },
        sandbox_permissions: { type: 'string', enum: ['read-only', 'workspace-write', 'danger-full-access'] },
        justification: { type: 'string' },
      },
      required: ['pattern'],
    },
  }
}

/** Pins over one class's header revisions, as the suite writes them. */
function pinsFor(className: string, capability: SnapshotCapability, sets: unknown[][]): ToolPins {
  return parseToolPins(formatToolPins({
    version: 2,
    class: className,
    capability,
    sets: sets.map(set => ({
      tools: set.map(schema => (schema as { name: string }).name),
      digest: toolSetDigest(set),
      capability,
    })),
  }))
}

describe('tool pins and their catalog', () => {
  it('catalogues a class that advertises different bodies per header revision', () => {
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', [
      [grepSchema('first revision')],
      [grepSchema('second revision')],
    ])
    const entry = catalog.classes['sdk/demo']
    expect(entry?.revisions).toHaveLength(2)
    expect(entry?.revisions[0]?.tools[0]?.digest).not.toBe(entry?.revisions[1]?.tools[0]?.digest)
    expect(Object.keys(catalog.schemas)).toHaveLength(2)
  })

  it('accepts a revision sequence whose tool surface changes mid-session', () => {
    const sets = [[grepSchema('first revision')], [grepSchema('second revision')]]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', sets)
    const check = { label: 'session 1', className: 'sdk/demo', pins: pinsFor('sdk/demo', 'unconfined', sets), catalog }
    expect(() => { assertToolPins(sets, check) }).not.toThrow()
  })

  it('reports the exact JSON path when an emitted body diverges', () => {
    const committed = [grepSchema('first revision')]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', [committed])
    const pins = pinsFor('sdk/demo', 'unconfined', [committed])
    let message = ''
    try {
      assertToolPin([grepSchema('edited revision')], { label: 'session 1', className: 'sdk/demo', pins, catalog }, 0)
    } catch (error) {
      message = String(error)
    }
    expect(message).toContain('tool schemas diverged from the pin')
    expect(message).toContain('grep.parameters.properties.pattern.description')
    expect(message).toContain('"first revision" → "edited revision"')
  })

  it('rejects a pin whose digests the catalog does not carry', () => {
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', [[grepSchema('catalogued')]])
    const pins = pinsFor('sdk/demo', 'unconfined', [[grepSchema('pinned')]])
    expect(() => {
      assertToolPin([grepSchema('pinned')], { label: 'session 1', className: 'sdk/demo', pins, catalog }, 0)
    }).toThrow(/the catalog carries sha256:[0-9a-f]+ but the header emits sha256:[0-9a-f]+[\s\S]*regenerate docs\/tool-catalog\.json/u)
  })

  it('rejects a catalog that records fewer revisions than the pin', () => {
    const sets = [[grepSchema('first revision')], [grepSchema('second revision')]]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', [sets[0] as unknown[]])
    const pins = pinsFor('sdk/demo', 'unconfined', sets)
    expect(() => { assertToolPins(sets, { label: 'session 1', className: 'sdk/demo', pins, catalog }) })
      .toThrow(/pins 2 header revision\(s\) but the catalog carries 1[\s\S]*regenerate docs\/tool-catalog\.json/u)
  })

  it('rejects a catalog that records more revisions than the pin', () => {
    const sets = [[grepSchema('only revision')]]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', [
      sets[0] as unknown[],
      [grepSchema('a revision the pin no longer claims')],
    ])
    const pins = pinsFor('sdk/demo', 'unconfined', sets)
    expect(() => { assertToolPins(sets, { label: 'session 1', className: 'sdk/demo', pins, catalog }) })
      .toThrow(/pins 1 header revision\(s\) but the catalog carries 2[\s\S]*regenerate docs\/tool-catalog\.json/u)
  })

  it('rejects a catalog revision that still lists a tool the header no longer emits', () => {
    const emitted = [grepSchema('kept')]
    const committed = [grepSchema('kept'), {
      name: 'removed_tool',
      description: 'A tool this surface no longer mounts.',
      parameters: { type: 'object', properties: {} },
    }]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', [committed])
    const pins = pinsFor('sdk/demo', 'unconfined', [emitted])
    let message = ''
    try {
      assertToolPin(emitted, { label: 'session 1', className: 'sdk/demo', pins, catalog }, 0)
    } catch (error) {
      message = String(error)
    }
    expect(message).toMatch(/revision 1 disagrees with docs\/tool-catalog\.json/u)
    expect(message).toContain('removed_tool: catalogued for revision 1')
    expect(message).toContain('regenerate docs/tool-catalog.json')
  })

  it('rejects an escalation class whose tools advertise no escalation parameters', () => {
    const sets = [[grepSchema('plain')]]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'escalation', sets)
    const pins = pinsFor('sdk/demo', 'escalation', sets)
    expect(() => { assertToolPins(sets, { label: 'session 1', className: 'sdk/demo', pins, catalog }) })
      .toThrow(/capability is "escalation" but no tool advertises sandbox_permissions\/justification[\s\S]*interactive/u)
  })

  it('accepts an interactive class without escalation parameters and rejects one that advertises them', () => {
    const sets = [[grepSchema('plain')]]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'interactive', sets)
    const pins = pinsFor('sdk/demo', 'interactive', sets)
    expect(() => { assertToolPins(sets, { label: 'session 1', className: 'sdk/demo', pins, catalog }) }).not.toThrow()
    const advertised = [escalatingGrepSchema('plain')]
    const advertisedCatalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'interactive', [advertised])
    const advertisedPins = pinsFor('sdk/demo', 'interactive', [advertised])
    expect(() => {
      assertToolPin(advertised, { label: 'session 1', className: 'sdk/demo', pins: advertisedPins, catalog: advertisedCatalog }, 0)
    }).toThrow(/capability is "interactive" but tool "grep" advertises sandbox_permissions\/justification/u)
  })

  it('rejects an unconfined class that advertises escalation parameters', () => {
    const sets = [[escalatingGrepSchema('plain')]]
    const catalog = registerCatalogClass(emptyToolCatalog(), 'sdk/demo', 'unconfined', sets)
    const pins = pinsFor('sdk/demo', 'unconfined', sets)
    expect(() => { assertToolPins(sets, { label: 'session 1', className: 'sdk/demo', pins, catalog }) })
      .toThrow(/capability is "unconfined" but tool "grep" advertises sandbox_permissions\/justification/u)
  })
})
