/**
 * Real-TypeSafe benchmark for two-pass skill adjudication. DSH contributes its proposed skills;
 * JEV scores the full bounded catalog, and disagreement triggers a second comparison across the
 * DSH proposal, JEV's primary choice, and every high-probability relevant skill. The final policy
 * separates primary, parallel-now, and later-depth skills so routing quality can be measured against
 * labeled ground truth rather than judged from confidence alone.
 */

import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, { type DecisionQuestion, type DecisionResult } from '@deepseek-ai/dsh-decision'
import * as jevPlugin from '@deepseek-ai/dsh-decision-jev'

interface SkillSpec {
  readonly name: string
  readonly description: string
}

interface GroundTruth {
  readonly primary: string
  readonly parallel: readonly string[]
  readonly later: readonly string[]
}

interface BenchmarkCase {
  readonly id: string
  readonly request: string
  readonly dshProposed: readonly string[]
  readonly truth: GroundTruth
}

const CATALOG: readonly SkillSpec[] = [
  { name: 'dsh-code-review', description: 'Review a DeepSeek Harness change and report file-backed defects and missing tests.' },
  { name: 'architecture', description: 'Produce a technical architecture from an accepted software brief.' },
  { name: 'diagram-design', description: 'Create a validated branded SVG diagram for architecture, workflows, or data flows.' },
  { name: 'penetration-testing-with-strix', description: 'Run a Strix penetration test against a repository, API, or web application.' },
  { name: 'fix-security-vulnerabilities-with-strix', description: 'Fix vulnerabilities after a Strix report already identifies them.' },
  { name: 'dsh-design-thinking-for-business-growth', description: 'Design and scale business ecosystems, value streams, and business models.' },
  { name: 'dsh-design-thinking-innovation-metrics', description: 'Define innovation metrics, North Stars, OKRs, and explore/exploit measures.' },
  { name: 'timesfm-forecasting', description: 'Forecast univariate or multivariate time series with Google TimesFM.' },
  { name: 'implement-code', description: 'Implement an accepted software brief and architecture in the workspace.' },
  { name: 'use-modern-go', description: 'Apply the Modern Go Guidelines while writing or modifying Go code.' },
  { name: 'go-current', description: 'Apply the current stable Go release and version-specific language guidance.' },
]

const REPETITIONS = Number.parseInt(process.env['JEV_SKILL_BENCHMARK_REPETITIONS'] ?? '1', 10)

const CASES: readonly BenchmarkCase[] = [
  {
    id: 'single-review',
    request: 'Review the current DeepSeek Harness pull request for regressions and missing tests. Do not implement fixes.',
    dshProposed: ['dsh-code-review'],
    truth: { primary: 'dsh-code-review', parallel: [], later: [] },
  },
  {
    id: 'architecture-plus-visual',
    request: 'Design the technical architecture for a multi-agent payment system and deliver an SVG architecture diagram.',
    dshProposed: ['diagram-design'],
    truth: { primary: 'architecture', parallel: [], later: ['diagram-design'] },
  },
  {
    id: 'security-stage-order',
    request: 'Pentest this API with Strix, then fix the vulnerabilities that the test confirms.',
    dshProposed: ['fix-security-vulnerabilities-with-strix'],
    truth: { primary: 'penetration-testing-with-strix', parallel: [], later: ['fix-security-vulnerabilities-with-strix'] },
  },
  {
    id: 'go-implementation-companions',
    request: 'Implement the accepted design as a production Go service using the current Go release and modern Go guidelines.',
    dshProposed: ['implement-code'],
    truth: { primary: 'implement-code', parallel: ['use-modern-go', 'go-current'], later: [] },
  },
  {
    id: 'single-forecast',
    request: 'Forecast the next 12 monthly demand values from this historical time series.',
    dshProposed: ['timesfm-forecasting'],
    truth: { primary: 'timesfm-forecasting', parallel: [], later: [] },
  },
]

interface RoutedSkills {
  readonly primary: string
  readonly parallel: readonly string[]
  readonly later: readonly string[]
  readonly firstPassMs: number
  readonly adjudicationMs: number
  readonly highProbability: readonly string[]
  readonly disagreement: boolean
}

/** Run the two-pass candidate expansion and DSH-owned final policy. */
async function routeSkills(ctx: Context, sample: BenchmarkCase): Promise<RoutedSkills> {
  const criteria = Object.fromEntries(CATALOG.map(skill => [skill.name, skill.description]))
  const questions: Record<string, DecisionQuestion> = {
    primary: {
      type: 'choice',
      instructions: 'Which single skill must own the first phase of this request?',
      criteria,
    },
  }
  for (const skill of CATALOG) {
    questions[`relevant:${skill.name}`] = {
      type: 'boolean',
      instructions: `Is ${skill.name} materially relevant to completing this request, either now, in parallel, or in a later phase?`,
    }
  }

  const state = [
    `User request: ${sample.request}`,
    `DSH proposed skills: ${sample.dshProposed.join(', ')}`,
    'Available skills:',
    ...CATALOG.map(skill => `- ${skill.name}: ${skill.description}`),
  ].join('\n')

  const firstStarted = performance.now()
  const first = await ctx.decision.evaluate({ state, questions })
  const firstPassMs = Math.round(performance.now() - firstStarted)
  const firstPrimary = choice(first, 'primary')
  const highProbability = CATALOG
    .filter(skill => probability(first, `relevant:${skill.name}`) >= 0.65)
    .map(skill => skill.name)
  const disagreement = !sample.dshProposed.includes(firstPrimary)
  const candidates = [...new Set([...sample.dshProposed, firstPrimary, ...highProbability])]

  if (!disagreement && candidates.length === 1) {
    return { primary: firstPrimary, parallel: [], later: [], firstPassMs, adjudicationMs: 0, highProbability, disagreement }
  }

  const candidateCriteria = Object.fromEntries(candidates.map(name => [name, criteria[name]!]))
  const adjudicationQuestions: Record<string, DecisionQuestion> = {
    primary: {
      type: 'choice',
      instructions: 'After comparing DSH and JEV candidates, which skill owns the first phase?',
      criteria: candidateCriteria,
    },
  }
  for (const name of candidates) {
    adjudicationQuestions[`phase:${name}`] = {
      type: 'choice',
      instructions: `When should ${name} be used for this request?`,
      criteria: {
        primary_now: 'Own the first phase now.',
        parallel_now: 'Run alongside the primary skill now because their work is independent or complementary.',
        later_depth: 'Load after the first phase produces information this skill needs.',
        not_needed: 'Do not load this skill for this request.',
      },
    }
  }

  const adjudicationStarted = performance.now()
  const adjudicated = await ctx.decision.evaluate({
    state: `${state}\nFirst-pass JEV primary: ${firstPrimary}\nHigh-probability relevant skills: ${highProbability.join(', ')}`,
    questions: adjudicationQuestions,
  })
  const adjudicationMs = Math.round(performance.now() - adjudicationStarted)
  const primary = choice(adjudicated, 'primary')
  const parallel: string[] = []
  const later: string[] = []
  for (const name of candidates) {
    if (name === primary) continue
    const phase = choice(adjudicated, `phase:${name}`)
    if (phase === 'parallel_now') parallel.push(name)
    if (phase === 'later_depth') later.push(name)
  }
  return { primary, parallel, later, firstPassMs, adjudicationMs, highProbability, disagreement }
}

function choice(result: DecisionResult, key: string): string {
  const answer = result.answers[key]
  if (answer?.type !== 'choice') throw new Error(`expected choice answer ${key}`)
  return answer.choice
}

function probability(result: DecisionResult, key: string): number {
  const answer = result.answers[key]
  if (answer?.type !== 'boolean') throw new Error(`expected boolean answer ${key}`)
  return answer.probability
}

function setF1(actual: readonly string[], expected: readonly string[]): number {
  if (actual.length === 0 && expected.length === 0) return 1
  const actualSet = new Set(actual)
  const expectedSet = new Set(expected)
  const truePositive = [...actualSet].filter(value => expectedSet.has(value)).length
  if (truePositive === 0) return 0
  const precision = truePositive / actualSet.size
  const recall = truePositive / expectedSet.size
  return 2 * precision * recall / (precision + recall)
}

function quality(route: RoutedSkills, truth: GroundTruth): number {
  return (route.primary === truth.primary ? 0.5 : 0)
    + 0.25 * setF1(route.parallel, truth.parallel)
    + 0.25 * setF1(route.later, truth.later)
}

function median(values: readonly number[]): number {
  const sorted = [...values].sort((a, b) => a - b)
  return sorted[Math.floor(sorted.length / 2)]!
}

describe.skipIf(!process.env['TYPESAFE_API_KEY'])('JEV skill-selection adjudication benchmark', () => {
  it('improves labeled routing quality while preserving staged and parallel skills', async () => {
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})
    jevPlugin.apply(ctx, {})

    expect(REPETITIONS).toBeGreaterThan(0)
    const rows: Array<Record<string, unknown>> = []
    for (let run = 1; run <= REPETITIONS; run += 1) {
      for (const sample of CASES) {
        const route = await routeSkills(ctx, sample)
        rows.push({
          run,
          case: sample.id,
          dsh: sample.dshProposed,
          primary: route.primary,
          parallel: route.parallel,
          later: route.later,
          truth: sample.truth,
          highProbability: route.highProbability,
          disagreement: route.disagreement,
          firstPassMs: route.firstPassMs,
          adjudicationMs: route.adjudicationMs,
          totalMs: route.firstPassMs + route.adjudicationMs,
          quality: quality(route, sample.truth),
        })
      }
    }

    const baselineAccuracy = CASES.filter(sample => sample.dshProposed[0] === sample.truth.primary).length / CASES.length
    const finalAccuracy = rows.filter((row) => {
      const sample = CASES.find(candidate => candidate.id === row['case'])
      return row['primary'] === sample?.truth.primary
    }).length / rows.length
    const meanQuality = rows.reduce((sum, row) => sum + Number(row['quality']), 0) / rows.length
    const medianTotalMs = median(rows.map(row => Number(row['totalMs'])))
    console.log(JSON.stringify({ baselineAccuracy, finalAccuracy, meanQuality, medianTotalMs, rows }, null, 2))

    expect(finalAccuracy).toBeGreaterThanOrEqual(baselineAccuracy)
    expect(finalAccuracy).toBeGreaterThanOrEqual(0.8)
    expect(meanQuality).toBeGreaterThanOrEqual(0.7)
    expect(medianTotalMs).toBeLessThan(3_000)
  }, 120_000)
})
