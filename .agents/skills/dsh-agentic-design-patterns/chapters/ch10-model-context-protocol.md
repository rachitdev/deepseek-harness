# Chapter 10: Model Context Protocol (MCP)

## Core Idea
MCP is an open, client-server standard that lets any compliant LLM discover and use any compliant tool — a universal adapter replacing per-integration custom code. But it is only a contract for an agentic *interface*; the quality of the underlying API and the agent-friendliness of its data still determine whether the agent can actually work.

## Frameworks Introduced
- **MCP as a client-server protocol**: an MCP **server** exposes three kinds of thing to an MCP **client** (an LLM host application or agent):
  - **Resources** — static data (a PDF file, a database record).
  - **Tools** — executable functions that perform actions (send an email, query an API).
  - **Prompts** — templates that guide the LLM in how to interact with a resource or tool, keeping the interaction structured and effective.
- **The four components**:
  1. **LLM** — the core intelligence: processes requests, formulates plans, decides when external access is needed.
  2. **MCP Client** — an application or wrapper around the LLM; translates intent into a standard-conformant request; discovers, connects to, and communicates with servers.
  3. **MCP Server** — the gateway to the external world; exposes tools/resources/prompts to authorized clients; typically owns one domain (internal database, email service, public API).
  4. **Optional Third-Party Service** — the actual external tool or data source the server manages.
- **The five-step interaction flow**:
  1. **Discovery** — the client queries a server for its capabilities; the server responds with a **manifest** of tools (`send_email`), resources (`customer_database`), and prompts.
  2. **Request Formulation** — the LLM chooses a tool and specifies parameters.
  3. **Client Communication** — the client sends a standardized call to the appropriate server.
  4. **Server Execution** — the server authenticates the client, validates the request, and interfaces with the underlying software.
  5. **Response and Context Update** — a standardized response returns (success plus output, e.g. a confirmation ID); the client passes it to the LLM, updating context.
- **The federated-model argument**: wrapping disparate and legacy services in an MCP-compliant interface brings them into a modern ecosystem. They keep operating independently but become composable into new workflows orchestrated by LLMs — agility and reuse without costly rewrites of foundational systems.

## Key Concepts
- **Discoverability / just-in-time discovery**: a client can query a server for new capabilities without the agent being redeployed.
- **Transport mechanisms**: local — **JSON-RPC over STDIO** for efficient inter-process communication; remote — **Streamable HTTP** and **Server-Sent Events (SSE)** for persistent client-server communication.
- **Local vs remote server**: local for speed and security with sensitive data; remote for shared, scalable access to common tools across an organization.
- **On-demand vs batch**: MCP supports interactive sessions and larger-scale batch processing.
- **`MCPToolset`** (ADK): the client-side handle; configured with `StdioServerParameters` (local process) or `HttpServerParameters` (remote URL), with an optional `tool_filter`.
- **`npx`**: runs Node.js packages from the npm registry without global installation — how most community MCP servers are launched.
- **`uvx`**: the Python equivalent, running tools in a temporary isolated environment via `uv`.
- **FastMCP**: a high-level Python framework abstracting protocol complexity; decorators define tools/resources/prompts, with **automatic schema generation** from function signatures, type hints, and docstrings. Also supports server composition and proxying.

## Mental Models
- **Function calling is a specific set of custom-built tools — a particular wrench and screwdriver.** MCP is a **universal standardized power outlet system**: it provides no tools itself, but lets any compliant tool from any manufacturer plug in.
- **MCP does not make a bad API good.** The protocol standardizes the connection, not the semantics — a ticketing API that only fetches full tickets one at a time will make an agent asked to "summarize high-priority tickets" slow and inaccurate at volume, MCP or not.
- **Agents do not replace deterministic workflows; they need stronger deterministic support to succeed.** Add filtering and sorting to the underlying API so the non-deterministic agent can work efficiently.
- **Data format is part of the contract.** An MCP server for a document store that returns PDFs is mostly useless if the agent cannot parse PDFs. Return Markdown instead.

## Anti-patterns
- **Wrapping legacy APIs unmodified**: the most-cited failure. The MCP surface inherits every inefficiency of the API beneath it.
- **Exposing agent-unfriendly data formats**: PDFs, binary blobs, anything the consuming agent cannot read.
- **No authentication/authorization**: exposing tools and data over any protocol requires controlling which clients reach which servers and what actions they may perform.
- **No error-handling strategy**: the protocol must define how tool-execution failure, unavailable servers, and invalid requests are communicated back to the LLM so it can understand the failure and try an alternative.
- **Reaching for MCP on a simple app**: for a fixed, limited set of predefined functions, direct tool function calling is sufficient.
- **Relative paths in server args**: the filesystem server requires an **absolute** path to the folder it may access.

## Code Examples
ADK client → local filesystem MCP server over STDIO:
```python
import os
from google.adk.agents import LlmAgent
from google.adk.tools.mcp_tool.mcp_toolset import MCPToolset, StdioServerParameters

TARGET_FOLDER_PATH = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "mcp_managed_files")
os.makedirs(TARGET_FOLDER_PATH, exist_ok=True)

root_agent = LlmAgent(
    model='gemini-2.0-flash',
    name='filesystem_assistant_agent',
    instruction=('Help the user manage their files. You can list files, read files, '
                 f'and write files. You are operating in the following directory: '
                 f'{TARGET_FOLDER_PATH}'),
    tools=[
        MCPToolset(
            connection_params=StdioServerParameters(
                command='npx',
                args=["-y",                                    # auto-confirm install
                      "@modelcontextprotocol/server-filesystem",
                      TARGET_FOLDER_PATH],                     # MUST be absolute
            ),
            # Optional: restrict which server tools are exposed
            # tool_filter=['list_directory', 'read_file'],
        )
    ],
)
```
Also requires `__init__.py` beside `agent.py` (`from . import agent`) so ADK discovers the package. Run with `adk web` from the parent directory.

Other launch commands for the same slot:
```python
# A local Python MCP server
connection_params = StdioConnectionParams(server_params={
    "command": "python3",
    "args": ["./agent/mcp_server.py"],
    "env": {"SERVICE_ACCOUNT_PATH": SERVICE_ACCOUNT_PATH, "DRIVE_FOLDER_ID": DRIVE_FOLDER_ID},
})

# A published Python MCP server, run in an isolated temp environment
connection_params = StdioConnectionParams(server_params={
    "command": "uvx",
    "args": ["mcp-google-sheets@latest"],
    "env": {"SERVICE_ACCOUNT_PATH": SERVICE_ACCOUNT_PATH, "DRIVE_FOLDER_ID": DRIVE_FOLDER_ID},
})
```

FastMCP — build a server in ~10 lines:
```python
# fastmcp_server.py     (pip install fastmcp)
from fastmcp import FastMCP

mcp_server = FastMCP()

@mcp_server.tool
def greet(name: str) -> str:
    """
    Generates a personalized greeting.
    Args:
        name: The name of the person to greet.
    Returns:
        A greeting string.
    """
    return f"Hello, {name}! Nice to meet you."

if __name__ == "__main__":
    mcp_server.run(transport="http", host="127.0.0.1", port=8000)
```
- **What it demonstrates**: the docstring becomes the tool description the LLM sees, and the type hints become the schema — FastMCP generates the interface specification automatically, so there is no separate manifest to maintain.

ADK client → the FastMCP server over HTTP:
```python
from google.adk.agents import LlmAgent
from google.adk.tools.mcp_tool.mcp_toolset import MCPToolset, HttpServerParameters

FASTMCP_SERVER_URL = "http://localhost:8000"

root_agent = LlmAgent(
    model='gemini-2.0-flash',
    name='fastmcp_greeter_agent',
    instruction='You are a friendly assistant that can greet people by their name. '
                'Use the "greet" tool.',
    tools=[
        MCPToolset(
            connection_params=HttpServerParameters(url=FASTMCP_SERVER_URL),
            tool_filter=['greet'],
        )
    ],
)
```

## Reference Tables

| Feature | Tool function calling | Model Context Protocol (MCP) |
|---|---|---|
| **Standardization** | proprietary and vendor-specific; format differs across LLM providers | open, standardized protocol promoting interoperability between LLMs and tools |
| **Scope** | a direct mechanism to request execution of a specific predefined function | a broader framework for how LLMs and external tools discover and communicate |
| **Architecture** | one-to-one between the LLM and the app's tool-handling logic | client-server; LLM apps (clients) connect to various MCP servers |
| **Discovery** | the LLM is explicitly told which tools exist in a given conversation | dynamic — a client can query a server for its capabilities |
| **Reusability** | integrations tightly coupled to the specific app and LLM | reusable, standalone MCP servers accessible by any compliant application |

| Component | Nature | Example |
|---|---|---|
| Resource | static data | a PDF file, a database record |
| Tool | executable function | send an email, query an API |
| Prompt | interaction template | how to structure a query against a resource |

| Deployment axis | Options | Choose by |
|---|---|---|
| Location | local / remote | speed & sensitive-data security vs shared scalable access |
| Mode | on-demand / batch | real-time conversational agent vs data pipeline |
| Transport | JSON-RPC over STDIO / Streamable HTTP + SSE | local IPC vs remote persistent connection |

| Use case | What MCP enables |
|---|---|
| Database integration | natural-language querying of BigQuery etc. via MCP Toolbox for Databases |
| Generative media orchestration | Imagen (image), Veo (video), Chirp 3 HD (voice), Lyria (music) |
| External API interaction | weather, stock prices, email, CRM |
| Reasoning-based extraction | return the precise clause or figure, not the whole document |
| Custom tool development | expose proprietary internal functions in standard form |
| Complex workflow orchestration | fetch customer data → generate image → draft email → send |
| IoT device control | smart appliances, industrial sensors, robotics |
| Financial services automation | market analysis, trade execution, regulatory reporting |

## Worked Example
**The ticketing-system trap** — the chapter's sharpest practical warning, and the reason "we added MCP" is not the same as "our agent works".

A team exposes their existing ticketing API through an MCP server. The API has one relevant endpoint: retrieve full ticket details, one ticket at a time. An agent is asked to *summarize high-priority tickets*. What happens: the agent must fetch every ticket individually to discover its priority, so latency scales linearly with ticket count, the context fills with irrelevant low-priority tickets, and accuracy degrades at volume. Nothing about MCP caused this and nothing about MCP fixes it.

The fix is on the API side: add **deterministic filtering and sorting** — `list_tickets(priority="high", sort="created_desc", limit=N)` — so the agent makes one call and receives exactly the relevant set. The generalization is the chapter's key line: *agents do not magically replace deterministic workflows; they often require stronger deterministic support to succeed.*

The paired warning is about format. An MCP server over a document store that returns PDFs satisfies the protocol perfectly and is still useless if the consuming agent cannot parse PDF. Return Markdown. **Consider not just the connection, but the nature of the data being exchanged.**

## Key Takeaways
1. MCP is an open standard for communication between LLMs and external applications, data sources, and tools.
2. It uses a client-server architecture defining how resources, prompts, and tools are exposed and consumed.
3. Dynamic discovery lets agents pick up new capabilities without redeployment — the main structural advantage over function calling.
4. Function calling is direct access to a few specific functions; MCP is the communication framework for discovering and using many.
5. MCP is a contract for an interface — it does not fix a poorly designed API beneath it. Add filtering, sorting, and other deterministic support.
6. Data must be agent-readable (Markdown, not PDF); MCP does not enforce this.
7. Security (authn/authz) and a defined error-handling strategy are required parts of any implementation.
8. ADK both consumes existing MCP servers and can expose ADK tools as one; FastMCP makes writing Python servers trivial via decorators and automatic schema generation.
9. For simple applications with a fixed set of functions, direct tool function calling is sufficient.

## Connects To
- **Ch 5 (Tool Use)**: MCP is the standardized, discoverable generalization of function calling.
- **Ch 15 (Inter-Agent Communication / A2A)**: the sibling protocol — A2A is agent-to-agent, MCP is agent-to-tool.
- **Ch 6 (Planning)**: the OpenAI Deep Research API uses MCP to reach private knowledge bases.
- **Ch 14 (Knowledge Retrieval / RAG)**: reasoning-based extraction over MCP resources.
- **Ch 18 (Guardrails)**: authorization boundaries and `tool_filter` are safety controls.
- **Ch 24 (Agentic Frameworks Overview)**: where MCP fits across the framework landscape.
