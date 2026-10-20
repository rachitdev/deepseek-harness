# Patterns — Agentic Design Patterns

The 21 core patterns plus the supporting techniques, in applicable form.

---

## Prompt Chaining (Pipeline)
**When to use**: A task is too complex for one prompt, has distinct stages, needs tools between steps, or must maintain state.
**How**: Decompose into ordered sub-tasks → one focused prompt each (optionally with a distinct role per stage) → force structured output (JSON) at each boundary → insert deterministic logic between calls → feed validated output forward.
**Trade-offs**: More calls, more latency. Buys debuggability, per-step optimization, and seams for validation and tool use.

## Routing
**When to use**: The agent must choose between distinct workflows, tools, or sub-agents — triage and classification.
**How**: Enumerate destinations plus an "unclear" fallback → pick a mechanism (LLM / embedding / rule / fine-tuned classifier) → emit one normalized identifier → branch on it, passing the *original* input to the handler → always define a default.
**Trade-offs**: LLM routing is flexible but costs latency and determinism; rules are fast but brittle; classifiers need labeled data.

## Parallelization
**When to use**: Multiple independent operations, especially I/O-bound calls to external services.
**How**: Map the dependency graph → fan out independents concurrently → carry the original input through with a passthrough → converge on one aggregation node → ground the synthesizer exclusively on branch outputs.
**Trade-offs**: Substantial added complexity in design, debugging, and logging. `asyncio` gives concurrency, not CPU parallelism.

## Reflection
**When to use**: Quality, accuracy, and detail matter more than speed and cost — long-form content, code, detailed plans.
**How**: Execute → critique against explicit criteria → refine → iterate. Use a **separate critic persona** for objectivity. Require a critique in structured form and an unambiguous stop sentinel; cap iterations.
**Trade-offs**: Every loop is another LLM call — latency, cost, growing context, throttling risk.

## Tool Use (Function Calling)
**When to use**: The agent must leave its training data — real-time data, private data, precise calculation, code execution, real-world actions.
**How**: Define tools with clear descriptions and typed parameters → the model decides and emits a structured call → the framework executes → the result returns as an observation → the model continues.
**Trade-offs**: Each tool is attack surface and failure surface. Return clean typed data and raise specific errors rather than encoding failure in strings.

## Planning
**When to use**: A request is too complex for a single action and needs a sequence of interdependent operations.
**How**: Establish initial and goal state → decompose into executable sub-goals in dependency order → execute while monitoring → on a blocked step, register the constraint and re-plan.
**Trade-offs**: Autonomy costs predictability. **If the "how" is already known, use a fixed workflow instead.**

## Multi-Agent Collaboration
**When to use**: A task decomposes into sub-tasks needing specialized skills or tools; diverse expertise, parallelism, or multi-stage workflow.
**How**: Delineate roles and responsibilities → establish communication channels → define the task flow. Choose a topology: Single, Network, Supervisor, Supervisor-as-Tool, Hierarchical, or Custom.
**Trade-offs**: Coordination overhead and communication cost; requires a standardized protocol and shared ontology to stay coherent.

## Memory Management
**When to use**: Any agent doing more than one-shot Q&A — conversational context, multi-step progress, personalization, learning.
**How**: Short-term in the context window (summarize older segments); long-term in an external vector/graph store retrieved by semantic search and injected into context.
**Trade-offs**: Persistence costs infrastructure; retrieval costs latency; context costs tokens every turn.

## Learning and Adaptation
**When to use**: Dynamic, uncertain, or evolving environments; personalization; continuous improvement; novel situations.
**How**: Pick a mechanism (RL, supervised, unsupervised, few-shot, online, memory-based). For self-modification: score past versions on success + time + cost, select the best, let it modify itself, benchmark, archive, repeat.
**Trade-offs**: Purpose drift, reward hacking, instability. Requires isolation (containers), an overseer that can halt, and observability.

## Model Context Protocol (MCP)
**When to use**: Complex, scalable, or enterprise systems needing a diverse and evolving set of tools; interoperability across LLMs; dynamic capability discovery without redeployment.
**How**: A server exposes resources, tools, and prompts; a client discovers via manifest, formulates a request, the server authenticates and executes, the response updates the client's context.
**Trade-offs**: Does not fix a bad underlying API. Add filtering and sorting; return agent-readable formats. For a fixed small tool set, plain function calling is enough.

## Goal Setting and Monitoring
**When to use**: An agent must autonomously execute a multi-step task and reliably achieve a high-level objective without constant supervision.
**How**: State the goal as SMART → define metrics and success criteria → monitor actions, environment state, and tool outputs → feed deviations back → cap iterations.
**Trade-offs**: An LLM judging its own work misreads goals and hallucinates success. Separate the judge from the author.

## Exception Handling and Recovery
**When to use**: Any agent in a dynamic real-world environment where failures are possible and reliability is required.
**How**: **Detect** (malformed output, API error codes, timeouts, incoherent responses, external monitors) → **Handle** (log, retry transients, fall back, degrade gracefully, notify) → **Recover** (rollback state, diagnose, self-correct, escalate).
**Trade-offs**: Retrying permanent errors wastes budget; silent degradation misleads users. Every failure needs a designed outcome.

## Human-in-the-Loop
**When to use**: Errors carry safety, ethical, or financial consequences; ambiguity and nuance beyond reliable LLM handling; generating high-quality training data.
**How**: Oversight (logs, dashboards) + intervention/correction + feedback for learning + decision augmentation + collaboration + **explicit escalation policies stating when to hand off**.
**Trade-offs**: Fundamentally unscalable — operators cannot handle millions of tasks. Depends on operator expertise. Raises privacy obligations requiring anonymization. Use hybrid: automation for volume, HITL for accuracy.

## Knowledge Retrieval (RAG)
**When to use**: Answers need specific, current, or proprietary information absent from training data, with citations.
**How**: Chunk documents preserving context → embed and store in a vector DB → semantic search (or hybrid with BM25) → augment the prompt with retrieved chunks → generate, grounded, with an explicit "say you don't know" clause.
**Trade-offs**: Fails when information is fragmented across chunks or documents; irrelevant chunks add noise; the knowledge base needs ongoing reconciliation; adds latency, cost, and prompt tokens.

## Inter-Agent Communication (A2A)
**When to use**: Orchestrating collaboration between agents, especially across frameworks; dynamic discovery of other agents' capabilities.
**How**: Publish an Agent Card (identity, URL, capabilities, skills, auth) → discover via well-known URI, registry, or direct config → exchange asynchronous tasks with messages and artifacts over JSON-RPC 2.0 → pick sync / polling / SSE / webhook by task duration.
**Trade-offs**: Protocol overhead; requires securing card endpoints, mTLS, audit logs, and header-based credentials.

## Resource-Aware Optimization
**When to use**: Strict budgets, latency-sensitive applications, constrained hardware, or workflows whose tasks have varying resource needs.
**How**: A Router classifies request complexity → dispatches to the cheapest sufficient model or tool → a Critique agent scores results and refines routing over time → fallbacks handle unavailability.
**Trade-offs**: The router itself costs a call; over-aggressive downgrading degrades quality. Prompt-tune or fine-tune the router on query→model pairs.

## Reasoning Techniques
**When to use**: Problems too complex for a single pass — decomposition, multi-step logic, external interaction, strategic planning, or where showing the work matters.
**How**: CoT for linear decomposition; ToT for branching with backtracking; Self-Correction for refinement; PALMs for precise computation; ReAct for environment interaction; CoD/GoD for multi-model debate.
**Trade-offs**: All buy accuracy with inference compute. Per the Scaling Inference Law, that trade is often better than a larger model.

## Guardrails / Safety Patterns
**When to use**: Any application where output can impact users, systems, or reputation — customer-facing agents, content platforms, finance, healthcare, legal.
**How**: Layer input validation, prompt-level behavioral constraints, tool restrictions, output filtering, external moderation, and human oversight. Screen with a fast cheap model at temperature 0. Force structured JSON verdicts and validate them with a schema.
**Trade-offs**: Over-blocking harms utility; under-blocking harms users. Guardrails need continuous monitoring and refinement as risks evolve.

## Evaluation and Monitoring
**When to use**: Live production deployments, version comparison, regulated domains, drift-prone environments, or when behavior quality must be assessed.
**How**: Track accuracy (semantically, not by string match), latency, and token usage to persistent storage. Compare trajectories against ground truth with an appropriately strict metric. Use LLM-as-a-Judge with a rubric for subjective quality. Unit-test with test files, integration-test with evalsets.
**Trade-offs**: Human evaluation captures nuance but doesn't scale; LLM-as-a-Judge scales but misses intermediate steps; automated metrics are objective but partial.

## Prioritization
**When to use**: Multiple, often conflicting tasks or goals under resource constraints in a dynamic environment.
**How**: Define criteria (urgency, importance, dependencies, resource availability, cost/benefit, user preference) → evaluate candidates → select via scheduling logic → **re-prioritize dynamically** as circumstances change.
**Trade-offs**: A closed priority vocabulary must be validated in code; document defaults so ambiguity produces a decision rather than a stall.

## Exploration and Discovery
**When to use**: Open-ended or rapidly evolving domains where the solution space is undefined; generating novel hypotheses, strategies, or insights; uncovering unknown unknowns.
**How**: Generate → debate → evolve, with specialized agents (generation, reflection, ranking via Elo tournament, evolution, proximity clustering, meta-review) under a supervisor, plus test-time compute scaling.
**Trade-offs**: Bounded by accessible literature; missing negative results; inherits LLM hallucination. Requires safety review of both goals and generated hypotheses.

---

## Supporting Techniques

### Context Engineering
**When to use**: Any serious agentic system.
**How**: Assemble system prompt + retrieved documents + tool outputs + implicit data (identity, history, environmental state) + memory, via runtime pipelines with feedback loops.
**Trade-offs**: Implicit data raises privacy and governance obligations, especially in enterprise, healthcare, and finance.

### Structured Output with Schema Validation
**When to use**: Any LLM output consumed by another component.
**How**: Request JSON with named keys → parse and validate with Pydantic (`model_validate_json`) → check logical constraints → raise on mismatch.
**Trade-offs**: None worth mentioning. This is a necessity for automation, not a convenience.

### Agent as a Tool
**When to use**: A specialized capability warrants its own agent but should be callable like a function.
**How**: Wrap the specialist in `AgentTool` with a description written as selection criteria; add it to the orchestrator's tool list.
**Trade-offs**: Adds a layer of latency; the wrapper's description is the only interface contract.

### Programmatic Prompt Optimization (DSPy-style)
**When to use**: A high-volume prompt whose quality materially affects outcomes, with a measurable objective.
**How**: Build a goldset of input/output pairs and an objective function → optimize few-shot example selection and instruction phrasing with a Bayesian optimizer.
**Trade-offs**: Requires labeled data and a scorer; worth it only above a certain usage volume.

### The Human-Agent Coding Team
**When to use**: Production software development with AI assistance.
**How**: Human orchestrates; a curated Context Staging Area supplies the brief; specialist personas (Scaffolder, Test Engineer, Documenter, Optimizer, Process Agent) handle tactical execution; a critique-then-reflect review runs on pre-commit.
**Trade-offs**: Context curation is real work. Output is always a proposal, never a command.

### The Contractor Model
**When to use**: High-stakes tasks where ambiguity causes failure and outcomes must be verifiable.
**How**: Write a formalized contract (deliverables, specifications, data sources, scope, cost, time) → allow negotiation before execution → quality-focused iterative self-validation → hierarchical decomposition into subcontracts.
**Trade-offs**: Specification cost up front, in exchange for objectively verifiable output and far fewer silent failures.
