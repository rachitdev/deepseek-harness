---
name: dsh-agentic-design-patterns
description: "Knowledge base from \"Agentic Design Patterns: A Hands-On Guide to Building Intelligent Systems\" by Antonio Gulli (Springer, 2025). Use when applying Gulli's 21 agentic patterns for prompt chaining, routing, tool use, planning, multi-agent collaboration, memory, RAG, MCP/A2A, guardrails, or agent evaluation — studying the book, or referencing its concepts and code examples."
---

<!-- argument-hint: [topic, pattern name, or chapter number] -->

# Agentic Design Patterns
**Author**: Antonio Gulli | **Pages**: ~458 | **Chapters**: 29 (21 patterns + 8 supplement) | **Generated**: 2026-08-22

## How to Use This Skill

- **Without arguments** — load the core patterns below for reference
- **With a topic** — ask about `routing`, `RAG`, `guardrails`, `MCP`, or another indexed topic; I read the relevant chapter
- **With a chapter** — ask for `ch14`; I load that chapter file
- **Browse** — ask "what chapters do you have?" for the full index

When you ask about a topic not covered in Core Frameworks below, I read the relevant chapter file before answering.

---

## Core Frameworks & Mental Models

### The four competency groups
1. **Core Execution & Task Decomposition** — Prompt Chaining, Routing, Parallelization, Planning
2. **Environment Interaction** — Tool Use, Knowledge Retrieval (RAG)
3. **State, Learning & Self-Improvement** — Memory Management, Reflection & Self-Correction, Learning and Adaptation
4. **Collaboration & Communication** — Multi-Agent Collaboration, A2A, MCP

The power comes from **composing** patterns, not applying one in isolation.

### The decision rules that matter most

- **Planning vs fixed workflow**: *Does the "how" need to be discovered, or is it already known?* If known and repeatable, constrain the agent to a predetermined workflow — dynamic planning is a specific tool, not a universal solution.
- **Structured output is not a convenience**: without JSON boundaries validated by a schema, an agent's internal components cannot communicate reliably, producing catastrophic failures in automated workflows.
- **Separate the judge from the author**: an agent reviewing its own work inherits its own blind spots. A distinct critic persona is the cheapest quality upgrade available.
- **Answer *after* reasoning**: generating the reasoning conditions the answer tokens. Putting the answer first wastes CoT entirely.
- **Scaling Inference Law**: a smaller model with a bigger "thinking budget" can beat a larger model using simple generation. Bigger is not automatically better — and is always more expensive.
- **MCP does not fix a bad API**: agents do not replace deterministic workflows; they *require stronger deterministic support* (filtering, sorting, agent-readable formats) to succeed.
- **Retrieval quality caps answer quality**: RAG's main failure is information fragmented across chunks or documents. The LLM cannot recover from a bad retrieval.
- **HITL does not scale**: human oversight gives accuracy but operators cannot manage millions of tasks. Design the hybrid split explicitly — automation for volume, humans for the subset that needs judgment.
- **Trajectory over output**: agents are probabilistic, so evaluate the *sequence of steps* against a ground-truth path, not just the final answer.

### Core patterns in one line each

- **Prompt Chaining** — decompose into focused sequential prompts with structured hand-offs; put validation and tool calls in the seams.
- **Routing** — conditional dispatch via LLM, embedding, rule, or fine-tuned classifier; always define a default branch.
- **Parallelization** — fan out independents concurrently, converge on one synthesis node grounded exclusively on branch outputs.
- **Reflection** — Producer/Critic loop with explicit criteria, a structured critique, an unambiguous stop sentinel, and an iteration cap.
- **Tool Use** — describe tools so the model emits a structured call the *framework* executes; return clean typed data, raise specific errors.
- **Planning** — decompose a goal into ordered sub-goals; treat the plan as a starting point and re-plan on obstacles; show long plans to the user first.
- **Multi-Agent Collaboration** — specialists plus a communication protocol; pick a topology (Single / Network / Supervisor / Supervisor-as-Tool / Hierarchical / Custom).
- **Memory Management** — ephemeral context window vs persistent semantic-search store; semantic (facts), episodic (experiences), procedural (rules) long-term types.
- **Learning and Adaptation** — PPO clips updates into a trust region; DPO removes the reward model; SICA modifies its own code against a scored archive.
- **MCP** — client-server standard exposing resources, tools, and prompts with dynamic discovery.
- **Goal Setting and Monitoring** — SMART goals, explicit success criteria, a monitor that collapses judgement to a testable boolean.
- **Exception Handling** — Detect → Handle (log, retry transients, fall back, degrade, notify) → Recover (rollback, diagnose, self-correct, escalate).
- **Human-in-the-Loop** — oversight, intervention, feedback, decision augmentation, collaboration, and explicit escalation policies.
- **RAG** — chunk, embed, semantic (or hybrid) search, augment, generate with citations; GraphRAG for fragmented sources; Agentic RAG to validate and reconcile.
- **A2A** — Agent Cards, asynchronous tasks, four interaction mechanisms matched to task duration.
- **Resource-Aware Optimization** — Router classifies complexity, cheap model for simple work, Critique agent refines routing, fallbacks for unavailability.
- **Reasoning Techniques** — CoT, ToT, Self-Correction, PALMs, RLVR, ReAct, CoD/GoD, MASS.
- **Guardrails** — layered defence: input validation, behavioral constraints, tool restrictions, output filtering, moderation, human oversight; plus least privilege and structured logging.
- **Evaluation and Monitoring** — semantic accuracy, latency, tokens, trajectory comparison, LLM-as-a-Judge, and the Contractor model for verifiable deliverables.
- **Prioritization** — criteria (urgency, importance, dependencies, resources, cost/benefit, preference), evaluation, selection, dynamic re-prioritization.
- **Exploration and Discovery** — generate, debate, evolve; Elo tournament ranking; test-time compute scaling; scientist-in-the-loop.

---

## Chapter Index

| # | Title | Key Frameworks |
|---|-------|----------------|
| [ch01](chapters/ch01-prompt-chaining.md) | Prompt Chaining | Pipeline pattern, Context Engineering, LCEL |
| [ch02](chapters/ch02-routing.md) | Routing | LLM/embedding/rule/ML routing, RunnableBranch, ADK Auto-Flow |
| [ch03](chapters/ch03-parallelization.md) | Parallelization | RunnableParallel, ParallelAgent, fan-out/fan-in |
| [ch04](chapters/ch04-reflection.md) | Reflection | Producer–Critic model, four-step loop, stop sentinel |
| [ch05](chapters/ch05-tool-use.md) | Tool Use (Function Calling) | six-step cycle, tool calling, Vertex AI Extensions |
| [ch06](chapters/ch06-planning.md) | Planning | the delegation test, Deep Research architecture |
| [ch07](chapters/ch07-multi-agent-collaboration.md) | Multi-Agent Collaboration | six topologies, six collaboration forms, AgentTool |
| [ch08](chapters/ch08-memory-management.md) | Memory Management | Session/State/Memory, state prefixes, semantic/episodic/procedural |
| [ch09](chapters/ch09-learning-and-adaptation.md) | Learning and Adaptation | PPO, DPO, SICA, AlphaEvolve, OpenEvolve |
| [ch10](chapters/ch10-model-context-protocol.md) | Model Context Protocol | client-server, resources/tools/prompts, FastMCP |
| [ch11](chapters/ch11-goal-setting-and-monitoring.md) | Goal Setting and Monitoring | SMART goals, boolean stop condition, the five-agent crew |
| [ch12](chapters/ch12-exception-handling-and-recovery.md) | Exception Handling and Recovery | detect/handle/recover, layered handler chain |
| [ch13](chapters/ch13-human-in-the-loop.md) | Human-in-the-Loop | six HITL aspects, human-on-the-loop, escalation policies |
| [ch14](chapters/ch14-knowledge-retrieval-rag.md) | Knowledge Retrieval (RAG) | embeddings, chunking, vector DBs, GraphRAG, Agentic RAG |
| [ch15](chapters/ch15-inter-agent-communication-a2a.md) | Inter-Agent Communication (A2A) | Agent Card, discovery, tasks/messages/artifacts, four mechanisms |
| [ch16](chapters/ch16-resource-aware-optimization.md) | Resource-Aware Optimization | Router+Critique, fallback, nine optimization techniques |
| [ch17](chapters/ch17-reasoning-techniques.md) | Reasoning Techniques | CoT, ToT, PALMs, RLVR, ReAct, CoD, GoD, MASS, Scaling Inference Law |
| [ch18](chapters/ch18-guardrails-safety.md) | Guardrails / Safety Patterns | six guardrail stages, layered defence, four engineering principles |
| [ch19](chapters/ch19-evaluation-and-monitoring.md) | Evaluation and Monitoring | trajectory metrics, LLM-as-a-Judge, the Contractor model |
| [ch20](chapters/ch20-prioritization.md) | Prioritization | six criteria, three levels, dynamic re-prioritization |
| [ch21](chapters/ch21-exploration-and-discovery.md) | Exploration and Discovery | AI co-scientist, Agent Laboratory, tripartite judgment |
| [ch22](chapters/ch22-advanced-prompting-techniques.md) | Advanced Prompting Techniques | five principles, shot levels, structuring, DSPy optimization |
| [ch23](chapters/ch23-agentic-interactions-gui-to-real-world.md) | AI Agentic Interactions | ACI four stages, Operator/Mariner/Computer Use, vibe coding |
| [ch24](chapters/ch24-agentic-frameworks-overview.md) | Agentic Frameworks Overview | LangChain vs LangGraph vs ADK vs CrewAI, and eight others |
| [ch25](chapters/ch25-building-an-agent-with-agentspace.md) | Building an Agent with AgentSpace | knowledge graph, Agent Designer, connected actions |
| [ch26](chapters/ch26-ai-agents-on-the-cli.md) | AI Agents on the CLI | Claude CLI, Gemini CLI, Aider, Copilot CLI, Terminal-Bench |
| [ch27](chapters/ch27-under-the-hood-reasoning-engines.md) | Under the Hood: Reasoning Engines | the convergent pipeline across six LLMs |
| [ch28](chapters/ch28-coding-agents.md) | Coding Agents | three foundations, specialist agent team, critique-then-reflect |
| [ch29](chapters/ch29-conclusion.md) | Conclusion | four competency groups, pattern composition, future directions |

## Topic Index

- **A2A / Agent Card** → ch15, ch25
- **Agentic RAG** → ch14
- **Chain-of-Thought** → ch17, ch22
- **Chunking / embeddings / vector DBs** → ch14
- **Coding agents** → ch26, ch28, ch11
- **Context Engineering** → ch01, ch22
- **CrewAI** → ch05, ch06, ch07, ch18, ch24
- **Deep Research** → ch06, ch17
- **DSPy / prompt optimization** → ch22
- **Evaluation / trajectories** → ch19
- **Fallback / graceful degradation** → ch12, ch16
- **Few-shot / many-shot** → ch22
- **Google ADK** → ch02, ch03, ch05, ch07, ch08, ch10, ch12, ch13, ch16, ch19, ch24
- **GraphRAG** → ch14
- **Guardrails / jailbreaks** → ch18
- **Human-in-the-loop / on-the-loop** → ch13, ch06, ch29
- **LangChain / LCEL** → ch01, ch02, ch03, ch05, ch14, ch20, ch24
- **LangGraph** → ch08, ch14, ch17, ch24
- **LLM-as-a-Judge** → ch19
- **MCP / FastMCP** → ch10, ch26
- **Memory (semantic/episodic/procedural)** → ch08
- **Multi-agent topologies** → ch07
- **Parallelization / concurrency** → ch03
- **Planning** → ch06, ch11
- **PPO / DPO / RLHF** → ch09
- **Prioritization** → ch20
- **Pydantic validation** → ch18, ch20, ch22
- **ReAct** → ch17, ch22
- **Reflection / self-correction** → ch04, ch17
- **Resource optimization / model routing** → ch16
- **Routing** → ch02, ch16
- **Scaling Inference Law** → ch17
- **Self-improving agents (SICA, AlphaEvolve)** → ch09
- **Structured output** → ch01, ch18, ch22
- **Tool Use / function calling** → ch05, ch22
- **Tree of Thoughts** → ch17, ch22
- **Vibe coding** → ch23, ch28

## Supporting Files

- [glossary.md](glossary.md) — all key terms with definitions and chapter references
- [patterns.md](patterns.md) — all 21 patterns plus supporting techniques in when/how/trade-off form
- [cheatsheet.md](cheatsheet.md) — decision rules, thresholds, tells and smells, cost levers

---

## Scope & Limits

This skill covers the book's content only. Framework APIs (LangChain, ADK, CrewAI) evolve — verify signatures against current documentation before use; the code here is faithful to the 2025 text. Model names and pricing in the book are point-in-time. For implementation in this repository, combine with project-specific tooling and conventions.
