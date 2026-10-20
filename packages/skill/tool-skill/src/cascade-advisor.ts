/**
 * Advisory two-stage skill cascade for `tool-skill`. On the first step of each user turn it groups
 * the live model-invocable catalog into configured categories and asks the decision provider for a
 * category and skill in the background; when the model later calls the `skill` tool, the advisor
 * logs whether the cascade agreed. Nothing reaches the model, so no session event is written.
 * @module @deepseek-ai/dsh-tool-skill/cascade-advisor
 */

import type { Context } from '@deepseek-ai/cordis'
import type { Agent } from '@deepseek-ai/dsh-agent'
import type { DecisionService } from '@deepseek-ai/dsh-decision'
import { cascadeSkillSelection, type SkillCascade, type SkillCascadeCategory } from '@deepseek-ai/dsh-decision-consumers'
import type { UserMessage } from '@deepseek-ai/dsh-session'
import type { SkillSummary } from '@deepseek-ai/dsh-skill'

/** One configured stage-one category. */
export interface SkillCascadeCategoryConfig {
  /** Category id reported in advisory logs. */
  name: string
  /** Routing description judged by the decision provider. */
  description: string
  /** Skill names in this category; names absent from the live catalog are skipped. */
  skills: string[]
}

/** Resolved advisor settings. */
export interface CascadeAdvisorOptions {
  readonly categories: readonly SkillCascadeCategoryConfig[]
  /** Category receiving every live skill that no configured category lists. */
  readonly ungrouped: { readonly name: string; readonly description: string }
  readonly maxTaskChars: number
  readonly timeoutMs: number
}

/** Reads the live model-invocable catalog under the cascade deadline. */
export type CatalogLoader = (signal: AbortSignal) => Promise<readonly SkillSummary[]>

/** Background cascade handle for one agent's current turn. */
interface PendingCascade {
  readonly turn: number
  readonly result: Promise<SkillCascade | undefined>
}

/** Advisor surface used by the `skill` tool and the pre-step listener. */
export interface CascadeAdvisor {
  /**
   * Start the background cascade for a turn's first step.
   * @param agent - agent whose step is starting.
   * @param turn - turn number of the step.
   * @param messages - claimed messages of the step; only direct user text starts a cascade.
   * @param loadSkills - catalog reader, called only when a cascade starts.
   */
  start(agent: Agent, turn: number, messages: readonly UserMessage[], loadSkills: CatalogLoader): void
  /**
   * Log agreement between the model's `skill` call and the turn's cascade verdict once it settles.
   * Never rejects: a failed or absent cascade logs nothing here.
   * @param agent - calling agent.
   * @param requested - skill name the model asked for.
   * @returns a promise that settles after the comparison is logged.
   */
  compare(agent: Agent, requested: string): Promise<void>
  /**
   * Drop the pending cascade of a disposed agent.
   * @param agent - disposed agent.
   */
  forget(agent: Agent): void
}

/**
 * Group the live catalog into cascade categories. Configured categories keep their order and
 * drop names missing from the catalog; every remaining live skill joins the ungrouped category.
 * Categories left without skills are omitted.
 * @param skills - live model-invocable catalog.
 * @param options - configured categories and the ungrouped category.
 * @returns non-empty categories for {@link cascadeSkillSelection}.
 */
export function groupCatalog(
  skills: readonly SkillSummary[],
  options: Pick<CascadeAdvisorOptions, 'categories' | 'ungrouped'>,
): SkillCascadeCategory[] {
  const byName = new Map(skills.map(skill => [skill.name, skill]))
  const grouped = new Set<string>()
  const categories: SkillCascadeCategory[] = []
  for (const category of options.categories) {
    const members = category.skills
      .map(name => byName.get(name))
      .filter((skill): skill is SkillSummary => skill !== undefined && !grouped.has(skill.name))
    for (const skill of members) grouped.add(skill.name)
    if (members.length > 0) {
      categories.push({ name: category.name, description: category.description, skills: members.map(candidate) })
    }
  }
  const rest = skills.filter(skill => !grouped.has(skill.name))
  if (rest.length > 0) categories.push({ ...options.ungrouped, skills: rest.map(candidate) })
  return categories
}

/**
 * Create the advisory cascade owner.
 * @param ctx - plugin context supplying the logger.
 * @param decision - mounted decision service.
 * @param options - resolved advisor settings.
 * @returns the advisor bound to this plugin's lifetime.
 */
export function createCascadeAdvisor(ctx: Context, decision: DecisionService, options: CascadeAdvisorOptions): CascadeAdvisor {
  const pending = new Map<Agent, PendingCascade>()
  return {
    start(agent, turn, messages, loadSkills) {
      if (pending.get(agent)?.turn === turn) return
      const request = userText(messages, options.maxTaskChars)
      if (request === undefined) return
      const started = performance.now()
      const deadline = AbortSignal.timeout(options.timeoutMs)
      const run = async (): Promise<SkillCascade | undefined> => {
        const categories = groupCatalog(await loadSkills(deadline), options)
        if (categories.length === 0) return undefined
        return cascadeSkillSelection(decision, { request, categories }, deadline)
      }
      const result = run()
        .then((cascade) => {
          if (cascade === undefined) return undefined
          ctx.logger.info(
            'skill cascade (advisory): agent %s turn %d -> %s/%s (category %s, skill %s, needs-skill %s) in %dms, %d call(s) via %s',
            agent.id, turn, cascade.category, cascade.skill,
            cascade.categoryConfidence.toFixed(2), cascade.skillConfidence.toFixed(2), cascade.needsSkillProbability.toFixed(2),
            Math.round(performance.now() - started), cascade.calls, cascade.providerIds.join(',') || 'local',
          )
          return cascade
        }, (error: unknown) => {
          ctx.logger.warn('skill cascade (advisory): agent %s turn %d failed: %o', agent.id, turn, error)
          return undefined
        })
      pending.set(agent, { turn, result })
    },
    async compare(agent, requested) {
      const cascade = await pending.get(agent)?.result
      if (cascade === undefined) return
      ctx.logger.info(
        'skill cascade (advisory): agent %s model loaded %s; cascade chose %s/%s (%s)',
        agent.id, requested, cascade.category, cascade.skill, cascade.skill === requested ? 'agree' : 'disagree',
      )
    },
    forget(agent) {
      pending.delete(agent)
    },
  }
}

function candidate(skill: SkillSummary): { name: string; description: string } {
  return { name: skill.name, description: skill.description }
}

function userText(messages: readonly UserMessage[], maxChars: number): string | undefined {
  const text = messages
    .filter(message => message.source.kind === 'user')
    .flatMap(message => message.content)
    .filter(block => block.type === 'text')
    .map(block => block.text)
    .join('\n')
    .trim()
  return text.length === 0 ? undefined : text.slice(0, maxChars)
}
