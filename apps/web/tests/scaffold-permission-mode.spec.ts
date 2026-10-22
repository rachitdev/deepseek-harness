import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { bootPermissionMode } from './scaffold.ts'

const MANIFEST = '/case/snapshot.yml'
/** Prefix every synthetic manifest with the header the parser requires. */
const manifest = (body: string): string => `version: 1\nprofile: web\n${body.replace(/^scenario: [^\n]*\n/u, '')}`
const FIXTURE = 'session.v4.jsonl'

/** One recorded event line, in the physical framing the replay parser reads. */
function event(type: string, data: Record<string, unknown>, _seq = 0): string {
  return `${JSON.stringify({ type, data })}\n`
}

// A committed fixture supplies the header, so the rows below are appended to a header the replay
// parser accepts instead of one this spec guesses at.
const HEADER = readFileSync(fileURLToPath(
  new URL('../../../snapshots/web/minimal-preset/session.v4.jsonl', import.meta.url)), 'utf8').split('\n')[0]

function recording(...events: string[]): string {
  return `${HEADER}\n${events.join('')}`
}

describe('boot permission mode', () => {
  it('boots a case that declares nothing under the composition default', () => {
    expect(bootPermissionMode(manifest('scenario: plain\n'), MANIFEST, FIXTURE, recording())).toBeUndefined()
  })

  it('refuses an undeclared case whose recording chose a preset, a mode, or a policy', () => {
    for (const chosen of [
      event('permission/preset', { preset: 'workspace-write' }, 1),
      event('sandbox/mode', { mode: 'danger-full-access' }, 1),
      event('approval/policy', { policy: 'never' }, 1),
    ]) {
      expect(() => bootPermissionMode(manifest('scenario: borrowed\n'), MANIFEST, FIXTURE, recording(chosen)))
        .toThrow(/declare the case's/u)
    }
  })

  it('returns the mode a capability-only manifest resolves, not the ambient one', () => {
    for (const [capability, mode] of [
      ['escalation', 'workspace-write'],
      ['interactive', 'workspace-write'],
      ['unconfined', 'danger-full-access'],
    ] as const) {
      expect(bootPermissionMode(manifest(`scenario: c\ncapability: ${capability}\n`), MANIFEST, FIXTURE, recording())).toBe(mode)
    }
  })

  it('returns the declared sandbox even when it narrows what the capability implies', () => {
    const declared = manifest('scenario: c\ncapability: escalation\nsandbox: read-only\n')
    expect(bootPermissionMode(declared, MANIFEST, FIXTURE, recording())).toBe('read-only')
  })

  it('refuses a recording whose boot preset, mode, or approval contradicts the declaration', () => {
    const declared = manifest('scenario: c\npermission: workspace-write\ncapability: escalation\nsandbox: workspace-write\n')
    expect(() => bootPermissionMode(declared, MANIFEST, FIXTURE,
      recording(event('permission/preset', { preset: 'read-only' }, 1)))).toThrow(/declares permission/u)
    expect(() => bootPermissionMode(declared, MANIFEST, FIXTURE,
      recording(event('sandbox/mode', { mode: 'danger-full-access' }, 1)))).toThrow(/declares sandbox/u)
    expect(() => bootPermissionMode(declared, MANIFEST, FIXTURE,
      recording(event('approval/policy', { policy: 'never' }, 1)))).toThrow(/boots with approval/u)
  })

  it('judges a takeover recording by the state it boots in and ignores the switches that follow', () => {
    const declared = manifest('scenario: takeover\ncapability: escalation\nsandbox: workspace-write\n')
    const takeover = recording(
      event('permission/preset', { preset: 'workspace-write' }, 1),
      event('sandbox/mode', { mode: 'workspace-write' }, 2),
      event('approval/policy', { policy: 'ask' }, 3),
      event('approval/policy', { policy: 'never' }, 4),
      event('permission/preset', { preset: 'read-only' }, 5),
      event('sandbox/mode', { mode: 'read-only' }, 6),
    )
    expect(bootPermissionMode(declared, MANIFEST, FIXTURE, takeover)).toBe('workspace-write')
  })
})
