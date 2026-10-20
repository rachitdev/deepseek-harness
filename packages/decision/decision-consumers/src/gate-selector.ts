/**
 * `GateSelector`: triages git diffs and changed paths using `ctx.decision` to select the minimal
 * sufficient verification gates before pushing. Avoids running heavy e2e or full coverage suites
 * on pure markdown or documentation edits, while ensuring high-risk seam changes receive thorough
 * verification.
 * @module @deepseek-ai/dsh-decision-consumers/gate-selector
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { GateSelection } from './types.ts'

/** Pre-configured command suites for repo verification tiers. */
export const SUITE_COMMANDS: Readonly<Record<GateSelection['recommendedSuite'], readonly string[]>> = {
  doc_only: [
    'pnpm run verify-md-links',
    'pnpm run verify-doc-budgets',
  ],
  lint_and_unit: [
    'npx tsc -b tsconfig.host.json',
    'pnpm run lint',
    'pnpm run test',
  ],
  full_coverage: [
    'npx tsc -b tsconfig.host.json',
    'pnpm run lint',
    'pnpm run test:coverage',
    'pnpm run hygiene',
  ],
  full_with_e2e: [
    'npx tsc -b tsconfig.host.json',
    'pnpm run lint',
    'pnpm run test:coverage',
    'pnpm run test:e2e',
    'pnpm run hygiene',
  ],
}

/**
 * Triage a git diff and select the minimal sufficient verification gates.
 * @param decisionService - the active decision seam service.
 * @param diff - git diff stat and list of changed paths.
 * @param signal - optional cancellation signal.
 * @returns the gate recommendation with corresponding shell commands.
 */
export async function selectVerificationGates(
  decisionService: DecisionService,
  diff: {
    readonly diffStat: string
    readonly changedFiles: readonly string[]
  },
  signal?: AbortSignal,
): Promise<GateSelection> {
  const state = [
    'Changed File Paths:',
    ...diff.changedFiles.map(f => `- ${f}`),
    'Git Diff Stat Summary:',
    diff.diffStat.slice(0, 1500),
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        suite: {
          type: 'choice',
          instructions: 'Which verification gate suite is minimally sufficient for this diff?',
          criteria: {
            doc_only: 'Only documentation, comments, markdown, or text artifacts changed',
            lint_and_unit: 'Standard code changes without touching capability seams, core runtime, or wire protocols',
            full_coverage: 'New package, capability seam, security policy, or core loop modification',
            full_with_e2e: 'Hardware, native driver, external cloud gateway, or network protocol changes',
          },
        },
        blast_radius: {
          type: 'score',
          instructions: 'What is the architectural blast radius of this change?',
          criteria: [
            'isolated-doc-only',
            'package-internal-logic',
            'cross-package-capability-seam',
            'repo-wide-runtime-or-wire-protocol',
          ],
        },
        needs_coverage: {
          type: 'boolean',
          instructions: 'Does this change require running the strict 100% CI coverage gate?',
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const suiteAns = result.answers['suite']
  const blastAns = result.answers['blast_radius']
  const covAns = result.answers['needs_coverage']

  if (suiteAns?.type !== 'choice' || blastAns?.type !== 'score' || covAns?.type !== 'boolean') {
    throw new Error('Decision provider returned unexpected answer types for gate selection')
  }

  const suite = suiteAns.choice as GateSelection['recommendedSuite']
  const confidence = suiteAns.probabilities[suiteAns.choice] ?? 0
  const needsCoverage = covAns.probability >= 0.5 || blastAns.score >= 2.0

  let recommendedSuite = suite
  if (needsCoverage && (recommendedSuite === 'doc_only' || recommendedSuite === 'lint_and_unit')) {
    recommendedSuite = 'full_coverage'
  }

  return {
    recommendedSuite,
    commands: SUITE_COMMANDS[recommendedSuite],
    confidence,
    blastRadius: blastAns.score,
    needsCoverage,
    latencyMs,
    providerId: result.providerId,
  }
}
