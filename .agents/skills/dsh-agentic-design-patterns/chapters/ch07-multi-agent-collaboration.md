# Chapter 7: Multi-Agent Collaboration

## Core Idea
Decompose a high-level objective into sub-problems and assign each to an agent with the tools, data access, and reasoning best suited to it — the efficacy comes not from division of labor alone but from the communication protocol and shared ontology that keep the collective output coherent.

## Frameworks Introduced
- **Multi-Agent Collaboration pattern**: a cooperative ensemble of distinct specialized agents, each with a defined role, aligned goals, and potentially different tools or knowledge bases.
  - When to use: a task is too complex for one agent and decomposes into sub-tasks needing specialized skills or tools; problems benefiting from diverse expertise, parallel processing, or a multi-stage structured workflow.
  - How: (1) delineate agent roles and responsibilities, (2) establish communication channels, (3) formulate the task flow / interaction protocol directing the collaboration.
- **Six collaboration forms**:
  - **Sequential Handoffs** — one agent completes a task and passes output to the next (a pipeline, but explicitly across agents).
  - **Parallel Processing** — agents work different parts simultaneously; results combined later.
  - **Debate and Consensus** — agents with varied perspectives and information sources discuss to evaluate options and reach a more informed decision.
  - **Hierarchical Structures** — a manager delegates dynamically based on tool access or plugin capabilities and synthesizes results; each agent handles a relevant *group* of tools rather than one agent owning all of them.
  - **Expert Teams** — domain specialists (researcher, writer, editor) collaborate on a complex output.
  - **Critic-Reviewer** — one group creates plans/drafts/answers; a second group assesses for policy adherence, security, compliance, correctness, quality, and organizational alignment; the creator or a final agent revises. Effective for code generation, research writing, logic checking, and ethical alignment; increases robustness and quality and reduces hallucinations.
- **The six interrelationship / communication topologies** — the chapter's central design taxonomy:
  1. **Single Agent** — autonomous, no interaction. Simple to implement and manage; limited by one agent's scope and resources. Suitable when the problem decomposes into independent sub-problems each solvable alone.
  2. **Network** — decentralized peer-to-peer interaction, sharing information, resources, and tasks. Resilient (one failure doesn't cripple the system), but communication overhead and coherent decision-making are hard at scale.
  3. **Supervisor** — a dedicated agent coordinates subordinates and acts as central hub for communication, task allocation, and conflict resolution. Clear authority, simpler control — but a single point of failure and a bottleneck when overwhelmed.
  4. **Supervisor as a Tool** — the supervisor provides resources, guidance, tools, data, or computational services rather than dictating actions. Leverages supervisor capability without rigid top-down control.
  5. **Hierarchical** — multiple supervisor layers over operational agents at the lowest tier. Structured scalability and complexity management with distributed decision-making inside defined boundaries.
  6. **Custom** — hybrids or novel designs tailored to the problem; arises from optimizing specific metrics, highly dynamic environments, or domain-specific knowledge. Requires deep understanding of coordination mechanisms and emergent behaviors.
- **Agent as a Tool** (`AgentTool`): wrap an agent so a higher-level agent can invoke it like a function, creating a layered system.

## Key Concepts
- **Task decomposition**: the principle the whole pattern rests on.
- **Standardized communication protocol + shared ontology**: required for agents to exchange data, delegate sub-tasks, and coordinate coherently.
- **Synergy**: collective performance exceeding any single agent in the ensemble.
- **Modularity, scalability, robustness**: one agent's failure does not necessarily fail the system.
- **`context=[research_task]`** (CrewAI): declares that a task consumes another task's output — the dependency edge.
- **`sub_agents`** (ADK): establishes the parent-child relationship; `agent.parent_agent` is set automatically.
- **`BaseAgent` subclass**: a custom, non-LLM agent implementing `_run_async_impl` and yielding `Event`s.
- **`EventActions(escalate=True)`**: the signal that terminates a `LoopAgent`.
- **`AgentTool`**: bridges one agent into another's tool list; its `description` is what the parent sees and selects on.

## Mental Models
- **Mirror a human team**: a research team has a searcher, a summarizer, a trend-spotter, and a synthesizer — model the agents on the roles you'd hire.
- Pick the topology by **failure mode you can least afford**: Supervisor for clarity and control (accept the bottleneck), Network for resilience (accept the overhead), Hierarchical when the problem itself nests.
- **Separate actions from reasoning**: give the specialist agent a plain function tool and let the LLM handle intent — the `generate_image` function does the work; `ImageGen` decides when and with what prompt.
- Multi-agent collaboration is the same three patterns composed: **sequential = chaining, parallel = parallelization, hierarchical = routing** — applied at agent rather than step granularity.

## Anti-patterns
- **One agent holding all the tools**: defeats specialization; give each agent a relevant *group* of tools.
- **Multi-agent for a well-defined single-domain problem**: a monolithic agent is effective there; the ensemble adds coordination cost for nothing.
- **Unstructured large Network topologies**: communication overhead and incoherent decisions.
- **An overloaded Supervisor**: single point of failure and throughput bottleneck.
- **Implicit task dependencies**: in CrewAI, declare `context=[prior_task]` — don't assume the writer sees the researcher's output.
- **A vague `AgentTool` description**: the parent selects on that string alone.

## Code Examples
CrewAI — researcher → writer with an explicit dependency:
```python
from crewai import Agent, Task, Crew, Process
from langchain_google_genai import ChatGoogleGenerativeAI

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash")

researcher = Agent(
    role='Senior Research Analyst',
    goal='Find and summarize the latest trends in AI.',
    backstory="You are an experienced research analyst with a knack for identifying "
              "key trends and synthesizing information.",
    verbose=True, allow_delegation=False)

writer = Agent(
    role='Technical Content Writer',
    goal='Write a clear and engaging blog post based on research findings.',
    backstory="You are a skilled writer who can translate complex technical topics "
              "into accessible content.",
    verbose=True, allow_delegation=False)

research_task = Task(
    description="Research the top 3 emerging trends in Artificial Intelligence in "
                "2024-2025. Focus on practical applications and potential impact.",
    expected_output="A detailed summary of the top 3 AI trends, including key points and sources.",
    agent=researcher)

writing_task = Task(
    description="Write a 500-word blog post based on the research findings. The post "
                "should be engaging and easy for a general audience to understand.",
    expected_output="A complete 500-word blog post about the latest AI trends.",
    agent=writer,
    context=[research_task])      # <- the dependency edge

blog_creation_crew = Crew(
    agents=[researcher, writer], tasks=[research_task, writing_task],
    process=Process.sequential, llm=llm, verbose=2)

result = blog_creation_crew.kickoff()
```

Google ADK — hierarchy with a custom non-LLM agent:
```python
from google.adk.agents import LlmAgent, BaseAgent
from google.adk.agents.invocation_context import InvocationContext
from google.adk.events import Event
from typing import AsyncGenerator

class TaskExecutor(BaseAgent):
    """A specialized agent with custom, non-LLM behavior."""
    name: str = "TaskExecutor"
    description: str = "Executes a predefined task."

    async def _run_async_impl(self, context: InvocationContext) -> AsyncGenerator[Event, None]:
        yield Event(author=self.name, content="Task finished successfully.")

greeter   = LlmAgent(name="Greeter", model="gemini-2.0-flash-exp",
                     instruction="You are a friendly greeter.")
task_doer = TaskExecutor()

coordinator = LlmAgent(
    name="Coordinator", model="gemini-2.0-flash-exp",
    description="A coordinator that can greet users and execute tasks.",
    instruction="When asked to greet, delegate to the Greeter. When asked to perform "
                "a task, delegate to the TaskExecutor.",
    sub_agents=[greeter, task_doer])

assert greeter.parent_agent == coordinator   # ADK wires this automatically
```

ADK — `LoopAgent` with a state-driven stop condition:
```python
from google.adk.agents import LoopAgent, LlmAgent, BaseAgent
from google.adk.events import Event, EventActions

class ConditionChecker(BaseAgent):
    """A custom agent that checks for a 'completed' status in the session state."""
    name: str = "ConditionChecker"
    description: str = "Checks if a process is complete and signals the loop to stop."

    async def _run_async_impl(self, context) -> AsyncGenerator[Event, None]:
        status = context.session.state.get("status", "pending")
        if status == "completed":
            yield Event(author=self.name, actions=EventActions(escalate=True))  # stop
        else:
            yield Event(author=self.name, content="Condition not met, continuing loop.")

process_step = LlmAgent(
    name="ProcessingStep", model="gemini-2.0-flash-exp",
    instruction="You are a step in a longer process. Perform your task. If you are the "
                "final step, update session state by setting 'status' to 'completed'.")

poller = LoopAgent(name="StatusPoller", max_iterations=10,
                   sub_agents=[process_step, ConditionChecker()])
```
- **What it demonstrates**: a loop needs *both* an escalation condition and `max_iterations`. Belt and braces.

ADK — Sequential and Parallel via session state:
```python
from google.adk.agents import SequentialAgent, ParallelAgent, Agent

step1 = Agent(name="Step1_Fetch", output_key="data")          # -> session.state["data"]
step2 = Agent(name="Step2_Process",
              instruction="Analyze the information found in state['data'] and provide a summary.")
pipeline = SequentialAgent(name="MyPipeline", sub_agents=[step1, step2])

weather_fetcher = Agent(name="weather_fetcher", model="gemini-2.0-flash-exp",
    instruction="Fetch the weather for the given location and return only the weather report.",
    output_key="weather_data")
news_fetcher = Agent(name="news_fetcher", model="gemini-2.0-flash-exp",
    instruction="Fetch the top news story for the given topic and return only that story.",
    output_key="news_data")
data_gatherer = ParallelAgent(name="data_gatherer", sub_agents=[weather_fetcher, news_fetcher])
```

ADK — Agent as a Tool:
```python
from google.adk.agents import LlmAgent
from google.adk.tools import agent_tool

# 1. A plain function tool for the core capability — separate actions from reasoning.
def generate_image(prompt: str) -> dict:
    """
    Generates an image based on a textual prompt.
    Args:
        prompt: A detailed description of the image to generate.
    Returns:
        A dictionary with the status and the generated image bytes.
    """
    return {"status": "success", "image_bytes": b"...", "mime_type": "image/png"}

# 2. The specialist agent that owns the tool.
image_generator_agent = LlmAgent(
    name="ImageGen", model="gemini-2.0-flash",
    description="Generates an image based on a detailed text prompt.",
    instruction="You are an image generation specialist. Your task is to take the user's "
                "request and use the 'generate_image' tool to create the image. The user's "
                "entire request should be used as the 'prompt' argument for the tool. "
                "After the tool returns the image bytes, you MUST output the image.",
    tools=[generate_image])

# 3. Wrap it — this description is what the parent agent sees.
image_tool = agent_tool.AgentTool(
    agent=image_generator_agent,
    description="Use this tool to generate an image. The input should be a descriptive "
                "prompt of the desired image.")

# 4. The orchestrating agent.
artist_agent = LlmAgent(
    name="Artist", model="gemini-2.0-flash",
    instruction="You are a creative artist. First, invent a creative and descriptive prompt "
                "for an image. Then, use the 'ImageGen' tool to generate the image using your prompt.",
    tools=[image_tool])
```

## Reference Tables

| Topology | Strength | Weakness | Use when |
|---|---|---|---|
| Single Agent | simplest to implement and manage | bounded by one agent's scope | sub-problems are independent and self-contained |
| Network | resilient; peer-to-peer sharing | communication overhead; incoherent decisions at scale | decentralization matters more than control |
| Supervisor | clear authority; simple control | single point of failure; bottleneck | a modest number of subordinates, clear task allocation |
| Supervisor as a Tool | leverages central capability without rigid control | less coordination guarantee | agents are competent but need shared resources/data |
| Hierarchical | structured scalability; distributed decisions in bounds | more layers, more latency | the problem itself nests into sub-problems |
| Custom | optimal fit | requires deep MAS expertise; emergent behavior risk | specific metrics, dynamic environment, domain knowledge |

| Domain | Agent team |
|---|---|
| Complex research | academic-database searcher, summarizer, trend identifier, synthesizer |
| Software development | requirements analyst, code generator, tester, documentation writer |
| Creative content | market research, copywriter, graphic design (image tools), social scheduling |
| Financial analysis | stock data fetcher, news sentiment, technical analysis, recommendation |
| Customer support | front-line handler → specialist (technical, billing) escalation |
| Supply chain | agents per node (suppliers, manufacturers, distributors) optimizing inventory/logistics |
| Network ops | triage and remediation agents integrating with existing ML models and tooling |

## Worked Example
**Blog post crew (CrewAI).** Two agents, two tasks, one dependency. The researcher's `expected_output` — "a detailed summary of the top 3 AI trends, including key points and sources" — is the contract the writer's task consumes via `context=[research_task]`. Without that `context` declaration the writer starts from nothing and invents trends. `Process.sequential` guarantees ordering; the dependency declaration guarantees the *data flow*. Those are two separate things and both are required.

**Artist → ImageGen (ADK "Agent as a Tool").** The layering matters: `generate_image` is a dumb function that makes an image; `ImageGen` is a specialist that decides how to call it and what to do with the bytes; `Artist` is a creative that invents the prompt and delegates. Each layer has exactly one job, and `AgentTool`'s description is the interface contract between the top two.

## Key Takeaways
1. Multi-agent collaboration decomposes a problem across specialized agents with defined roles, tools, and goals.
2. Efficacy depends on inter-agent communication — a standardized protocol and shared ontology — not only on division of labor.
3. Six collaboration forms: sequential handoff, parallel, debate/consensus, hierarchical, expert teams, critic-reviewer.
4. Six topologies: Single, Network, Supervisor, Supervisor-as-Tool, Hierarchical, Custom — choose by task complexity, agent count, autonomy, robustness needs, and acceptable communication overhead.
5. Distribute tools by agent; don't let one agent own them all.
6. Declare data dependencies explicitly (CrewAI `context`, ADK `output_key` + state references).
7. `AgentTool` makes an agent callable as a function, enabling layered systems.
8. Loops need both an escalation condition and a `max_iterations` cap.

## Connects To
- **Ch 2 (Routing)**: the Supervisor topology is routing at agent granularity.
- **Ch 3 (Parallelization)**: `ParallelAgent` is this pattern's concurrent form.
- **Ch 4 (Reflection)**: Critic-Reviewer is Producer–Critic scaled to teams.
- **Ch 5 (Tool Use)**: Agent-as-a-Tool unifies tools and agents under one interface.
- **Ch 8 (Memory Management)**: session state is the shared blackboard agents communicate through.
- **Ch 15 (Inter-Agent Communication / A2A)**: the standardized protocol this pattern depends on.
- **Ch 17 (Reasoning Techniques)**: MASS covers multi-agent-system optimization.
