/**
 * `ToolRiskGate`: evaluates proposed tool calls before execution or approval escalation using
 * `ctx.decision`. Quantifies risk levels and human-confirmation probability so policies can
 * automatically allow safe inspections, prompt on mutations, and block destructive actions.
 * @module @deepseek-ai/dsh-decision-consumers/tool-risk-gate
 */

import type { DecisionService } from '@deepseek-ai/dsh-decision'
import type { ToolRiskAssessment } from './types.ts'

/**
 * Options configuring tool risk gating policy.
 */
export interface ToolRiskGateOptions {
  /** Maximum risk score (0..3) permitted for auto-approval; defaults to 1.0 (read-only/safe). */
  readonly maxAutoApproveRisk?: number
  /** Probability above which human confirmation is mandatory; defaults to 0.40. */
  readonly humanConfirmThreshold?: number
}

/**
 * Assess the risk of a proposed tool execution.
 * @param decisionService - the active decision seam service.
 * @param call - the proposed tool name, arguments, and active session policy.
 * @param options - risk thresholds.
 * @param signal - optional cancellation signal.
 * @returns the structured risk assessment.
 */
export async function assessToolRisk(
  decisionService: DecisionService,
  call: {
    readonly toolName: string
    readonly toolArgs: Record<string, unknown>
    readonly sessionPolicy?: string
  },
  options: ToolRiskGateOptions = {},
  signal?: AbortSignal,
): Promise<ToolRiskAssessment> {
  const maxRisk = options.maxAutoApproveRisk ?? 1.0
  const confirmThreshold = options.humanConfirmThreshold ?? 0.4

  const state = [
    `Tool Name: ${call.toolName}`,
    `Tool Arguments: ${JSON.stringify(call.toolArgs, null, 2)}`,
    `Session File/Approval Policy: ${call.sessionPolicy ?? 'default'}`,
  ].join('\n')

  const t0 = performance.now()
  const result = await decisionService.evaluate(
    {
      state,
      questions: {
        risk_score: {
          type: 'score',
          instructions: 'How risky or destructive is this tool call to system files, git history, or external systems?',
          criteria: [
            'read-only-inspection',
            'safe-workspace-scratch-edit',
            'destructive-overwrite-or-deletion',
            'system-critical-unrecoverable',
          ],
        },
        requires_human: {
          type: 'boolean',
          instructions: 'Does this action require explicit human confirmation under safe engineering practices?',
        },
        category: {
          type: 'choice',
          instructions: 'Which primary operational category describes this action?',
          criteria: {
            read: 'Pure inspection, query, or dry-run without persistent mutations',
            local_edit: 'Scoped workspace file change or test execution',
            destructive_edit: 'Deletion, force-overwrite, or uncommitted permanent purge',
            external_action: 'Remote network request, cloud API change, or credential use',
          },
        },
      },
    },
    signal,
  )
  const latencyMs = Math.round(performance.now() - t0)

  const scoreAns = result.answers['risk_score']
  const humanAns = result.answers['requires_human']
  const catAns = result.answers['category']

  if (scoreAns?.type !== 'score' || humanAns?.type !== 'boolean' || catAns?.type !== 'choice') {
    throw new Error('Decision provider returned unexpected answer types for tool risk')
  }

  const riskLevel = scoreAns.score
  const requiresConfirmProb = humanAns.probability
  const catConfidence = catAns.probabilities[catAns.choice] ?? 0

  let recommendedAction: ToolRiskAssessment['recommendedAction']
  if (riskLevel >= 2.5) {
    recommendedAction = 'deny'
  } else if (riskLevel > maxRisk || requiresConfirmProb >= confirmThreshold) {
    recommendedAction = 'prompt'
  } else {
    recommendedAction = 'allow'
  }

  return {
    recommendedAction,
    riskLevel,
    riskCategory: catAns.choice,
    requiresConfirmationProbability: requiresConfirmProb,
    confidence: catConfidence,
    latencyMs,
    providerId: result.providerId,
  }
}
