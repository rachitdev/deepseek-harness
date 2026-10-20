# Chapter 9: Learning and Adaptation

## Core Idea
Learning is the agent changing its thinking, actions, or knowledge from new experience and data; **adaptation is the visible change in behavior that results** — the mechanism by which agents evolve from instruction-followers into systems that improve without manual reprogramming.

## Frameworks Introduced
- **The six learning mechanisms for agents**:
  - **Reinforcement Learning** — try actions, receive rewards/penalties, learn optimal behavior in changing situations. For robot control and games.
  - **Supervised Learning** — learn from labeled examples mapping inputs to desired outputs. For sorting email, predicting trends.
  - **Unsupervised Learning** — discover hidden patterns in unlabeled data; builds a mental map of the environment. For exploring data without guidance.
  - **Few-Shot / Zero-Shot Learning with LLM agents** — adapt to new tasks from minimal examples or clear instructions; enables rapid response to new commands.
  - **Online Learning** — continuously update knowledge with new data; essential for real-time reaction and continuous data streams.
  - **Memory-Based Learning** — recall past experiences to adjust current actions in similar situations.
- **PPO (Proximal Policy Optimization)** — RL for continuous action spaces (robot joints, game characters); goal is to reliably and *stably* improve the policy.
  - How: (1) collect a batch of (state, action, reward) experiences with the current policy; (2) evaluate a **surrogate objective** — how a candidate update would change expected reward; (3) apply the **clipping mechanism**, a trust region around the current policy that prevents updates too different from the current strategy.
  - Why it works: clipping is a safety brake. PPO balances improving performance against staying near a known working strategy, preventing catastrophic collapse during training.
- **DPO (Direct Preference Optimization)** — aligns LLMs with human preferences without a reward model.
  - The PPO alignment baseline (two steps): (1) collect human comparisons ("Response A is better than B") and train a separate **reward model** to predict human scores; (2) fine-tune the LLM with PPO to maximize the reward model's score. Complex and unstable — the LLM can find a loophole and **hack the reward model**, getting high scores for bad responses.
  - The DPO alternative: skip the reward model. Use a mathematical relationship linking preference data directly to the optimal policy — "increase the probability of generating responses like the preferred one and decrease the probability of ones like the disfavored one."
  - Why it works: removes the reward-model training and its instability, making alignment more efficient and robust.
- **SICA (Self-Improving Coding Agent)** — an agent that modifies its **own source code**; both modifier and modified, rather than one agent training another.
  - The self-improvement cycle: (1) review an archive of past versions and their benchmark performance; (2) select the highest-scoring version by a **weighted formula over success, time, and computational cost**; (3) that version analyzes the archive to identify improvements and directly alters its codebase; (4) test the modified agent against benchmarks; (5) record results in the archive; repeat.
  - Architecture: a foundational toolkit (file ops, command execution, arithmetic), result submission, and specialized sub-agents (coding, problem-solving, reasoning) that decompose tasks and manage context length across long improvement cycles.
- **AlphaEvolve** (Google) — LLM ensemble + automated evaluation + evolutionary algorithm framework. Gemini **Flash generates a wide range of initial algorithm proposals**; **Pro provides deeper analysis and refinement**; proposals are auto-evaluated and scored against predefined criteria; feedback drives iterative improvement.

## Key Concepts
- **Adaptation vs learning**: learning is the internal update; adaptation is the observable change in strategy, understanding, or goals.
- **Trust region / clipping**: the stability mechanism at the heart of PPO.
- **Reward hacking**: the failure mode DPO exists to avoid.
- **Asynchronous overseer** (SICA): a second LLM running concurrently that assesses the main agent for pathological deviation, loops, or stagnation, and can notify or cancel execution. It receives a **callgraph** and an **event stream** of LLM messages, tool calls, and responses.
- **Structured context window** (SICA): System Prompt (goals, tool/sub-agent documentation, system instructions) → Core Prompt (problem statement, contents of open files, directory map) → Assistant Messages (step-by-step reasoning, tool/sub-agent call records and results, overseer communications).
- **Docker containerization**: SICA runs in a dedicated container for isolation from the host, mitigating inadvertent filesystem manipulation given the agent's shell access.
- **Observability**: an interactive webpage visualizing the event bus and callgraph, letting users inspect events, read overseer messages, and collapse sub-agent traces.
- **Knowledge Base Learning**: use RAG to maintain a dynamic base of problem descriptions and proven solutions, so the agent applies previously successful patterns and avoids known pitfalls.

## Mental Models
- **Small careful steps beat big risky ones** — the PPO lesson generalizes past RL: any self-modifying system needs a trust region.
- **Remove the proxy when you can measure the thing** — DPO's insight. A reward model is a proxy for preference; optimizing a proxy invites gaming it.
- **Selection pressure needs a scoring function you actually believe** — SICA's weighted score over success, time, and cost is what determines what it evolves toward.
- **Self-modification needs three safety layers**: an overseer that can halt it, container isolation, and observability into the callgraph.
- **Ensemble by role**: cheap-and-fast model for breadth of proposals, strong model for depth of refinement (AlphaEvolve's Flash/Pro split).

## Anti-patterns
- **Optimizing a learned reward model without watching for reward hacking**: the model learns the loophole, not the task.
- **Large unclipped policy updates**: performance collapse; the update undoes prior learning.
- **Self-modifying agents without isolation**: an agent with shell access and no container can manipulate the host filesystem.
- **No overseer on a long self-improvement run**: loops and stagnation go undetected and burn budget.
- **Unstructured context in long improvement cycles**: SICA's explicit System/Core/Assistant organization exists to control processing time and cost.
- **Tuning multi-agent systems on partial trajectories**: tuning data must reflect the *complete* interaction trajectory, capturing each participating agent's individual inputs and outputs.

## Code Examples
OpenEvolve — evolutionary optimization of a program:
```python
from openevolve import OpenEvolve

evolve = OpenEvolve(
    initial_program_paths="path/to/initial_program.py",
    evaluation_file="path/to/evaluator.py",
    config_path="path/to/config.yaml",
)

best_program = await evolve.run(iterations=1000)
print("Best program metrics:")
for name, value in best_program.metrics.items():
    print(f"  {name}: {value:.4f}")
```
- **What it demonstrates**: the three inputs that define an evolutionary coding agent — a seed program, an **evaluator** (the fitness function; this is the load-bearing artifact), and a config. OpenEvolve evolves *entire code files* rather than single functions, supports multiple languages and any OpenAI-compatible API, and does multi-objective optimization with distributed evaluation.
- Architecture: a controller orchestrates a program sampler, a Program Database, an Evaluator Pool, and LLM Ensembles, in an asynchronous pipeline optimized for throughput.

## Reference Tables

| Mechanism | Signal | Good for |
|---|---|---|
| Reinforcement Learning | reward/penalty | robots, games, continuous control |
| Supervised Learning | labeled examples | classification, prediction |
| Unsupervised Learning | structure in unlabeled data | insight, organization, environment mapping |
| Few-shot / zero-shot (LLM) | examples or instructions in-context | rapid adaptation to new commands |
| Online Learning | continuous data stream | real-time reaction in dynamic environments |
| Memory-Based | recalled past experience | context-aware decisions in similar situations |

| | PPO alignment | DPO |
|---|---|---|
| Steps | train reward model, then fine-tune with PPO | optimize directly on preference pairs |
| Reward model | required | none |
| Failure mode | reward hacking; instability | — |
| Complexity | high | lower, more robust |

| Application | What adapts |
|---|---|
| Personalized assistant | interaction protocols, from longitudinal user behavior |
| Trading bot | model parameters, from real-time market data |
| Application agent | UI and functionality, from observed user behavior |
| Robotics / AV | navigation and response, from sensor data + historical actions |
| Fraud detection | predictive models, from newly identified fraud patterns |
| Recommendation | content selection, from preference learning |
| Game AI | strategic algorithms, to raise challenge |
| Knowledge Base agent | a RAG store of problems and proven solutions |

| AlphaEvolve result | Impact |
|---|---|
| Data center scheduling | 0.7% reduction in global compute resource usage |
| TPU hardware design | Verilog optimizations for upcoming TPUs |
| Gemini architecture | 23% speedup in a core kernel |
| FlashAttention | up to 32.5% optimization of low-level GPU instructions |
| Matrix multiplication | 4×4 complex-valued in 48 scalar multiplications, surpassing prior solutions |
| Open math problems | rediscovered SOTA in 75% of >50 problems; improved on 20% (e.g. kissing number) |

## Worked Example
**SICA's evolution of its own code-editing tools** — the clearest demonstration in the chapter of learning producing visible adaptation.

SICA began with a crude capability: **file overwriting** for every code change. Across iterations, guided only by the benchmark-weighted score, it built:

1. **Smart Editor** — intelligent, contextual edits instead of whole-file writes.
2. **Diff-Enhanced Smart Editor** — diffs for targeted modification plus pattern-based editing.
3. **Quick Overwrite Tool** — a cheap path back, to reduce processing demands when a full edit isn't warranted.
4. **Minimal Diff Output Optimization** and **Context-Sensitive Diff Minimization** — using **AST parsing** for efficiency.
5. **SmartEditor Input Normalizer**.

In parallel, for navigation: an **AST Symbol Locator** (using the code's structural map to find definitions), then a **Hybrid Symbol Locator** combining a quick text search with AST checking, then **Optimized AST Parsing in the Hybrid Symbol Locator** to focus only on relevant sections.

Two things are worth extracting. First, the trajectory is recognizably *engineering judgment* — from blunt to precise, then adding a cheap fast path once the precise one exists. Second, the honest limitation the authors report: prompting the agent to independently propose **novel, innovative, feasible, and engaging** modifications each iteration was the hard part. Open-ended learning and authentic creativity in LLM agents remain open research problems — the loop reliably refines, but does not reliably invent.

## Key Takeaways
1. Learning and adaptation let agents improve from experience; adaptation is the visible behavioral change learning produces.
2. PPO's clipping keeps policy updates inside a trust region — stability over greedy improvement.
3. DPO removes the reward model from alignment, avoiding reward hacking and the instability of the two-step PPO pipeline.
4. SICA self-improves by modifying its own code against an archive of past versions scored on success, time, and cost — producing the Smart Editor and AST Symbol Locator lines of tools.
5. Self-improving systems need specialized sub-agents (task decomposition and context management) and an asynchronous overseer (loop/stagnation detection with the power to halt).
6. Context-window organization — system prompt, core prompt, assistant messages — materially affects efficiency, processing time, and cost.
7. Isolation (Docker) and observability (event bus, callgraph) are non-optional for agents that execute shell commands on themselves.
8. AlphaEvolve pairs an LLM ensemble with automated evaluation and evolution to discover new algorithms, with deployed gains in scheduling, hardware design, and kernel performance.
9. For multi-agent tuning, capture the complete interaction trajectory — every agent's inputs and outputs.

## Connects To
- **Ch 4 (Reflection)**: self-critique is learning within one task; SICA generalizes it to the agent's own code.
- **Ch 8 (Memory Management)**: long-term memory is where learned strategies persist; procedural memory is self-modified instructions.
- **Ch 14 (Knowledge Retrieval / RAG)**: the substrate for Knowledge Base Learning agents.
- **Ch 19 (Evaluation and Monitoring)**: benchmarks are the selection pressure; the overseer is monitoring made active.
- **Ch 21 (Exploration and Discovery)**: AlphaEvolve and Co-scientist are autonomous discovery systems.
- **Ch 18 (Guardrails)**: containerization and the halting overseer are safety patterns for self-modifying agents.
