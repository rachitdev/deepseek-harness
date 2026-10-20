# Chapter 6: Planning

## Core Idea
Planning is the agent's ability to formulate a sequence of actions from an initial state to a goal state when the *how* is not known in advance — and to re-plan when reality invalidates the first attempt.

## Frameworks Introduced
- **Planning pattern**: given a goal and constraints, autonomously chart a course; treat the initial plan as a starting point, not a script.
  - When to use: a request is too complex for a single action or tool, and requires a sequence of interdependent operations to reach a synthesized outcome — research reports, employee onboarding, competitive analysis.
  - How:
    1. Establish the **initial state** (budget, participants, dates, available tools) and the **goal state**.
    2. Decompose the objective into discrete, executable sub-goals in dependency order.
    3. Execute; monitor for new information and obstacles.
    4. On a blocked step, register the new constraint, re-evaluate options, and formulate a new plan rather than failing.
  - Why it works: LLMs generate plausible and effective plans from vast training data; explicitly prompting or structuring the task to require a planning step is what elicits the behavior.
- **The delegation test — the pattern's central decision rule**: *does the "how" need to be discovered, or is it already known?* When a solution is well-understood and repeatable, constrain the agent to a predetermined fixed workflow. Dynamic planning is a specific tool, not a universal solution; it trades predictability for flexibility.
- **Deep Research architecture** (Google Gemini DeepResearch): a managed, long-running agentic pipeline —
  1. Deconstruct the user prompt into a multi-point research plan.
  2. **Present the plan to the user for review and modification** before execution (collaborative shaping of the trajectory).
  3. Run an iterative search-and-analysis loop: dynamically formulate and refine queries, identify knowledge gaps, corroborate data points, resolve discrepancies.
  4. Run **asynchronously** so the investigation survives single-point failures and the user can disengage and be notified on completion.
  5. Synthesize: critically evaluate collected information, identify major themes, organize into a coherent narrative with logical sections.
  6. Return the structured multi-page report **plus the full list of sources consulted** as citations.

## Key Concepts
- **Initial state / goal state**: planning is state-space traversal — the plan is the path between them.
- **Adaptability**: the agent's real power is incorporating new information and steering around obstacles, not producing a perfect first plan.
- **Flexibility/predictability trade-off**: autonomy reduces determinism; constraining the agent reduces risk of unpredictable behavior and guarantees consistent outcomes.
- **Plan-then-execute task shape**: a single task whose `expected_output` demands both a `### Plan` section and a `### Summary` section, forcing the plan into the artifact.
- **`Process.sequential`** (CrewAI): the crew's ordered execution mode.
- **Intermediate step transparency** (OpenAI Deep Research API): the API exposes reasoning steps, the exact web-search queries executed, and any code run — unlike the abstracted ChatGPT experience.
- **Grounded, cited output**: annotations carry `start_index`/`end_index` spans, `title`, and `url`, so every claim is traceable.

## Mental Models
- Think of a planning agent as a **specialist you delegate a goal to**: you define the *what* (objective and constraints), it discovers the *how*.
- **"Organize a team offsite"** is the canonical prompt: budget and dates are the initial state, a booked offsite is the goal, and a fully-booked caterer is the obstacle that separates a planner from a script.
- A plan presented for **human review before execution** converts a risky autonomous run into a collaborative one — the cheapest correction point in the whole pipeline.
- **Automate the iterative search-and-filter cycle**, which is the actual bottleneck in manual research; comprehensiveness comes from processing more sources than a human can, which also reduces selection bias.

## Anti-patterns
- **Planning a solved problem**: when the solution is well-understood and repeatable, dynamic planning adds uncertainty for no gain. Use a fixed workflow.
- **Treating the initial plan as a rigid script**: a capable agent re-plans on a blocked step; a brittle one fails.
- **Executing a long research plan without showing it first**: hours of work aimed at the wrong question.
- **Synchronous long-running research**: an investigation across hundreds of sources needs asynchrony to be resilient and to let the user disengage.
- **Reports without citations**: unverifiable claims. Return the source list and inline annotations.
- **Hiding intermediate steps**: without visible reasoning, queries, and code, you cannot debug why the answer came out that way.

## Code Examples
CrewAI — force the plan into the deliverable:
```python
from crewai import Agent, Task, Crew, Process
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(model="gpt-4-turbo")

planner_writer_agent = Agent(
    role='Article Planner and Writer',
    goal='Plan and then write a concise, engaging summary on a specified topic.',
    backstory=('You are an expert technical writer and content strategist. '
               'Your strength lies in creating a clear, actionable plan before writing, '
               'ensuring the final summary is both informative and easy to digest.'),
    verbose=True, allow_delegation=False, llm=llm)

topic = "The importance of Reinforcement Learning in AI"

high_level_task = Task(
    description=(f"1. Create a bullet-point plan for a summary on the topic: '{topic}'.\n"
                 f"2. Write the summary based on your plan, keeping it around 200 words."),
    expected_output=("A final report containing two distinct sections:\n\n"
                     "### Plan\n"
                     "A bulleted list outlining the main points of the summary.\n\n"
                     "### Summary\n"
                     "A concise and well-structured summary of the topic."),
    agent=planner_writer_agent)

crew = Crew(agents=[planner_writer_agent], tasks=[high_level_task],
            process=Process.sequential)
result = crew.kickoff()
```
- **What it demonstrates**: the plan is elicited by making it a *required section of the expected output*, not by hoping the model plans internally. That also makes the plan inspectable.

OpenAI Deep Research API — plan, search, cite, and expose every step:
```python
from openai import OpenAI
client = OpenAI(api_key="YOUR_OPENAI_API_KEY")

system_message = """You are a professional researcher preparing a structured,
data-driven report. Focus on data-rich insights, use reliable sources, and
include inline citations."""
user_query = "Research the economic impact of semaglutide on global healthcare systems."

response = client.responses.create(
    model="o3-deep-research-2025-06-26",   # o4-mini-deep-research-... for lower latency
    input=[
        {"role": "developer", "content": [{"type": "input_text", "text": system_message}]},
        {"role": "user",      "content": [{"type": "input_text", "text": user_query}]},
    ],
    reasoning={"summary": "auto"},
    tools=[{"type": "web_search_preview"}],   # + code_interpreter or MCP tools as needed
)

final_report = response.output[-1].content[0].text

# --- Inline citations and metadata ---
for i, citation in enumerate(response.output[-1].content[0].annotations):
    cited_text = final_report[citation.start_index:citation.end_index]
    print(f"Citation {i+1}: {citation.title} — {citation.url}")
    print(f"  Cited text: {cited_text}")

# --- Intermediate steps: reasoning, searches, code ---
reasoning_step = next(i for i in response.output if i.type == "reasoning")
search_step    = next(i for i in response.output if i.type == "web_search_call")
code_step      = next(i for i in response.output if i.type == "code_interpreter_call")
print(search_step.action['query'], search_step.status)
```
- **What it demonstrates**: three things worth copying — a system message that sets persona *and* output discipline; citations as character spans into the report so each claim is anchored; and `response.output` as a step log you can audit (`reasoning`, `web_search_call`, `code_interpreter_call`).

## Reference Tables

| Domain | What planning produces |
|---|---|
| Procedural task automation | onboarding decomposed into account creation, training assignment, department coordination, in dependency order |
| Robotics / autonomous navigation | a path or action sequence optimizing time or energy under environmental constraints |
| Structured information synthesis | phases for gathering, summarization, structuring, iterative refinement |
| Customer support | systematic plan for diagnosis, solution implementation, escalation |

| | Planning agent | Fixed-workflow agent |
|---|---|---|
| The "how" | discovered at runtime | known in advance |
| Strength | adapts around obstacles | predictable, consistent, auditable |
| Cost | uncertainty, variable behavior | cannot handle novel obstacles |
| Use when | goal is complex/novel, environment dynamic | solution is well-understood and repeatable |

| Deep Research capability | Why it matters |
|---|---|
| Plan shown before execution | cheapest correction point |
| Asynchronous long-running | resilient to single-point failure; user can disengage |
| User-provided documents | blends private sources with web research |
| Inline citations + full source list | verifiability and transparency |
| Interactive report (audio overview, charts, links) | supports verification and further exploration |
| MCP support (OpenAI API) | connects private knowledge bases and internal data |

## Worked Example
**European VC investment analysis** (the book's DeepResearch walkthrough). The user asks about VC investment growth in Europe. The agent does not search immediately — it emits a research plan and shows it for editing:

1. Find the total volume of VC investment in Europe for the current year and the previous year.
2. Find reports detailing the top European countries by VC investment volume.
3. Look for year-over-year growth rate data across European countries.
4. Identify countries with the largest recent percentage increase.
5. Search for analyses specifically highlighting countries with accelerating activity.
6. Investigate which sectors are driving the acceleration.

Only after the user approves (or edits) does the search-and-analysis loop run. During synthesis the agent hits a real problem the plan did not anticipate: sources disagree — Crunchbase reports ~$51B (−5% YoY), Invest Europe reports a 26% increase, GoingVC reports a 39% decline, PitchBook estimates €57.3B (−6.7%). A concatenating summarizer would pick one. The synthesizing agent instead **reports the discrepancy and explains it** — differing methodologies, deal-size thresholds, scope of what counts as VC, collection timing — then extracts the trend all sources do agree on: volume remains well below the 2021 peak. That is the difference between a list of findings and a report.

## Key Takeaways
1. Planning breaks complex goals into actionable, sequential steps and is essential for multi-step tasks, workflow automation, and complex environments.
2. The decision rule is whether the "how" must be discovered — if it is already known, constrain the agent to a fixed workflow instead.
3. An initial plan is a starting point; adaptability on obstacles is the real capability.
4. LLMs plan well when explicitly prompted or when the task structure demands a plan as output.
5. Show long research plans to the user before execution; it is the cheapest place to correct course.
6. Run long investigations asynchronously and return citations plus the full source list.
7. Expose intermediate steps — reasoning, queries, code — so the process is debuggable.
8. Google DeepResearch reflects, plans, and executes, using Google Search as a tool on your behalf.

## Connects To
- **Ch 1 (Prompt Chaining)**: a plan executed step-by-step is a chain generated at runtime.
- **Ch 4 (Reflection)**: evaluating a plan's feasibility before executing it is reflection applied to plans.
- **Ch 5 (Tool Use)**: plans orchestrate tools; web search is the canonical one.
- **Ch 11 (Goal Setting and Monitoring)**: goals define the target state a plan traverses to; monitoring triggers re-planning.
- **Ch 10 (Model Context Protocol)**: how Deep Research reaches private knowledge bases.
- **Ch 21 (Exploration and Discovery)**: Google Co-scientist extends iterative planning to hypothesis generation.
