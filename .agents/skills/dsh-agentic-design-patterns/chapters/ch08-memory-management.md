# Chapter 8: Memory Management

## Core Idea
Agents need two distinct memories: an ephemeral short-term context for the current conversation, and a persistent, searchable long-term store queried by semantic similarity and pulled into context on demand.

## Frameworks Introduced
- **The dual-component memory model**:
  - **Short-Term Memory (Contextual Memory)** — working memory. For LLM agents this *is* the context window: recent messages, agent replies, tool results, reflections from the current interaction. Limited capacity; managed by summarizing older segments or emphasizing key details. Long-context models simply expand this window — the context is still ephemeral, lost when the session concludes, and costly to reprocess every turn.
  - **Long-Term Memory (Persistent Memory)** — a repository outside the processing environment: databases, knowledge graphs, vector databases. In vector stores, information is embedded and retrieved by **semantic similarity** rather than exact keyword match (semantic search). The agent queries it, retrieves relevant data, and integrates it into short-term context.
  - When to use: any agent that must maintain context across a conversation, track progress in multi-step tasks, personalize by recalling preferences and history, or learn from past successes and failures.
- **ADK's three concepts**:
  - **Session** — one chat thread: `id`, `app_name`, `user_id`, a chronological list of `Event`s, a `state` dict, and `last_update_time`.
  - **State (`session.state`)** — the session's scratchpad; data relevant only to the active thread.
  - **Memory** — a searchable repository sourced from past chats or external sources, beyond the immediate conversation.
  - Managed by **`SessionService`** (thread lifecycle) and **`MemoryService`** (long-term knowledge).
- **The three types of long-term memory** (LangGraph framing, analogous to human memory):
  - **Semantic Memory — remembering facts**: user preferences, domain knowledge. Grounds responses; managed as a continuously updated user "profile" (a JSON document) or a "collection" of individual factual documents.
  - **Episodic Memory — remembering experiences**: past events or actions; for agents, usually *how to accomplish a task*. Frequently implemented through few-shot example prompting from past successful interaction sequences.
  - **Procedural Memory — remembering rules**: the agent's core instructions and behaviors, typically in the system prompt. Agents commonly modify their own prompts to adapt — via **Reflection**: prompt the agent with its current instructions plus recent interactions and ask it to refine those instructions.

## Key Concepts
- **State key prefixes** (ADK) — scope and persistence encoded in the key:
  - no prefix → session-specific
  - `user:` → tied to a user ID across all their sessions
  - `app:` → shared among all users of the application
  - `temp:` → valid only for the current processing turn; not persisted
- **`output_key`** — the simple state update: the Runner sees it and automatically emits a `state_delta` when appending the event.
- **`EventActions.state_delta`** — the standard update for anything more complex: multiple keys, non-text values, scoped keys, or updates not tied to a final text reply.
- **`ToolContext.state`** — a tool's handle on session state; encapsulating state changes inside tools co-locates the logic.
- **`add_session_to_memory` / `search_memory`** — the `BaseMemoryService` interface.
- **`checkpointer`** (LangGraph) — persists short-term memory as agent state so a thread can be resumed at any time.
- **Namespace + key** (LangGraph store) — memories are JSON documents filed under a namespace (folder) and key (filename).
- **`memory_key` / `return_messages`** (LangChain `ConversationBufferMemory`) — the prompt variable holding history, and whether it renders as a formatted string (standard LLMs) or a list of message objects (chat models — recommended).
- **Vertex Memory Bank** — a managed service that uses Gemini models to *asynchronously* analyze conversation histories, extract key facts and preferences, store them scoped by user ID, and intelligently consolidate new data and **resolve contradictions**.

## Mental Models
- **Short-term = RAM, long-term = disk.** Retrieval is the load; `add_session_to_memory` is the save.
- **Long context is a bigger short-term memory, not a substitute for long-term memory** — it's still ephemeral and still costs tokens every turn.
- Store state through the **event pipeline**, never by mutating the dict — the same reason you don't write to a database file behind the ORM's back.
- Procedural memory means **the agent's prompt is data**, not code — and Reflection is the update mechanism.

## Anti-patterns
- **Directly mutating `session.state` after retrieving a session** — strongly discouraged: it bypasses event processing, so changes are not recorded in event history, may not be persisted by the `SessionService`, can cause concurrency issues, and won't update metadata like timestamps. `session.state` should primarily be used for *reading*.
- **Deeply nested state**: keep it simple, use basic serializable types, clear key names, correct prefixes.
- **Missing prefixes**: a user preference stored without `user:` disappears when the session ends.
- **Relying on an in-memory service in production**: `InMemorySessionService` / `InMemoryMemoryService` lose everything on restart — testing only.
- **`return_messages=False` with a chat model**: history collapses to a string and loses role structure.
- **Replaying the entire history every turn**: costly and inefficient; summarize older segments.

## Code Examples
ADK — session services by persistence tier:
```python
# Testing: no persistence across restarts
from google.adk.sessions import InMemorySessionService
session_service = InMemorySessionService()

# Production / persistent: your own database
from google.adk.sessions import DatabaseSessionService
session_service = DatabaseSessionService(db_url="sqlite:///./my_agent_data.db")

# Scalable production on GCP
from google.adk.sessions import VertexAiSessionService
session_service = VertexAiSessionService(project=PROJECT_ID, location=LOCATION)
# app_name for this service is the Reasoning Engine resource name
```

ADK — the simple state update via `output_key`:
```python
from google.adk.agents import LlmAgent
from google.adk.sessions import InMemorySessionService
from google.adk.runners import Runner
from google.genai.types import Content, Part

greeting_agent = LlmAgent(
    name="Greeter", model="gemini-2.0-flash",
    instruction="Generate a short, friendly greeting.",
    output_key="last_greeting")          # final text reply -> state["last_greeting"]

session_service = InMemorySessionService()
runner = Runner(agent=greeting_agent, app_name="state_app", session_service=session_service)
session = session_service.create_session(app_name="state_app", user_id="user1",
                                         session_id="session1")

for event in runner.run(user_id="user1", session_id="session1",
                        new_message=Content(parts=[Part(text="Hello")])):
    if event.is_final_response():
        pass

# Re-fetch after the runner finishes processing all events
updated = session_service.get_session("state_app", "user1", "session1")
print(updated.state)
```

ADK — encapsulate state changes in a tool (the recommended organization):
```python
import time
from google.adk.tools.tool_context import ToolContext

def log_user_login(tool_context: ToolContext) -> dict:
    """
    Updates the session state upon a user login event.
    This tool encapsulates all state changes related to a user login.
    """
    state = tool_context.state
    login_count = state.get("user:login_count", 0) + 1
    state["user:login_count"]      = login_count      # persists across the user's sessions
    state["task_status"]           = "active"         # session-scoped
    state["user:last_login_ts"]    = time.time()
    state["temp:validation_needed"] = True            # this turn only, not stored
    return {"status": "success", "message": f"User login tracked. Total logins: {login_count}"}
```

ADK — long-term memory service:
```python
# Testing
from google.adk.memory import InMemoryMemoryService
memory_service = InMemoryMemoryService()

# Production: semantic, persistent, scalable via Vertex AI RAG
from google.adk.memory import VertexAiRagMemoryService
memory_service = VertexAiRagMemoryService(
    rag_corpus="projects/<proj>/locations/us-central1/ragCorpora/<corpus-id>",
    similarity_top_k=5,
    vector_distance_threshold=0.7)

# Managed extraction/consolidation of facts and preferences
from google.adk.memory import VertexAiMemoryBankService
memory_service = VertexAiMemoryBankService(project="PROJECT_ID", location="LOCATION",
                                           agent_engine_id=agent_engine_id)
session = await session_service.get_session(app_name=app_name, user_id="USER_ID",
                                            session_id=session.id)
await memory_service.add_session_to_memory(session)
```

LangChain — buffer memory wired into a chat chain:
```python
from langchain_openai import ChatOpenAI
from langchain.chains import LLMChain
from langchain.memory import ConversationBufferMemory
from langchain_core.prompts import (ChatPromptTemplate, MessagesPlaceholder,
                                    SystemMessagePromptTemplate, HumanMessagePromptTemplate)

prompt = ChatPromptTemplate(messages=[
    SystemMessagePromptTemplate.from_template("You are a friendly assistant."),
    MessagesPlaceholder(variable_name="chat_history"),
    HumanMessagePromptTemplate.from_template("{question}"),
])

# return_messages=True is essential for chat models
memory = ConversationBufferMemory(memory_key="chat_history", return_messages=True)
conversation = LLMChain(llm=ChatOpenAI(), prompt=prompt, memory=memory)

conversation.predict(question="Hi, I'm Jane.")
conversation.predict(question="Do you remember my name?")
```

LangGraph — a namespaced long-term store:
```python
from langgraph.store.memory import InMemoryStore

store = InMemoryStore(index={"embed": embed, "dims": 2})   # production: database-backed
namespace = ("my-user", "chitchat")                        # (user_id, application_context)

store.put(namespace, "a-memory", {
    "rules": ["User likes short, direct language",
              "User only speaks English & python"],
    "my-key": "my-value",
})

item = store.get(namespace, "a-memory")
items = store.search(namespace, filter={"my-key": "my-value"}, query="language preferences")
```

LangGraph — procedural memory updated by reflection:
```python
def update_instructions(state: State, store: BaseStore):
    """Reflect on the conversation and rewrite the agent's own instructions."""
    current = store.search(("instructions",))[0]
    prompt = prompt_template.format(instructions=current.value["instructions"],
                                    conversation=state["messages"])
    new_instructions = llm.invoke(prompt)["new_instructions"]
    store.put(("agent_instructions",), "agent_a", {"instructions": new_instructions})

def call_model(state: State, store: BaseStore):
    instructions = store.get(("agent_instructions",), key="agent_a")[0]
    prompt = prompt_template.format(instructions=instructions.value["instructions"])
    # ... application logic continues
```

## Reference Tables

| | Short-term | Long-term |
|---|---|---|
| Where | LLM context window / agent state | external DB, knowledge graph, vector store |
| Scope | one session/thread | across sessions, shared between threads |
| Retrieval | already present | query → semantic search → inject into context |
| Lifetime | lost at session end | persistent |
| ADK | `Session` + `State` | `MemoryService` |
| LangGraph | agent state + `checkpointer` | `store` under namespaces |

| ADK state prefix | Scope | Persisted |
|---|---|---|
| *(none)* | this session | per `SessionService` |
| `user:` | this user, all sessions | yes |
| `app:` | all users of the app | yes |
| `temp:` | current processing turn | no |

| Long-term type | Content | Typical implementation |
|---|---|---|
| Semantic | facts, preferences, domain knowledge | user profile JSON, or a collection of fact documents |
| Episodic | past events/actions, how a task was done | few-shot examples from successful past sequences |
| Procedural | rules, core instructions, behaviors | the system prompt, self-refined via Reflection |

| Service tier | ADK Session | ADK Memory |
|---|---|---|
| Testing | `InMemorySessionService` | `InMemoryMemoryService` |
| Persistent | `DatabaseSessionService` (SQLite/Postgres) | — |
| Managed cloud | `VertexAiSessionService` | `VertexAiRagMemoryService`, `VertexAiMemoryBankService` |

## Worked Example
**The message-exchange cycle in ADK**, which is where the "don't mutate state directly" rule comes from:

1. A message arrives.
2. The `Runner` retrieves or establishes a `Session` via the `SessionService`.
3. The agent processes the message using the session's context — `state` plus historical `events`.
4. The agent generates a response and may update state.
5. The `Runner` encapsulates this as an `Event`.
6. `session_service.append_event` records the event **and** persists the state change.
7. The session awaits the next message; `delete_session` terminates it when done.

Every guarantee — event history, persistence, timestamp metadata, concurrency safety — lives in step 6. Writing `session.state["x"] = y` after step 2 skips it, so the change exists only in the local object. The two sanctioned paths back into step 6 are `output_key` (for a final text reply) and `EventActions.state_delta` (for everything else); putting the delta inside a tool via `ToolContext.state`, as `log_user_login` does, keeps the logic beside the action that caused it.

## Key Takeaways
1. Memory lets agents keep track, learn, and personalize; without it they are stateless and limited to one-shot interactions.
2. Short-term memory is the ephemeral context window; long-term memory is external, persistent, and retrieved by semantic search.
3. Long context enlarges short-term memory but does not replace persistence.
4. ADK models this as `Session` (thread), `State` (temporary data), and `MemoryService` (searchable long-term knowledge).
5. State prefixes (`user:`, `app:`, `temp:`) declare scope and persistence — use them deliberately.
6. Update state through `output_key` or `EventActions.state_delta` on `append_event`; never mutate the dict directly.
7. Long-term memory splits into semantic (facts), episodic (experiences, as few-shot examples), and procedural (rules, refined by Reflection).
8. LangChain's `ConversationBufferMemory` handles single-conversation recall; LangGraph stores handle cross-session memory; Vertex Memory Bank manages extraction, consolidation, and contradiction resolution across ADK, LangGraph, and CrewAI.

## Connects To
- **Ch 4 (Reflection)**: memory makes reflection cumulative; reflection updates procedural memory.
- **Ch 9 (Learning and Adaptation)**: long-term memory is where learned strategies persist.
- **Ch 14 (Knowledge Retrieval / RAG)**: the retrieval machinery behind semantic long-term memory.
- **Ch 7 (Multi-Agent Collaboration)**: session state is the shared blackboard between agents.
- **Ch 1 (Prompt Chaining)**: conversational state accumulated across chained turns.
- **Ch 16 (Resource-Aware Optimization)**: context is a budget; summarization is the compression lever.
