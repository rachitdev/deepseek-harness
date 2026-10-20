/**
 * Real-TypeSafe e2e for the Jev decision provider. Self-skips without `$TYPESAFE_API_KEY`, so a
 * keyless checkout still passes. It exists because a unit test with a stubbed `fetch` cannot catch
 * the direct System One endpoint, bearer authentication, model placement, or `noul` vocabulary
 * drifting.
 */

import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, { type DecisionRequest } from '@deepseek-ai/dsh-decision'
import * as jevPlugin from '@deepseek-ai/dsh-decision-jev'

/** A browser-step decision: pick the next element, rate the risk, check for completion. */
const request: DecisionRequest = {
  state: [
    'Page: search results for "usb cable".',
    'Elements: 1 USB-C Cable 2m link, 2 Adapter link, 3 Sort button.',
    'Task: open the 2m USB-C cable page.',
  ].join(' '),
  questions: {
    element: {
      type: 'choice',
      instructions: 'Which element should be clicked next?',
      criteria: { '1': 'USB-C Cable 2m link', '2': 'Adapter link', '3': 'Sort button' },
    },
    risk: {
      type: 'score',
      instructions: 'How risky is this action?',
      criteria: ['safe', 'caution', 'destructive'],
    },
    done: { type: 'boolean', instructions: 'Is the task already complete?' },
  },
}

describe.skipIf(!process.env['TYPESAFE_API_KEY'])('decision-jev e2e (direct TypeSafe)', () => {
  it('answers every question arm in one round trip', async () => {
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    jevPlugin.apply(ctx, {})

    const result = await ctx.decision.evaluate(request)

    expect(result.providerId).toBe('jev')
    // The decisive option: a model that cannot read the page state would not concentrate here.
    expect(result.answers['element']).toMatchObject({ type: 'choice', choice: '1' })
    const risk = result.answers['risk']
    expect(risk).toMatchObject({ type: 'score' })
    if (risk?.type !== 'score') throw new Error('risk answer lost its arm')
    expect(risk.score).toBeGreaterThanOrEqual(0)
    expect(risk.score).toBeLessThanOrEqual(2)
    const done = result.answers['done']
    expect(done).toMatchObject({ type: 'boolean' })
    if (done?.type !== 'boolean') throw new Error('done answer lost its arm')
    expect(done.probability).toBeLessThan(0.5)
  }, 60_000)
})
