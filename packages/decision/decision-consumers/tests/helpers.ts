import type { DecisionRequest, DecisionResult, DecisionService } from '@deepseek-ai/dsh-decision'

/** Create a mock decision service responding with scripted answers. */
export function createMockDecisionService(
  evaluator: (request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult>,
): DecisionService {
  return {
    registerProvider: () => () => {},
    evaluate: (request: DecisionRequest, signal?: AbortSignal) => evaluator(request, signal),
  } as unknown as DecisionService
}
