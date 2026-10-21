/**
 * Request-header tool pins and the generated tool catalog they point at.
 *
 * A pin records what a header class *commits* to: the ordered model-facing tool
 * names and one schema digest per header revision. The schemas themselves live
 * in the generated catalog (`docs/tool-catalog.json`), so evolving a tool schema
 * never rewrites a committed pin: the catalog is regenerated from the emitted
 * headers and one digest per class is bumped, both keylessly.
 *
 * A header class also declares its {@link SnapshotCapability}: the sandbox and
 * approval capability the composition must mount. The declaration is forced by
 * the scenario environment and verified against the emitted schemas, so a
 * composition that silently changes capability fails instead of re-recording.
 *
 * @module @deepseek-ai/dsh-session-snapshot/tool-pins
 */

import { createHash } from 'node:crypto'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { basename, join } from 'node:path'

/**
 * The capability a header class pins: the sandbox mode and approval policy its
 * composition must mount, and therefore whether the mutating tool families
 * advertise sandbox escalation. The pairing is exhaustive and mutually
 * exclusive, so "interactive approvals because a backend can escalate" and
 * "interactive approvals without one" stay distinguishable:
 *
 * - `unconfined` — approval `never`, sandbox `danger-full-access` (permission
 *   preset `danger-full-access`): the mutating families MUST NOT advertise
 *   `sandbox_permissions`/`justification`.
 * - `escalation` — approval `ask`, sandbox `workspace-write` (preset
 *   `workspace-write`): at least one mutating family MUST advertise both
 *   parameters, pinned to the escalation vocabulary.
 * - `interactive` — approval `ask`, sandbox `workspace-write` (preset `auto`
 *   resolves it as `danger-full-access` + `ask`): no mutating family mounts an
 *   escalation-capable backend, so NONE may advertise either parameter.
 */
export type SnapshotCapability = 'unconfined' | 'escalation' | 'interactive'

/**
 * Whether a parsed value names one of the three capabilities.
 *
 * @param value The parsed `capability` field.
 * @returns True when the value is a {@link SnapshotCapability}.
 */
function isSnapshotCapability(value: unknown): value is SnapshotCapability {
  return value === 'unconfined' || value === 'escalation' || value === 'interactive'
}

/** The approval policy a capability requires in the composed session. */
export const CAPABILITY_APPROVAL: Readonly<Record<SnapshotCapability, 'ask' | 'never'>> = {
  unconfined: 'never',
  escalation: 'ask',
  interactive: 'ask',
}

/**
 * The `DSH_PERMISSION_MODE` a capability forces, so the host cannot pick it:
 * every shipped composition resolves its sandbox mode and approval policy from
 * this variable before falling back to a snapshot-run default.
 */
export const CAPABILITY_PERMISSION_MODE: Readonly<Record<SnapshotCapability, string>> = {
  unconfined: 'danger-full-access',
  escalation: 'workspace-write',
  interactive: 'workspace-write',
}

/**
 * The two escalation arguments a confining mutating tool advertises together.
 * Mirrors `@deepseek-ai/dsh-sandbox` (`ESCALATION_TARGETS`,
 * `sandboxPermissionsDescription`); `tests/tool-pins.spec.ts` pins the mirror
 * against the shipped constants so drift fails in unit tests.
 */
export const ESCALATION_PARAMETERS = ['sandbox_permissions', 'justification'] as const

/** The closed sandbox-mode vocabulary an advertised `sandbox_permissions` enum must equal. */
export const ESCALATION_MODES: readonly string[] = ['read-only', 'workspace-write', 'danger-full-access']

/** The committed pin of one header revision: ordered tool names plus one digest over their schemas. */
export interface ToolPin {
  /** Ordered model-facing tool names, exactly as the request advertises them. */
  tools: string[]
  /** `sha256:` digest over the ordered `{name, schema digest}` pairs of {@link ToolPin.tools}. */
  digest: string
  /**
   * The capability this header revision runs under. Defaults to the class
   * capability; a delegated child keeps the class but pins its own `never`
   * policy, so its schemas omit the escalation parameters.
   */
  capability: SnapshotCapability
}

/** The committed pins of one header class: one entry per request-header revision. */
export interface ToolPins {
  /** Pin format version; `2` stores names and digests instead of schema bodies. */
  version: 2
  /** The generated-catalog class these pins belong to. */
  class: string
  /** The capability this class forces. */
  capability: SnapshotCapability
  /** One pin per header revision, in request order. */
  sets: ToolPin[]
}

/** One catalogued tool schema, addressed by its own digest. */
export interface CatalogSchema {
  /** Model-facing tool name. */
  name: string
  /** The complete model-facing schema body. */
  schema: Record<string, unknown>
}

/** One header revision's catalogued tool surface. */
export interface CatalogRevision {
  /** The capability that revision was harvested under. */
  capability: SnapshotCapability
  /** One digest per tool that revision advertises, ordered by name. */
  tools: { name: string; digest: string }[]
}

/**
 * One header class's catalogs: its capability and the tool digests of every
 * header revision. Revisions are keyed individually because one class may
 * legitimately change its tool surface between requests, such as a delegated
 * child that pins its own narrower policy, or a mid-session mode switch.
 */
export interface CatalogClass {
  /** The capability recorded when the class was harvested. */
  capability: SnapshotCapability
  /** One entry per request-header revision, in header order. */
  revisions: CatalogRevision[]
}

/** The generated schema store the pins point at. */
export interface ToolCatalog {
  /** Catalog format version; `2` keys class tools per header revision. */
  version: 2
  /** Schema bodies keyed by digest, so a body is stored once however many classes advertise it. */
  schemas: Record<string, CatalogSchema>
  /** Per-class tool digests, keyed by header class. */
  classes: Record<string, CatalogClass>
}

/** A newly parsed JSON value, before narrowing. */
export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue }

/**
 * Serialize a value with object keys sorted, so a digest covers content and not
 * key order. Absent object members are skipped; `null` and empty containers
 * survive because a schema can legitimately carry them.
 *
 * @param value The value to serialize.
 * @returns A compact, key-sorted JSON string.
 */
export function canonicalJson(value: unknown): string {
  if (value === null) return 'null'
  if (typeof value === 'number') return Number.isFinite(value) ? JSON.stringify(value) : 'null'
  if (typeof value === 'string' || typeof value === 'boolean') return JSON.stringify(value)
  if (Array.isArray(value)) return `[${value.map(item => canonicalJson(item)).join(',')}]`
  if (typeof value === 'object') {
    const entries = Object.entries(value as Record<string, unknown>)
      .filter(([, member]) => member !== undefined)
      .sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0)
      .map(([key, member]) => `${JSON.stringify(key)}:${canonicalJson(member)}`)
    return `{${entries.join(',')}}`
  }
  return 'null'
}

/**
 * Digest one schema body.
 *
 * @param schema The model-facing tool schema.
 * @returns `sha256:` plus the digest of its {@link canonicalJson} text.
 */
export function schemaDigest(schema: unknown): string {
  return `sha256:${createHash('sha256').update(canonicalJson(schema)).digest('hex')}`
}

/**
 * Digest one header revision's tool set: the ordered `{name, digest}` pairs, so
 * a rename, a reorder, and a schema change are all distinct pins.
 *
 * @param schemas The emitted tool schemas, in advertisement order.
 * @returns `sha256:` plus the digest of the tool set.
 */
export function toolSetDigest(schemas: readonly unknown[]): string {
  const pairs = schemas.map((schema) => {
    const name = toolSchemaName(schema)
    return { name, digest: schemaDigest(schema) }
  })
  return `sha256:${createHash('sha256').update(canonicalJson(pairs)).digest('hex')}`
}

/**
 * Render a pin as canonical JSON.
 *
 * @param pins The pins to render.
 * @returns Pretty-printed JSON ending in one newline.
 */
export function formatToolPins(pins: ToolPins): string {
  return `${JSON.stringify({
    version: pins.version,
    class: pins.class,
    capability: pins.capability,
    sets: pins.sets.map(pin => ({
      ...(pin.capability === pins.capability ? {} : { capability: pin.capability }),
      tools: pin.tools,
      digest: pin.digest,
    })),
  }, null, 2)}\n`
}

/** Assert a value is a JSON object. */
function asObject(value: unknown, message: string): Record<string, unknown> {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) throw new Error(message)
  return value as Record<string, unknown>
}

/** Read a required string field. */
function requireString(source: Record<string, unknown>, field: string, message: string): string {
  const value = source[field]
  if (typeof value !== 'string' || value.length === 0) throw new Error(message)
  return value
}

/**
 * Parse and validate a pin sidecar.
 *
 * @param text The sidecar text.
 * @returns The pins it carries.
 */
export function parseToolPins(text: string): ToolPins {
  const parsed: unknown = JSON.parse(text)
  const root = asObject(parsed, 'acp-snapshot: tool pins must be an object')
  if (root.version !== 2) throw new Error('acp-snapshot: tool pins must declare version 2')
  if (typeof root.class !== 'string' || root.class.length === 0) {
    throw new Error('acp-snapshot: tool pins must name the generated-catalog class they belong to')
  }
  const capability = root.capability
  if (!isSnapshotCapability(capability)) {
    throw new Error('acp-snapshot: tool pins must declare capability "unconfined", "escalation", or "interactive"')
  }
  if (!Array.isArray(root.sets) || root.sets.length === 0) {
    throw new Error('acp-snapshot: tool pins must carry a non-empty array-valued sets field')
  }
  const sets = root.sets.map((entry, index): ToolPin => {
    const set = asObject(entry, `acp-snapshot: tool pin ${index} must be an object`)
    const tools = set.tools
    if (!Array.isArray(tools) || tools.length === 0 || !tools.every(name => typeof name === 'string')) {
      throw new Error(`acp-snapshot: tool pin ${index} must carry a non-empty array-valued tools field`)
    }
    const digest = requireString(set, 'digest', `acp-snapshot: tool pin ${index} must carry a digest`)
    if (!digest.startsWith('sha256:')) throw new Error(`acp-snapshot: tool pin ${index} digest must be a sha256: digest`)
    const setCapability = set.capability === undefined ? capability : set.capability
    if (!isSnapshotCapability(setCapability)) {
      throw new Error(`acp-snapshot: tool pin ${index} capability must be unconfined, escalation, or interactive`)
    }
    return { tools: tools, digest, capability: setCapability }
  })
  return { version: 2, class: root.class, capability, sets }
}

/**
 * An empty catalog ready for harvesting.
 *
 * @returns A version-2 catalog with no schema bodies and no classes.
 */
export function emptyToolCatalog(): ToolCatalog {
  return { version: 2, schemas: {}, classes: {} }
}

/**
 * Merge one catalog into another. Duplicate digests must carry identical
 * bodies and duplicate classes must agree, so a concurrent harvest cannot hide
 * a divergence.
 *
 * @param base The catalog to merge into.
 * @param addition The catalog to merge in.
 * @returns The merged catalog.
 */
export function mergeToolCatalog(base: ToolCatalog, addition: ToolCatalog): ToolCatalog {
  const schemas: Record<string, CatalogSchema> = { ...base.schemas }
  for (const [digest, entry] of Object.entries(addition.schemas)) {
    const existing = schemas[digest]
    if (existing === undefined) {
      schemas[digest] = entry
      continue
    }
    if (canonicalJson(existing) !== canonicalJson(entry)) {
      throw new Error(`acp-snapshot: catalog digest ${digest} carries two different schema bodies`)
    }
  }
  const classes: Record<string, CatalogClass> = { ...base.classes }
  for (const [name, entry] of Object.entries(addition.classes)) {
    const existing = classes[name]
    if (existing === undefined) {
      classes[name] = entry
      continue
    }
    if (canonicalJson(existing) !== canonicalJson(entry)) {
      throw new Error(`acp-snapshot: catalog class "${name}" was harvested twice with different content`)
    }
  }
  return { version: 2, schemas, classes }
}

/**
 * Register one class's harvested header revisions in a catalog.
 *
 * @param catalog The catalog to extend.
 * @param className The header class the revisions belong to.
 * @param capability The capability the class declares.
 * @param sets Every emitted tool-schema set of that class, in header order.
 * @param setCapabilities Per-revision capabilities when one class's revisions
 *   differ, such as a delegated child pinned at `never`; omitted when every
 *   revision runs under `capability`.
 * @returns The extended catalog.
 */
export function registerCatalogClass(
  catalog: ToolCatalog,
  className: string,
  capability: SnapshotCapability,
  sets: readonly unknown[][],
  setCapabilities?: readonly SnapshotCapability[],
): ToolCatalog {
  const schemas: Record<string, CatalogSchema> = {}
  const revisions = sets.map((set, index): CatalogRevision => {
    const tools = set.map((schema) => {
      const name = toolSchemaName(schema)
      const digest = schemaDigest(schema)
      schemas[digest] = { name, schema: asObject(schema, `acp-snapshot: tool "${name}" must be a schema object`) }
      return { name, digest }
    })
    return {
      capability: setCapabilities?.[index] ?? capability,
      tools: tools.sort((left, right) => left.name < right.name ? -1 : left.name > right.name ? 1 : 0),
    }
  })
  return mergeToolCatalog(catalog, {
    version: 2,
    schemas,
    classes: { [className]: { capability, revisions } },
  })
}

/**
 * Render a catalog as canonical JSON: classes and tools sorted by name, schema
 * bodies reachable through their digests.
 *
 * @param catalog The catalog to render.
 * @returns Pretty-printed JSON ending in one newline.
 */
export function formatToolCatalog(catalog: ToolCatalog): string {
  const schemas = Object.fromEntries(
    Object.entries(catalog.schemas).sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0),
  )
  const classes = Object.fromEntries(
    Object.entries(catalog.classes)
      .sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0)
      .map(([name, entry]) => [name, {
        capability: entry.capability,
        revisions: entry.revisions.map(revision => ({
          capability: revision.capability,
          tools: [...revision.tools]
            .sort((left, right) => left.name < right.name ? -1 : left.name > right.name ? 1 : 0)
            .map(tool => ({ name: tool.name, digest: tool.digest })),
        })),
      }]),
  )
  return `${JSON.stringify({ version: 2, schemas, classes }, null, 2)}\n`
}

/**
 * Parse and validate a generated catalog.
 *
 * @param text The catalog text.
 * @returns The catalog it carries.
 */
export function parseToolCatalog(text: string): ToolCatalog {
  const parsed: unknown = JSON.parse(text)
  const root = asObject(parsed, 'acp-snapshot: tool catalog must be an object')
  if (root.version !== 2) throw new Error('acp-snapshot: tool catalog must declare version 2')
  const rawSchemas = asObject(root.schemas, 'acp-snapshot: tool catalog must carry an object-valued schemas field')
  const schemas: Record<string, CatalogSchema> = {}
  for (const [digest, entry] of Object.entries(rawSchemas)) {
    const record = asObject(entry, `acp-snapshot: catalog digest ${digest} must carry a schema entry`)
    const name = requireString(record, 'name', `acp-snapshot: catalog digest ${digest} must name its tool`)
    schemas[digest] = { name, schema: asObject(record.schema, `acp-snapshot: catalog digest ${digest} must carry its schema`) }
  }
  const rawClasses = asObject(root.classes, 'acp-snapshot: tool catalog must carry an object-valued classes field')
  const classes: Record<string, CatalogClass> = {}
  for (const [name, entry] of Object.entries(rawClasses)) {
    const record = asObject(entry, `acp-snapshot: catalog class "${name}" must be an object`)
    const capability = record.capability
    if (!isSnapshotCapability(capability)) {
      throw new Error(`acp-snapshot: catalog class "${name}" must declare a capability`)
    }
    if (!Array.isArray(record.revisions) || record.revisions.length === 0) {
      throw new Error(`acp-snapshot: catalog class "${name}" must carry a non-empty array-valued revisions field`)
    }
    const revisions = record.revisions.map((revision, revisionIndex): CatalogRevision => {
      const parsedRevision = asObject(
        revision,
        `acp-snapshot: catalog class "${name}" revision ${revisionIndex} must be an object`,
      )
      const label = `acp-snapshot: catalog class "${name}" revision ${revisionIndex}`
      if (!Array.isArray(parsedRevision.tools)) throw new Error(`${label} must carry an array-valued tools field`)
      const revisionCapability = parsedRevision.capability
      if (!isSnapshotCapability(revisionCapability)) {
        throw new Error(`${label} must declare a capability`)
      }
      return {
        capability: revisionCapability,
        tools: parsedRevision.tools.map((tool, toolIndex): { name: string; digest: string } => {
          const parsed = asObject(tool, `${label} tool ${toolIndex} must be an object`)
          return {
            name: requireString(parsed, 'name', `${label} tool ${toolIndex} must be named`),
            digest: requireString(parsed, 'digest', `${label} tool ${toolIndex} must carry a digest`),
          }
        }),
      }
    })
    classes[name] = { capability, revisions }
  }
  return { version: 2, schemas, classes }
}

/**
 * Read one emitted tool's model-facing name.
 *
 * @param schema The emitted tool schema.
 * @returns Its name.
 * @throws When the schema carries no name.
 */
export function toolSchemaName(schema: unknown): string {
  const record = asObject(schema, 'acp-snapshot: every emitted tool must be a schema object')
  return requireString(record, 'name', 'acp-snapshot: every emitted tool must carry a name')
}

/** The advertised parameter properties of one tool schema, or an empty record. */
function parameterProperties(schema: unknown): Record<string, unknown> {
  const record = asObject(schema, 'acp-snapshot: every emitted tool must be a schema object')
  const parameters = record.parameters
  if (parameters === null || typeof parameters !== 'object' || Array.isArray(parameters)) return {}
  const properties = (parameters as Record<string, unknown>).properties
  if (properties === null || typeof properties !== 'object' || Array.isArray(properties)) return {}
  return properties as Record<string, unknown>
}

/**
 * The leaf paths where two JSON values differ, in document order.
 *
 * @param expected The committed value.
 * @param actual The emitted value.
 * @param path The JSON path of the values being compared.
 * @param into The accumulator for reported paths.
 * @param limit The maximum number of paths to report.
 * @returns The differing paths.
 */
export function diffPaths(
  expected: unknown,
  actual: unknown,
  path: string,
  into: string[] = [],
  limit = 8,
): string[] {
  if (into.length >= limit) return into
  if (Array.isArray(expected) && Array.isArray(actual)) {
    if (expected.length !== actual.length) {
      into.push(`${path} (array length ${expected.length} → ${actual.length})`)
      if (into.length >= limit) return into
    }
    for (const index of Array.from({ length: Math.max(expected.length, actual.length) }, (_, i) => i)) {
      if (into.length >= limit) break
      diffPaths(expected[index], actual[index], `${path}[${index}]`, into, limit)
    }
    return into
  }
  const expectedObject = expected !== null && typeof expected === 'object' && !Array.isArray(expected)
  const actualObject = actual !== null && typeof actual === 'object' && !Array.isArray(actual)
  if (expectedObject && actualObject) {
    const expectedRecord = expected as Record<string, unknown>
    const actualRecord = actual as Record<string, unknown>
    const keys = [...new Set([...Object.keys(expectedRecord), ...Object.keys(actualRecord)])].sort()
    for (const key of keys) {
      if (into.length >= limit) break
      const memberPath = path.length === 0 ? key : `${path}.${key}`
      if (!(key in expectedRecord)) {
        into.push(`${memberPath} (unexpected member)`)
        continue
      }
      if (!(key in actualRecord)) {
        into.push(`${memberPath} (missing member)`)
        continue
      }
      diffPaths(expectedRecord[key], actualRecord[key], memberPath, into, limit)
    }
    return into
  }
  if (canonicalJson(expected) !== canonicalJson(actual)) {
    into.push(`${path.length === 0 ? '<root>' : path} (${describe(expected)} → ${describe(actual)})`)
  }
  return into
}

/** Render a leaf value for a diagnostic, truncating long text. */
function describe(value: unknown): string {
  const text = canonicalJson(value)
  return text.length > 220 ? `${text.slice(0, 217)}…` : text
}

/**
 * The generated-catalog key of one header class. A suite's own directory names
 * its namespace, so classes of different suites never collide.
 *
 * @param snapshotsDir The suite's `snapshots/` directory.
 * @param headerClass The class name the scenario declares.
 * @returns The catalog key.
 */
export function catalogClassName(snapshotsDir: string, headerClass: string): string {
  return `${basename(snapshotsDir)}/${headerClass}`
}

/**
 * Remove the `tools` field from a request header so the remaining fields can be
 * compared against a tokenized pin.
 *
 * @param header The request header.
 * @returns Every other request field.
 */
export function headerWithoutTools(header: unknown): Record<string, unknown> {
  const record = asObject(header, 'acp-snapshot: a pinned request header must be an object')
  return Object.fromEntries(Object.entries(record).filter(([field]) => field !== 'tools'))
}

/**
 * Assert one emitted request header against its class pin: every field except
 * `tools` must equal the tokenized pin, and the tools must match the pin's
 * names, digest, catalogued bodies, and capability.
 *
 * @param header The emitted request header.
 * @param pinned The class pin's header for this revision, with its `tools` token.
 * @param check The class's pins, catalog, and diagnostic label.
 * @param revision The zero-based header revision this request belongs to.
 * @returns Nothing.
 * @throws When any request field, tool name, schema, or capability diverges.
 */
export function assertPinnedHeader(
  header: unknown,
  pinned: unknown,
  check: ToolPinCheck,
  revision: number,
): void {
  const record = asObject(header, 'acp-snapshot: a pinned request header must be an object')
  const { tools } = record
  if (!Array.isArray(tools)) {
    throw new Error(`${check.label}: the request header must carry an array-valued tools field`)
  }
  const expected = headerWithoutTools(pinned)
  const actual = headerWithoutTools(record)
  if (canonicalJson(expected) !== canonicalJson(actual)) {
    const paths = diffPaths(expected, actual, '')
    throw new Error(`${check.label}: request header diverged from the pin at ${paths.join(', ')}`)
  }
  assertToolPin(tools, check, revision)
}

/** One header class's check inputs. */
export interface ToolPinCheck {
  /** Prefix for every diagnostic, naming the session and request header. */
  label: string
  /** The header class being compared. */
  className: string
  /** The class's committed pins. */
  pins: ToolPins
  /** The generated catalog the pins point at. */
  catalog: ToolCatalog
}

/**
 * Assert the emitted tool schemas against a class's pins and the generated
 * catalog: names and digests come from the pin, schema bodies from the catalog,
 * and the capability from the class declaration. Every failure names the exact
 * tool and JSON path, so a schema change is a readable diff rather than a
 * re-record.
 *
 * @param sets The emitted tool-schema sets of one session, in request order.
 * @param check The class's pins, catalog, and diagnostic label.
 * @returns Nothing.
 * @throws When the emitted names, digests, bodies, or capability diverge.
 */
export function assertToolPins(sets: readonly unknown[][], check: ToolPinCheck): void {
  const { label, className, pins, catalog } = check
  if (sets.length !== pins.sets.length) {
    throw new Error(`${label}: emitted ${sets.length} tool-schema set(s) but class "${className}" pins ${pins.sets.length}`)
  }
  const classEntry = catalog.classes[className]
  if (classEntry === undefined) {
    throw new Error(`${label}: the generated catalog has no entry for class "${className}"`)
  }
  if (classEntry.capability !== pins.capability) {
    throw new Error(
      `${label}: class "${className}" pins capability "${pins.capability}" but the catalog recorded "${classEntry.capability}"`,
    )
  }
  // Exact, in both directions: a catalog with extra revisions names a header
  // this class no longer emits, and one with fewer is missing a revision the
  // pin still claims.
  if (classEntry.revisions.length !== pins.sets.length) {
    throw new Error(
      `${label}: class "${className}" pins ${pins.sets.length} header revision(s) but the catalog carries `
      + `${classEntry.revisions.length}; regenerate docs/tool-catalog.json`,
    )
  }
  assertCatalogBodies(className, catalog, label)
  for (const [index, set] of sets.entries()) {
    const revisionLabel = `${label} (class "${className}" revision ${index + 1})`
    const revision = catalogRevision(classEntry, className, index, revisionLabel)
    const pin = pins.sets[index] as ToolPin
    assertRevisionCapability(revision, pin, className, index, revisionLabel)
    assertToolNames(set, pin, revisionLabel)
    const digest = toolSetDigest(set)
    if (digest !== pin.digest) {
      throw new Error(`${revisionLabel}: tool schemas diverged from the pin\n${describeSchemaDivergence(set, revision, catalog, index)}`)
    }
    assertCatalogCarries(set, revision, className, index, revisionLabel)
    assertCapabilityAdvertisement(set, pin.capability, revisionLabel)
  }
}

/**
 * Assert one emitted request header's tool set against its pin and the
 * generated catalog. This is the per-header form of {@link assertToolPins};
 * {@link ToolPinCheck} carries the class's complete pin sequence and this
 * function checks the revision the header belongs to.
 *
 * @param schemas The emitted tool schemas of one request header, in order.
 * @param check The class's pins, catalog, and diagnostic label.
 * @param revision The zero-based header revision this request belongs to.
 * @returns Nothing.
 * @throws When the names, digest, bodies, or capability diverge.
 */
export function assertToolPin(schemas: readonly unknown[], check: ToolPinCheck, revision: number): void {
  const pin = check.pins.sets[revision]
  if (pin === undefined) {
    throw new Error(`${check.label}: class "${check.className}" pins no tool set for revision ${revision + 1}`)
  }
  const classEntry = check.catalog.classes[check.className]
  if (classEntry === undefined) {
    throw new Error(`${check.label}: the generated catalog has no entry for class "${check.className}"`)
  }
  if (classEntry.capability !== check.pins.capability) {
    throw new Error(
      `${check.label}: class "${check.className}" pins capability "${check.pins.capability}" but the catalog recorded "${classEntry.capability}"`,
    )
  }
  if (classEntry.revisions.length !== check.pins.sets.length) {
    throw new Error(
      `${check.label}: class "${check.className}" pins ${check.pins.sets.length} header revision(s) but the catalog carries `
      + `${classEntry.revisions.length}; regenerate docs/tool-catalog.json`,
    )
  }
  assertCatalogBodies(check.className, check.catalog, check.label)
  const revisionEntry = catalogRevision(classEntry, check.className, revision, check.label)
  assertRevisionCapability(revisionEntry, pin, check.className, revision, check.label)
  assertToolNames(schemas, pin, check.label)
  if (toolSetDigest(schemas) !== pin.digest) {
    throw new Error(
      `${check.label}: tool schemas diverged from the pin\n${describeSchemaDivergence(schemas, revisionEntry, check.catalog, revision)}`,
    )
  }
  assertCatalogCarries(schemas, revisionEntry, check.className, revision, check.label)
  assertCapabilityAdvertisement(schemas, pin.capability, check.label)
}

/**
 * Assert every body one catalogued class references still hashes to its digest,
 * so a hand-edited catalog fails the replay instead of silently describing a
 * different tool surface.
 *
 * @param className The catalogued class to verify.
 * @param catalog The generated catalog.
 * @param label The failing check's label.
 * @returns Nothing.
 * @throws When a referenced body is absent or does not hash to its digest.
 */
function assertCatalogBodies(className: string, catalog: ToolCatalog, label: string): void {
  const entry = catalog.classes[className]
  if (entry === undefined) return
  for (const revision of entry.revisions) {
    for (const tool of revision.tools) {
      const schema = catalog.schemas[tool.digest]
      if (schema === undefined) {
        throw new Error(`${label}: the generated catalog has no body for ${tool.name} (${tool.digest})`)
      }
      if (schemaDigest(schema.schema) !== tool.digest) {
        throw new Error(
          `${label}: catalogued body for ${tool.name} does not hash to ${tool.digest};`
          + ' regenerate docs/tool-catalog.json',
        )
      }
    }
  }
}

/**
 * Assert the capability one header revision runs under, which the catalog and
 * the pin must agree on. A class whose revisions legitimately differ, such as a
 * delegated child with its own `never` policy, states it per revision.
 *
 * @param revision The catalogued capability and tools of that revision.
 * @param pin The pinned tool set of that revision.
 * @param className The class's catalog key, for the diagnostic.
 * @param index The zero-based header revision, for the diagnostic.
 * @param label Prefix for the diagnostic.
 * @returns Nothing.
 * @throws When the pin and the catalog record different capabilities.
 */
function assertRevisionCapability(
  revision: CatalogRevision,
  pin: ToolPin,
  className: string,
  index: number,
  label: string,
): void {
  if (revision.capability === pin.capability) return
  throw new Error(
    `${label}: class "${className}" revision ${index + 1} pins capability "${pin.capability}" but the catalog `
    + `recorded "${revision.capability}"`,
  )
}

/**
 * The catalogued tool surface of one header revision.
 *
 * @param entry The catalogued class.
 * @param className The class's catalog key, for the diagnostic.
 * @param revision The zero-based header revision.
 * @param label Prefix for the diagnostic.
 * @returns That revision's catalogued tools.
 * @throws When the catalog records no such revision.
 */
function catalogRevision(
  entry: CatalogClass,
  className: string,
  revision: number,
  label: string,
): CatalogRevision {
  const found = entry.revisions[revision]
  if (found !== undefined) return found
  throw new Error(
    `${label}: class "${className}" is catalogued with ${entry.revisions.length} header revision(s) but this `
    + `header is revision ${revision + 1}; regenerate docs/tool-catalog.json`,
  )
}

/**
 * Assert the catalog carries exactly the tool digests this header emitted. The
 * digest comparison against the pin proves the emitted surface has not moved;
 * this proves the catalog the pins point at still describes that surface, so a
 * bumped pin without a regenerated catalog fails here instead of passing.
 *
 * @param set The emitted tool schemas of one header, in order.
 * @param revision The catalogued tool surface of that header's revision.
 * @param className The class's catalog key, for the diagnostic.
 * @param index The zero-based header revision, for the diagnostic.
 * @param label Prefix for the diagnostic.
 * @returns Nothing.
 * @throws When the catalog is missing an emitted tool or carries another digest for it.
 */
function assertCatalogCarries(
  set: readonly unknown[],
  revision: CatalogRevision,
  className: string,
  index: number,
  label: string,
): void {
  const catalogued = new Map(revision.tools.map(tool => [tool.name, tool.digest]))
  const emitted = new Set(set.map(toolSchemaName))
  const extra = revision.tools.flatMap(tool => emitted.has(tool.name)
    ? []
    : [`  ${tool.name}: catalogued for revision ${index + 1} (${tool.digest}) but the header no longer emits it`])
  const stale = set.flatMap((schema) => {
    const name = toolSchemaName(schema)
    const digest = schemaDigest(schema)
    const known = catalogued.get(name)
    if (known === digest) return []
    return [
      known === undefined
        ? `  ${name}: not catalogued for revision ${index + 1} (${digest})`
        : `  ${name}: the catalog carries ${known} but the header emits ${digest}`,
    ]
  })
  if (stale.length === 0 && extra.length === 0) return
  throw new Error(
    `${label}: class "${className}" revision ${index + 1} disagrees with docs/tool-catalog.json\n`
    + `${[...stale, ...extra].join('\n')}\n  regenerate docs/tool-catalog.json`,
  )
}

/** Assert the ordered tool names of one emitted set against its pin. */
function assertToolNames(set: readonly unknown[], pin: ToolPin, label: string): void {
  const names = set.map(toolSchemaName)
  if (names.length === pin.tools.length && names.every((name, index) => name === pin.tools[index])) return
  const emitted = new Set(names)
  const pinned = new Set(pin.tools)
  const missing = pin.tools.filter(name => !emitted.has(name))
  const unexpected = names.filter(name => !pinned.has(name))
  const moved = pin.tools
    .filter(name => emitted.has(name))
    .flatMap((name) => {
      const pinIndex = pin.tools.indexOf(name)
      const emittedIndex = names.indexOf(name)
      return pinIndex === emittedIndex ? [] : [`${name} (pinned #${pinIndex + 1}, emitted #${emittedIndex + 1})`]
    })
  throw new Error(
    `${label}: emitted tool names diverged from the pin (${pin.tools.length} pinned, ${names.length} emitted)\n`
    + [
      missing.length === 0 ? '' : `  missing: ${missing.join(', ')}`,
      unexpected.length === 0 ? '' : `  unexpected: ${unexpected.join(', ')}`,
      moved.length === 0 ? '' : `  reordered: ${moved.join(', ')}`,
    ].filter(line => line.length > 0).join('\n'),
  )
}

/**
 * Describe how each emitted tool diverges from the catalogued body of its
 * revision, as exact JSON paths. The emitted schema is compared directly
 * because the catalog only ever holds the committed digest, so looking the
 * emitted digest up there would find nothing and report no paths.
 */
function describeSchemaDivergence(
  set: readonly unknown[],
  revision: CatalogRevision,
  catalog: ToolCatalog,
  index: number,
): string {
  const catalogued = new Map(revision.tools.map(tool => [tool.name, tool.digest]))
  const lines: string[] = []
  for (const schema of set) {
    const name = toolSchemaName(schema)
    const committed = catalogued.get(name)
    if (committed === undefined) {
      lines.push(`  ${name}: not catalogued for revision ${index + 1}; regenerate docs/tool-catalog.json`)
      continue
    }
    if (committed === schemaDigest(schema)) continue
    const body = catalog.schemas[committed]
    if (body === undefined) {
      lines.push(`  ${name}: the catalog lost the committed schema ${committed}`)
      continue
    }
    const paths = diffPaths(body.schema, schema, name)
    lines.push(...(paths.length === 0
      ? [`  ${name}: the committed body is unchanged, so only its catalogued digest is stale (${committed})`]
      : paths.map(path => `  ${path}`)))
  }
  return lines.length === 0
    ? '  no catalogued tool diverged; the pin digest covers the whole set, so the pinned digest itself is stale'
    : lines.join('\n')
}

/**
 * Assert the escalation advertisement a capability implies: `unconfined` and
 * `interactive` omit `sandbox_permissions`/`justification` everywhere, and
 * `escalation` advertises the pair with the closed target vocabulary from at
 * least one mutating family.
 */
function assertCapabilityAdvertisement(set: readonly unknown[], capability: SnapshotCapability, label: string): void {
  let advertised = 0
  for (const schema of set) {
    const name = toolSchemaName(schema)
    const properties = parameterProperties(schema)
    const permissions = properties.sandbox_permissions
    const justification = properties.justification
    if (permissions === undefined && justification === undefined) continue
    advertised++
    if (permissions === undefined || justification === undefined) {
      throw new Error(`${label}: tool "${name}" advertises only one of ${ESCALATION_PARAMETERS.join('/')}`)
    }
    if (capability !== 'escalation') {
      throw new Error(
        `${label}: class capability is "${capability}" but tool "${name}" advertises ${ESCALATION_PARAMETERS.join('/')}; `
        + `force DSH_PERMISSION_MODE=${CAPABILITY_PERMISSION_MODE[capability]} or declare capability "escalation"`,
      )
    }
    const enumValues = asObject(permissions, `acp-snapshot: tool "${name}" sandbox_permissions must be an object`).enum
    if (!Array.isArray(enumValues) || enumValues.length !== ESCALATION_MODES.length
      || enumValues.some((mode, index) => mode !== ESCALATION_MODES[index])) {
      throw new Error(
        `${label}: tool "${name}" advertises sandbox_permissions enum ${canonicalJson(enumValues)}; `
        + `expected ${canonicalJson(ESCALATION_MODES)}`,
      )
    }
  }
  if (capability === 'escalation' && advertised === 0) {
    throw new Error(
      `${label}: class capability is "escalation" but no tool advertises ${ESCALATION_PARAMETERS.join('/')}; `
      + 'a confining backend with interactive approvals must advertise them, '
      + 'or the class is "interactive"',
    )
  }
}

/**
 * Assert the approval policy the composed session recorded for a capability,
 * which is what makes the declaration independent of the host: the scenario
 * forces `DSH_PERMISSION_MODE`, so a divergent policy means the composition
 * ignored the declaration.
 *
 * @param capability The class's declared capability.
 * @param policy The approval policy recorded in the session log, if any.
 * @param label Prefix for the diagnostic.
 * @returns Nothing.
 * @throws When the recorded policy contradicts the capability.
 */
export function assertCapabilityApproval(
  capability: SnapshotCapability,
  policy: string | undefined,
  label: string,
): void {
  const expected = CAPABILITY_APPROVAL[capability]
  // An unconfined composition may mount no approval service at all; both that
  // and an explicit `never` policy satisfy the capability.
  if (policy === expected || (capability === 'unconfined' && policy === undefined)) return
  const recorded = policy === undefined ? 'no approval policy' : `approval policy "${policy}"`
  throw new Error(
    `${label}: class capability "${capability}" requires approval policy "${expected}" but the session recorded `
    + `${recorded}; the composition ignored the forced DSH_PERMISSION_MODE`,
  )
}

/** Where one header class's pins and catalog entry live. */
export interface HeaderPinLocation {
  /** The suite's `snapshots/` directory, which namespaces the catalog key. */
  snapshotsDir: string
  /** Directory that owns this class's pin file. */
  ownerDir: string
  /** The header class the scenario declares. */
  headerClass: string
}

/**
 * The generated catalog key of a header class, namespaced by its suite directory.
 *
 * @param location The suite's snapshots directory and the class the scenario declares.
 * @returns The catalog key the class is recorded under.
 */
export function catalogClassOf(location: HeaderPinLocation): string {
  return catalogClassName(location.snapshotsDir, location.headerClass)
}

/**
 * Write one class's header pin and its catalog fragment. The pin records names
 * and digests only; the fragment carries the schema bodies the generator merges
 * into the committed catalog, so a device that cannot run the merge still
 * leaves reviewable evidence.
 *
 * @param location Where the pin and catalog fragment belong.
 * @param capability The class's declared capability.
 * @param schemaSets Every emitted tool-schema set of the class, in header order.
 * @param catalogPartsDir Directory for per-class catalog fragments, or
 *   `undefined` to write only the pin. The catalog generator sets it, so an
 *   ordinary replay never stages fragments.
 * @param setCapabilities Per-revision capabilities when one class's revisions
 *   legitimately differ, such as a delegated child pinned at `never`; omitted
 *   when every revision runs under `capability`.
 * @returns The pins and their canonical text.
 */
export async function writeHeaderPin(
  location: HeaderPinLocation,
  capability: SnapshotCapability,
  schemaSets: readonly unknown[][],
  catalogPartsDir: string | undefined,
  setCapabilities?: readonly SnapshotCapability[],
): Promise<{ pins: ToolPins; text: string }> {
  if (schemaSets.length === 0) throw new Error(`acp-snapshot: ${location.headerClass} has no tool-schema set to pin`)
  const pins: ToolPins = {
    version: 2,
    class: catalogClassOf(location),
    capability,
    sets: schemaSets.map((set, index) => ({
      tools: set.map(toolSchemaName),
      digest: toolSetDigest(set),
      capability: setCapabilities?.[index] ?? capability,
    })),
  }
  const text = formatToolPins(pins)
  await writeFile(join(location.ownerDir, 'tool-pins.expected.json'), text)
  const key = catalogClassOf(location)
  if (catalogPartsDir === undefined) return { pins, text }
  const catalog = registerCatalogClass(emptyToolCatalog(), key, capability, schemaSets, setCapabilities)
  await mkdir(catalogPartsDir, { recursive: true })
  await writeFile(
    join(catalogPartsDir, `${key.replace(/[^A-Za-z0-9]+/g, '-')}.json`),
    formatToolCatalog(catalog),
  )
  return { pins, text }
}

/**
 * Read one class's committed pin and the generated catalog it points at.
 *
 * @param ownerDir Directory that owns the class's pin file.
 * @param catalogPath The generated catalog's path.
 * @returns The pins and the catalog.
 */
export async function readHeaderPin(
  ownerDir: string,
  catalogPath: string,
): Promise<{ pins: ToolPins; text: string; catalog: ToolCatalog }> {
  const text = await readFile(join(ownerDir, 'tool-pins.expected.json'), 'utf8')
  let catalogText: string
  try {
    catalogText = await readFile(catalogPath, 'utf8')
  } catch (error) {
    throw new Error(`acp-snapshot: cannot read the generated tool catalog ${catalogPath}: ${String(error)}`)
  }
  return { pins: parseToolPins(text), text, catalog: parseToolCatalog(catalogText) }
}
