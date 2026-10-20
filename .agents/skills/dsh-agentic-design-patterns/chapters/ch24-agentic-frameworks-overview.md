# Chapter 24: A Quick Overview of Agentic Frameworks

## Core Idea
The framework landscape is a spectrum of abstraction levels — from low-level libraries for defining an agent's internal logic to opinionated platforms for orchestrating teams. The choice hinges on whether your application needs **a simple sequence, a dynamic reasoning loop, or a managed team of specialists**.

## Frameworks Introduced
- **LangChain** — components and standardized interfaces for LLM applications. Core strength is **LCEL**, piping components into a chain where each step's output feeds the next. Built for **Directed Acyclic Graphs**: one direction, no loops.
  - Use for: simple RAG (retrieve → prompt → answer), summarization, structured extraction.
- **LangGraph** — built on LangChain for advanced agentic systems. Workflow as a graph of **nodes** (functions or LCEL chains) and **edges** (conditional logic). Main advantage: **cycles** — loop, retry, or call tools in flexible order until complete. Explicitly manages application state passed between nodes.
  - Use for: multi-agent systems (a supervisor routing to workers, looping until the goal is met); plan-and-execute agents (create plan → execute step → loop back to update plan); human-in-the-loop (the graph waits for human input before choosing the next node).
- **Google ADK** — a high-level, structured, **opinionated and production-oriented** framework for orchestrating multiple interacting agents. Abstracts away low-level graph construction with pre-built architectural patterns: built-in agent types like `SequentialAgent` and `ParallelAgent` manage control flow automatically. Architected around a **team** of agents with a primary delegating to specialized sub-agents. State and session management handled implicitly — more cohesive, less granular than LangGraph's explicit state passing.
- **CrewAI** — orchestration for multi-agent systems focused on **collaborative roles and structured processes**. The developer defines actors and assignments rather than logic flow; CrewAI manages their interaction. Three core components:
  - **Agent** — defined by a persona: a specific **role**, **goal**, and **backstory** guiding behavior and communication style.
  - **Task** — a discrete unit of work with a clear description and expected output, assigned to an Agent.
  - **Crew** — the cohesive unit containing Agents and Tasks, executing a **Process**: typically *sequential* (each task's output feeds the next) or *hierarchical* (a manager-like agent delegates and coordinates).
  - Positioning: "Instead of building a state machine, the developer designs a team charter."

## Key Concepts
- **The abstraction ladder**: LangChain (components) → LangGraph (control flow) → ADK/CrewAI (team orchestration).
- **The metaphor**: "LangGraph gives you the detailed tools to design the intricate wiring of a single robot or a team; Google's ADK gives you a factory assembly line designed to build and manage a fleet of robots that already know how to work together."
- **The core trade-off**: granular control of graph-based systems vs streamlined development of opinionated platforms.

## Mental Models
- **Can you draw the flow A → B → C without an arrow going backwards?** If yes, LangChain. If any arrow loops back, LangGraph.
- **Are you wiring a machine or hiring a team?** Graph frameworks want nodes and edges; CrewAI wants roles and assignments.
- **Opinionated frameworks trade granularity for cohesion.** ADK's implicit state is a feature until the moment you need to control it.

## Anti-patterns
- **LangChain for a task requiring retry or reflection**: LCEL is acyclic; you will end up hand-rolling the loop.
- **LangGraph for a three-step linear pipeline**: unnecessary state machinery.
- **Reaching for a data framework for orchestration**: LlamaIndex's agentic control flow is less developed than agent-first frameworks.
- **Conversation-driven orchestration where determinism matters**: AutoGen's paradigm can produce less predictable execution paths.

## Code Examples
LangChain — the LCEL shape:
```python
# A simple LCEL chain, conceptually
chain = prompt | model | output_parser
```

LangGraph — parallel fan-out and aggregation:
```python
class State(TypedDict):
    topic: str
    joke: str
    story: str
    poem: str
    combined_output: str

def call_llm_1(state: State):
    """First LLM call to generate initial joke"""
    return {"joke": llm.invoke(f"Write a joke about {state['topic']}").content}

def call_llm_2(state: State):
    """Second LLM call to generate story"""
    return {"story": llm.invoke(f"Write a story about {state['topic']}").content}

def call_llm_3(state: State):
    """Third LLM call to generate poem"""
    return {"poem": llm.invoke(f"Write a poem about {state['topic']}").content}

def aggregator(state: State):
    """Combine the joke, story and poem into a single output"""
    combined = f"Here's a story, joke, and poem about {state['topic']}!\n\n"
    combined += f"STORY:\n{state['story']}\n\n"
    combined += f"JOKE:\n{state['joke']}\n\n"
    combined += f"POEM:\n{state['poem']}"
    return {"combined_output": combined}

parallel_builder = StateGraph(State)
parallel_builder.add_node("call_llm_1", call_llm_1)
parallel_builder.add_node("call_llm_2", call_llm_2)
parallel_builder.add_node("call_llm_3", call_llm_3)
parallel_builder.add_node("aggregator", aggregator)

# Three edges from START = three parallel branches
parallel_builder.add_edge(START, "call_llm_1")
parallel_builder.add_edge(START, "call_llm_2")
parallel_builder.add_edge(START, "call_llm_3")
# All three converge on the aggregator
parallel_builder.add_edge("call_llm_1", "aggregator")
parallel_builder.add_edge("call_llm_2", "aggregator")
parallel_builder.add_edge("call_llm_3", "aggregator")
parallel_builder.add_edge("aggregator", END)

parallel_workflow = parallel_builder.compile()
state = parallel_workflow.invoke({"topic": "cats"})
```
- **What it demonstrates**: parallelism in LangGraph is purely topological — three edges out of `START` and three into `aggregator`. No special construct.

Google ADK — a search-augmented agent in eight lines:
```python
from google.adk.agents import LlmAgent
from google.adk.tools import google_search

dice_agent = LlmAgent(
    model="gemini-2.0-flash-exp",
    name="question_answer_agent",
    description="A helpful assistant agent that can answer questions.",
    instruction="""Respond to the query using google search""",
    tools=[google_search],
)
```

CrewAI — the crew as team charter:
```python
@crew
def crew(self) -> Crew:
    """Creates the research crew"""
    return Crew(
        agents=self.agents,
        tasks=self.tasks,
        process=Process.sequential,
        verbose=True,
    )
```

## Reference Tables

| Feature | LangChain | LangGraph |
|---|---|---|
| **Core Abstraction** | Chain (using LCEL) | Graph of Nodes |
| **Workflow Type** | Linear (Directed Acyclic Graph) | Cyclical (Graphs with loops) |
| **State Management** | Generally stateless per run | Explicit and persistent state object |
| **Primary Use** | Simple, predictable sequences | Complex, dynamic, stateful agents |

| Framework | Abstraction level | Organizing concept |
|---|---|---|
| LangChain | lowest | chains of components |
| LangGraph | low-mid | stateful graph of nodes and edges |
| Google ADK | high | teams of agents with pre-built patterns |
| CrewAI | high | roles, tasks, processes — a team charter |

| Other framework | Strength | Trade-off |
|---|---|---|
| **Microsoft AutoGen** | flexible, conversation-driven multi-agent interaction; complex problem decomposition | less predictable execution paths; needs sophisticated prompt engineering for convergence |
| **LlamaIndex** | data framework: sophisticated ingestion and retrieval pipelines, excellent for RAG | native agentic control flow and multi-agent orchestration less developed |
| **Haystack** | modular nodes for production search pipelines; performance and scalability at enterprise scale | optimized for search, more rigid for dynamic creative agentic behavior |
| **MetaGPT** | Standard Operating Procedure-driven roles mimicking a software company; highly structured coherent output | highly specialized, less adaptable outside its design |
| **SuperAGI** | full lifecycle management: provisioning, monitoring, GUI, loop-detection, observability | platform approach adds complexity and overhead |
| **Semantic Kernel** (Microsoft) | plugins and planners integrating LLMs with conventional code; excellent .NET/Python enterprise integration | conceptual overhead of plugin/planner architecture; steeper learning curve |
| **Strands Agents** (AWS) | lightweight, model-driven, model-agnostic; native MCP integration; customizable agent loop | you build more surrounding operational infrastructure yourself |

## Worked Example
**Choosing between LangChain and LangGraph for the same feature.**

*Requirement A — "summarize an uploaded document and extract the key dates as JSON."* The flow is: load → chunk → summarize → extract → parse. Every arrow points forward; nothing retries or branches. This is LCEL: `prompt | model | output_parser`, composed twice. Adding LangGraph here means defining a `State` TypedDict, four nodes, and five edges to express a pipe.

*Requirement B — "answer the question, but if the answer isn't grounded in retrieved context, search again with a refined query."* The retry arrow points backwards. LCEL cannot express it; you would write a Python `while` loop around the chain and hand-manage the accumulating state. That loop *is* LangGraph, badly: nodes for retrieve / generate / grade, a conditional edge from grade back to retrieve, and an explicit state object carrying the query, documents, and attempt count.

The same test scales up. *Requirement C — "a researcher, a writer, and an editor produce a report."* You could build it in LangGraph as three nodes with handoff edges — but you would be re-implementing role, goal, backstory, and task dependency by hand. That is CrewAI's job. And if the deliverable must run in production on GCP with session and artifact services, ADK's opinionated `SequentialAgent` over sub-agents removes the wiring entirely.

## Key Takeaways
1. The landscape spans low-level logic libraries to high-level orchestration platforms.
2. LangChain (LCEL) handles simple linear DAG workflows; LangGraph adds cycles and explicit state for reasoning loops.
3. Google ADK is opinionated and production-oriented, with pre-built patterns like `SequentialAgent` and `ParallelAgent` and implicit state/session management.
4. CrewAI models a human team — Agents with role/goal/backstory, Tasks, and a Crew executing a sequential or hierarchical Process.
5. The central trade-off is granular control versus streamlined development.
6. LlamaIndex is the right pick when the core challenge is data retrieval and synthesis, not orchestration.
7. Specialized options exist for conversation-driven multi-agent (AutoGen), production search (Haystack), SOP-driven software teams (MetaGPT), lifecycle management (SuperAGI), enterprise code integration (Semantic Kernel), and lightweight model-driven agents with MCP (Strands).
8. Choose by whether the application needs a simple sequence, a dynamic reasoning loop, or a managed team of specialists.

## Connects To
- **Ch 1 (Prompt Chaining)**: LCEL is the chaining primitive.
- **Ch 3 (Parallelization)**: `RunnableParallel`, graph fan-out, `ParallelAgent`.
- **Ch 7 (Multi-Agent Collaboration)**: CrewAI and ADK are that pattern's frameworks.
- **Ch 8 (Memory Management)**: state handling differs sharply across frameworks.
- **Ch 15 (Inter-Agent Communication / A2A)**: how agents from different frameworks interoperate.
- **Ch 10 (Model Context Protocol)**: Strands' native MCP integration; ADK's `MCPToolset`.
