# Cheatsheet — Agentic Design Patterns

## Pattern selection: decision rules

| If… | Then | Why |
|---|---|---|
| the *how* is already known and repeatable | fixed workflow, **not** Planning | autonomy adds uncertainty for no gain (Ch 6) |
| the *how* must be discovered | Planning agent | the plan is created in response to the request |
| steps are ordered and dependent | Prompt Chaining | each output informs the next |
| steps are independent and I/O-bound | Parallelization | latency becomes the slowest branch, not the sum |
| the next step depends on input type | Routing | conditional dispatch with a default branch |
| quality matters more than speed and cost | Reflection with a separate critic | objectivity beats self-review |
| the answer needs current/private data | RAG | grounding + citations |
| the answer needs precise arithmetic | PALMs / code executor | never let the LLM compute |
| information is fragmented across documents | GraphRAG | traditional RAG's main failure mode |
| retrieved sources conflict or may be stale | Agentic RAG | a layer that can *reject* what it retrieved |
| one agent lacks the needed skills or tools | Multi-Agent Collaboration | specialization + division of labor |
| agents span different frameworks | A2A | cross-framework interoperability |
| the tool set is diverse and evolving | MCP | dynamic discovery without redeployment |
| the tool set is small and fixed | plain function calling | MCP is overhead here |
| errors carry safety/ethical/financial weight | Human-in-the-Loop | accountable judgment cannot be delegated |
| the decision is repetitive but policy-bound | Human-**on**-the-loop | human sets policy, AI executes each case |
| you're under a cost or latency budget | Router + cheap/expensive model tiers | Pro for planning, Flash for fetching |

## Thresholds and defaults

| Setting | Value | Source |
|---|---|---|
| Few-shot examples | 3–5, adjust for task complexity and token limits | Ch 22 |
| CoT temperature, single-answer tasks | **0** (greedy decoding) | Ch 22 |
| Self-Consistency temperature | high, to diversify reasoning paths | Ch 22 |
| Guardrail / judge temperature | **0.0–0.2** for deterministic strictness | Ch 18, 19 |
| Reflection loop cap | ~3 iterations plus a sentinel | Ch 4 |
| ADK LoopAgent | `max_iterations` **and** an escalate condition — both | Ch 7 |
| RAG chunking | ~500 chars, 50 overlap as a starting point | Ch 14 |
| RAG retrieval | `similarity_top_k=5`, `vector_distance_threshold=0.7` | Ch 14 |
| Task priorities | closed vocabulary: P0 / P1 / P2 | Ch 20 |
| Missing priority/assignee | default to P1 + a named default worker | Ch 20 |
| Reasoning-model output | thousands of tokens is normal, not a bug | Ch 17 |

## Tells and smells

| If you see… | You're probably… | Fix |
|---|---|---|
| free-form prose passed between chain steps | one bad parse from a silent failure | force JSON + Pydantic validation |
| an agent grading its own work | missing systematic errors | separate critic with a distinct persona |
| a loop with no sentinel or cap | burning budget indefinitely | add both |
| retries on "insufficient funds" | retrying a permanent error | classify transient vs permanent |
| a fallback agent that always runs | doing redundant work | gate it on an explicit state flag |
| an exact string-match accuracy metric | scoring correct answers as 0.0 | semantic similarity or LLM-as-a-Judge |
| metrics printed to console | no alerting, no history | persist to logs/TSDB/warehouse |
| `session.state["x"] = y` after a read | bypassing event processing | `output_key` or `EventActions.state_delta` |
| a user preference stored without `user:` | it vanishes at session end | prefix state keys deliberately |
| an MCP server over a one-record-at-a-time API | slow and inaccurate at volume | add filtering/sorting to the API first |
| an MCP server returning PDFs | agent cannot parse them | return Markdown |
| a review that lists 40 equal findings | not a review | add a reflection pass that prioritizes |
| one agent holding every tool | no specialization | distribute tools by agent role |
| credentials in a URL or message body | leaking | HTTP headers only |
| "Borderline Accept" in a judge's output | unusable verdict | ban hedged decisions |
| the answer printed before the reasoning | CoT doing nothing | reasoning must precede the answer |

## Cost / quality levers, cheapest first

1. **Route by complexity** — cheap model for simple queries (Ch 16)
2. **Prune and summarize context** — prompt tokens are a direct cost line (Ch 16)
3. **Reference-depth output** — omit worked examples where lookup is the use case
4. **Zero-shot CoT** — one phrase, large gain, small cost (Ch 22)
5. **Few-shot examples** — longer prompt, better format adherence (Ch 22)
6. **Reflection loop** — 2× calls per cycle (Ch 4)
7. **Self-Consistency** — N× calls, majority vote (Ch 17)
8. **Tree of Thoughts** — branching exploration, highest cost (Ch 17)
9. **A larger model** — often *worse value* than more inference compute on a smaller one (Scaling Inference Law, Ch 17)

## Multi-agent topology trade-offs

| Topology | Strength | Failure mode | Pick when |
|---|---|---|---|
| Single | simplest | bounded by one agent's scope | sub-problems are independent |
| Network | resilient, peer-to-peer | overhead, incoherent decisions at scale | decentralization > control |
| Supervisor | clear authority, simple control | single point of failure, bottleneck | modest agent count |
| Supervisor-as-Tool | central capability without rigid control | weaker coordination guarantee | competent agents needing shared resources |
| Hierarchical | scales, bounded distributed decisions | latency per layer | the problem itself nests |
| Custom | optimal fit | emergent behavior risk | specific metrics, dynamic environment |

## Trajectory evaluation: how strict?

| Metric | Allows | Use for |
|---|---|---|
| Exact match | nothing extra, exact order | prescribed high-stakes procedures |
| In-order match | extra steps, order preserved | order matters, efficiency doesn't |
| Any-order match | extra steps, any order | genuinely independent steps |
| Precision | — | penalizing unnecessary tool calls |
| Recall | — | ensuring nothing critical is skipped |
| Single-tool use | — | verifying one required capability fired |

## Framework selection

| Need | Framework |
|---|---|
| A → B → C, no loops | LangChain (LCEL) |
| loops, retries, explicit state | LangGraph |
| production multi-agent on GCP | Google ADK |
| a team of role-defined specialists | CrewAI |
| data ingestion and retrieval is the hard part | LlamaIndex |
| production search pipelines at scale | Haystack |
| enterprise .NET/Python code integration | Semantic Kernel |
| lightweight, model-agnostic, native MCP | Strands Agents (AWS) |

## Prompt engineering, in order of leverage

1. **Provide examples** — the single most effective guidance
2. **Be specific about output** — format, length, style, content
3. **Instructions over constraints** — say what to do, not what to avoid
4. **Use action verbs** — Summarize, Extract, Classify, Rank, Compare
5. **Delimit sections** — XML tags or backticks around instruction / context / input
6. **Request structured output** — JSON forces structure and limits hallucination
7. **Add CoT** — "Let's think step by step"; answer *after* reasoning
8. **Iterate and document** — record prompts, configs, results
9. **Store prompts as code** — separate files, version control, automated tests

## Guardrail layers, defence in depth

| Layer | Catches |
|---|---|
| Input validation + moderation API | jailbreaks, off-policy prompts |
| Schema validation (Pydantic) | malformed structured input |
| Behavioral constraints in the prompt | scope drift |
| Tool restrictions + `before_tool_callback` | unauthorized actions |
| Least privilege | limits blast radius when the above fail |
| Output filtering | harmful or non-compliant generations |
| Human oversight / escalation | ambiguity, high-consequence decisions |
| Sandboxing + network boundaries | escapes and lateral movement |
| Output sanitization before UI render | XSS from model output |
