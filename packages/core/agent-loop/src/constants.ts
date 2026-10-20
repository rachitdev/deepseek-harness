/** Shared agent-loop scheduler defaults.
 * @module dsh-agent-loop/constants
 */

/** Default maximum in-flight parallel-safe calls per agent step. */
export const DEFAULT_MAX_PARALLEL_TOOL_CALLS = 10

/** Maximum attempts to resolve a request route through `prepareCall()` before giving up. */
export const MAX_PREPARE_ATTEMPTS = 5
