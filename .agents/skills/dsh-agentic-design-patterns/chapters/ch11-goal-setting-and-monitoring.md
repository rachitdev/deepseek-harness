# Chapter 11: Goal Setting and Monitoring

## Core Idea
Give an agent explicit, measurable objectives *and* a mechanism to track progress against them — the feedback loop between the two is what turns a reactive executor into a proactive, self-assessing system.

## Frameworks Introduced
- **Goal Setting and Monitoring pattern**: define clear measurable objectives; establish monitoring that continuously tracks the agent's progress and environment state against those goals; close the loop so the agent can assess performance, correct course, and adapt its plan when it deviates.
  - When to use: an agent must autonomously execute a multi-step task, adapt to dynamic conditions, and reliably achieve a specific high-level objective without constant human intervention.
  - How:
    1. State the goal as **SMART** — specific, measurable, achievable, relevant, time-bound.
    2. Define the metrics and success criteria explicitly; ambiguous criteria make monitoring meaningless.
    3. Monitor agent actions, environmental states, and tool outputs.
    4. Feed deviations back so the agent revises the plan, adapts, or escalates.
    5. Cap iterations so the loop terminates.
- **The generate → self-evaluate → refine loop with a binary verdict**: rather than asking the model whether the work is "good", ask a reviewer prompt to respond **with only one word: True or False**, making the stop condition mechanically testable.
- **Separation of concerns via a crew of role-specific agents** — the chapter's recommended upgrade over single-agent self-judging:
  - **The Peer Programmer** — helps write and brainstorm code.
  - **The Code Reviewer** — catches errors and suggests improvements (holds the judge prompt).
  - **The Documenter** — generates clear, concise documentation.
  - **The Test Writer** — creates comprehensive unit tests.
  - **The Prompt Refiner** — optimizes interactions with the AI.
  - Why it works: a Code Reviewer distinct from the programmer agent significantly improves objective evaluation, and having a Test Writer naturally produces the tests the single-agent version skips.

## Key Concepts
- **Goal state vs initial state**: the trip-planning analogy — decide the destination, establish where you start, consider options and constraints, then map the dependent sequence of steps.
- **Success criteria**: what "done" means, stated so a monitor can evaluate it.
- **Iteration cap** (`max_iterations`): the guard against a monitoring loop running forever.
- **Self-assessment prompt**: a separate LLM invocation whose only job is to judge feedback against goals and emit `True`/`False`.
- **Escalation**: what happens when the goal cannot be met — a first-class outcome, not a failure of the pattern.
- In **Google ADK**, goals are conveyed through agent **instructions**, and monitoring is accomplished through **state management** and tool interactions.

## Mental Models
- **A goal is the benchmark reflection measures against.** Goal setting supplies the yardstick, monitoring supplies the reading, reflection is the corrective engine.
- **Ask for a boolean, not a judgement.** "Respond with only one word: True or False" turns a subjective assessment into a loop condition.
- **Don't let the author grade the exam.** When the same LLM writes the code and judges it, it has a harder time discovering it is going in the wrong direction.
- Monitoring watches three surfaces: **the agent's own actions, the environment's state, and tool outputs.**

## Anti-patterns
- **Vague goals**: "make it good". The LLM may not grasp the intended meaning of a goal and will incorrectly assess its performance as successful.
- **Same model as author and judge**: the central caveat of the chapter's example — self-evaluation without an independent reviewer misses systematic errors.
- **Trusting the verdict without running the code**: LLMs do not produce flawless code by magic; you still need to run and test what they produce.
- **No iteration cap**: basic monitoring creates a real risk of the process running forever.
- **Treating the example as production-ready**: the chapter says so explicitly — hallucination, misread goals, and weak monitoring are all live risks.
- **Monitoring without a feedback path**: measuring progress that never changes behavior is telemetry, not monitoring.

## Code Examples
The three prompts that make the loop work (LangChain + GPT-4o):
```python
def generate_prompt(use_case: str, goals: list[str],
                    previous_code: str = "", feedback: str = "") -> str:
    base_prompt = f"""
You are an AI coding agent. Your job is to write Python code based on the following use case:

Use Case: {use_case}

Your goals are:
{chr(10).join(f"- {g.strip()}" for g in goals)}
"""
    if previous_code:
        base_prompt += f"\nPreviously generated code:\n{previous_code}"
    if feedback:
        base_prompt += f"\nFeedback on previous version:\n{feedback}\n"
    base_prompt += ("\nPlease return only the revised Python code. "
                    "Do not include comments or explanations outside the code.")
    return base_prompt


def get_code_feedback(code: str, goals: list[str]) -> str:
    """The critic: produce a critique, not a verdict."""
    feedback_prompt = f"""
You are a Python code reviewer. A code snippet is shown below.
Based on the following goals:
{chr(10).join(f"- {g.strip()}" for g in goals)}

Please critique this code and identify if the goals are met.
Mention if improvements are needed for clarity, simplicity, correctness,
edge case handling, or test coverage.

Code:
{code}
"""
    return llm.invoke(feedback_prompt)


def goals_met(feedback_text: str, goals: list[str]) -> bool:
    """The monitor: collapse the critique to a single testable boolean."""
    review_prompt = f"""
You are an AI reviewer.
Here are the goals:
{chr(10).join(f"- {g.strip()}" for g in goals)}

Here is the feedback on the code:
\"\"\"
{feedback_text}
\"\"\"

Based on the feedback above, have the goals been met?
Respond with only one word: True or False.
"""
    return llm.invoke(review_prompt).content.strip().lower() == "true"
```

The monitored loop:
```python
def run_code_agent(use_case: str, goals_input: str, max_iterations: int = 5) -> str:
    goals = [g.strip() for g in goals_input.split(",")]
    previous_code, feedback = "", ""

    for i in range(max_iterations):
        prompt = generate_prompt(use_case, goals, previous_code,
                                 feedback if isinstance(feedback, str) else feedback.content)
        code = clean_code_block(llm.invoke(prompt).content.strip())

        feedback = get_code_feedback(code, goals)
        feedback_text = feedback.content.strip()

        if goals_met(feedback_text, goals):
            break                       # goal achieved -> stop
        previous_code = code            # otherwise carry forward for refinement

    final_code = add_comment_header(code, use_case)
    return save_code_to_file(final_code, use_case)


# Example invocation — note how specific the goals are
run_code_agent(
    use_case="Write code to find BinaryGap of a given positive integer",
    goals_input=("Code simple to understand, Functionally correct, "
                 "Handles comprehensive edge cases, Takes positive integer input only, "
                 "prints the results with few examples"))
```

The reviewer prompt worth stealing (the chapter's anti-hallucination code-review persona):
```
Act as an expert code reviewer with a deep commitment to producing clean, correct,
and simple code. Your core mission is to eliminate code "hallucinations" by ensuring
every suggestion is grounded in reality and best practices.

When I provide you with a code snippet, I want you to:
-- Identify and Correct Errors: Point out any logical flaws, bugs, or potential runtime errors.
-- Simplify and Refactor: Suggest changes that make the code more readable, efficient,
   and maintainable without sacrificing correctness.
-- Provide Clear Explanations: For every suggested change, explain why it is an
   improvement, referencing principles of clean code, performance, or security.
-- Offer Corrected Code: Show the "before" and "after" of your suggested changes so
   the improvement is clear.

Your feedback should be direct, constructive, and always aimed at improving the
quality of the code.
```

## Reference Tables

| Domain | Goal | What is monitored | Failure action |
|---|---|---|---|
| Customer support | "resolve customer's billing inquiry" | conversation, database entries, billing change confirmed, customer feedback | escalate |
| Personalized learning | "improve student's understanding of algebra" | exercise progress, accuracy, completion time | adapt materials |
| Project management | "ensure milestone X completed by date Y" | task statuses, team comms, resource availability | flag delay, suggest corrective action |
| Automated trading | "maximize gains within risk tolerance" | market data, portfolio value, risk indicators | adjust strategy on threshold breach |
| Autonomous vehicles | "safely transport passengers A→B" | environment, own state (speed, fuel), route progress | adapt driving behavior |
| Content moderation | "identify and remove harmful content" | incoming content, false positive/negative rates | adjust criteria, escalate ambiguous cases to humans |

| Loop component | Prompt's job | Output shape |
|---|---|---|
| Generator | produce/refine code from goals + prior code + feedback | code only, no prose |
| Critic | critique against each goal | prose critique naming gaps |
| Monitor | decide whether goals are met | exactly `True` or `False` |

## Worked Example
**The BinaryGap agent.** The user supplies two things: a use case ("write code to find the BinaryGap of a given positive integer") and a **quality checklist** as comma-separated goals — simple to understand, functionally correct, handles comprehensive edge cases, positive integer input only, prints results with a few examples.

Iteration 1: the generator produces a first draft. Instead of returning it, the agent invokes the *critic* prompt, which walks the code against each goal and produces a prose critique. That critique then goes to a *third* prompt — the monitor — whose entire job is to answer `True` or `False`. False, so the code and the critique both feed forward into iteration 2's generator prompt.

The cycle repeats until `True` or `max_iterations = 5`. The final code gets a header comment naming the use case and is saved to a filename the LLM itself summarizes into ≤10 characters plus a random suffix.

**Why the three-prompt split matters:** collapsing critic and monitor into one call would produce prose the loop cannot branch on. Collapsing generator and critic would produce the failure the chapter warns about. And the honest limitation stands: the same LLM plays all three roles here, so it can confidently declare `True` on code that does not run. The recommended fix is the five-agent crew — a Code Reviewer that is a genuinely separate entity from the programmer, plus a Test Writer that produces the unit tests this version never writes.

## Key Takeaways
1. Goal Setting and Monitoring equips agents with purpose and a mechanism to track progress.
2. Goals should be **SMART** — specific, measurable, achievable, relevant, time-bound.
3. Clearly defined metrics and success criteria are essential for effective monitoring.
4. Monitoring observes agent actions, environmental states, and tool outputs.
5. Feedback loops from monitoring let agents adapt, revise plans, or escalate.
6. Force the stop condition into a single testable token (`True`/`False`) rather than a prose judgement.
7. Separate the judge from the author — a distinct reviewer agent materially improves evaluation objectivity.
8. Always cap iterations, and always run the produced code; an LLM's self-verdict is not verification.
9. In ADK, goals live in agent instructions and monitoring lives in state management and tool interactions.

## Connects To
- **Ch 4 (Reflection)**: reflection is the corrective engine; goals supply its criteria.
- **Ch 6 (Planning)**: a plan is the route from initial state to goal state; monitoring triggers replanning.
- **Ch 7 (Multi-Agent Collaboration)**: the five-role crew is the multi-agent upgrade to single-agent self-judging.
- **Ch 12 (Exception Handling and Recovery)**: escalation is the shared exit path.
- **Ch 19 (Evaluation and Monitoring)**: the same measurement discipline applied systematically.
- **Ch 28 (Coding Agents)**: this chapter's crew of role-specific coding agents in full.
