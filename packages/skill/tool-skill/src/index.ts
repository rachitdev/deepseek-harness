/**
 * Durable session skill catalog and model-facing `skill` loader tool.
 *
 * @module @deepseek-ai/dsh-tool-skill
 */

import type {} from '@deepseek-ai/dsh-working-directory'
import { createHash } from 'node:crypto'
import type { Context } from '@deepseek-ai/cordis'
import z from '@deepseek-ai/schemastery'
import { latestUserTaskText } from '@deepseek-ai/dsh-agent'
import type { Agent, PreStepDecision } from '@deepseek-ai/dsh-agent'
import type {} from '@deepseek-ai/dsh-decision'
import { adjudicateSkillSelection, type SkillAdjudication } from '@deepseek-ai/dsh-decision-consumers'
import { defineTool } from '@deepseek-ai/dsh-tools'
import { createCascadeAdvisor, type CascadeAdvisor, type CascadeAdvisorOptions, type SkillCascadeCategoryConfig } from './cascade-advisor.ts'
import { createUserMessage } from '@deepseek-ai/dsh-llm'
import { SessionSeq, type UserMessage } from '@deepseek-ai/dsh-session'
import {
  escapeText,
  isModelInvocable,
  isSkillName,
  isUserInvocable,
  renderSkillContent,
  type SkillDefinition,
  type SkillInvocationSource,
  type SkillSummary,
  type SkillViewOptions,
} from '@deepseek-ai/dsh-skill'
import type { AssembledSection, PromptAssembly } from '@deepseek-ai/dsh-system-prompt'

export const name = 'tool-skill'
export const inject = ['agents', 'tools', 'skills', 'workingDirectory']

const DEFAULT_CATALOG_DESCRIPTION_MAX_LENGTH = 500
/**
 * Skills whose full instructions join every assembled system prompt by default.
 * The deployment owner pinned the Design Thinking General Skill (DTGS) router
 * here so intent clarification and series routing are guaranteed before the
 * first token of every conversation, regardless of topic.
 */
const DEFAULT_PINNED_SKILLS = ['dsh-design-thinking-general']
/**
 * Durable provider and item records for one published session skill catalog. The catalog is a
 * `catalog`-form context, so it records the entries it published beside the
 * model-facing prose: a consumer presenting the list must not re-parse the
 * `<available_skills>` block, whose framing exists for the model.
 */
export interface SkillCatalogSource {
  readonly kind: 'skill-catalog'
  readonly form: 'catalog'
  /** Marks a replacement catalog rather than this session's first publication. */
  readonly update?: true
  /** Exactly the entries this message published, in catalog order. */
  readonly entries: readonly { readonly name: string; readonly description: string }[]
}

declare module '@deepseek-ai/dsh-llm' {
  interface MessageSourceMap {
    'skill-catalog': SkillCatalogSource
  }
}

/** Durable entry list mirroring the rendered catalog lines, for non-model consumers. */
function catalogSourceEntries(
  skills: SkillSummary[],
  descriptionMaxLength: number,
): SkillCatalogSource['entries'] {
  return skills.map(skill => ({
    name: skill.name,
    description: catalogDescription(skill.description, descriptionMaxLength),
  }))
}

/** Model-facing skill catalog configuration. */
export interface Config {
  /** Maximum normalized description length rendered in the session catalog; minimum 3. */
  catalogDescriptionMaxLength?: number
  /**
   * Skills whose full instructions join every assembled system prompt, loaded
   * fresh per assembly with the calling agent's cwd and scope. Names must be
   * valid skill names; a name that resolves to nothing, fails to load, or is
   * not model-invocable is skipped for that assembly rather than failing the
   * step. An empty array disables pinned injection. Defaults to
   * {@link DEFAULT_PINNED_SKILLS} (the DTGS router).
   */
  pinnedSkills?: string[]
  /** Adjudicate model-selected skills through the mounted decision provider. Defaults to false. */
  skillAdjudication?: boolean
  /** Maximum lexical shortlist size, including the model-proposed skill. Defaults to 16. */
  skillAdjudicationMaxCandidates?: number
  /** JEV relevance probability required for final comparison. Defaults to 0.65. */
  skillAdjudicationRelevanceThreshold?: number
  /** JEV phase probability required to load a companion skill. Defaults to 0.55. */
  skillAdjudicationPhaseConfidence?: number
  /** Maximum user-task characters sent to the decision provider. Defaults to 4000. */
  skillAdjudicationMaxTaskChars?: number
  /**
   * Advisory two-stage (category, then skill) cascade over the live catalog, run in the
   * background on each user turn and compared with the model's `skill` call in the log only.
   * Requires a mounted decision service when enabled.
   */
  skillCascade?: SkillCascadeConfig
}

/** Advisory skill-cascade configuration. */
export interface SkillCascadeConfig {
  /** Run the advisory cascade. Defaults to false. */
  enabled?: boolean
  /** Stage-one categories in judging order; a skill listed twice belongs to its first category. */
  categories?: SkillCascadeCategoryConfig[]
  /** Name of the category holding every live skill no configured category lists. Defaults to `other`. */
  ungroupedName?: string
  /** Description of the ungrouped category. */
  ungroupedDescription?: string
  /** Maximum user-task characters sent to the decision provider. Defaults to 4000. */
  maxTaskChars?: number
  /** Deadline for both cascade stages together. Defaults to 10000 ms. */
  timeoutMs?: number
}

const SkillCascadeConfig: z<SkillCascadeConfig> = z.object({
  enabled: z.boolean().default(false),
  categories: z.array(z.object({
    name: z.string().required(),
    description: z.string().required(),
    skills: z.array(z.string()).required(),
  })).default([]),
  ungroupedName: z.string().default('other'),
  ungroupedDescription: z.string().default('Skills outside every other category.'),
  maxTaskChars: z.number().default(4000),
  timeoutMs: z.number().default(10000),
})

/** Validate and default the model-facing skill catalog configuration. */
export const Config: z<Config> = z.object({
  catalogDescriptionMaxLength: z.number().default(DEFAULT_CATALOG_DESCRIPTION_MAX_LENGTH),
  pinnedSkills: z.array(z.string()).default(DEFAULT_PINNED_SKILLS),
  skillAdjudication: z.boolean().default(false),
  skillAdjudicationMaxCandidates: z.number().default(16),
  skillAdjudicationRelevanceThreshold: z.number().min(0).max(1).default(0.65),
  skillAdjudicationPhaseConfidence: z.number().min(0).max(1).default(0.55),
  skillAdjudicationMaxTaskChars: z.number().default(4000),
  skillCascade: SkillCascadeConfig,
})

/**
 * Register the model-facing skill loader and its visibility-matched
 * durable session catalog. The catalog is emitted only when the calling agent
 * resolves this plugin's exact tool registration; a restriction or scoped
 * same-name shadow therefore removes both the schema and its call guidance.
 */
export function apply(ctx: Context, config: Config = {}): void {
  const catalogDescriptionMaxLength = config.catalogDescriptionMaxLength ?? DEFAULT_CATALOG_DESCRIPTION_MAX_LENGTH
  assertPositiveInteger('catalogDescriptionMaxLength', catalogDescriptionMaxLength, 3)
  const pinnedSkills = config.pinnedSkills ?? DEFAULT_PINNED_SKILLS
  for (const pinned of pinnedSkills) {
    if (!isSkillName(pinned)) {
      throw new Error(`tool-skill: pinnedSkills entry "${pinned}" is not a valid skill name`)
    }
  }
  const pinnedNames = new Set(pinnedSkills)
  const skillAdjudication = config.skillAdjudication ?? false
  const skillAdjudicationMaxCandidates = config.skillAdjudicationMaxCandidates ?? 16
  const skillAdjudicationRelevanceThreshold = config.skillAdjudicationRelevanceThreshold ?? 0.65
  const skillAdjudicationPhaseConfidence = config.skillAdjudicationPhaseConfidence ?? 0.55
  const skillAdjudicationMaxTaskChars = config.skillAdjudicationMaxTaskChars ?? 4000
  assertPositiveInteger('skillAdjudicationMaxCandidates', skillAdjudicationMaxCandidates, 2)
  assertPositiveInteger('skillAdjudicationMaxTaskChars', skillAdjudicationMaxTaskChars, 1)
  const decisionService = skillAdjudication || config.skillCascade?.enabled === true ? ctx.get('decision') : undefined
  if (skillAdjudication && decisionService === undefined) {
    throw new Error('tool-skill: skillAdjudication requires a mounted decision service')
  }
  const decision = skillAdjudication ? decisionService : undefined
  const cascadeOptions = resolveSkillCascade(config.skillCascade)
  let cascade: CascadeAdvisor | undefined
  if (cascadeOptions !== undefined) {
    if (decisionService === undefined) throw new Error('tool-skill: skillCascade requires a mounted decision service')
    cascade = createCascadeAdvisor(ctx, decisionService, cascadeOptions)
  }

  const skillTool = defineTool({
    name: 'skill',
    description: 'Load the full instructions for a skill. Call it before acting on a task that names or clearly matches a skill in the session skill catalog.',
    parameters: {
      name: { type: 'string', required: true, description: 'The exact skill name from the available skills list.' },
    },
    output: {
      schema: {
        type: 'object',
        additionalProperties: false,
        properties: {
          name: { type: 'string', required: true },
          provider: { type: 'string', required: true },
          resourceBase: {
            oneOf: [
              {
                type: 'object',
                additionalProperties: false,
                properties: {
                  kind: { type: 'string', required: true, const: 'directory' },
                  path: { type: 'string', required: true },
                },
              },
              {
                type: 'object',
                additionalProperties: false,
                properties: {
                  kind: { type: 'string', required: true, const: 'url' },
                  url: { type: 'string', required: true },
                },
              },
              {
                type: 'object',
                additionalProperties: false,
                properties: {
                  kind: { type: 'string', required: true, const: 'opaque' },
                  description: { type: 'string', required: true },
                },
              },
            ],
          },
          content: { type: 'string', required: true },
        },
      },
      render: (_args, value) => [{ type: 'text', text: renderSkillContent(value) }],
    },
    async execute(args, exec) {
      if (!isSkillName(args.name)) {
        throw new Error(`invalid skill name "${args.name}"`)
      }
      // The agent is its own scope key, so the lookup resolves the layered
      // registry exactly as this agent's composition sees it.
      const cwd = exec.agent === undefined ? undefined : await ctx.workingDirectory.ensure(exec.agent, exec.signal)
      const lookup = { cwd, signal: exec.signal, scope: exec.agent }
      const summaries = await ctx.skills.list(lookup)
      const summary = summaries.find(skill => skill.name === args.name)
      if (!summary) {
        throw new Error(`skill "${args.name}" is unknown or no longer available`)
      }
      if (!isModelInvocable(summary)) {
        throw new Error(`skill "${args.name}" is not available for model invocation`)
      }
      // Advisory: the comparison logs once the background cascade settles and never delays the load.
      if (cascade !== undefined && exec.agent !== undefined) void cascade.compare(exec.agent, args.name)
      let routing: SkillAdjudication | undefined
      if (decision !== undefined && exec.agent !== undefined) {
        const request = latestUserTaskText(exec.agent, skillAdjudicationMaxTaskChars, 'Load the model-selected skill for the current task.')
        const candidates = shortlistSkills(summaries.filter(isModelInvocable), args.name, request, skillAdjudicationMaxCandidates)
        try {
          routing = await adjudicateSkillSelection(decision, {
            request,
            dshProposed: [args.name],
            candidates,
          }, {
            relevanceThreshold: skillAdjudicationRelevanceThreshold,
            minPhaseConfidence: skillAdjudicationPhaseConfidence,
          }, exec.signal)
        } catch (error: unknown) {
          if (exec.signal.aborted) throw error
          ctx.logger.warn('skill adjudication failed; preserving model-selected skill: %o', error)
        }
      }
      const primaryName = routing?.primary ?? args.name
      const primary = await loadModelSkill(ctx, primaryName, lookup)
      const companions: SkillDefinition[] = []
      for (const name of routing?.parallel ?? []) {
        companions.push(await loadModelSkill(ctx, name, lookup))
      }
      return {
        name: primary.name,
        provider: primary.provider,
        ...primary.resourceBase !== undefined ? {
          resourceBase: { ...primary.resourceBase },
        } : {},
        content: renderAdjudicatedContent(primary, companions, routing, args.name),
      }
    },
    presentCall(args) {
      return { card: 'generic', title: `Load skill ${args.name}`, kind: 'read', rawInput: args.name }
    },
  })
  ctx.tools.register(skillTool)

  // User-explicit skill invocation: a claimed user message whose first line
  // starts with `/<name>` naming a user-invocable skill is a deterministic
  // load gesture. The rendered body enters this step as injected
  // instructions context appended after every other injection — background
  // first (workspace rules, runtime policy, the catalog), the material the
  // model must act on last, closest to its answer. Registration order makes
  // that placement deterministic: this listener registers before the catalog
  // listener, so the waterfall hands it the catalog-bearing list to extend.
  // Only `source.kind === 'user'` messages are scanned — external text
  // cannot forge the gesture — and a token naming no user-invocable skill
  // stays ordinary prose (the command registry is a different closed
  // namespace, resolved client-side before a line ever becomes a prompt).
  // This is the only entry point for `disable-model-invocation` skills; the
  // catalog and the `skill` tool below never see them.
  ctx.on('agent/pre-step', async (
    { agent, messages, signal },
    next,
  ): Promise<PreStepDecision> => {
    const decision = await next()
    if (decision.kind === 'reject') return decision
    const names = invokedSkillNames(messages)
    if (names.length === 0) return decision
    signal.throwIfAborted()
    const lookup = { cwd: await ctx.workingDirectory.ensure(agent, signal), signal, scope: agent }
    const injections: UserMessage[] = []
    for (const name of names) {
      const skill = await ctx.skills.get(name, lookup)
      signal.throwIfAborted()
      // Unknown names and user-disabled skills stay plain prose: the
      // gesture was never a claim this boundary recognizes. The check sits
      // on the loaded definition — the single lookup that produces what is
      // actually injected.
      if (skill === undefined || !isUserInvocable(skill)) continue
      const source: SkillInvocationSource = { kind: 'skill-invocation', name, form: 'instructions' }
      injections.push(createUserMessage({
        content: [{ type: 'text', text: renderSkillContent(skill) }],
        source,
      }))
    }
    if (injections.length === 0) return decision
    return { ...decision, messages: [...decision.messages, ...injections] }
  })

  if (cascade !== undefined) {
    const advisor = cascade
    // Advisory only: the cascade starts in the background on a turn's first step and never
    // changes the step's messages, so the model's request is identical with or without it.
    ctx.on('agent/pre-step', async ({ agent, messages, turn }, next): Promise<PreStepDecision> => {
      const decision = await next()
      if (decision.kind === 'reject') return decision
      if (ctx.tools.get(skillTool.name, agent) !== skillTool) return decision
      advisor.start(agent, turn, messages, async (signal) => {
        const summaries = await ctx.skills.list({ cwd: await ctx.workingDirectory.ensure(agent, signal), signal, scope: agent })
        return summaries.filter(isModelInvocable)
      })
      return decision
    })
    ctx.on('agent/disposed', ({ agent }) => {
      advisor.forget(agent)
    })
  }

  // Register after the tool so reverse teardown removes guidance first. Exact definition
  // identity prevents a scoped shadow merely named `skill` from inheriting this catalog.
  //
  // The comparison is against the definition this plugin registered, not against
  // a lookup of its own name: `register()` files into the CALLING context's
  // scope, so a plugin mounted inside an agent preset registers for that agent
  // alone and an unscoped lookup correctly finds nothing.
  ctx.on('agent/pre-step', async (
    { agent, signal },
    next,
  ): Promise<PreStepDecision> => {
    const decision = await next()
    if (decision.kind === 'reject') return decision
    signal.throwIfAborted()
    const toolVisible = ctx.tools.get(skillTool.name, agent) === skillTool
    const snapshot = toolVisible
      ? await ctx.skills.snapshot({ cwd: await ctx.workingDirectory.ensure(agent, signal), signal, scope: agent })
      : { skills: [], complete: true }
    signal.throwIfAborted()
    if (!snapshot.complete) return decision
    // A pinned skill's full body already rides the system prompt, so listing it again would
    // spend catalog tokens on a skill the model never needs to load.
    const skills = snapshot.skills.filter(skill => isModelInvocable(skill) && !pinnedNames.has(skill.name))
    const entries = catalogSourceEntries(skills, catalogDescriptionMaxLength)
    const digest = digestCatalogEntries(entries)
    const history = catalogHistory(agent)
    const existing = catalogMessage(decision.messages)
    if (history.visibleDigest === digest) {
      return existing === undefined
        ? decision
        : { ...decision, messages: decision.messages.filter(message => message.id !== existing.message.id) }
    }
    if (existing !== undefined && digestCatalogEntries(existing.entries) === digest) return decision
    if (!history.published && skills.length === 0) {
      return existing === undefined
        ? decision
        : { ...decision, messages: decision.messages.filter(message => message.id !== existing.message.id) }
    }
    const catalog = history.published
      ? renderCatalogUpdate(entries)
      : renderCatalogMessage(entries)
    return {
      ...decision,
      messages: existing === undefined
        ? [...decision.messages, catalog]
        : decision.messages.map(message => message.id === existing.message.id ? catalog : message),
    }
  })

  // Pinned skills ride the system prompt itself, so their instructions are
  // guaranteed before the first token of every conversation — no catalog
  // routing step, tool call, or user gesture can bypass them. Section text
  // providers are synchronous while skill bodies load asynchronously, so the
  // contribution joins through the assembly waterfall after `next()` has
  // produced the registry-built sections. Appending at the end places the
  // instruction material after the deployment's identity and policies,
  // closest to the conversation, mirroring where the invocation injection
  // sits in a step's message list. Registration after both pre-step
  // listeners keeps reverse teardown removing this prompt material first.
  //
  // A pinned name that resolves to nothing, fails to load (for example
  // malformed frontmatter), or lost model invocability degrades to
  // catalog-only routing for that assembly instead of failing every model
  // step of every agent.
  ctx.on('system-prompt/assemble', async (
    _assembly,
    context,
    next,
  ): Promise<PromptAssembly> => {
    const assembled = await next()
    const agent = context.agent
    if (agent === undefined || pinnedSkills.length === 0) return assembled
    const lookup = { cwd: await ctx.workingDirectory.ensure(agent, context.signal), signal: context.signal, scope: agent }
    // Several tool-skill instances can serve one agent (an agent-preset row beside a profile row),
    // each hooked into the same assembly; a skill another instance already pinned must not repeat.
    const present = new Set(assembled.sections.map(section => section.name))
    const pinned: AssembledSection[] = []
    for (const pinnedName of pinnedSkills) {
      if (present.has(`skill:pinned:${pinnedName}`)) continue
      let skill: SkillDefinition | undefined
      try {
        skill = await ctx.skills.get(pinnedName, lookup)
      } catch (error: unknown) {
        // A pinned skill that fails to load degrades to catalog-only routing for this assembly.
        ctx.logger.warn('tool-skill: pinned skill "%s" failed to load; skipping it for this assembly: %o', pinnedName, error)
        continue
      }
      context.signal?.throwIfAborted()
      if (skill === undefined || !isModelInvocable(skill)) continue
      pinned.push({ name: `skill:pinned:${skill.name}`, text: renderSkillContent(skill) })
    }
    if (pinned.length === 0) return assembled
    return { ...assembled, sections: [...assembled.sections, ...pinned] }
  })
}

/**
 * Resolve the advisory cascade settings, or `undefined` when the cascade is disabled.
 * @param config - raw `skillCascade` configuration.
 * @returns advisor options with every default applied.
 */
function resolveSkillCascade(config: SkillCascadeConfig | undefined): CascadeAdvisorOptions | undefined {
  if (config?.enabled !== true) return undefined
  const resolved = SkillCascadeConfig(config) as Required<SkillCascadeConfig>
  assertPositiveInteger('skillCascade.maxTaskChars', resolved.maxTaskChars, 1)
  assertPositiveInteger('skillCascade.timeoutMs', resolved.timeoutMs, 1)
  const names = new Set([resolved.ungroupedName])
  for (const category of resolved.categories) {
    if (names.has(category.name)) throw new Error(`tool-skill: skillCascade category "${category.name}" is duplicated`)
    names.add(category.name)
    const invalid = category.skills.find(skill => !isSkillName(skill))
    if (invalid !== undefined) {
      throw new Error(`tool-skill: skillCascade category "${category.name}" lists invalid skill name "${invalid}"`)
    }
  }
  return {
    categories: resolved.categories,
    ungrouped: { name: resolved.ungroupedName, description: resolved.ungroupedDescription },
    maxTaskChars: resolved.maxTaskChars,
    timeoutMs: resolved.timeoutMs,
  }
}

function renderCatalogMessage(entries: SkillCatalogSource['entries']): UserMessage {
  return createUserMessage({
    content: [{
      type: 'text',
      text: [
        '<system-reminder>',
        'A skill is a reusable set of task-specific instructions. The following skills are available in this session:',
        '',
        '<available_skills>',
        ...renderCatalogEntries(entries),
        '</available_skills>',
        '',
        "If the user names a skill, or the task clearly matches a skill's description, call the `skill` tool with the exact skill name before taking task actions. Load all applicable skills, then follow their full instructions. This catalog contains summaries only; do not infer or follow a skill's instructions until it has been loaded.",
        'A user may also invoke a skill directly; its <skill_content> block then appears in this conversation. Follow it, and do not call the `skill` tool again for that skill.',
        '</system-reminder>',
      ].join('\n'),
    }],
    source: {
      kind: 'skill-catalog',
      form: 'catalog',
      entries,
    },
  })
}

function renderCatalogUpdate(entries: SkillCatalogSource['entries']): UserMessage {
  const availability = entries.length === 0
    ? [
      'No skills are currently available through the `skill` tool. Do not use names from earlier skill catalogs.',
      'A user may still invoke a skill directly; its <skill_content> block then appears in this conversation. Follow it, and do not call the `skill` tool for it.',
    ]
    : [
      'Use only names in this replacement catalog. If the user names a listed skill, or the task clearly matches its description, call the `skill` tool with the exact name before acting.',
      'A user may also invoke a skill directly; its <skill_content> block then appears in this conversation. Follow it, and do not call the `skill` tool again for that skill.',
    ]
  return createUserMessage({
    content: [{
      type: 'text',
      text: [
        '<system-reminder>',
        'The available skill catalog changed. This complete catalog replaces every earlier available-skills list in this session:',
        '',
        '<available_skills>',
        ...renderCatalogEntries(entries),
        '</available_skills>',
        '',
        ...availability,
        '</system-reminder>',
      ].join('\n'),
    }],
    source: {
      kind: 'skill-catalog',
      form: 'catalog',
      update: true,
      entries,
    },
  })
}

/**
 * Model-facing catalog lines, projected from the same entries the source records.
 * The pseudo-XML escaping belongs to this frame, not to the published fact, so it
 * is applied here and never stored. Names are `isSkillName`-validated and carry
 * no escapable character.
 */
function renderCatalogEntries(entries: SkillCatalogSource['entries']): string[] {
  return entries.map(entry => `- \`${entry.name}\`: ${escapeText(entry.description)}`)
}

/**
 * Catalog identity over the durable entry list rather than the rendered prose.
 * The entries are what changes; the surrounding `<system-reminder>` framing is
 * written for the model and must not decide whether a republish is needed.
 */
function digestCatalogEntries(entries: SkillCatalogSource['entries']): string {
  // JSON per entry rather than a separator character: every separator is itself
  // a legal description character, so only quoting makes the boundary exact.
  const canonical = entries.map(entry => JSON.stringify([entry.name, entry.description])).join('\n')
  return createHash('sha256')
    .update(canonical)
    .digest('hex')
}

/** Load one model-invocable skill after routing has selected it. */
async function loadModelSkill(ctx: Context, name: string, lookup: SkillViewOptions): Promise<SkillDefinition> {
  const skill = await ctx.skills.get(name, lookup)
  if (!skill) throw new Error(`skill "${name}" is unknown or no longer available`)
  if (!isModelInvocable(skill)) throw new Error(`skill "${name}" is not available for model invocation`)
  return skill
}

/** Render routing evidence, primary instructions, parallel instructions, and deferred names. */
function renderAdjudicatedContent(
  primary: SkillDefinition,
  companions: readonly SkillDefinition[],
  routing: SkillAdjudication | undefined,
  proposed: string,
): string {
  if (routing === undefined) return primary.content
  const sections = [
    '<skill-routing>',
    `DSH proposed: ${proposed}`,
    `Primary now: ${routing.primary}`,
    `Parallel now: ${routing.parallel.join(', ') || 'none'}`,
    `Later depth: ${routing.later.join(', ') || 'none'}`,
    `Decision provider: ${routing.providerId}`,
    '</skill-routing>',
    primary.content,
  ]
  for (const companion of companions) {
    sections.push(`Parallel skill selected by routing:\n${renderSkillContent(companion)}`)
  }
  if (routing.later.length > 0) {
    sections.push(`Later-depth skills: ${routing.later.join(', ')}. Call the skill tool with the exact name when the current phase produces the context they need.`)
  }
  return sections.join('\n\n')
}

/** Produce a bounded lexical shortlist while always retaining DSH's proposed skill. */
function shortlistSkills(
  skills: readonly SkillSummary[],
  proposed: string,
  request: string,
  maxCandidates: number,
): SkillSummary[] {
  const queryTokens = tokens(request)
  const scored = skills
    .filter(skill => skill.name !== proposed)
    .map(skill => ({ skill, score: shortlistScore(skill, request, queryTokens) }))
    .sort((left, right) => right.score - left.score || left.skill.name.localeCompare(right.skill.name))
  const selected = skills.find(skill => skill.name === proposed)
  if (selected === undefined) return []
  return [selected, ...scored.slice(0, maxCandidates - 1).map(entry => entry.skill)]
}

function shortlistScore(skill: SkillSummary, request: string, queryTokens: ReadonlySet<string>): number {
  const normalizedRequest = request.toLowerCase()
  let score = normalizedRequest.includes(skill.name.toLowerCase()) ? 100 : 0
  for (const token of tokens(`${skill.name} ${skill.description}`)) {
    if (queryTokens.has(token)) score += skill.name.toLowerCase().includes(token) ? 4 : 1
  }
  return score
}

function tokens(text: string): Set<string> {
  return new Set(text.toLowerCase().split(/[^a-z0-9]+/u).filter(token => token.length >= 3))
}

/**
 * Entries of one durable catalog message, or undefined when the record is not a
 * usable catalog.
 *
 * `agent.session.snapshotEvents()` may contain a resumed, forked, or externally written seed,
 * and seed validation only guarantees a source object with a non-empty `kind`;
 * no per-kind field is checked there. An unreadable record is therefore treated
 * as "not this plugin's catalog" — the posture the replaced content digest had —
 * rather than throwing inside the step listener, which would fail every
 * subsequent turn of that session.
 */
function readCatalogEntries(source: unknown): SkillCatalogSource['entries'] | undefined {
  const entries = (source as { entries?: unknown }).entries
  if (!Array.isArray(entries)) return undefined
  const readable: { name: string; description: string }[] = []
  for (const entry of entries as readonly unknown[]) {
    if (typeof entry !== 'object' || entry === null) return undefined
    const { name, description } = entry as { name?: unknown; description?: unknown }
    if (typeof name !== 'string' || name === '' || typeof description !== 'string') return undefined
    readable.push({ name, description })
  }
  return readable
}

function catalogHistory(agent: Agent): { visibleDigest?: string; published: boolean } {
  const visible = new Set(agent.session.surface.nodes)
  let published = false
  for (let index = agent.session.seq - 1; index >= 0; index -= 1) {
    // oxlint-disable-next-line typescript/no-deprecated -- Existing Session history read; migration deferred.
    const event = agent.session.eventAt(SessionSeq(index))
    if (event === undefined) {
      throw new Error(`skill catalog cannot read seq ${String(index)} below the current Session length`)
    }
    if (event.type !== 'user/message' || event.data.source.kind !== 'skill-catalog') continue
    const entries = readCatalogEntries(event.data.source)
    if (entries === undefined) continue
    const digest = digestCatalogEntries(entries)
    published = true
    if (visible.has(event.seq)) return { visibleDigest: digest, published }
  }
  return { published }
}

function catalogMessage(
  messages: readonly UserMessage[],
): { message: UserMessage; entries: SkillCatalogSource['entries'] } | undefined {
  for (const message of messages) {
    if (message.source.kind !== 'skill-catalog') continue
    const entries = readCatalogEntries(message.source)
    if (entries !== undefined) return { message, entries }
  }
  return undefined
}

function catalogDescription(value: string, maxLength: number): string {
  const normalized = value.replaceAll(/\s+/g, ' ').trim()
  return normalized.length <= maxLength ? normalized : `${normalized.slice(0, maxLength - 3)}...`
}

function assertPositiveInteger(name: string, value: number, minimum = 1): void {
  if (!Number.isInteger(value) || value < minimum) {
    throw new Error(`tool-skill: ${name} must be an integer greater than or equal to ${minimum}`)
  }
}

/**
 * A whitespace-bounded `/name` token (the public skill-name grammar) anywhere
 * in the text — the same word-boundary shape the transcript chip decoration
 * uses, so a gesture reads as one wherever it sits in the sentence. A second
 * `/` or any non-boundary character breaks the match, which keeps file paths
 * (`/usr/bin`) and fractions (`5/8`) out.
 */
const SKILL_GESTURE = /(^|\s)\/([a-z0-9]+(?:-[a-z0-9]+)*)(?=\s|$)/g

/**
 * `/name` gesture tokens from the claimed user messages, deduplicated in
 * first-seen order. Every text block of direct user input is scanned; no
 * other source can forge a gesture.
 * @param messages - the step's claimed batch.
 * @returns candidate skill names, unvalidated against the registry.
 */
function invokedSkillNames(messages: readonly UserMessage[]): string[] {
  const names: string[] = []
  for (const message of messages) {
    if ((message.source as { kind?: unknown }).kind !== 'user') continue
    for (const block of message.content) {
      if (block.type !== 'text') continue
      for (const match of block.text.matchAll(SKILL_GESTURE)) {
        const name = match[2]
        if (name !== undefined && !names.includes(name)) names.push(name)
      }
    }
  }
  return names
}
