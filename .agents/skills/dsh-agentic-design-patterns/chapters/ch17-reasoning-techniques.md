# Chapter 17: Reasoning Techniques

## Core Idea
Make the agent's internal reasoning **explicit** and spend more computation at inference time. Decomposition, exploration of multiple paths, self-critique, and tool use during generation buy accuracy, coherence, and robustness that a single forward pass cannot.

## Frameworks Introduced
- **Chain-of-Thought (CoT)**: prompt the model to generate a sequence of intermediate reasoning steps rather than a direct answer.
  - When to use: multi-step reasoning — arithmetic, commonsense reasoning, symbolic manipulation.
  - How: few-shot examples demonstrating step-by-step reasoning, or simply instructing "think step by step"; better still, define a persona and an explicit numbered process.
  - Why it works: transforms a difficult single-step problem into a series of simpler steps, and increases transparency — valuable for debugging, comprehension, and **auditable** agent actions.
- **Tree-of-Thought (ToT)**: builds on CoT by branching into different intermediate steps, forming a tree. Supports **backtracking, self-correction, and exploration of alternatives**; the model evaluates multiple reasoning trajectories before committing. For tasks requiring strategic planning and decision-making.
- **Self-Correction (self-refinement)**: the agent internally evaluates its generated content and intermediate thoughts, identifying ambiguities, information gaps, or inaccuracies, then adjusts. A quality-control measure embedded directly into generation.
- **Program-Aided Language Models (PALMs)**: integrate LLMs with symbolic reasoning by generating and executing code. Offload complex calculation, logical operations, and data manipulation to a **deterministic programming environment**, then convert results back to natural language.
- **Reinforcement Learning from Verifiable Rewards (RLVR)**: the training strategy behind specialized "reasoning models". Standard CoT generates a single predetermined line of thought without adapting to problem complexity. Reasoning models instead dedicate a *variable* amount of thinking time, producing a dynamic chain of thought thousands of tokens long that supports self-correction and backtracking. Trained on problems with **known correct answers** (math, code), the model learns effective long-form reasoning by trial and error — evolving problem-solving ability without direct human supervision. The output is a **reasoning trajectory** demonstrating planning, monitoring, and evaluation.
- **ReAct (Reasoning and Acting)**: integrates CoT with tool use. The agent reasons about which action to take, acts (tool or function call), observes the outcome, and folds that observation into subsequent reasoning — the interleaved **Thought → Action → Observation → Thought…** loop. Unlike linear CoT, the agent responds to real-time feedback, adapting plans and correcting errors.
- **Chain of Debates (CoD)** (Microsoft): multiple *diverse models* collaborate and argue — an "AI council meeting" where models present initial ideas, critique each other's reasoning, and exchange counterarguments. Enhances accuracy, reduces bias, and creates a transparent, trustworthy record of the reasoning process. AI peer review.
- **Graph of Debates (GoD)**: reimagines debate as a dynamic non-linear network. Arguments are **nodes**; **edges** signify `supports` or `refutes`. Lines of inquiry branch off, evolve independently, and merge. A conclusion is reached not at the end of a sequence but by **identifying the most robust, well-supported cluster of arguments in the graph** — where "well-supported" means ground truth, factual evidence validated by search grounding, or consensus reached by multiple models in debate.
- **MASS (Multi-Agent System Search)**: automates and optimizes multi-agent system design, whose effectiveness depends critically on both prompt quality and interaction topology. Three stages:
  1. **Block-Level Prompt Optimization** — locally optimize prompts per agent type before integration, so topology search builds on well-performing agents rather than compounding poorly configured ones.
  2. **Workflow Topology Optimization** — select and arrange agent interactions from a customizable design space, using an **influence-weighted method** that computes each topology's "incremental influence" (performance gain relative to a baseline agent) to guide the search.
  3. **Workflow-Level Prompt Optimization** — after the best topology is found, globally re-tune all prompts as one integrated entity so agent interdependencies are optimized.
- **The Scaling Inference Law**: performance improves predictably with computation allocated **at inference**, distinct from training scaling laws. Superior results can often be achieved from a *smaller* LLM by investing more compute at inference — generating multiple candidate answers (diverse beam search, self-consistency) and selecting the best. A smaller model with a substantial "thinking budget" can surpass a much larger model using simple generation. Balances **model size**, **response latency**, and **operational cost**.

## Key Concepts
- **Reasoning trajectory**: the extended, dynamic chain a reasoning model produces, exhibiting planning, monitoring, and evaluation.
- **Thinking budget**: additional computational steps or algorithms applied during inference.
- **Thought frequency tuning**: for knowledge-intensive reasoning (fact-checking), interleave thoughts with *every* action; for decision-making tasks with many actions (navigating a simulated environment), use thoughts sparingly and let the agent decide when thinking is necessary.
- **Few-shot trajectories**: ReAct commonly supplies examples of human-like problem-solving trajectories showing how to combine thoughts and actions.
- **Deep Research**: agentic research tools (Perplexity, Gemini, ChatGPT) where you grant a **time budget** of a few minutes and receive a report — Initial Exploration → Reasoning and Refinement (identify gaps, contradictions) → Follow-up Inquiry → Final Synthesis.

## Mental Models
- **CoT is the agent's internal monologue; ReAct is the agent's operational loop.** One formulates the plan, the other executes and adapts it.
- **Bigger is not always better.** The Scaling Inference Law challenges that intuition: allocate the thinking budget where it has most impact rather than reaching for the largest model.
- **Delegate what must be exact.** PALMs exist because LLMs are unreliable at precise computation; a deterministic environment is not a fallback, it is the right tool.
- **Optimize agents before you compose them.** MASS's first principle — a great topology over badly-prompted agents compounds their weakness.
- **Debate over monologue.** CoD/GoD replace one model's confident chain with a council whose disagreements surface errors and bias.

## Anti-patterns
- **Asking for a direct answer on a multi-step problem**: no intermediate steps means no decomposition and no auditability.
- **Linear CoT where the environment matters**: without observation, the agent cannot correct on real-world feedback. Use ReAct.
- **Letting the LLM do arithmetic**: generate and execute code instead.
- **Assuming a bigger model beats more inference compute**: often false, and always more expensive.
- **Searching an unconstrained multi-agent design space**: MASS's second principle — compose *influential* topologies instead.
- **Optimizing prompts once, before topology is settled**: prompts tuned in isolation are not tuned for orchestration; MASS re-optimizes at workflow level.
- **Thinking on every step of a long action sequence**: for decision-making tasks with many actions, constant deliberation wastes budget.

## Code Examples
A Chain-of-Thought system prompt with an explicit process:
```
You are an Information Retrieval Agent. Your goal is to answer the user's question
comprehensively and accurately by thinking step-by-step.

Here's the process you must follow:

1. **Analyze the Query:** Understand the core subject and specific requirements of the
   user's question. Identify key entities, keywords, and the type of information sought.
2. **Formulate Search Queries (for Knowledge Base):** Based on your analysis, generate a
   list of precise search queries you would use to retrieve relevant information.
3. **Simulate Information Retrieval (Self-Correction/Reasoning):** For each search query,
   mentally consider what kind of information you expect to find. What would be the most
   relevant snippets? Think about potential ambiguities or missing pieces.
4. **Synthesize Information:** Synthesize the gathered information into a coherent and
   complete answer. Ensure all aspects of the query are addressed.
5. **Review and Refine:** Before finalizing, critically evaluate your answer. Is it
   accurate? Comprehensive? Easy to understand? Concise? If not, identify what needs
   improvement and how.
```

A Self-Correction agent prompt:
```
You are a highly critical and detail-oriented Self-Correction Agent. Your task is to
review a previously generated piece of content against its original requirements and
identify areas for improvement.

1. **Understand Original Requirements:** What was the *original intent*? What were the
   key constraints or goals?
2. **Analyze Current Content:** Read the provided content carefully.
3. **Identify Discrepancies/Weaknesses:** Compare against the original requirements:
   * **Accuracy Issues:** factual errors or misleading statements?
   * **Completeness Gaps:** does it fully address all aspects? Is anything missing?
   * **Clarity & Coherence:** clear, concise, logically flowing?
   * **Tone & Style:** does it match the desired tone?
   * **Engagement:** is it captivating?
   * **Redundancy/Verbosity:** can anything be condensed or removed?
4. **Propose Specific Improvements:** Do not just state the problem; propose a solution.
5. **Generate Revised Content:** Rewrite to incorporate all necessary changes.
```

PALMs — search and code-execution specialists exposed as tools to a root agent (ADK):
```python
from google.adk.agents import Agent
from google.adk.tools import agent_tool, google_search
from google.adk.code_executors import BuiltInCodeExecutor

search_agent = Agent(
    model='gemini-2.0-flash', name='SearchAgent',
    instruction="You're a specialist in Google Search",
    tools=[google_search])

coding_agent = Agent(
    model='gemini-2.0-flash', name='CodeAgent',
    instruction="You're a specialist in Code Execution",
    code_executor=[BuiltInCodeExecutor])

root_agent = Agent(
    name="RootAgent", model="gemini-2.0-flash", description="Root Agent",
    tools=[agent_tool.AgentTool(agent=search_agent),
           agent_tool.AgentTool(agent=coding_agent)])
```

DeepSearch as a LangGraph cycle (from `backend/src/agent/graph.py` in `gemini-fullstack-langgraph-quickstart`):
```python
builder = StateGraph(OverallState, config_schema=Configuration)

builder.add_node("generate_query", generate_query)
builder.add_node("web_research", web_research)
builder.add_node("reflection", reflection)
builder.add_node("finalize_answer", finalize_answer)

builder.add_edge(START, "generate_query")

# Fan out queries into parallel research branches
builder.add_conditional_edges("generate_query", continue_to_web_research, ["web_research"])

builder.add_edge("web_research", "reflection")

# The loop: reflection either sends us back for more research, or finalizes
builder.add_conditional_edges("reflection", evaluate_research,
                              ["web_research", "finalize_answer"])

builder.add_edge("finalize_answer", END)

graph = builder.compile(name="pro-search-agent")
```
- **What it demonstrates**: the entire Deep Research behavior is four nodes and one conditional back-edge. `evaluate_research` is the knowledge-gap check that decides whether to loop; everything else is linear. React frontend + LangGraph backend, Gemini 2.5, Redis for streaming and Postgres for data in production. Apache 2.0.

## Reference Tables

| Technique | Shape | Adds |
|---|---|---|
| Chain-of-Thought | linear sequence of steps | decomposition, transparency |
| Tree-of-Thought | branching tree | backtracking, alternatives, deliberation |
| Self-Correction | review-and-revise loop | accuracy, completeness |
| PALMs | generate + execute code | deterministic precision |
| RLVR reasoning models | variable-length trajectory | adaptive thinking time, learned self-correction |
| ReAct | Thought → Action → Observation loop | environment interaction, real-time adaptation |
| Chain of Debates | multiple models arguing | reduced bias, collective accuracy |
| Graph of Debates | argument graph with supports/refutes edges | non-linear, cluster-based conclusions |
| MASS | automated MAS design search | optimized prompts + topology |

| Application | Technique emphasis |
|---|---|
| Complex (multi-hop) QA | multiple reasoning paths, extended inference to synthesize |
| Mathematical problem solving | decomposition + code execution for precise computation |
| Code debugging and generation | Self-Correction over iterative test-driven refinement |
| Strategic planning | reasoning over options, consequences, preconditions; ReAct for feedback |
| Medical diagnosis | systematic assessment, articulated reasoning, ReAct for data retrieval |
| Legal analysis | logical steps detailed, self-correction for consistency |

| MASS stage | Optimizes | Example finding |
|---|---|---|
| Block-level prompt | each agent type in isolation | HotpotQA "Debater" framed as an "expert fact checker for a major publication" cross-referencing context passages |
| Workflow topology | interaction structure, influence-weighted | MBPP coding: one predictor doing several rounds of reflection + one executor verifying code against test cases |
| Workflow-level prompt | all prompts jointly, for orchestration | DROP "Predictor" prompt combining dataset meta-knowledge, few-shot examples, and a high-stakes role ("a live broadcast is relying on your accuracy and speed") |

| Scaling Inference Law balances | Consideration |
|---|---|
| Model size | smaller models demand less memory and storage |
| Response latency | more inference compute adds latency; find where gains outweigh it |
| Operational cost | larger models cost more in power and infrastructure |

## Worked Example
**Chain-of-Thought on "Explain the main differences between classical and quantum computers, and briefly describe a potential application of quantum computing."** The prompt above produces five visible thoughts before any answer:

- **Thought 1 (Analyze Query)** — the user wants two things: the differences, and one application. Keywords: classical computers, quantum computers, differences, potential application.
- **Thought 2 (Formulate Search Queries)** — "differences classical vs quantum computers", "how quantum computers work", "how classical computers work", "applications of quantum computing".
- **Thought 3 (Simulate Retrieval)** — from *differences*: bits vs qubits, superposition, entanglement, probabilistic nature. From *how they work*: transistors/binary vs quantum mechanics. From *applications*: drug discovery, material science, cryptography, optimization.
- **Thought 4 (Synthesize)** — assemble the two-part answer.
- **Thought 5 (Review and Refine)** — differences covered; application given; brief; clear. Good.

The value is not that the answer is better in isolation — it is that steps 1 and 5 are **checkable**. A reviewer can see that the agent identified *both* required parts in Thought 1 and verified both in Thought 5. That is what "auditable" means in practice.

**Self-Correction on a social media post.** Requirements: short, engaging, max 150 characters, announcing "GreenTech Gadgets", an eco-friendly line. Initial draft: *"We have new products. They are green and techy. Buy GreenTech Gadgets now!"* (64 chars). The agent's critique is specific rather than general: not engaging, sounds generic; eco-friendliness only *implied* by "green", not highlighted as a benefit; the call to action is okay but not compelling; short but weak verbs; lacks launch excitement. Proposed fixes name the mechanism — stronger verbs, active voice, emphasize eco-friendly and innovation, add hashtags, stay under 150. Revised: *"⚡ Discover GreenTech Gadgets! Our new eco-friendly line blends innovation with sustainability. Go green, go smart! Shop now! #EcoFriendly #GreenTech"* (148 chars).

Note that the draft satisfied the *stated* constraint (under 150 characters) and still failed the intent. Self-correction catches that because step 1 recovers the original intent, not just the checklist.

## Key Takeaways
1. Explicit reasoning gives agents transparent, multi-step plans — the foundation for autonomous action and user trust.
2. CoT is the internal monologue; ToT and Self-Correction add deliberation, backtracking, and self-improvement before execution.
3. ReAct is the core agentic loop — Thought, Action, Observation — letting agents act and adapt within an environment.
4. PALMs offload precise computation to code, where LLMs are unreliable.
5. RLVR trains models to produce long, adaptive reasoning trajectories by rewarding verifiable correct answers.
6. The Scaling Inference Law: performance depends on allocated thinking time, not just model size — a smaller model with a bigger budget can beat a larger one.
7. CoD and GoD move from a solitary reasoner to collaborative agent societies, reducing individual bias.
8. MASS automates multi-agent design: optimize agents' prompts first, compose influential topologies, then jointly re-optimize for interdependencies.
9. Deep Research shows these techniques combining into autonomous, long-running investigation on a user's behalf.

## Connects To
- **Ch 4 (Reflection)**: Self-Correction is Reflection at the reasoning-step level.
- **Ch 5 (Tool Use)**: ReAct's Action step; PALMs' code executor.
- **Ch 6 (Planning)**: Deep Research applies these techniques to long-running research plans.
- **Ch 3 (Parallelization)**: Self-Consistency generates diverse reasoning paths concurrently and votes.
- **Ch 7 (Multi-Agent Collaboration)**: CoD, GoD, and MASS are multi-agent reasoning.
- **Ch 16 (Resource-Aware Optimization)**: the Scaling Inference Law is the quantitative basis for the cost/quality trade-off.
- **Ch 22 (Advanced Prompting Techniques)**: CoT, Self-Consistency, Step-Back, ToT, ReAct in prompt form.
