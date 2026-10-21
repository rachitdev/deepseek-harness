import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime from '@deepseek-ai/dsh-decision'
import type { DecisionRequest, DecisionResult, DecisionService } from '@deepseek-ai/dsh-decision'

/** A decision service whose `evaluate` answers from a script instead of a selected provider. */
class ScriptedDecisionService extends DecisionRuntime {
  readonly #evaluator: (request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult>

  /**
   * @param evaluator - the scripted answer for every request, called with the consumer's own arguments.
   */
  constructor(evaluator: (request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult>) {
    super(new Context())
    this.#evaluator = evaluator
  }

  /** @returns the scripted answer, without provider selection or request validation. */
  override evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult> {
    return this.#evaluator(request, signal)
  }
}

/** Create a mock decision service responding with scripted answers. */
export function createMockDecisionService(
  evaluator: (request: DecisionRequest, signal?: AbortSignal) => Promise<DecisionResult>,
): DecisionService {
  return new ScriptedDecisionService(evaluator)
}
