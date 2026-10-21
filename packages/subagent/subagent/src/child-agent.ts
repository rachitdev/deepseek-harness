/**
 * Shared in-process child composition: the delegation-depth budget, the
 * durable session metadata, the resolved child `AgentOptions`, the delegated
 * policy seed, and the scoped setup used by local activations.
 *
 * @module @deepseek-ai/dsh-subagent/child-agent
 */

import type { Context } from '@deepseek-ai/cordis'
import type { Agent, AgentOptions, CreateAgentOptions } from '@deepseek-ai/dsh-agent'
import type { SandboxMode } from '@deepseek-ai/dsh-sandbox'
import type { Session, SessionId } from '@deepseek-ai/dsh-session'
import type {} from '@deepseek-ai/dsh-system-prompt'
import type { ToolRestriction } from '@deepseek-ai/dsh-tools'
// Type-only: make `ctx.get('sandboxPolicy')`, `ctx.get('approval')`, and
// `ctx.get('permissionPresets')` resolve to their services when composed — delegation consumes them
// opportunistically (the documented `ctx.get` pattern), never as a hard dep —
// and merge the inherited permission session-event payloads.
import type {} from '@deepseek-ai/dsh-sandbox-policy'
import type {} from '@deepseek-ai/dsh-user-approval'
import type {} from '@deepseek-ai/dsh-permission-presets'
// Type-only: make `ctx.get('agentPresets')` resolve to the preset roster when
// composed — a child inherits its parent's composition opportunistically (the
// documented `ctx.get` pattern), never as a hard dep. A rosterless deployment
// keeps its model-facing rows on the host plane, where the child already sees
// them through the tool registry's global layer.
import type {} from '@deepseek-ai/dsh-agent-preset-registry'
import { delegationDepthOf } from './depth.ts'
import { SubagentError } from './error.ts'
import { createDelegationRejectionMessage } from './continuation-messages.ts'

/** Thrown when starting a child would exceed the requested depth cap. */
export class SubagentDepthError extends Error {
  constructor(public readonly attemptedDepth: number, public readonly maxDepth: number) {
    super(`subagent depth ${attemptedDepth} exceeds maxDepth ${maxDepth}`)
    this.name = 'SubagentDepthError'
  }
}

/**
 * Resolve the child's delegation depth from its parent and enforce an optional
 * cap. The persisted parent header is the monotone floor, so a resumed parent
 * cannot delegate as if it were top-level.
 * @param parent - the delegating parent agent.
 * @param maxDepth - optional absolute cap the resolved depth must not exceed.
 * @returns the child's non-negative safe-integer depth.
 * @throws {SubagentDepthError} when the resolved depth exceeds `maxDepth`.
 * @throws {RangeError} when the resolved depth leaves the safe-integer range.
 */
export function resolveChildDepth(parent: Agent, maxDepth: number | undefined): number {
  const childDepth = delegationDepthOf(parent) + 1
  if (!Number.isSafeInteger(childDepth)) {
    throw new RangeError('subagent child depth exceeds the safe-integer range')
  }
  if (maxDepth !== undefined && childDepth > maxDepth) {
    throw new SubagentDepthError(childDepth, maxDepth)
  }
  return childDepth
}

/**
 * Resolve the parent values inherited by a child. The latest request header
 * owns provider, model, and reasoning effort after request-time selection;
 * creation options remain the fallback before the first request and retain
 * the configured output-token limit.
 * @param parent - delegating parent Agent.
 * @returns detached Agent options for child-option merging.
 */
export function parentAgentOptionsForDelegation(parent: Agent): AgentOptions {
  const requestConfig = parent.session.requestHeader()?.config
  if (requestConfig === undefined) return { ...parent.options }
  const {
    provider: _createdProvider,
    model: _createdModel,
    reasoningEffort: _createdReasoningEffort,
    ...createdOptions
  } = parent.options
  return {
    ...createdOptions,
    provider: requestConfig.provider,
    model: requestConfig.model,
    ...requestConfig.reasoningEffort === undefined
      ? {}
      : { reasoningEffort: requestConfig.reasoningEffort },
  }
}

/**
 * Resolve the child's `AgentOptions`: the parent's provider/model,
 * reasoning-effort, and maxTokens values unless the request overrides them,
 * stamped with the child's own delegation depth. Changing the route without
 * naming an effort clears the parent's route-owned effort so the selected
 * model resolves its own default.
 * @param parent - the delegating parent whose route the child inherits.
 * @param requested - per-child overrides, if any.
 * @param childDepth - the resolved delegation depth to stamp.
 * @returns the resolved options for `ctx.agents.create()`.
 */
export function resolveChildAgentOptions(
  parent: Agent,
  requested: AgentOptions | undefined,
  childDepth: number,
): AgentOptions {
  const parentOptions = parentAgentOptionsForDelegation(parent)
  const parentProvider = parentOptions.provider
  const parentModel = parentOptions.model
  const parentReasoningEffort = parentOptions.reasoningEffort
  const parentMaxTokens = parentOptions.maxTokens
  const resolved: AgentOptions = {
    ...parentProvider !== undefined ? { provider: parentProvider } : {},
    ...parentModel !== undefined ? { model: parentModel } : {},
    ...parentReasoningEffort !== undefined ? { reasoningEffort: parentReasoningEffort } : {},
    ...parentMaxTokens !== undefined ? { maxTokens: parentMaxTokens } : {},
    ...requested,
    subagentDepth: childDepth,
  }
  const routeChanged = resolved.provider !== parentProvider || resolved.model !== parentModel
  if (routeChanged && requested?.reasoningEffort === undefined) delete resolved.reasoningEffort
  return resolved
}

/**
 * Build the child session's durable creation metadata: the parent's workspace,
 * its direct lineage, coarse product origin, the recursion budget that must
 * survive persistence, the seed boundary that separates inherited parent
 * history from child work, and the composition the child runs under.
 *
 * Omitted selection uses the parent's live composition, including any switch
 * made while the parent was blank.
 * @param parent - the delegating parent agent.
 * @param childDepth - the resolved delegation depth to persist.
 * @param isSeeded - whether this child inherits a parent-log prefix, including an explicitly empty one.
 * @param selectedPreset - independent selection, or omission to inherit the parent.
 * @returns the `meta` for `ctx.agents.create()`.
 */
export function childSessionMeta(
  parent: Agent,
  childDepth: number,
  isSeeded: boolean,
  selectedPreset?: string,
): NonNullable<CreateAgentOptions['meta']> {
  const parentHeader = parent.session.header
  const agentPreset = selectedPreset ?? parent.ctx.get('agentPresets')?.composedPreset(parent.ctx)
  return {
    ...parentHeader.cwd !== undefined ? { cwd: parentHeader.cwd } : {},
    ...agentPreset === undefined ? {} : { agentPreset },
    parentSession: parentHeader.id,
    isSeeded,
    // Navigation classification only; the descriptor remains the authority
    // for mode and continuation capability.
    origin: 'subagent',
    // Durable: the recursion budget must survive persistence and resume.
    delegationDepth: childDepth,
  }
}

/** The scoped composition a child agent's creation window applies. */
export interface ChildComposition {
  /** Independently mounted preset; omission shares the parent's standing generation. */
  readonly agentPreset?: string | undefined
  /** Historical descriptor persona reapplied only during cold resume. */
  readonly persona?: string | undefined
  /** Historical descriptor tool restriction reapplied only during cold resume. */
  readonly toolFilter?: ToolRestriction | undefined
}

/**
 * Model-facing delegation-scope statement for every in-process child. A
 * runtime-context contribution rather than a system-prompt section, so the
 * deployment's system prompt stays uniform across parents and children.
 */
export const SUBAGENT_DELEGATION_CONTEXT
  = 'You are a delegated subagent: your permission scope was fixed when you were started and cannot be '
    + 'widened from inside this session — operations that require approval are rejected automatically. '
    + 'When the task needs access beyond that scope, do not retry the denied operation; state the '
    + 'limitation in your reply so the delegating agent can handle it.'

/**
 * Compose a child before publication and restore historical descriptor overrides.
 * Explicit selection mounts the named preset; omission synchronously joins the
 * parent's current generation. Registrations belong to the child's scope.
 * @param childCtx - the child agent's scoped creation context.
 * @param parent - the delegating parent used when no preset is selected.
 * @param composition - independent preset and any overrides read from a historical descriptor.
 * @returns A promise when mounting an independent preset; otherwise completes synchronously.
 */
export function applyChildComposition(
  childCtx: Context,
  parent: Agent,
  composition: ChildComposition,
): void | Promise<void> {
  if (composition.agentPreset !== undefined) {
    const presets = childCtx.get('agentPresets')
    if (presets === undefined) {
      throw new SubagentError('selecting a child agent preset requires the agent-presets service', 'PRESET_UNAVAILABLE')
    }
    return presets.mount(childCtx, composition.agentPreset).then(() => {
      applyChildOverrides(childCtx, composition)
    })
  }
  childCtx.get('agentPresets')?.composeFrom(childCtx, parent.ctx)
  applyChildOverrides(childCtx, composition)
}

function applyChildOverrides(childCtx: Context, composition: ChildComposition): void {
  childCtx.systemPrompt.context({
    name: 'subagent:delegation',
    order: childCtx.systemPrompt.getContextOrder('SUBAGENT_DELEGATION'),
    text: SUBAGENT_DELEGATION_CONTEXT,
  })
  if (composition.persona !== undefined) {
    childCtx.systemPrompt.section({
      name: 'deployment:persona-prefix',
      order: childCtx.systemPrompt.getSectionOrder('DEPLOYMENT_PERSONA_PREFIX'),
      text: composition.persona,
    })
  }
  if (composition.toolFilter !== undefined) childCtx.tools.restrict(composition.toolFilter)
}

/** Policy seeded onto a child session's log at the delegation boundary. */
export interface DelegatedPolicyOverrides {
  /**
   * The parent's current preset identity when it runs in Auto or Full access;
   * the child keeps it under the pinned `never` approval policy.
   */
  readonly permissionPreset: 'auto' | 'danger-full-access' | undefined
  /**
   * The parent session's effective sandbox mode, or `undefined` when no
   * sandbox-policy service is composed. Resolved through that service, so a
   * parent confined by the deployment default or by a preset is captured as
   * confined as it actually runs.
   */
  readonly sandboxMode: SandboxMode | undefined
  /**
   * `'never'` whenever the approval capability is composed, `undefined`
   * otherwise: a delegated child acts only within the sandbox scope fixed at
   * delegation, so its asks are rejected deterministically.
   */
  readonly approvalPolicy: 'never' | undefined
}

/**
 * Capture the permission state to seed into one delegation. Call synchronously before
 * the child start's first await: a later parent switch belongs to the
 * parent's future, not to this child. Auto and Full access identities are
 * inherited only through the in-process DSH path so either can replace a stale
 * same-bundle fork value. The sandbox mode is the parent's EFFECTIVE mode —
 * resolved through the sandbox-policy service exactly as the parent's own tool
 * calls resolve it, so a parent confined by a preset or by the deployment
 * default confines its child too — while the approval policy is pinned to
 * `'never'` regardless of the parent's own policy. One-shot grants are never
 * captured: they belong to the parent's finished call.
 * @param parent - the delegating parent agent.
 * @returns the effective sandbox mode (or `undefined` without the service) and the approval pin.
 */
export function captureDelegatedPolicyOverrides(parent: Agent): DelegatedPolicyOverrides {
  const preset = parent.ctx.get('permissionPresets')?.current(parent.session)
  return {
    permissionPreset: preset === 'auto' || preset === 'danger-full-access' ? preset : undefined,
    sandboxMode: parent.ctx.get('sandboxPolicy')?.resolve({ session: parent.session }).mode,
    approvalPolicy: parent.ctx.get('approval') === undefined ? undefined : 'never',
  }
}

/**
 * Append the captured delegation policy onto the child's own log as
 * `source: 'delegation'` events inside the unpublished creation window, so the
 * child's effective policy is reconstructable from its log alone. Appends land
 * after any fork seed, so fresh policy wins stale seed state; later child
 * switches still win over these events.
 * @param childSession - the unpublished child's session.
 * @param overrides - the policy captured at delegation.
 */
export function appendDelegatedPolicyOverrides(
  childSession: Session,
  overrides: DelegatedPolicyOverrides,
): void {
  if (overrides.sandboxMode !== undefined) {
    childSession.append('sandbox/mode', { mode: overrides.sandboxMode, source: 'delegation' })
  }
  if (overrides.approvalPolicy !== undefined) {
    childSession.append('approval/policy', { policy: overrides.approvalPolicy, source: 'delegation' })
  }
  if (overrides.permissionPreset !== undefined) {
    childSession.append('permission/preset', { preset: overrides.permissionPreset })
  }
}

/**
 * Map a decided approval request back to the tool its ask named. Every
 * decision — granted, rejected, cancelled, or unavailable — consumes its ask,
 * so the tracking stays bounded by in-flight asks rather than by all asks a
 * residency ever made.
 */
export class PendingApprovalAsks {
  private readonly names = new Map<string, string>()

  /**
   * Record one ask.
   * @param id - the request id pairing this ask with its decision.
   * @param toolName - the tool the ask is about.
   */
  asked(id: string, toolName: string): void {
    this.names.set(id, toolName)
  }

  /**
   * Consume the ask behind one decision.
   * @param id - the decided request id.
   * @returns the asked tool name, or `undefined` for an unpaired decision.
   */
  decide(id: string): string | undefined {
    const toolName = this.names.get(id)
    this.names.delete(id)
    return toolName
  }

  /** How many asks still await a decision. */
  get pending(): number {
    return this.names.size
  }
}

/** Per-child state the caller owns across one delegation-notice subscription. */
export interface DelegationRejectionSubscription {
  /**
   * Whether the delegation-sourced `never` pin is the child's latest policy.
   * The caller seeds it from its own creation-time knowledge and the observer
   * writes every observed policy back here, so the retained fact tracks
   * reality for the next residency instead of the creation-time value.
   */
  pinned: boolean
  /**
   * Tool names already reported for this child. The caller owns the set, so it
   * outlives one residency epoch and a resume does not notify twice; it is
   * dropped with the rest of the caller's record when the child is released.
   */
  readonly reported: Set<string>
}

/**
 * Report each auto-rejected approval ask from a delegated child to its direct
 * parent as one runtime notice, so a refusal the child cannot undo is visible
 * where the scope was chosen. Reports under the delegation-sourced `'never'`
 * pin only: a rejection under any other policy, or under a `never` the child
 * switched to itself, belongs to whoever set or answered it. One notice per
 * reported tool name keeps a retrying child quiet.
 * @param child - the locally created delegated child to observe.
 * @param subscription - the pin this residency began under and the tools already reported.
 * @param deliver - delivery hook for the parent-visible message.
 */
export function observeDelegatedApprovalRejections(
  child: Agent,
  subscription: DelegationRejectionSubscription,
  deliver: (message: ReturnType<typeof createDelegationRejectionMessage>) => void,
): void {
  if (child.ctx.get('approval') === undefined) return
  const asks = new PendingApprovalAsks()
  child.ctx.on('session/event', (session, event) => {
    if (session !== child.session) return
    if (event.type === 'approval/policy') {
      subscription.pinned = event.data.policy === 'never' && event.data.source === 'delegation'
      return
    }
    if (event.type === 'approval/asked') {
      asks.asked(event.data.id, event.data.toolName)
      return
    }
    if (event.type !== 'approval/decided') return
    const toolName = asks.decide(event.data.id)
    if (!subscription.pinned || event.data.outcome !== 'rejected' || toolName === undefined) return
    if (subscription.reported.has(toolName)) return
    deliver(createDelegationRejectionMessage(child.session.id, toolName))
    subscription.reported.add(toolName)
  })
}

/** Identity and lineage inputs shared by every in-process child creation. */
export interface ChildCreateInputs {
  /** The child's reserved session id. */
  readonly sessionId: SessionId
  /** The delegating parent agent. */
  readonly parent: Agent
  /** The resolved delegation depth. */
  readonly childDepth: number
  /** How many leading seed events came from the parent's log. */
  readonly lineageSeedLength: number
}
