# Chapter 15: Inter-Agent Communication (A2A)

## Core Idea
A2A is an open, HTTP-based standard that lets agents built on *different frameworks* — ADK, LangGraph, CrewAI — discover each other, delegate tasks, and exchange results. Where MCP standardizes agent-to-tool, A2A standardizes agent-to-agent.

## Frameworks Introduced
- **The Agent2Agent (A2A) protocol**: interoperability between agent frameworks, so agents collaborate regardless of origin.
  - When to use: orchestrating collaboration between two or more agents, especially across frameworks; modular applications where specialists handle parts of a workflow; and when an agent must **dynamically discover and consume** other agents' capabilities.
  - Backed by Atlassian, Box, LangChain, MongoDB, Salesforce, SAP, ServiceNow; Microsoft planning integration into Azure AI Foundry and Copilot Studio; Auth0 and SAP integrating support.
- **Three Core Actors**:
  - **User** — initiates requests for agent assistance.
  - **A2A Client (Client Agent)** — an application or agent acting on the user's behalf to request actions or information.
  - **A2A Server (Remote Agent)** — an agent providing an HTTP endpoint that processes requests and returns results. **The remote agent is an "opaque" system**: the client need not understand its internal operation.
- **Agent Card** — the agent's digital identity, usually a JSON file, carrying identity, endpoint URL, version, supported capabilities (streaming, push notifications, state transition history), skills, default input/output modes, and authentication requirements.
- **Three Agent Discovery strategies**:
  - **Well-Known URI** — the card is hosted at a standardized path (`/.well-known/agent.json`). Broad, often automated accessibility for public or domain-specific use.
  - **Curated Registries** — a centralized catalog where cards are published and queried by criteria. Suited to enterprises needing centralized management and access control.
  - **Direct Configuration** — card information embedded or privately shared. For closely coupled or private systems where dynamic discovery isn't crucial.
  - Secure card endpoints regardless of method: access control, mutual TLS, or network restrictions — the card may contain sensitive (though non-secret) information.
- **Four Interaction Mechanisms**:
  - **Synchronous Request/Response** — client sends and waits for a complete response in one exchange. For quick operations.
  - **Asynchronous Polling** — server immediately acknowledges with a "working" status and task ID; the client does other work and periodically polls until "completed" or "failed". For longer tasks.
  - **Streaming Updates (Server-Sent Events)** — a persistent one-way connection from server to client; the remote agent continuously pushes status changes and partial results. For real-time incremental results.
  - **Push Notifications (Webhooks)** — the client registers a webhook URL; the server pushes asynchronously when status changes significantly. For very long-running or resource-intensive tasks where constant connections or frequent polling are inefficient.

## Key Concepts
- **Task**: the fundamental unit of work; asynchronous, uniquely identified, moving through states (`submitted`, `working`, `completed`), supporting parallel processing.
- **Message**: carries **attributes** (key-value metadata like priority or creation time) and one or more **parts** holding the actual content — plain text, files, or structured JSON.
- **Artifact**: the tangible output an agent generates during a task; also composed of parts, and **can be streamed incrementally** as results become available.
- **`contextId`**: server-generated, groups related tasks to preserve context across interactions.
- **Transport**: all communication over HTTP(S) using **JSON-RPC 2.0** for payloads.
- **`sendTask` vs `sendTaskSubscribe`**: the synchronous method expects one complete answer; the streaming method establishes a persistent connection for multiple incremental updates.
- **`input-required` state**: supports multi-turn conversations where the agent requests additional information while maintaining context.
- **Modality-agnostic**: A2A facilitates these patterns for audio and video as well as text, enabling multimodal applications.
- **Security**: mutual TLS for encrypted authenticated connections; comprehensive audit logs of all inter-agent communication for accountability, troubleshooting, and security analysis; authentication requirements declared in the Agent Card; credentials (OAuth 2.0 tokens, API keys) passed via **HTTP headers**, never in URLs or message bodies.

## Mental Models
- **The Agent Card is a service contract plus a business card**: it is simultaneously how you find an agent, how you know what it can do, and how you know how to authenticate to it.
- **Opacity is the point.** The client treats the remote agent as a black box, which is exactly what makes cross-framework interoperability possible.
- **Match the interaction mechanism to task duration**: seconds → synchronous; minutes → polling; incremental output → SSE; hours → webhooks.
- **A2A and MCP are complements, not competitors**: MCP structures an agent's context and its interaction with external data and tools; A2A handles coordination, delegation, and collaboration *between* agents.

## Anti-patterns
- **Credentials in URLs or message bodies**: pass them in HTTP headers.
- **Unsecured Agent Card endpoints**: even non-secret card content warrants access control, mTLS, or network restrictions.
- **Polling a long-running task frequently**: inefficient; register a webhook instead.
- **Holding a connection open for an hours-long task**: same problem from the other direction.
- **Skipping audit logs**: without them, multi-agent failures are undebuggable and unaccountable.
- **Vague skill descriptions and no examples in the Agent Card**: discovery depends on other agents understanding what yours does.
- **Assuming A2A replaces MCP** (or vice versa): different layers of the stack.

## Code Examples
An Agent Card (WeatherBot):
```json
{
  "name": "WeatherBot",
  "description": "Provides accurate weather forecasts and historical data.",
  "url": "http://weather-service.example.com/a2a",
  "version": "1.0.0",
  "capabilities": {
    "streaming": true,
    "pushNotifications": false,
    "stateTransitionHistory": true
  },
  "authentication": { "schemes": ["apiKey"] },
  "defaultInputModes": ["text"],
  "defaultOutputModes": ["text"],
  "skills": [
    {
      "id": "get_current_weather",
      "name": "Get Current Weather",
      "description": "Retrieve real-time weather for any location.",
      "inputModes": ["text"],
      "outputModes": ["text"],
      "examples": ["What's the weather in Paris?", "Current conditions in Tokyo"],
      "tags": ["weather", "current", "real-time"]
    },
    {
      "id": "get_forecast",
      "name": "Get Forecast",
      "description": "Get 5-day weather predictions.",
      "inputModes": ["text"],
      "outputModes": ["text"],
      "examples": ["5-day forecast for New York", "Will it rain in London this weekend?"],
      "tags": ["weather", "forecast", "prediction"]
    }
  ]
}
```

Synchronous vs streaming request (JSON-RPC 2.0):
```json
// Synchronous — one complete answer
{
  "jsonrpc": "2.0", "id": "1", "method": "sendTask",
  "params": {
    "id": "task-001", "sessionId": "session-001",
    "message": { "role": "user",
                 "parts": [{ "type": "text",
                             "text": "What is the exchange rate from USD to EUR?" }] },
    "acceptedOutputModes": ["text/plain"], "historyLength": 5
  }
}

// Streaming — persistent connection, incremental updates
{
  "jsonrpc": "2.0", "id": "2", "method": "sendTaskSubscribe",
  "params": {
    "id": "task-002", "sessionId": "session-001",
    "message": { "role": "user",
                 "parts": [{ "type": "text",
                             "text": "What's the exchange rate for JPY to GBP today?" }] },
    "acceptedOutputModes": ["text/plain"], "historyLength": 5
  }
}
```

Building the ADK agent behind an A2A server:
```python
import datetime
from google.adk.agents import LlmAgent
from google.adk.tools.google_api_tool import CalendarToolset

async def create_agent(client_id, client_secret) -> LlmAgent:
    """Constructs the ADK agent."""
    toolset = CalendarToolset(client_id=client_id, client_secret=client_secret)
    return LlmAgent(
        model='gemini-2.0-flash-001',
        name='calendar_agent',
        description="An agent that can help manage a user's calendar",
        instruction=f"""
You are an agent that can help manage a user's calendar.
Users will request information about the state of their calendar or to make changes
to their calendar. Use the provided tools for interacting with the calendar API.
If not specified, assume the calendar the user wants is the 'primary' calendar.
When using the Calendar API tools, use well-formed RFC3339 timestamps.
Today is {datetime.datetime.now()}.
""",
        tools=await toolset.get_tools(),
    )
```

Exposing it as an A2A server:
```python
def main(host: str, port: int):
    if os.getenv('GOOGLE_GENAI_USE_VERTEXAI') != 'TRUE' and not os.getenv('GOOGLE_API_KEY'):
        raise ValueError('GOOGLE_API_KEY environment variable not set and '
                         'GOOGLE_GENAI_USE_VERTEXAI is not TRUE.')

    skill = AgentSkill(
        id='check_availability',
        name='Check Availability',
        description="Checks a user's availability for a time using their Google Calendar",
        tags=['calendar'],
        examples=['Am I free from 10am to 11am tomorrow?'])

    agent_card = AgentCard(
        name='Calendar Agent',
        description="An agent that can manage a user's calendar",
        url=f'http://{host}:{port}/',
        version='1.0.0',
        defaultInputModes=['text'], defaultOutputModes=['text'],
        capabilities=AgentCapabilities(streaming=True),
        skills=[skill])

    adk_agent = asyncio.run(create_agent(client_id=os.getenv('GOOGLE_CLIENT_ID'),
                                         client_secret=os.getenv('GOOGLE_CLIENT_SECRET')))

    runner = Runner(app_name=agent_card.name, agent=adk_agent,
                    artifact_service=InMemoryArtifactService(),
                    session_service=InMemorySessionService(),
                    memory_service=InMemoryMemoryService())

    agent_executor = ADKAgentExecutor(runner, agent_card)

    async def handle_auth(request: Request) -> PlainTextResponse:
        await agent_executor.on_auth_callback(
            str(request.query_params.get('state')), str(request.url))
        return PlainTextResponse('Authentication successful.')

    request_handler = DefaultRequestHandler(agent_executor=agent_executor,
                                            task_store=InMemoryTaskStore())
    a2a_app = A2AStarletteApplication(agent_card=agent_card, http_handler=request_handler)

    routes = a2a_app.routes()
    routes.append(Route(path='/authenticate', methods=['GET'], endpoint=handle_auth))
    uvicorn.run(Starlette(routes=routes), host=host, port=port)
```
- **What it demonstrates**: the shape of an A2A server. The `AgentSkill` (with `examples` and `tags`) is what other agents discover and match against; the `AgentCard` declares the URL and streaming capability; `A2AStarletteApplication` supplies the protocol routes and you append your own (here, an OAuth callback).

## Reference Tables

| Interaction mechanism | Method / transport | Best for |
|---|---|---|
| Synchronous request/response | `sendTask` | quick, immediate operations |
| Asynchronous polling | `sendTask` + status polls | longer tasks; client free to do other work |
| Streaming (SSE) | `sendTaskSubscribe` | real-time incremental results, partial output |
| Push notifications (webhooks) | registered callback URL | very long-running or resource-intensive tasks |

| Discovery strategy | Mechanism | Fits |
|---|---|---|
| Well-Known URI | `/.well-known/agent.json` | public or domain-wide, automated |
| Curated Registry | centralized queryable catalog | enterprise: central management, access control |
| Direct Configuration | embedded/privately shared card | closely coupled or private systems |

| | A2A | MCP |
|---|---|---|
| Standardizes | agent ↔ agent coordination, delegation, collaboration | agent ↔ external data and tools; context structuring |
| Unit | tasks, messages, artifacts | tools, resources, prompts |
| Identity artifact | Agent Card | server manifest |
| Level | high-level workflow orchestration | resource interface |

| A2A concept | Composition |
|---|---|
| Task | id + state (`submitted`/`working`/`completed`) |
| Message | attributes (metadata) + parts (text, files, JSON) |
| Artifact | parts; streamable incrementally |
| contextId | groups related tasks for continuity |

| Security mechanism | Purpose |
|---|---|
| Mutual TLS | encrypted, authenticated connections; prevents interception |
| Comprehensive audit logs | accountability, troubleshooting, security analysis |
| Agent Card declaration | centralizes authentication requirements |
| Credentials in HTTP headers | OAuth 2.0 tokens / API keys, never in URLs or bodies |

## Worked Example
**The Calendar Agent as an A2A server** — the sample walks through the full path from capability to network service:

1. **Define the skill.** `check_availability`, described as "checks a user's availability for a time using their Google Calendar", tagged `calendar`, with the example `"Am I free from 10am to 11am tomorrow?"`. The example is not decoration — it is what a discovering client matches an incoming user request against.
2. **Publish the Agent Card.** Name, description, URL derived from host/port, version, text in/out modes, and `AgentCapabilities(streaming=True)` — which tells clients they may use `sendTaskSubscribe`.
3. **Build the underlying agent.** An ADK `LlmAgent` with the `CalendarToolset`, instructed to default to the `primary` calendar and to use well-formed RFC3339 timestamps, with today's date injected for temporal grounding.
4. **Wire the runtime.** A `Runner` with in-memory artifact, session, and memory services; an `ADKAgentExecutor` bridging the runner to the protocol; a `DefaultRequestHandler` with a task store (tasks are stateful and need somewhere to live).
5. **Serve.** `A2AStarletteApplication` provides the protocol routes; an extra `/authenticate` route handles the OAuth callback; Uvicorn runs it.

Note the layering: the *agent* knows nothing about A2A, and the *A2A server* knows nothing about calendars. The `ADKAgentExecutor` is the only piece that touches both — which is why the same agent could be exposed over a different protocol, and why a LangGraph or CrewAI agent can sit behind an identical card.

## Key Takeaways
1. A2A is an open, HTTP-based standard for communication between agents built with different frameworks.
2. The Agent Card is the digital identity enabling automatic discovery and capability understanding.
3. Four interaction mechanisms — synchronous, polling, SSE streaming, webhooks — match different task durations.
4. Communication is structured as asynchronous **tasks** with states; **messages** carry attributes and parts; **artifacts** are the streamable outputs.
5. Multi-turn conversation is supported through an `input-required` state that preserves context.
6. Discovery can be well-known URI, curated registry, or direct configuration; secure the endpoint either way.
7. Security is built in: mTLS, audit logs, card-declared auth requirements, credentials in headers.
8. A2A encourages modular architecture with specialized agents on independent ports, enabling scale and distribution.
9. A2A handles high-level task and workflow coordination between agents; MCP provides the interface to external resources. Use both.

## Connects To
- **Ch 10 (Model Context Protocol)**: the complementary protocol — tools vs agents.
- **Ch 7 (Multi-Agent Collaboration)**: A2A is the standardized communication protocol that pattern depends on.
- **Ch 5 (Tool Use)**: a remote agent reached over A2A is a tool in the broad sense.
- **Ch 2 (Routing)**: skill matching from Agent Cards is discovery-driven routing.
- **Ch 19 (Evaluation and Monitoring)**: audit logs and trace tooling for multi-agent debugging.
- **Ch 24 (Agentic Frameworks Overview)**: which frameworks interoperate through A2A.
