/**
 * Shared types for the decision consumers: the 11 decision-driven plugins and workflows that branch
 * on typed judgements from `ctx.decision`.
 * @module @deepseek-ai/dsh-decision-consumers/types
 */

/** Candidate action the ego-browser driver decides between. */
export interface EgoCandidateElement {
  readonly id: string
  readonly label: string
  readonly description?: string
}

/** Result of an ego-browser navigation step decision. */
export interface EgoStepDecision {
  readonly action:
    | { readonly type: 'click'; readonly elementId: string }
    | { readonly type: 'complete' }
    | { readonly type: 'escalate'; readonly reason: string }
  readonly confidence: number
  readonly probabilities: Readonly<Record<string, number>>
  readonly taskComplete: boolean
  readonly pageBlocked: boolean
  readonly riskLevel: number
  readonly latencyMs: number
  readonly providerId: string
}

/** Assessment of proposed tool execution risk. */
export interface ToolRiskAssessment {
  readonly recommendedAction: 'allow' | 'prompt' | 'deny'
  readonly riskLevel: number
  readonly riskCategory: string
  readonly requiresConfirmationProbability: number
  readonly confidence: number
  readonly latencyMs: number
  readonly providerId: string
}

/** Model tier decision for a conversation turn. */
export interface ModelRouteDecision {
  readonly selectedTier: 'fast' | 'frontier'
  readonly recommendedModel: string
  readonly confidence: number
  readonly complexityScore: number
  readonly needsDeepReasoning: boolean
  readonly latencyMs: number
  readonly providerId: string
}

/** One candidate document or chunk for retrieval filtering. */
export interface RetrievalCandidate {
  readonly id: string
  readonly content: string
  readonly title?: string
}

/** Scored and filtered retrieval item. */
export interface ScoredRetrievalItem {
  readonly id: string
  readonly relevanceScore: number
  readonly keepProbability: number
  readonly retained: boolean
}

/** Result of filtering retrieved items. */
export interface FilteredRetrievalResult {
  readonly retained: readonly ScoredRetrievalItem[]
  readonly dropped: readonly ScoredRetrievalItem[]
  readonly latencyMs: number
  readonly providerId: string
}

/** One recalled memory item as injected into model context. */
export interface RecallScreenItem {
  /** Stable item address, kept in the state so the provider can judge where the item came from. */
  readonly uri: string
  /** Item body; empty when the recall carried only the address. */
  readonly content: string
}

/** Screening outcome for one recalled item. */
export type RecallItemJudgement =
  | {
    readonly uri: string
    readonly kind: 'judged'
    /** Position on the 0..3 usefulness rubric (`irrelevant`..`essential`). */
    readonly relevance: number
    /** Probability that the item is a stored prompt-injection attempt, when screened. */
    readonly injectionProbability?: number
  }
  | {
    readonly uri: string
    /** The provider refused the single-item request itself, so the item was never judged. */
    readonly kind: 'rejected'
  }

/** Item considered during transcript compaction. */
export interface CompactionTranscriptItem {
  readonly id: string
  readonly role: string
  readonly summary: string
  readonly bytes: number
}

/** Prioritized compaction plan. */
export interface CompactionPlan {
  readonly keepIds: readonly string[]
  readonly dropIds: readonly string[]
  readonly summarizeIds: readonly string[]
  readonly retainedBytes: number
  readonly prunedBytes: number
  readonly latencyMs: number
  readonly providerId: string
}

/** Summary of one recent agent action. */
export interface RecentAgentAction {
  readonly tool: string
  readonly inputSummary: string
  readonly resultSummary: string
  readonly error?: boolean
}

/** Loop stall assessment. */
export interface LoopStallVerdict {
  readonly stalled: boolean
  readonly loopState: string
  readonly stallSeverity: number
  readonly stallProbability: number
  readonly suggestedIntervention: 'continue' | 'warn' | 'interrupt'
  readonly latencyMs: number
  readonly providerId: string
}

/** One browser operation the policy can choose. */
export type BrowserOperation =
  | 'CLICK'
  | 'TYPE_TEXT'
  | 'SELECT'
  | 'SCROLL_DOWN'
  | 'SCROLL_UP'
  | 'WAIT'
  | 'DONE'
  | 'BLOCKED'

/** One interactive control observed on the current page. */
export interface BrowserObservedControl {
  /** Stable index for this control within the current observation. */
  readonly id: string
  /** Accessible role, such as `button` or `combobox`. */
  readonly role: string
  /** Accessible name or visible label. */
  readonly label: string
  /** Current field value, when the control holds one. */
  readonly value?: string
  /** Current state for a checkbox, switch, or radio. */
  readonly checked?: boolean
  /** Operations this control supports; a control offers an operation or it is not a candidate. */
  readonly operations: readonly BrowserOperation[]
}

/** The goal, observed page, and recent history a browser action is chosen from. */
export interface BrowserActionPolicyRequest {
  readonly goal: string
  readonly pageUrl: string
  readonly pageTitle: string
  /** Visible page text; treated as untrusted data by the policy instructions. */
  readonly pageText: string
  readonly controls: readonly BrowserObservedControl[]
  /** Recent executed actions, oldest first. */
  readonly history: readonly string[]
}

/** The chosen browser operation and its resolved target. */
export interface BrowserActionDecision {
  readonly operation: BrowserOperation
  /** Control id to act on; absent for operations that need no element. */
  readonly targetId?: string
  readonly confidence: number
  /** Whether confidence reached the threshold; false means observe again rather than act. */
  readonly executable: boolean
  readonly targetProbabilities?: Readonly<Record<string, number>>
  readonly latencyMs: number
  readonly providerId: string
}

/** Repository instructions paired with the work to judge against them. */
export interface InstructionDriftEvidence {
  /** The binding instruction text, typically the concatenated `AGENTS.md` files in scope. */
  readonly instructions: string
  /** What the agent did: changed files, commands run, and decisions taken. */
  readonly workSummary: string
}

/** Advisory judgement that work diverged from the repository instructions. */
export interface InstructionDriftVerdict {
  readonly drifted: boolean
  readonly driftProbability: number
  /** Distance from the threshold the probability was judged against; negative means below it. */
  readonly thresholdMargin: number
  readonly latencyMs: number
  readonly providerId: string
}

/** Subagent task brief and evaluation. */
export interface SubagentEvaluation {
  readonly verdict: 'accept' | 'retry' | 'escalate'
  readonly confidence: number
  readonly qualityScore: number
  readonly criteriaMet: boolean
  readonly feedbackRecommendation?: string
  readonly latencyMs: number
  readonly providerId: string
}

/** Selected pre-push verification gates. */
export interface GateSelection {
  readonly recommendedSuite: 'doc_only' | 'lint_and_unit' | 'full_coverage' | 'full_with_e2e'
  readonly commands: readonly string[]
  readonly confidence: number
  readonly blastRadius: number
  readonly needsCoverage: boolean
  readonly latencyMs: number
  readonly providerId: string
}

/** One candidate agent skill offered for selection. */
export interface SkillCandidate {
  /** Skill catalog name that the caller will load on `load_skill`. */
  readonly name: string
  /** One-line description the decision provider judges against. */
  readonly description: string
}

/** Selected next step for one user turn against an available skill catalog. */
export interface SkillSelection {
  readonly action:
    | { readonly type: 'load_skill'; readonly skill: string }
    | { readonly type: 'load_skill_with_approval'; readonly skill: string }
    | { readonly type: 'direct_answer' }
    | { readonly type: 'clarify'; readonly reason: string }
  /** Probability that the turn needs a listed skill at all. */
  readonly needsSkillProbability: number
  /** Position (0..2) on the modification-risk rubric for acting on this turn. */
  readonly riskScore: number
  /** Probability of the chosen skill option, or of `other`/`none` when no skill was chosen. */
  readonly confidence: number
  readonly latencyMs: number
  readonly providerId: string
}

/** One fetched web document to guard before it reaches model context. */
export interface FetchedWebContent {
  /** Source URL, kept in the state so the provider can judge which site served the text. */
  readonly url: string
  /** Document title, when the fetcher surfaced one. */
  readonly title?: string
  /** Raw document text or excerpt. */
  readonly content: string
}

/** Guardrail result for one fetched web document. */
export interface FetchGuardrailVerdict {
  readonly verdict: 'pass' | 'review' | 'block'
  /** Probability that the content attempts to override harness instructions. */
  readonly injectionProbability: number
  /** Probability that the content tries to harvest credentials or secrets. */
  readonly credentialHarvestProbability: number
  /**
   * Margin between the larger hazard probability and the policy threshold it was judged against,
   * in `[0, 1]`: larger means further from flipping to the next band.
   */
  readonly thresholdMargin: number
  readonly latencyMs: number
  readonly providerId: string
}

/** Result of one parallel model canary probe during failover. */
export interface FailoverCandidateProbe {
  /** Registered LLM provider serving the probe. */
  readonly provider: string
  /** Provider-owned model id. */
  readonly model: string
  /** Probe round-trip time. */
  readonly latencyMs: number
  /** Input and output tokens consumed by the probe. */
  readonly tokenCount: number
  /** Bounded model response judged for task fitness. */
  readonly probeOutput: string
  /** Terminal probe failure, when this candidate is unhealthy. */
  readonly error?: string
}

/** Recommended exact model connection after parallel probing and decision arbitration. */
export interface FailoverDecision {
  /** Registered provider selected for takeover. */
  readonly recommendedProvider: string
  /** Provider-owned model selected for takeover. */
  readonly recommendedModel: string
  /** Choice probability assigned to the selected exact route. */
  readonly confidence: number
  /** Selected route's task-relative token-efficiency score. */
  readonly tokenEfficiencyScore: number
  /** Selected route's task-relative code-validity score. */
  readonly codeValidityScore: number
  /** Selected route's latency-penalty score, where a higher value is better. */
  readonly latencyPenaltyScore: number
  /** Decision-provider round-trip time. */
  readonly latencyMs: number
  /** Decision provider that made the judgement. */
  readonly providerId: string
}
