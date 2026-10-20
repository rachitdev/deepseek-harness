/**
 * Real-TypeSafe benchmark for the dynamic browser action policy. Each case is an observed element
 * table with one correct next operation and target, so the measurement is routing quality against
 * labeled ground truth rather than confidence read back from the provider.
 */

import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime from '@deepseek-ai/dsh-decision'
import * as jevPlugin from '@deepseek-ai/dsh-decision-jev'
import { decideBrowserAction } from '../src/browser-action-policy.ts'
import type { BrowserActionPolicyRequest, BrowserOperation } from '../src/types.ts'

interface Case {
  readonly id: string
  readonly request: BrowserActionPolicyRequest
  readonly operation: BrowserOperation
  readonly targetId?: string
}

const CASES: readonly Case[] = [
  {
    id: 'type-into-empty-destination',
    request: {
      goal: 'Find one-way flights from Zurich to London on 20 September 2026',
      pageUrl: 'https://flights.test/',
      pageTitle: 'Flights',
      pageText: 'Book your trip',
      controls: [
        { id: '1', role: 'combobox', label: 'Where from?', value: 'Zurich', operations: ['CLICK', 'TYPE_TEXT'] },
        { id: '2', role: 'combobox', label: 'Where to?', value: '', operations: ['CLICK', 'TYPE_TEXT'] },
        { id: '3', role: 'button', label: 'Search', operations: ['CLICK'] },
      ],
      history: ['TYPE_TEXT into 1 -> Zurich'],
    },
    operation: 'TYPE_TEXT',
    targetId: '2',
  },
  {
    id: 'submit-when-fields-ready',
    request: {
      goal: 'Find one-way flights from Zurich to London on 20 September 2026',
      pageUrl: 'https://flights.test/',
      pageTitle: 'Flights',
      pageText: 'Book your trip',
      controls: [
        { id: '1', role: 'combobox', label: 'Where from?', value: 'Zurich', operations: ['CLICK', 'TYPE_TEXT'] },
        { id: '2', role: 'combobox', label: 'Where to?', value: 'London', operations: ['CLICK', 'TYPE_TEXT'] },
        { id: '3', role: 'combobox', label: 'Departure', value: '20 Sep 2026', operations: ['CLICK', 'TYPE_TEXT'] },
        { id: '4', role: 'button', label: 'Search', operations: ['CLICK'] },
      ],
      history: ['TYPE_TEXT into 2 -> London', 'CLICK 3 -> date chosen'],
    },
    operation: 'CLICK',
    targetId: '4',
  },
  {
    id: 'click-matching-article-link',
    request: {
      goal: "Open the Wikipedia article about Godel's incompleteness theorems",
      pageUrl: 'https://en.wikipedia.test/wiki/Kurt_Godel',
      pageTitle: 'Kurt Godel',
      pageText: 'Austrian logician known for his incompleteness theorems.',
      controls: [
        { id: '1', role: 'link', label: 'Mathematical logic', operations: ['CLICK'] },
        { id: '2', role: 'link', label: "Godel's incompleteness theorems", operations: ['CLICK'] },
        { id: '3', role: 'link', label: 'Vienna Circle', operations: ['CLICK'] },
      ],
      history: [],
    },
    operation: 'CLICK',
    targetId: '2',
  },
  {
    id: 'done-when-goal-visibly-met',
    request: {
      goal: "Open the Wikipedia article about Godel's incompleteness theorems",
      pageUrl: "https://en.wikipedia.test/wiki/Godel's_incompleteness_theorems",
      pageTitle: "Godel's incompleteness theorems",
      pageText: "Godel's incompleteness theorems are two theorems of mathematical logic.",
      controls: [
        { id: '1', role: 'link', label: 'Mathematical logic', operations: ['CLICK'] },
        { id: '2', role: 'link', label: 'Peano arithmetic', operations: ['CLICK'] },
      ],
      history: ['CLICK 2 -> navigated'],
    },
    operation: 'DONE',
  },
]

describe.skipIf(!process.env['TYPESAFE_API_KEY'])('JEV browser action policy benchmark', () => {
  it('chooses the labeled operation and target in one request per step', async () => {
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    jevPlugin.apply(ctx, {})

    const rows: Array<Record<string, unknown>> = []
    let correct = 0
    for (const sample of CASES) {
      const decision = await decideBrowserAction(ctx.decision, sample.request)
      const operationOk = decision.operation === sample.operation
      const targetOk = sample.targetId === undefined || decision.targetId === sample.targetId
      if (operationOk && targetOk) correct += 1
      rows.push({
        case: sample.id,
        got: `${decision.operation}${decision.targetId === undefined ? '' : ` [${decision.targetId}]`}`,
        want: `${sample.operation}${sample.targetId === undefined ? '' : ` [${sample.targetId}]`}`,
        confidence: Number(decision.confidence.toFixed(3)),
        executable: decision.executable,
        latencyMs: decision.latencyMs,
      })
    }

    const accuracy = correct / CASES.length
    console.log(JSON.stringify({ accuracy, rows }, null, 2))

    // The policy exists to replace click-only stepping; a wrong operation executes the wrong action
    // on a real page, so every labeled step must route correctly.
    expect(accuracy).toBe(1)
  }, 120_000)
})
