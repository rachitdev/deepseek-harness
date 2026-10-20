/**
 * Real-TypeSafe scale benchmark for compaction scoring. A transcript large enough to exceed the
 * System One request ceiling is the case compaction exists to serve, so this measures that the
 * batched scorer answers it at all, and that its retention policy still separates user constraints
 * and failing tests from boilerplate tool output at that size.
 */

import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime from '@deepseek-ai/dsh-decision'
import * as jevPlugin from '@deepseek-ai/dsh-decision-jev'
import { scoreCompactionItems } from '../src/compaction-scorer.ts'
import type { CompactionTranscriptItem } from '../src/types.ts'

const GOAL = 'refactor the dispatch layer without breaking the public API'

/**
 * Build a labeled transcript: every tenth item is a standing user constraint, every seventh a
 * failing test, and the rest boilerplate file reads.
 * @param count - number of items to generate.
 * @returns the transcript paired with the expected class of each id.
 */
function buildTranscript(count: number): {
  items: CompactionTranscriptItem[]
  constraints: Set<string>
  noise: Set<string>
} {
  const items: CompactionTranscriptItem[] = []
  const constraints = new Set<string>()
  const noise = new Set<string>()
  for (let i = 0; i < count; i += 1) {
    const id = `t${String(i)}`
    if (i % 10 === 0) {
      constraints.add(id)
      items.push({
        id,
        role: 'user',
        summary: 'USER CONSTRAINT: never edit files under src/generated/, and the public API must stay backward compatible.',
        bytes: 300,
      })
    } else if (i % 7 === 0) {
      items.push({
        id,
        role: 'tool',
        summary: `Test run ${String(i)} FAILED: TypeError at dispatch.ts:88 cannot read property 'handler' of undefined`,
        bytes: 900,
      })
    } else {
      noise.add(id)
      items.push({
        id,
        role: 'tool',
        summary: `Read src/module_${String(i)}.ts -> boilerplate imports and a class skeleton, no logic relevant to dispatch.`,
        bytes: 4200,
      })
    }
  }
  return { items, constraints, noise }
}

describe.skipIf(!process.env['TYPESAFE_API_KEY'])('JEV compaction scale benchmark', () => {
  it('scores a transcript larger than one request and keeps constraints while dropping boilerplate', async () => {
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    jevPlugin.apply(ctx, {})

    const { items, constraints, noise } = buildTranscript(400)
    const started = performance.now()
    const plan = await scoreCompactionItems(ctx.decision, items, GOAL)
    const elapsedMs = Math.round(performance.now() - started)

    const planned = new Set([...plan.keepIds, ...plan.dropIds, ...plan.summarizeIds])
    expect(planned.size).toBe(items.length)

    const retained = new Set([...plan.keepIds, ...plan.summarizeIds])
    const constraintsRetained = [...constraints].filter(id => retained.has(id)).length
    const noiseDropped = [...noise].filter(id => plan.dropIds.includes(id)).length

    console.log(
      JSON.stringify(
        {
          items: items.length,
          elapsedMs,
          keep: plan.keepIds.length,
          summarize: plan.summarizeIds.length,
          drop: plan.dropIds.length,
          constraintsRetained: `${String(constraintsRetained)}/${String(constraints.size)}`,
          noiseDropped: `${String(noiseDropped)}/${String(noise.size)}`,
          prunedBytes: plan.prunedBytes,
          retainedBytes: plan.retainedBytes,
        },
        null,
        2,
      ),
    )

    // A transcript this size must not lose standing user direction, and must actually reclaim
    // budget from boilerplate; otherwise compaction has not earned its decision call.
    expect(constraintsRetained).toBe(constraints.size)
    expect(noiseDropped / noise.size).toBeGreaterThanOrEqual(0.8)
    expect(plan.prunedBytes).toBeGreaterThan(plan.retainedBytes)
  }, 120_000)
})
