# Chapter 2: Routing

## Core Idea
Routing introduces conditional logic into an agent's operational framework: instead of a fixed execution path, the agent evaluates the input or state and dynamically dispatches to the most appropriate tool, sub-agent, or chain.

## Frameworks Introduced
- **Routing (dispatcher) pattern**: analyze the query → classify intent → direct control flow to a specialized handler.
  - When to use: an agent must choose between multiple distinct workflows, tools, or sub-agents based on user input or current state — triage and classification problems above all (support bot separating sales / technical support / account management).
  - How:
    1. Enumerate the possible destinations (handlers, tools, sub-agents, chains) plus a fallback for "unclear".
    2. Pick a routing mechanism (see the four below).
    3. Emit a single normalized route identifier — one word, constrained vocabulary.
    4. Branch on it (`RunnableBranch`, graph edge, or framework auto-delegation) and pass the *original* input to the chosen handler.
    5. Always define a default branch.
- **The four routing mechanisms**:
  - **LLM-based Routing** — prompt the model to output only a category identifier ("Analyze the following user query and output only the category: 'Order Status', 'Product Info', 'Technical Support', or 'Other'"). Most flexible for nuance and novel input.
  - **Embedding-based Routing** — embed the query, compare against embeddings representing each route, pick the most similar. Semantic routing: decides on *meaning*, not keywords.
  - **Rule-based Routing** — if/else, switch, keyword and pattern matching on structured extracted fields. Faster and deterministic, but brittle on nuanced or novel input.
  - **ML Model-Based Routing** — a discriminative classifier fine-tuned on a small labeled corpus. Routing logic lives in learned weights, not a runtime prompt; no generative model in the decision path. LLMs may generate synthetic training data but are absent at inference.

## Key Concepts
- **Logical arbitration**: the capacity to choose among actions — what turns a static executor into a dynamic system.
- **Router placement**: routing can sit at the outset (classify the primary task), mid-chain (choose the next action), or inside a subroutine (select a tool).
- **RunnableBranch** (LangChain): takes the router chain's decision and dispatches the original input to a matching branch, with a final default branch.
- **Auto-Flow** (Google ADK): defining `sub_agents` on a parent agent enables LLM-driven delegation automatically — no explicit graph.
- **FunctionTool** (ADK): wraps a plain Python function (its docstring is the tool contract) so the framework can select it.
- **Coordinator agent**: a parent whose *only* job is to analyze and delegate — explicitly instructed not to answer directly.

## Mental Models
- Think of routing as **`switch` for agents**: the router is the discriminant, handlers are the cases, "unclear" is `default`.
- Choose the mechanism by **cost of a wrong route vs. variability of input**: rules for narrow deterministic input; embeddings for paraphrase-heavy semantic input; LLM for open-ended; fine-tuned classifier when the routing task is high-volume, stable, and latency-sensitive.
- **LangGraph vs ADK**: LangGraph makes states and transitions explicit in a graph — best for complex, multi-step, state-dependent routing. ADK defines discrete capabilities as tools and lets the framework match intent — simpler when the agent has a well-defined set of discrete actions.

## Anti-patterns
- **No fallback branch**: an unrecognized route identifier crashes the dispatch or silently picks the wrong handler.
- **Letting the router answer the question**: a coordinator that tries to respond directly instead of delegating collapses the pattern. State "Do not try to answer the user directly" in the instruction.
- **Unbounded router output**: without "ONLY output one word", the model returns prose and the branch predicate fails. Also `.strip()` the decision before comparing.
- **Rule-based routing for nuanced natural language**: fast and deterministic but fails on novel phrasing.
- **Routing on keywords when the decision is semantic**: use embeddings instead.

## Code Examples
LangChain — LLM router + `RunnableBranch`:
```python
from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain_core.runnables import RunnablePassthrough, RunnableBranch

llm = ChatGoogleGenerativeAI(model="gemini-2.5-flash", temperature=0)

def booking_handler(request: str) -> str:
    return f"Booking Handler processed request: '{request}'."

def info_handler(request: str) -> str:
    return f"Info Handler processed request: '{request}'."

def unclear_handler(request: str) -> str:
    return f"Coordinator could not delegate request: '{request}'. Please clarify."

coordinator_router_prompt = ChatPromptTemplate.from_messages([
    ("system",
     "Analyze the user's request and determine which specialist handler should process it.\n"
     "- If the request is related to booking flights or hotels, output 'booker'.\n"
     "- For all other general information questions, output 'info'.\n"
     "- If the request is unclear or doesn't fit either category, output 'unclear'.\n"
     "ONLY output one word: 'booker', 'info', or 'unclear'."),
    ("user", "{request}")
])

coordinator_router_chain = coordinator_router_prompt | llm | StrOutputParser()

branches = {
    "booker":  RunnablePassthrough.assign(output=lambda x: booking_handler(x['request']['request'])),
    "info":    RunnablePassthrough.assign(output=lambda x: info_handler(x['request']['request'])),
    "unclear": RunnablePassthrough.assign(output=lambda x: unclear_handler(x['request']['request'])),
}

delegation_branch = RunnableBranch(
    (lambda x: x['decision'].strip() == 'booker', branches["booker"]),
    (lambda x: x['decision'].strip() == 'info',   branches["info"]),
    branches["unclear"],   # default branch
)

coordinator_agent = (
    {"decision": coordinator_router_chain, "request": RunnablePassthrough()}
    | delegation_branch
    | (lambda x: x['output'])
)

coordinator_agent.invoke({"request": "Book me a flight to London."})
```
- **What it demonstrates**: the router's decision travels *alongside* the original request through the branch, so the handler receives the untouched input. Note the `.strip()` — model output carries whitespace.

Google ADK — delegation via `sub_agents` (Auto-Flow):
```python
from google.adk.agents import Agent
from google.adk.tools import FunctionTool
from google.adk.runners import InMemoryRunner

booking_tool = FunctionTool(booking_handler)
info_tool = FunctionTool(info_handler)

booking_agent = Agent(name="Booker", model="gemini-2.0-flash",
    description="A specialized agent that handles all flight and hotel booking "
                "requests by calling the booking tool.",
    tools=[booking_tool])

info_agent = Agent(name="Info", model="gemini-2.0-flash",
    description="A specialized agent that provides general information and "
                "answers user questions by calling the info tool.",
    tools=[info_tool])

coordinator = Agent(name="Coordinator", model="gemini-2.0-flash",
    instruction=("You are the main coordinator. Your only task is to analyze incoming "
                 "user requests and delegate them to the appropriate specialist agent. "
                 "Do not try to answer the user directly.\n"
                 "For any requests related to booking flights or hotels, delegate to the 'Booker' agent.\n"
                 "For all other general information questions, delegate to the 'Info' agent."),
    description="A coordinator that routes user requests to the correct specialist agent.",
    # The presence of sub_agents enables LLM-driven delegation (Auto-Flow) by default.
    sub_agents=[booking_agent, info_agent])

runner = InMemoryRunner(coordinator)
```
- **What it demonstrates**: no explicit branch code. The `description` field of each sub-agent is what the coordinator matches intent against — descriptions are load-bearing, not documentation.

## Reference Tables

| Mechanism | Decision made by | Strengths | Weaknesses |
|---|---|---|---|
| LLM-based | generative model at inference | flexible, handles novel/nuanced input, zero training data | latency, cost, non-determinism |
| Embedding-based | vector similarity to route embeddings | semantic (meaning, not keywords), cheap, fast | needs good route exemplars; opaque near boundaries |
| Rule-based | if/else, switch, regex | fastest, fully deterministic, auditable | brittle on nuance and novel inputs |
| ML classifier | fine-tuned discriminative model | fast, specialized, no prompt at runtime | needs labeled corpus + retraining |

| Framework | Routing style |
|---|---|
| LangGraph | state-based graph; explicit nodes and conditional transitions — best for complex state-dependent routing |
| LangChain | `RunnableBranch` over a router chain's output |
| Google ADK | discrete tools + `sub_agents`; framework matches intent to handler (Auto-Flow) |

## Worked Example
**Customer inquiry agent.** The naive design answers everything with one prompt. The routed design:

1. Analyze the user's query.
2. Route on intent:
   - `check order status` → sub-agent/chain that queries the order database.
   - `product information` → chain that searches the product catalog.
   - `technical support` → chain that reads troubleshooting guides, or escalates to a human.
   - unclear → clarification sub-agent.

The same skeleton reappears across domains: document pipelines classify incoming emails/tickets/API payloads by content, metadata, or format and dispatch to sales-lead ingestion, a JSON vs CSV transformer, or an urgent-escalation path. A coding assistant routes on detected language *and* intent (debug / explain / translate) before handing the snippet to the right specialized tool.

## Key Takeaways
1. Routing lets agents make dynamic decisions about the next step based on conditions, moving beyond linear execution.
2. Four mechanisms — LLM, embedding, rule-based, fine-tuned classifier — trade flexibility against determinism, latency, and cost.
3. Constrain the router's output to a small identifier vocabulary and always define a default/unclear branch.
4. Route the *original* input to the handler; the decision is metadata, not a replacement for the payload.
5. In ADK, sub-agent `description` strings are the routing signal — write them as selection criteria.
6. A coordinator must be instructed not to answer directly, or it will short-circuit the delegation.
7. LangGraph suits complex state-contingent routing; ADK suits a well-defined set of discrete actions.

## Connects To
- **Ch 1 (Prompt Chaining)**: routing adds the conditional branching a linear chain lacks.
- **Ch 5 (Tool Use)**: tool selection is routing at subroutine granularity.
- **Ch 7 (Multi-Agent Collaboration)**: the Supervisor topology is routing at the agent level.
- **Ch 14 (Knowledge Retrieval / RAG)**: embedding-based routing reuses the same embedding machinery.
- **Ch 16 (Resource-Aware Optimization)**: routing by cost/latency budget rather than intent.
- **Ch 13 (Human-in-the-Loop)**: escalation is a route.
