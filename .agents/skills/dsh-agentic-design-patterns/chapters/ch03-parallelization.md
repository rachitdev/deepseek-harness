# Chapter 3: Parallelization

## Core Idea
Identify the parts of a workflow that do not depend on each other's output and run them concurrently, so total latency becomes the duration of the slowest branch rather than the sum of all steps.

## Frameworks Introduced
- **Parallelization pattern**: execute independent components — LLM calls, tool uses, whole sub-agents — simultaneously, then converge on a synthesis step.
  - When to use: a workflow contains multiple independent operations — fetching from several APIs, processing different data chunks, generating multiple content pieces for later synthesis. Especially effective when the sub-tasks are I/O-bound on external services with latency.
  - How:
    1. Map the dependency graph of the workflow; mark nodes with no upstream dependency on each other.
    2. Fan out those nodes concurrently (`RunnableParallel`, `ParallelAgent`, multiple graph edges from one node).
    3. Pass the original input through alongside (`RunnablePassthrough`) so the synthesis step still has it.
    4. Converge at a single aggregation/synthesis node that waits for all branches.
    5. Ground the synthesizer *exclusively* on the branch outputs.
- **Fan-out / fan-in (map → reduce) shape**: `map_chain` runs N branches; `synthesis_prompt` reduces them.

## Key Concepts
- **Concurrency ≠ parallelism**: `asyncio` gives concurrency on a single thread via an event loop that switches when a task is idle (waiting on network). The code still runs on one thread under Python's GIL. Fine for I/O-bound LLM/API work; not for CPU-bound work.
- **RunnableParallel** (LCEL): bundles named runnables into a dict; the runtime executes them side by side and produces a dict of results.
- **RunnablePassthrough**: carries the original input through a parallel block unchanged.
- **ParallelAgent** (ADK): orchestrates concurrent sub-agents; completes once all have finished and written to session state.
- **SequentialAgent** (ADK): orchestrates ordered stages — typically `[ParallelAgent, MergerAgent]`.
- **`output_key`** (ADK): the session-state key each sub-agent writes its result to; the merger reads those keys by name.
- **LLM-Driven Delegation**: an ADK Coordinator whose LLM recognizes independent sub-tasks and triggers concurrent handling by specialized sub-agents.
- **Convergence point**: the graph node where parallel branches rejoin.

## Mental Models
- Think **Amdahl's law for agents**: parallelize the independent gather, accept that synthesis stays sequential. The research example goes from 5 serial steps to 3 stages.
- **Latency hiding, not compute scaling**: the win comes from overlapping waits on external services, not from more CPU.
- Concurrency is a **cost you pay in design, debugging, and logging** — adopt it where the latency win is real, not by default.

## Anti-patterns
- **Parallelizing dependent steps**: if B needs A's output, concurrency produces garbage or races. Only fan out true independents.
- **Assuming `asyncio` gives CPU parallelism**: it does not; the GIL still applies.
- **Letting the merger add outside knowledge**: without an explicit "ground your entire response exclusively on the Input Summaries" instruction, the synthesizer hallucinates beyond what the branches found.
- **Dropping the original input**: without a passthrough, the synthesis prompt loses the topic it is synthesizing about.
- **Parallelizing everything reflexively**: the added complexity in debugging and logging can outweigh a small latency win.

## Code Examples
LangChain — `RunnableParallel` fan-out into a synthesis prompt:
```python
import asyncio
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain_core.runnables import Runnable, RunnableParallel, RunnablePassthrough

llm = ChatOpenAI(model="gpt-4o-mini", temperature=0.7)

summarize_chain: Runnable = (
    ChatPromptTemplate.from_messages([
        ("system", "Summarize the following topic concisely:"), ("user", "{topic}")])
    | llm | StrOutputParser())

questions_chain: Runnable = (
    ChatPromptTemplate.from_messages([
        ("system", "Generate three interesting questions about the following topic:"),
        ("user", "{topic}")])
    | llm | StrOutputParser())

terms_chain: Runnable = (
    ChatPromptTemplate.from_messages([
        ("system", "Identify 5-10 key terms from the following topic, separated by commas:"),
        ("user", "{topic}")])
    | llm | StrOutputParser())

# 1. The parallel block
map_chain = RunnableParallel({
    "summary":   summarize_chain,
    "questions": questions_chain,
    "key_terms": terms_chain,
    "topic":     RunnablePassthrough(),   # keep the original input
})

# 2. The convergence / synthesis step
synthesis_prompt = ChatPromptTemplate.from_messages([
    ("system", """Based on the following information:
     Summary: {summary}
     Related Questions: {questions}
     Key Terms: {key_terms}
     Synthesize a comprehensive answer."""),
    ("user", "Original topic: {topic}")
])

full_parallel_chain = map_chain | synthesis_prompt | llm | StrOutputParser()

asyncio.run(full_parallel_chain.ainvoke("The history of space exploration"))
```
- **What it demonstrates**: three independent LLM calls issued concurrently, their named outputs plus the passthrough topic feeding one reduce step. Invoke with `ainvoke` — the concurrency only materializes on the async path.

Google ADK — `ParallelAgent` inside a `SequentialAgent`:
```python
from google.adk.agents import LlmAgent, ParallelAgent, SequentialAgent
from google.adk.tools import google_search

GEMINI_MODEL = "gemini-2.0-flash"

researcher_agent_1 = LlmAgent(
    name="RenewableEnergyResearcher", model=GEMINI_MODEL,
    instruction="You are an AI Research Assistant specializing in energy. Research the "
                "latest advancements in 'renewable energy sources'. Use the Google Search "
                "tool provided. Summarize your key findings concisely (1-2 sentences). "
                "Output *only* the summary.",
    description="Researches renewable energy sources.",
    tools=[google_search],
    output_key="renewable_energy_result")   # -> session state

# ... researcher_agent_2 (EVResearcher, output_key="ev_technology_result")
# ... researcher_agent_3 (CarbonCaptureResearcher, output_key="carbon_capture_result")

parallel_research_agent = ParallelAgent(
    name="ParallelWebResearchAgent",
    sub_agents=[researcher_agent_1, researcher_agent_2, researcher_agent_3],
    description="Runs multiple research agents in parallel to gather information.")

merger_agent = LlmAgent(
    name="SynthesisAgent", model=GEMINI_MODEL,
    instruction="""You are an AI Assistant responsible for combining research findings
into a structured report... **Crucially: Your entire response MUST be grounded
*exclusively* on the information provided in the 'Input Summaries' below. Do NOT add
any external knowledge, facts, or details not present in these specific summaries.**

**Input Summaries:**
*  **Renewable Energy:** {renewable_energy_result}
*  **Electric Vehicles:** {ev_technology_result}
*  **Carbon Capture:** {carbon_capture_result}
""",
    description="Combines research findings from parallel agents into a structured, "
                "cited report, strictly grounded on provided inputs.")

sequential_pipeline_agent = SequentialAgent(
    name="ResearchAndSynthesisPipeline",
    sub_agents=[parallel_research_agent, merger_agent],
    description="Coordinates parallel research and synthesizes the results.")

root_agent = sequential_pipeline_agent
```
- **What it demonstrates**: parallel branches communicate through *session state* via `output_key`, not by return value. The merger's prompt interpolates those state keys by name and is hard-constrained against adding outside knowledge.

## Reference Tables

| Use case | Parallel tasks | Benefit |
|---|---|---|
| Company research | news articles, stock data, social mentions, company DB | comprehensive view far faster |
| Customer feedback analysis | sentiment, keyword extraction, categorization, urgency detection | multi-faceted analysis quickly |
| Travel planning | flight prices, hotel availability, local events, restaurants | complete plan faster |
| Marketing email | subject line, body draft, image search, CTA text | assembles final email efficiently |
| Input validation | email format, phone, address lookup, profanity check | faster validity feedback |
| Multi-modal post analysis | text sentiment/keywords ∥ image objects/scene | integrates modalities quickly |
| A/B option generation | three headlines with different prompts/models | quick comparison and selection |

| Framework | Construct |
|---|---|
| LangChain (LCEL) | `RunnableParallel` — dict of runnables executed side by side |
| LangGraph | multiple nodes reachable from one state transition; converge downstream |
| Google ADK | `ParallelAgent` (explicit) or LLM-Driven Delegation from a Coordinator |

## Worked Example
**Research-and-summarize agent.** Sequential shape:

1. Search Source A → 2. Summarize A → 3. Search Source B → 4. Summarize B → 5. Synthesize.

Parallel shape:

1. Search A **and** Search B simultaneously.
2. Once both complete, Summarize A **and** Summarize B simultaneously.
3. Synthesize the final answer from summaries A and B — sequential, waiting on the parallel stages.

Fully realized in the ADK example above: three researchers hit Google Search concurrently, each writing a 1–2 sentence summary to its own state key; a `SequentialAgent` then runs a merger that emits a report with a heading per topic and a short overall conclusion, grounded only on those three summaries.

## Key Takeaways
1. Parallelization runs independent tasks concurrently to cut total latency; it shines when tasks wait on external resources.
2. `asyncio` gives concurrency (event-loop switching on idle), not GIL-defeating parallelism — right tool for I/O-bound LLM/API work.
3. `RunnableParallel` is the LCEL construct; `ParallelAgent` + `output_key` state is the ADK equivalent; LangGraph does it topologically.
4. Always keep a convergence step, and pass the original input through to it.
5. Hard-ground the synthesizer on the parallel outputs to prevent it inventing content.
6. Concurrency adds real cost in design, debugging, and logging — apply it where the latency win justifies that.
7. Combine parallel gather with sequential chaining and conditional routing to build high-performance workflows.

## Connects To
- **Ch 1 (Prompt Chaining)**: parallel gather → chained synthesis is the standard hybrid.
- **Ch 2 (Routing)**: routing chooses one path; parallelization takes several at once.
- **Ch 7 (Multi-Agent Collaboration)**: ParallelAgent is a multi-agent topology.
- **Ch 16 (Resource-Aware Optimization)**: concurrency trades cost/complexity for latency.
- **Ch 17 (Reasoning Techniques)**: Self-Consistency generates diverse reasoning paths in parallel and majority-votes.
