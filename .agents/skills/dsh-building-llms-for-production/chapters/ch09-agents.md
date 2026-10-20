# Chapter IX — Agents

## Core Idea

An agent is an LLM used as a **reasoning engine rather than a content
generator**: it does not invent the answer, it decides which tool to call,
reads the result, and decides what to do next until it can respond. Three parts
make one — **tools** (functions that touch the world), a **core** (the model
doing the deciding), and **orchestration** (the loop that connects them). The
chapter walks the design space from a two-tool ReAct agent up to fully
autonomous loops (AutoGPT, BabyAGI), multi-agent simulations (CAMEL, Generative
Agents), and three production frameworks: LangChain, LlamaIndex, and OpenAI's
Assistants API.

## Frameworks Introduced

### Agent anatomy

| Component | Role |
|---|---|
| **Tools** | task-specific functions: Google Search API, SQL database, Python REPL, calculator |
| **Reasoning Engine / Core** | the large pre-trained model that plans and selects |
| **Agent orchestration** | the system managing LLM↔tool interaction |

### Two agent categories

| Type | Behavior | Suited to | Cost |
|---|---|---|---|
| **Action Agent** | decides and executes a single action at a time | simple, straightforward tasks | fewer calls |
| **Plan-and-Execute Agent** | builds a plan of actions up front, then executes them (sequentially or in parallel); intermediate results can revise the plan | sustaining long-term goals | more API calls, higher latency |

### The Action Agent loop

1. Process the user's input query.
2. Core selects a tool (if needed) and defines its input.
3. Tool executes; output is recorded.
4. Core receives tool input + output, produces an **observation**, decides the
   next action.
5. Repeat until no tool is needed and the agent can answer directly.

This is the **ReAct** (Reasoning and Acting, Yao et al. 2022) loop, running
until a stopping criterion is met.

### AutoGPT

Three defining characteristics:

- Internet-connected — real-time research and retrieval.
- **Self-prompting** — generates its own list of sub-tasks and works them one
  at a time.
- Executes tasks, including spinning up other AI agents.

External memory is a context-aware component: the agent assesses its situation,
formulates new tasks, corrects itself, and updates its task queue — a dynamic
recurrent flow rather than a fixed script. The stated failure modes: **getting
caught in loops** and **wrongly assuming a task is complete**. The community's
own correction is instructive — AutoGPT proved ineffective *because of its broad
scope*, and development shifted toward specialized task-specific agents.

The AutoGPT system prompt is the chapter's best artifact, because it shows what
an autonomous loop actually needs (see Worked Example).

### BabyAGI — four sub-agents in a loop

| Sub-agent | Job |
|---|---|
| **Execution Agent** | takes objective + task, builds a prompt, calls the LLM, produces a result |
| **Task Creation Agent** | generates new tasks from the objective and the last result; outputs a list of task dicts |
| **Prioritization Agent** | reorders the task list |
| **Context Agent** | merges the Execution Agent's result with cumulative prior results |

BabyAGI stores enriched outcomes in a database, which makes it a *learning*
system that adapts to new knowledge and priorities rather than just a task runner.

### Agent simulation projects

**CAMEL** (Communicative Agents for "Mind" Exploration) — a **role-playing**
framework where autonomy replaces human prompting. The stock-trading-bot flow:

1. A human states an intent (build a trading bot).
2. Two AI agents with distinct roles: an AI *assistant* with Python expertise,
   an AI *user* knowledgeable in stock trading.
3. A **task specifier agent** turns the vague concept into a concrete task.
4. The two agents chat, follow directions, and cooperate to completion.

Guided by **inception prompting** to keep interactions aligned with the original
human intent. Stated challenges: hallucinations, conversation deviation, **role
flipping**, and setting appropriate termination conditions.

**Generative Agents** ('Interactive Simulacra of Human Behavior') — 25 agents in
a Sims-like simulated world. Its contribution is the **long-term memory system**:

| Component | Function |
|---|---|
| **Memory Stream** | chronological database of the agent's experiences |
| **Observations** | raw memories from interactions and dialogue |
| **Importance reflection step** | assigns each memory an importance score, used to prioritize retrieval |
| **Reflection step** | the agent generalizes from experience; reflections stored alongside plain memories |
| **Retriever** | combines **recency + relevancy + importance** — implemented in LangChain as `TimeWeightedVectorStoreRetriever`, which also tracks last-accessed time |

The three-factor retriever is the transferable idea: cosine similarity alone is
not how memory should be recalled.

### Plan and Execute in LangChain

Inspired by BabyAGI and the Plan-and-Solve paper. Two components:

- **Planner** — uses the model's reasoning to devise the plan's steps, handling
  ambiguity and unusual scenarios.
- **Executor** — interprets each step and identifies the tools/actions to run it.

Separating them improves reliability and adaptability.

### OpenAI Assistants API — three built-in capabilities

| Capability | What it does |
|---|---|
| **Code Interpreter** | generates and runs Python in a sandbox; solves math, produces files and charts; activated by conversation or file upload |
| **Knowledge Retrieval** | OpenAI-hosted RAG — uploaded documents are segmented, indexed, embedded, and vector-searched |
| **Function Calling** | user-supplied functions; the model returns the function name and arguments |

Four objects: **Assistant** (the configured agent), **Thread** (one
conversation, no size limit — the API truncates to fit the context window),
**Message** (user input or assistant reply appended to a Thread), **Run**
(activation; the Assistant decides which prior Messages to include). Inspect
**Run Steps** to see how it reached the result.

Configuration parameters: `instructions` (personality/role/goals — like a system
message), `tools` (**up to 128 in parallel**), `file_ids` (files for the tools).

## Key Concepts

- **Reasoning engine vs. content generator.** Same model, two modes. As a
  reasoning engine it gathers, processes, and synthesizes external information —
  tools required, `temperature=0`. As a content generator it draws on training
  patterns — no tools needed, raise the temperature for creativity.
- **The tool description is the routing signal.** `ToolMetadata`/`Tool`
  descriptions are what the agent reads when choosing. In LlamaIndex,
  `FunctionTool.from_defaults()` builds that description from the function's
  **type annotations and docstring** — so annotations are functional, not
  cosmetic.
- **`tool_choice`** — pass a tool name to force it, `"auto"` to let the agent
  decide.
- **Debug an agent by interviewing it.** Ask the agent what tools it has, what
  arguments they take, what those arguments mean, and what each tool is for.
  Its answers expose prompt flaws and explain why a tool goes unused.
- **`max_iterations`** — "if `max_iterations` is None, the agent may go on
  forever if stuck in loops." Always set it.
- **Deep Memory** — Activeloop's embedding-space adaptation: pull chunks from
  the dataset, have GPT-3.5 generate a question per chunk, train on those
  (question, chunk-id) pairs to reshape the embedding space toward your data.
  Stated gain: ~25% improvement in expected chunks returned, minimum ~100
  chunks recommended. The book's own caveat: this is *one* metric (correct chunk
  returned) and does not necessarily mean better end performance.
- **Data-source tagging from day one.** Tag and track the origin of every data
  point (which book, doc site, blog). The Towards AI RAG tutor uses five sources;
  tagging is what lets **routers** send a query to the right one.
- **Start small.** Begin the RAG/agent pipeline with a manageable, rich dataset
  such as web articles — it makes testing, debugging, and evaluation tractable.

## Mental Models

- **An agent is a while-loop with a language model as the branch predictor.**
  Everything else — memory, tools, planners — exists to make the next branch
  better informed.
- **Autonomy trades control for reach.** AutoGPT's advantage (no human in the
  loop) is exactly its risk surface: loops, false completion, misuse,
  unexpected outcomes.
- **Specialization beat generality.** The AutoGPT community's pivot from
  general-purpose to task-specific agents is the chapter's most practical
  lesson about scoping agent projects.
- **Separate what to do from how to do it.** Planner/Executor, and
  BabyAGI's four sub-agents, are both the same move: decompose the agent so
  each part has one job and can be improved independently.
- **Memory retrieval should mimic human recall.** Recency, relevancy, and
  importance together — not similarity alone.

## Anti-patterns

- **Letting an agent answer from parametric knowledge when a tool exists.** The
  World Cup example works because the agent searched instead of recalling.
- **Vague tool descriptions.** The agent cannot choose a tool it does not
  understand; a bad description is a silent routing bug.
- **Unbounded autonomous loops.** No `max_iterations`, no termination
  condition — AutoGPT and CAMEL both list this as a core difficulty.
- **A single query engine over heterogeneous sources.** The Tesla example splits
  predictions and biography into two engines with two tools precisely to avoid
  "uniformly sourcing information from both data sources regardless of relevance."
- **Trusting a single retrieval metric.** The book flags this about Deep
  Memory's 25% figure in the same breath as quoting it.
- **Overriding Assistant instructions accidentally.** Additional instructions
  passed at Run creation **override** the Assistant's default instructions.
- **Using Hugging Face's free Inference API in production.** Rate-limited and
  explicitly not for production; Inference Endpoints is the paid path.

## Code Examples

### Minimal ReAct agent with two tools

```python
from langchain.agents import load_tools, initialize_agent, AgentType
from langchain.chat_models import ChatOpenAI

llm = ChatOpenAI(model="gpt-3.5-turbo", temperature=0)
tools = load_tools(["google-search", "llm-math"], llm=llm)

agent = initialize_agent(tools, llm,
                         agent=AgentType.ZERO_SHOT_REACT_DESCRIPTION,
                         verbose=True)

agent.run("What's the result of 1000 plus the number of goals scored in the "
          "soccer world cup in 2018?")
```

Trace: `Action: Google Search` → `Observation: ...169 goals...` →
`Action: Calculator` → `Action Input: 1000 + 169` → `Final Answer: 1169`.
Two tools, two hops, correct answer — the agent decomposed a compound question
without being told to.

### AutoGPT with search + file tools and FAISS memory

```python
from langchain.utilities import GoogleSearchAPIWrapper
from langchain.agents import Tool
from langchain.tools.file_management.write import WriteFileTool
from langchain.tools.file_management.read import ReadFileTool

search = GoogleSearchAPIWrapper()
tools = [
    Tool(name="search", func=search.run, return_direct=True,
         description="Useful for when you need to answer questions about "
                     "current events. You should ask targeted questions"),
    WriteFileTool(),
    ReadFileTool(),
]

import faiss
from langchain.vectorstores import FAISS
from langchain.docstore import InMemoryDocstore
from langchain.embeddings import OpenAIEmbeddings

embeddings_model = OpenAIEmbeddings(model="text-embedding-ada-002")
index = faiss.IndexFlatL2(1536)
vectorstore = FAISS(embeddings_model.embed_query, index, InMemoryDocstore({}), {})

from langchain.experimental import AutoGPT
agent = AutoGPT.from_llm_and_tools(
    ai_name="Jim", ai_role="Assistant", tools=tools,
    llm=ChatOpenAI(model="gpt-3.5-turbo", temperature=0),
    memory=vectorstore.as_retriever(),
)
agent.chain.verbose = True
agent.run(["Provide an analysis of the major historical events that led to the French Revolution"])
```

Note the pairing: `WriteFileTool` + `ReadFileTool` exist because the prompt tells
the agent its short-term memory is ~4000 words and it must **save important
information to files**. Files are the agent's long-term memory.

> As of August 2023 LangChain moved these classes out of `langchain.experimental`
> into `langchain_experimental` (`pip install langchain-experimental`). Code
> pinned to `langchain==0.0.208` uses the old path.

### BabyAGI

```python
from langchain.experimental import BabyAGI

baby_agi = BabyAGI.from_llm(
    llm=OpenAI(model="text-davinci-003", temperature=0),
    vectorstore=vectorstore,
    verbose=False,
    max_iterations=3,        # None = may loop forever
)
baby_agi({"objective": "Plan a trip to the Grand Canyon"})
```

Its trace shows the loop and its pathology: task 1 "Make a todo list" produced
10 items; the Task Creation Agent then expanded that to 19, including
near-duplicates ("best places to take a break from the heat / the crowds / the
sun / the wind"). Task lists grow faster than they are consumed — which is what
`max_iterations` is for.

### Plan and Execute over a private corpus

```python
from langchain.agents.tools import Tool
from langchain.experimental.plan_and_execute import (
    PlanAndExecute, load_agent_executor, load_chat_planner)

retriever = db.as_retriever()
retriever.search_kwargs['k'] = 3

def retrieve_n_docs_tool(query: str) -> str:
    """Searches for relevant documents that may contain the answer to the query."""
    docs = retriever.get_relevant_documents(query)
    return "\n---------------\n".join(d.page_content for d in docs)

tools = [Tool(name="Search Private Docs", func=retrieve_n_docs_tool,
              description="useful for when you need to answer questions about "
                          "current events about Artificial Intelligence")]

model = ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0)
agent = PlanAndExecute(planner=load_chat_planner(model),
                       executor=load_agent_executor(model, tools, verbose=True),
                       verbose=True)
agent.run("Write an overview of Artificial Intelligence Regulations by governments by country")
```

The planner emitted five steps (research → identify key countries → summarize
per country → organize by country → produce the overview), and the executor ran
each as its own retrieval-and-answer cycle.

### LlamaIndex: two sources, two tools, one agent

```python
from llama_index.tools import QueryEngineTool, ToolMetadata
from llama_index.agent import OpenAIAgent

tesla_engine = tesla_index.as_query_engine(similarity_top_k=3)
webtext_engine = webtext_index.as_query_engine(similarity_top_k=3)

query_engine_tools = [
    QueryEngineTool(query_engine=tesla_engine, metadata=ToolMetadata(
        name="tesla_lk",
        description="Provides information about Tesla's statements that refer to "
                    "future times and predictions. "
                    "Use a detailed plain text question as input to the tool.")),
    QueryEngineTool(query_engine=webtext_engine, metadata=ToolMetadata(
        name="webtext_lk",
        description="Provides information about Tesla's life and biographical data. "
                    "Use a detailed plain text question as input to the tool.")),
]

agent = OpenAIAgent.from_tools(query_engine_tools, verbose=True)
agent.chat_repl()
```

### Custom function tools

```python
def multiply(a: int, b: int) -> int:
    """Multiply two integers and returns the result integer"""
    return a * b

from llama_index.tools import FunctionTool
from llama_index.objects import ObjectIndex, SimpleToolNodeMapping
from llama_index.agent import FnRetrieverOpenAIAgent

all_tools = [FunctionTool.from_defaults(fn=multiply, name="multiply"),
             FunctionTool.from_defaults(fn=add, name="add")]

tool_mapping = SimpleToolNodeMapping.from_objects(all_tools)
obj_index = ObjectIndex.from_objects(all_tools, tool_mapping, VectorStoreIndex)

agent = FnRetrieverOpenAIAgent.from_retriever(obj_index.as_retriever(), verbose=True)
agent.chat("What's 12 multiplied by 22? Make sure to use Tools")   # -> 264
agent.chat("What is 5 + 2?", tool_choice="add")                    # -> 7
```

`ObjectIndex` makes the *tools themselves* retrievable — the agent semantic-searches
its own toolbox, which is how you scale past a handful of tools.

### OpenAI Assistant with knowledge retrieval

```python
from openai import OpenAI
client = OpenAI()

file = client.beta.files.upload(file=open("tech_manual.pdf", "rb"),
                                filetype="application/pdf",
                                description="Tech product manual")

assistant = client.beta.assistants.create(
    instructions="You are a tech support chatbot. Use the product manual to "
                 "respond accurately to customer inquiries.",
    model="gpt-4-turbo",
    tools=[{"type": "retrieval"}],
    file_ids=[file.id],
)

thread = client.beta.threads.create()
client.beta.threads.messages.create(thread_id=thread.id, role="user",
                                    content="How do I reset my Model X device?")
run = client.beta.threads.runs.create(thread_id=thread.id, assistant_id=assistant.id)

messages = client.beta.threads.messages.list(thread_id=thread.id)
assistant_response = messages.data[0].content[0].text.value
```

The Run enters a **queued** state before executing — this API is asynchronous by
design.

### Deep Memory training

```python
job_id = db.vectorstore.deep_memory.train(
    queries=questions,          # LLM-generated, one per chunk
    relevance=relevances,       # [[(chunk_id, 1)], ...]
    embedding_function=OpenAIEmbeddings().embed_documents,
)
print(db.vectorstore.dataset.embedding.info)
# {'base_recall@10': 0.25, 'recall@10': 0.5, 'delta': 0.25, ...}
```

## Reference Tables

### Framework selection

| Framework | Strength |
|---|---|
| **LangChain** | broadest agent-type catalogue, AutoGPT/BabyAGI/Plan-and-Execute implementations, simulation projects |
| **LlamaIndex** | query-engine tools, `ObjectIndex` over tools, LlamaHub (30+ shareable agent tools in one line) |
| **OpenAI Assistants API** | hosted — no orchestration to build; Code Interpreter, Retrieval, Function Calling, up to 128 tools |

### Hugging Face APIs for agent tools

| Task | Example model | Call |
|---|---|---|
| Summarization | `facebook/bart-large-cnn` | Inference API POST |
| Sentiment analysis | `distilbert-base-uncased-finetuned-sst-2-english` | Inference API POST |
| Text-to-image | `runwayml/stable-diffusion-v1-5` | `StableDiffusionPipeline` |
| Embeddings | any sentence-transformer | `SentenceTransformer.encode()` |
| Image captioning | `nlpconnect/vit-gpt2-image-captioning` | `pipeline("image-to-text")` |
| Image classification | `google/vit-base-patch16-224` | `ViTForImageClassification` |

Endpoint form: `https://api-inference.huggingface.co/models/<MODEL_ID>`,
authenticated with `HUGGINGFACEHUB_API_TOKEN`. Not every model is served — check
the Model card.

## Worked Example — Reading the AutoGPT system prompt

The most valuable thing in this chapter is not a code snippet but the prompt
AutoGPT constructs, because it enumerates everything an autonomous loop needs.
Its eight parts:

1. **Role prompting** — "You are Jim, Assistant. Your decisions must always be
   made independently without seeking user assistance."
2. **Goals** — the user's objective, verbatim.
3. **Constraints** — "~4000 word limit for short term memory… immediately save
   important information to files"; recall past events by thinking about similar
   ones; no user assistance; use only the listed commands.
4. **Commands** — `search`, `write_file`, `read_file`, `finish`, each with a
   JSON args schema. `finish` is the termination mechanism.
5. **Resources** — internet access, long-term memory management, GPT-3.5-powered
   sub-agents for delegation, file output.
6. **Performance Evaluation** — continuously review your actions; self-criticize
   your big-picture behavior; reflect on past decisions; *"Every command has a
   cost, so be smart and efficient."*
7. **Response format** — enforced JSON:

```json
{
  "thoughts": {
    "text": "thought",
    "reasoning": "reasoning",
    "plan": "- short bulleted\n- list that conveys\n- long-term plan",
    "criticism": "constructive self-criticism",
    "speak": "thoughts summary to say to user"
  },
  "command": { "name": "command name", "args": { "arg name": "value" } }
}
```

8. **Situational context** — current time and date, plus retrieved similar past
   events (empty on the first turn).

The response format is doing the real work. It **forces** the model to write its
reasoning, produce an explicit plan, criticize that plan, and only then emit a
command. The French Revolution run took seven iterations — research, summarize,
analyze, summarize again, analyze in more detail, reflect, compile — and left
four text files behind. Every one of those steps was a `plan` field the model
wrote for itself in the previous turn.

Copy the structure, not the framework: role, goals, hard constraints, an
enumerated command set with a `finish`, a cost reminder, and a response schema
that makes thinking and self-criticism mandatory before action.

## Key Takeaways

- An agent = tools + reasoning core + orchestration, looping until it can answer
  (ReAct).
- Action Agents for simple tasks; Plan-and-Execute for long-horizon goals, at
  the cost of more calls and latency.
- Tool descriptions and type annotations are the agent's routing table — write
  them as carefully as the prompt.
- Autonomous agents (AutoGPT, BabyAGI) work through self-prompting plus external
  memory; both fail by looping and by falsely declaring completion. Bound the
  iterations and give the agent an explicit `finish`.
- Narrow scope beats general capability — the AutoGPT community's own pivot.
- Multi-agent designs (CAMEL role-play, Generative Agents) contribute two ideas
  worth stealing: explicit role assignment with a task specifier, and memory
  retrieval weighted by recency + relevance + importance.
- One query engine per data source, each behind its own described tool, beats
  one engine over everything.
- OpenAI's Assistants API trades control for speed: Code Interpreter, hosted
  retrieval, and function calling, with Threads and Runs as the concurrency model.

## Connects To

- [Chapter VIII](ch08-advanced-rag.md) — query engines, reranking, and the
  evaluation machinery that agent tools are built on.
- [Chapter VII](ch07-retrieval-augmented-generation.md) — retrievers and vector
  stores, which become the agent's knowledge tools.
- [Chapter VI](ch06-prompting-with-langchain.md) — output parsers, which are how
  a tool call is made machine-readable.
- [Chapter V](ch05-langchain-and-llamaindex.md) — the six LangChain agent types
  and the framework comparison introduced there.
