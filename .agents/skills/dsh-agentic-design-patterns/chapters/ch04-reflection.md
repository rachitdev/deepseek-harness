# Chapter 4: Reflection

## Core Idea
Reflection adds a feedback loop: the agent (or a dedicated critic) evaluates the output it just produced against explicit criteria and uses that critique to generate a better version — turning a single-pass execution into iterative self-correction.

## Frameworks Introduced
- **Reflection pattern (the four-step loop)**:
  1. **Execution** — the agent performs the task or generates an initial output.
  2. **Evaluation/Critique** — the agent (usually another LLM call, or a rule set) analyzes the result for factual accuracy, coherence, style, completeness, instruction adherence.
  3. **Reflection/Refinement** — based on the critique, generate a refined output, adjust parameters, or modify the plan.
  4. **Iteration** (optional but common) — repeat until satisfactory or a stopping condition is met.
  - When to use: quality, accuracy, and detail of the final output matter more than speed and cost — polished long-form content, code writing and debugging, detailed plans.
- **Producer–Critic model** (a.k.a. Generator-Critic, Producer-Reviewer): split reflection into two logical roles.
  - **Producer Agent** — sole responsibility is initial execution: write the code, draft the post, create the plan.
  - **Critic Agent** — sole purpose is evaluating the Producer's output, under a *different persona* ("You are a senior software engineer", "You are a meticulous fact-checker") and explicit criteria. Designed to find flaws and return structured feedback.
  - Why it works: separation of concerns prevents the cognitive bias of an agent reviewing its own work. The critic arrives with a fresh perspective dedicated entirely to finding errors.
  - When to use two agents rather than self-reflection: when the task requires high objectivity or specialized evaluation a generalist producer would miss.

## Key Concepts
- **Feedback loop**: unlike chaining (output → next step) or routing (choose a path), reflection sends output *back* to be examined.
- **Meta-cognition**: the agent reasons about its own output and process, not just the task.
- **Stopping condition**: an explicit sentinel (`CODE_IS_PERFECT`) or a `max_iterations` cap; without one the loop never terminates.
- **Reflector prompt**: a system message that installs the critic persona and the evaluation criteria, distinct from the producer's prompt.
- **Structured critique**: the critic returns a dictionary (`{"status": "ACCURATE"|"INACCURATE", "reasoning": ...}`) or a bulleted list, so the next stage can act on it mechanically.
- **Message history as context**: the accumulated task + code + critique history is what lets each refinement build on the last.
- **LoopAgent** (ADK): an alternative to `SequentialAgent` for genuinely iterative reflection.

## Mental Models
- Think **code review, not proofreading**: the value is the second, differently-instructed reader — persona and criteria do the work.
- **Memory turns reflection cumulative**: without conversational memory each reflection is a self-contained event; with memory, each cycle builds on the last and the agent stops repeating errors.
- **Goals supply the benchmark**: a goal is the yardstick for self-evaluation, monitoring tracks progress, and reflection is the corrective engine that analyzes deviations and adjusts strategy.
- Reflection is a **control structure**, composable with the other patterns rather than a standalone workflow.

## Anti-patterns
- **Self-review without role separation**: an agent grading its own work inherits its own blind spots. Use a distinct critic persona at minimum.
- **No stopping sentinel or iteration cap**: the loop runs forever or burns budget on diminishing returns.
- **Unstructured critique**: free-form praise the producer cannot act on. Demand a bulleted list of defects or a status/reasoning dict.
- **Reflecting on time-sensitive work**: every loop is another LLM call — latency and cost multiply. Suboptimal for latency-bound applications.
- **Ignoring context growth**: history expands with the initial output, critique, and every refinement — risking context-window overflow and API throttling.
- **A critic with no criteria**: "review this" produces vague notes; name the axes (bugs, style, missing edge cases, requirement coverage).

## Code Examples
LangChain — iterative generate → critique → refine with a stopping sentinel:
```python
from langchain_openai import ChatOpenAI
from langchain_core.messages import SystemMessage, HumanMessage

llm = ChatOpenAI(model="gpt-4o", temperature=0.1)

task_prompt = """
Your task is to create a Python function named 'calculate_factorial'.
1. Accept a single integer 'n' as input.
2. Calculate its factorial (n!).
3. Include a clear docstring explaining what the function does.
4. Handle edge cases: The factorial of 0 is 1.
5. Handle invalid input: Raise a ValueError if the input is a negative number.
"""

max_iterations = 3
current_code = ""
message_history = [HumanMessage(content=task_prompt)]

for i in range(max_iterations):
    # --- 1. GENERATE / REFINE ---
    if i == 0:
        response = llm.invoke(message_history)
    else:
        message_history.append(
            HumanMessage(content="Please refine the code using the critiques provided."))
        response = llm.invoke(message_history)
    current_code = response.content
    message_history.append(response)

    # --- 2. REFLECT (critic persona, separate call) ---
    reflector_prompt = [
        SystemMessage(content="""
            You are a senior software engineer and an expert in Python.
            Your role is to perform a meticulous code review.
            Critically evaluate the provided Python code based on the original task
            requirements. Look for bugs, style issues, missing edge cases, and areas
            for improvement.
            If the code is perfect and meets all requirements, respond with the single
            phrase 'CODE_IS_PERFECT'. Otherwise, provide a bulleted list of your critiques.
        """),
        HumanMessage(content=f"Original Task:\n{task_prompt}\n\nCode to Review:\n{current_code}")
    ]
    critique = llm.invoke(reflector_prompt).content

    # --- 3. STOPPING CONDITION ---
    if "CODE_IS_PERFECT" in critique:
        break
    message_history.append(
        HumanMessage(content=f"Critique of the previous code:\n{critique}"))
```
- **What it demonstrates**: the critic is a *separate invocation with its own system message* — it never sees the producer's history, only the task and the code. The critique is then folded back into the producer's history for the next pass.

Google ADK — Producer/Critic as two agents over shared state:
```python
from google.adk.agents import SequentialAgent, LlmAgent

generator = LlmAgent(
    name="DraftWriter",
    description="Generates initial draft content on a given subject.",
    instruction="Write a short, informative paragraph about the user's subject.",
    output_key="draft_text")

reviewer = LlmAgent(
    name="FactChecker",
    description="Reviews a given text for factual accuracy and provides a structured critique.",
    instruction="""
    You are a meticulous fact-checker.
    1. Read the text provided in the state key 'draft_text'.
    2. Carefully verify the factual accuracy of all claims.
    3. Your final output must be a dictionary containing two keys:
      - "status": A string, either "ACCURATE" or "INACCURATE".
      - "reasoning": A string providing a clear explanation for your status,
        citing specific issues if any are found.
    """,
    output_key="review_output")

review_pipeline = SequentialAgent(
    name="WriteAndReview_Pipeline",
    sub_agents=[generator, reviewer])

# 1. generator runs -> state['draft_text']
# 2. reviewer runs  -> reads state['draft_text'], writes state['review_output']
```
- **What it demonstrates**: the critic's contract is a *structured dictionary*, so downstream logic can branch on `status` without parsing prose. ADK's `LoopAgent` is the alternative when you want true iteration rather than one pass.

## Reference Tables

| Use case | Reflection cycle | Benefit |
|---|---|---|
| Blog post | draft → critique flow/tone/clarity → rewrite, repeat to standard | more polished content |
| Python function | write → run tests/static analysis → fix | more robust code |
| Logic puzzle | propose step → check for contradictions → backtrack or continue | better navigation of the problem space |
| Long-document summary | summarize → compare against source key points → refine | more accurate, complete summaries |
| Plan | generate plan → simulate/evaluate feasibility against constraints → revise | more realistic plans |
| Support chatbot | after a user turn, review history + last message for coherence | more natural conversation |

| Variant | Objectivity | Cost | Use when |
|---|---|---|---|
| Self-reflection | lower (own blind spots) | 2 calls/cycle | quick quality bump, generalist task |
| Producer–Critic (separate persona) | higher | 2 calls/cycle + persona design | objectivity or specialized evaluation needed |
| Iterative loop (LangGraph / LoopAgent) | highest | N cycles | quality dominates speed and budget |

## Worked Example
**Refining `calculate_factorial`.** Iteration 1: the producer emits a first implementation from the task prompt. The reflector — instructed as a senior Python engineer — returns a bulleted critique (missing negative-input `ValueError`, no docstring detail, recursion depth concern). Iteration 2: the producer receives "Please refine the code using the critiques provided" with the full history and emits v2. The reflector reviews again; if it responds with the bare phrase `CODE_IS_PERFECT`, the loop breaks and v2 is final. Otherwise iteration 3 runs and the cap ends the loop regardless.

Note the shape of the stopping condition: a **single unambiguous sentinel string** the surrounding code can test with `in`, not a judgement the code has to interpret.

## Key Takeaways
1. Reflection iteratively self-corrects, materially raising quality, accuracy, and adherence to complex instructions.
2. The loop is execute → evaluate/critique → refine → (iterate).
3. The Producer–Critic split is the powerful implementation: a separate agent or prompted role with its own persona and criteria yields objective, structured feedback.
4. Give the critic explicit evaluation axes and an explicit "perfect" sentinel; give the loop an iteration cap.
5. Costs are real: extra latency, extra spend, growing context (window-overflow and throttling risk).
6. A single reflection step fits in plain LCEL; true iterative reflection wants stateful orchestration (LangGraph, ADK `LoopAgent`).
7. Memory makes reflection cumulative rather than episodic; goals give it a benchmark.

## Connects To
- **Ch 1 (Prompt Chaining)**: a reflection cycle is a chain with a backward edge.
- **Ch 8 (Memory Management)**: conversational memory turns each cycle into learning rather than a fresh start.
- **Ch 11 (Goal Setting and Monitoring)**: goals supply the criteria; monitoring supplies the deviation signal reflection corrects.
- **Ch 9 (Learning and Adaptation)**: self-improving agents (SICA) generalize reflection to the agent's own code.
- **Ch 19 (Evaluation and Monitoring)**: the critic is an evaluator run inline instead of offline.
- **Ch 17 (Reasoning Techniques)**: Self-Consistency and Tree of Thoughts are alternative ways to buy quality with extra inference.
