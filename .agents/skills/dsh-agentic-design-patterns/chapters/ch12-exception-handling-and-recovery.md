# Chapter 12: Exception Handling and Recovery

## Core Idea
Real environments guarantee failure — tool errors, unavailable services, malformed outputs, network faults. The pattern makes agents durable by combining proactive detection, graceful handling, and explicit recovery to a stable state, rather than crashing or silently producing nonsense.

## Frameworks Introduced
- **The three-stage pattern** — the chapter's core structure:
  - **Error Detection** — identify operational issues as they arise:
    - invalid or malformed tool outputs
    - specific API error codes (404 Not Found, 500 Internal Server Error, 503 Service Unavailable)
    - unusually long response times from services or APIs (timeouts)
    - incoherent or nonsensical responses deviating from expected formats
    - monitoring by other agents or specialized monitoring systems, for proactive anomaly detection before issues escalate
  - **Error Handling** — the response plan once detected:
    - **Logging** — record error details meticulously for later debugging and analysis
    - **Retries** — reattempt the action, sometimes with adjusted parameters; suited to *transient* errors
    - **Fallbacks** — alternative strategies or methods that maintain some functionality
    - **Graceful degradation** — maintain partial functionality to provide at least some value when full recovery isn't possible
    - **Notification** — alert human operators or other agents when intervention or collaboration is required
  - **Recovery** — restore the agent or system to a stable, operational state:
    - **State rollback** — reverse recent changes or transactions to undo the error's effects
    - **Diagnosis** — thorough investigation of the cause, to prevent recurrence
    - **Self-correction / replanning** — adjust the agent's plan, logic, or parameters to avoid the same error in future
    - **Escalation** — delegate to a human operator or higher-level system in complex or severe cases
  - When to use: any agent deployed in a dynamic, real-world environment where system failures, tool errors, network issues, or unpredictable inputs are possible and operational reliability is a key requirement.
- **Layered handler chain**: a `SequentialAgent` of `[primary_handler, fallback_handler, response_agent]`, where each stage reads shared state to decide whether it should act.
- **Reflection-driven retry**: when an initial attempt raises an exception, a reflective process analyzes the failure and reattempts with a refined approach — an improved prompt, different parameters — rather than blindly retrying the same call.

## Key Concepts
- **Transient vs persistent error**: retries help the first, waste budget on the second. "Not repeatedly trying the same invalid trade" is the canonical example.
- **Controlled failure**: when recovery is impossible, failing predictably and informatively still counts as success for this pattern.
- **State flag as the coordination channel** (`state["primary_location_failed"]`): how downstream handlers learn whether upstream succeeded, without exceptions crossing agent boundaries.
- **Skip-and-continue**: for batch work, a bad item should be logged and skipped, not halt the whole run.
- **Sensor feedback as detection**: in robotics, failure detection comes from the physical world, not an error code.
- **Operational integrity**: the property the pattern preserves — the agent stays functional and trustworthy despite anomalies.

## Mental Models
- **Detect → Handle → Recover** is the whole pattern; most fragile agents implement only the middle stage, or none.
- **Every failure has a designed outcome.** Retry, fall back, degrade, notify, roll back, or escalate — but never "undefined".
- **Layer handlers instead of nesting try/except in the model's head**: a primary agent, a fallback agent, and a presenter agent are individually simple and jointly robust.
- Exception handling **transforms agents from fragile and unreliable systems into robust, dependable components** — the framing to use when justifying the engineering effort.

## Anti-patterns
- **Crashing on a downed dependency**: a chatbot whose database is temporarily unavailable should report the issue, suggest retrying, or escalate — not fail.
- **Retrying a permanent error**: "insufficient funds" and "market closed" are not transient; log, stop, and notify or adjust strategy.
- **Halting a batch on one bad item**: skip the corrupted file, log it, keep processing, and report skipped items at the end.
- **No timeout**: an unusually long response is an error condition and must be detected as one.
- **Silent fallback**: degrading without telling the user leaves them acting on partial information.
- **No fallback path at all**: a single handler with no alternative is a single point of failure.
- **A presenter that assumes success**: the final agent must handle the case where the state key does not exist or is empty.

## Code Examples
ADK — layered primary / fallback / presenter chain:
```python
from google.adk.agents import Agent, SequentialAgent

# Agent 1: tries the primary tool. Narrow and clear.
primary_handler = Agent(
    name="primary_handler",
    model="gemini-2.0-flash-exp",
    instruction="""
Your job is to get precise location information.
Use the get_precise_location_info tool with the user's provided address.
""",
    tools=[get_precise_location_info])

# Agent 2: the fallback, gated on a state flag.
fallback_handler = Agent(
    name="fallback_handler",
    model="gemini-2.0-flash-exp",
    instruction="""
Check if the primary location lookup failed by looking at state["primary_location_failed"].
- If it is True, extract the city from the user's original query and use the
  get_general_area_info tool.
- If it is False, do nothing.
""",
    tools=[get_general_area_info])

# Agent 3: presents whatever ended up in state — including nothing.
response_agent = Agent(
    name="response_agent",
    model="gemini-2.0-flash-exp",
    instruction="""
Review the location information stored in state["location_result"].
Present this information clearly and concisely to the user.
If state["location_result"] does not exist or is empty, apologize that you could not
retrieve the location.
""",
    tools=[])   # reasons over final state only

robust_location_agent = SequentialAgent(
    name="robust_location_agent",
    sub_agents=[primary_handler, fallback_handler, response_agent])
```
- **What it demonstrates**: three things worth copying. The fallback is **explicitly told to do nothing** when the primary succeeded — otherwise it runs redundantly. The failure signal travels through **session state**, not an exception, so each agent stays independently simple. And the presenter has an **explicit empty-state branch**, so total failure still produces a coherent user-facing message rather than a crash or a hallucinated location.

## Reference Tables

| Stage | Mechanisms |
|---|---|
| Detection | malformed tool output, API error codes (404/500/503), timeouts, incoherent responses, external monitoring agents |
| Handling | logging, retries (transient only), fallbacks, graceful degradation, notification |
| Recovery | state rollback, diagnosis, self-correction/replanning, escalation |

| Domain | Failure | Correct response |
|---|---|---|
| Customer service chatbot | customer database temporarily down | detect API error, inform user, suggest retry later, or escalate to a human |
| Automated trading | "insufficient funds" / "market closed" | log; do **not** retry the invalid trade; notify user or adjust strategy |
| Smart home | light fails on network issue or device malfunction | retry; if still failing, notify user and suggest manual intervention |
| Data processing | corrupted file in a batch | skip it, log it, continue the batch, report skipped files at the end |
| Web scraping | CAPTCHA, changed structure, 404/503 | pause, use a proxy, or report the specific failing URL |
| Robotics / manufacturing | arm fails to pick a component (misalignment) | detect via sensor feedback, readjust, retry, then alert an operator or switch component |

| Error type | Right handler |
|---|---|
| Transient (network blip, 503, rate limit) | retry, possibly with backoff |
| Permanent-but-expected (insufficient funds, closed market) | log + notify, do not retry |
| Data-level (corrupted item in a batch) | skip, log, continue, report |
| Structural (API changed, tool gone) | fallback, then escalate |
| Unknown / severe | roll back state, escalate to a human |

## Worked Example
**Robust location lookup.** A naive agent calls `get_precise_location_info` and, when the address is unparseable or the geocoding service errors, either crashes or returns an invented location. The layered version handles it in three stages that each read and write shared state:

1. **`primary_handler`** attempts precise lookup. On failure it sets `state["primary_location_failed"] = True`; on success it writes `state["location_result"]`.
2. **`fallback_handler`** runs unconditionally but is instructed to check that flag first. If the primary failed, it degrades gracefully — extract just the *city* from the original query and call `get_general_area_info`, producing an approximate answer instead of none. If the primary succeeded, it explicitly does nothing.
3. **`response_agent`** presents `state["location_result"]`, whatever produced it, and apologizes if it's absent.

The user experience across the three outcomes: precise location, approximate area, or an honest apology. There is no fourth outcome where the agent crashes or fabricates — which is the entire point.

## Key Takeaways
1. Exception Handling and Recovery is essential for robust, reliable agents.
2. The pattern is detect → handle gracefully → recover to a stable state.
3. Detection uses tool-output validation, API error codes, and timeouts, plus proactive monitoring by other agents.
4. Handling strategies: logging, retries, fallbacks, graceful degradation, notification.
5. Recovery focuses on restoring stable operation through rollback, diagnosis, self-correction, or escalation.
6. Retry transient errors only; permanent errors need logging, notification, or a strategy change.
7. Coordinate layered handlers through shared state, and gate each fallback explicitly so it doesn't run redundantly.
8. Always give the final presenter an empty-state branch.
9. Combine with Reflection: analyze the failure and retry with a refined approach rather than repeating the same call.

## Connects To
- **Ch 4 (Reflection)**: analyzing a failure and retrying with an improved prompt.
- **Ch 5 (Tool Use)**: tools should raise specific errors so this pattern has something to detect.
- **Ch 11 (Goal Setting and Monitoring)**: monitoring detects deviation; escalation is the shared exit.
- **Ch 13 (Human-in-the-Loop)**: escalation's destination.
- **Ch 18 (Guardrails / Safety)**: preventing bad actions vs recovering from failed ones.
- **Ch 19 (Evaluation and Monitoring)**: logs and error rates are the raw material for evaluation.
