# Chapter 5: Tool Use (Function Calling)

## Core Idea
Tool Use breaks the LLM out of its training data: describe external functions to the model, let it decide when one is needed and emit a structured call, have the orchestration layer execute it, and feed the result back as context.

## Frameworks Introduced
- **Tool Use pattern (the six-step cycle)**:
  1. **Tool Definition** — external functions are defined and described to the LLM: purpose, name, parameters with types and descriptions.
  2. **LLM Decision** — given the request plus tool definitions, the model decides whether calling one or more tools is necessary.
  3. **Function Call Generation** — the model emits structured output (usually JSON) naming the tool and the arguments extracted from the request.
  4. **Tool Execution** — the framework intercepts that output, identifies the tool, and runs the real function.
  5. **Observation/Result** — the tool's output returns to the agent.
  6. **LLM Processing** — the model consumes the result and either answers, calls another tool, reflects, or advances the workflow.
  - When to use: whenever the agent must leave the LLM's internal knowledge — real-time data, private/proprietary data, precise calculation, code execution, or triggering actions in other systems.
- **"Tool calling" as the broader frame**: a tool need not be a local function. It can be an API endpoint, a database query, or *an instruction directed at another specialized agent* — a primary agent delegating data analysis to a dedicated "analyst agent". This framing captures agents as orchestrators across a diverse ecosystem of digital resources and other intelligent entities.

## Key Concepts
- **Function calling**: the technical mechanism bridging the LLM's reasoning and external functionality.
- **Tool description as contract**: the docstring/description is what the model routes on — it is executable specification, not documentation.
- **`@tool` decorator** (LangChain / CrewAI): wraps a Python function into a tool, deriving name, signature, and description.
- **`create_tool_calling_agent` + `AgentExecutor`** (LangChain): the agent decides; the executor is the runtime that actually invokes tools and loops.
- **`agent_scratchpad`**: the required prompt placeholder holding the agent's intermediate tool-call steps.
- **`BuiltInCodeExecutor`** (ADK): a sandboxed Python interpreter for deterministic logic and precise calculation, outside the scope of probabilistic generation.
- **`VSearchAgent`** (ADK): answers from a Vertex AI Search datastore, returning `grounding_metadata` with source attributions.
- **Vertex AI Extension**: a structured API wrapper with enterprise security, privacy, and performance guarantees. **Key difference from function calling: Vertex AI executes extensions automatically, whereas function calls require manual execution by the user or client.**

## Mental Models
- Tool Use is what turns a **text generator into an agent that senses, reasons, and acts** in the digital or physical world.
- The LLM is the **dispatcher, not the executor** — it produces intent (a JSON call), the framework produces effect.
- **Raise errors, don't return error strings**: a tool that raises `ValueError` gives the agent a real outcome to handle; a stringified error is indistinguishable from data.
- Think of tools as **capabilities you'd give a new hire**: named clearly, scoped narrowly, documented by when to reach for them.

## Anti-patterns
- **Vague tool descriptions**: the model can only select on what you wrote. "Use this tool to find answers to phrases like 'capital of France' or 'weather in London?'" beats "searches things".
- **Returning error text as a normal result**: forces the model to parse prose to detect failure. Raise a specific exception and let the agent decide the next action.
- **Asking the LLM to do arithmetic itself**: delegate to a calculator or code executor; precise calculation is outside probabilistic generation.
- **Omitting `agent_scratchpad`** from the prompt template — the agent loses its intermediate steps.
- **Hardcoding API keys**: use environment variables loaded at runtime or a secret manager.
- **No task-level failure instruction**: tell the agent explicitly what to do when the tool fails ("If the ticker is not found, you must report that you were unable to retrieve the price").

## Code Examples
LangChain — define a tool, bind it, run the executor:
```python
from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.tools import tool as langchain_tool
from langchain.agents import create_tool_calling_agent, AgentExecutor

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash", temperature=0)

@langchain_tool
def search_information(query: str) -> str:
    """
    Provides factual information on a given topic. Use this tool to find answers
    to phrases like 'capital of France' or 'weather in London?'.
    """
    simulated_results = {
        "weather in london": "The weather in London is currently cloudy, 15°C.",
        "capital of france": "The capital of France is Paris.",
        "population of earth": "The estimated population of Earth is around 8 billion.",
        "default": f"Simulated search result for '{query}': no specific information found.",
    }
    return simulated_results.get(query.lower(), simulated_results["default"])

tools = [search_information]

agent_prompt = ChatPromptTemplate.from_messages([
    ("system", "You are a helpful assistant."),
    ("human", "{input}"),
    ("placeholder", "{agent_scratchpad}"),   # required: holds the agent's internal steps
])

agent = create_tool_calling_agent(llm, tools, agent_prompt)
agent_executor = AgentExecutor(agent=agent, tools=tools, verbose=True)

await agent_executor.ainvoke({"input": "What's the weather like in London?"})
```
- **What it demonstrates**: the docstring *is* the tool contract the model selects on. `AgentExecutor` is the runtime loop; the agent alone only decides.

CrewAI — a tool that returns clean data and raises on failure:
```python
from crewai import Agent, Task, Crew
from crewai.tools import tool

@tool("Stock Price Lookup Tool")
def get_stock_price(ticker: str) -> float:
    """
    Fetches the latest simulated stock price for a given stock ticker symbol.
    Returns the price as a float. Raises a ValueError if the ticker is not found.
    """
    simulated_prices = {"AAPL": 178.15, "GOOGL": 1750.30, "MSFT": 425.50}
    price = simulated_prices.get(ticker.upper())
    if price is None:
        # Raising a specific error beats returning a string; the agent can handle
        # the exception and decide on the next action.
        raise ValueError(f"Simulated price for ticker '{ticker.upper()}' not found.")
    return price

financial_analyst_agent = Agent(
    role='Senior Financial Analyst',
    goal='Analyze stock data using provided tools and report key prices.',
    backstory="You are an experienced financial analyst adept at using data sources "
              "to find stock information. You provide clear, direct answers.",
    tools=[get_stock_price], allow_delegation=False, verbose=True)

analyze_aapl_task = Task(
    description=("What is the current simulated stock price for Apple (ticker: AAPL)? "
                 "Use the 'Stock Price Lookup Tool' to find it. "
                 "If the ticker is not found, you must report that you were unable "
                 "to retrieve the price."),
    expected_output=("A single, clear sentence stating the simulated stock price for AAPL. "
                     "For example: 'The simulated stock price for AAPL is $178.15.' "
                     "If the price cannot be found, state that clearly."),
    agent=financial_analyst_agent)

financial_crew = Crew(agents=[financial_analyst_agent], tasks=[analyze_aapl_task], verbose=True)
result = financial_crew.kickoff()
```
- **What it demonstrates**: `expected_output` pins the response shape; the task description covers *both* the success and the failure path. The tool returns a raw `float`, keeping it reusable.

Google ADK — pre-built tools and the code executor:
```python
from google.adk.agents import Agent, LlmAgent
from google.adk.runners import Runner
from google.adk.sessions import InMemorySessionService
from google.adk.tools import google_search
from google.adk.code_executors import BuiltInCodeExecutor

# 1. Pre-built web search
root_agent = Agent(
    name="basic_search_agent", model="gemini-2.0-flash-exp",
    description="Agent to answer questions using Google Search.",
    instruction="I can answer your questions by searching the internet. Just ask me anything!",
    tools=[google_search])

# 2. Sandboxed code execution for exact arithmetic
code_agent = LlmAgent(
    name="calculator_agent", model="gemini-2.0-flash",
    code_executor=BuiltInCodeExecutor(),
    instruction="""You are a calculator agent.
    When given a mathematical expression, write and execute Python code to calculate
    the result. Return only the final numerical result as plain text, without markdown
    or code blocks.""",
    description="Executes Python code to perform calculations.")

# 3. Enterprise search over a Vertex AI Search datastore
vsearch_agent = agents.VSearchAgent(
    name="q2_strategy_vsearch_agent",
    description="Answers questions about Q2 strategy documents using Vertex AI Search.",
    model="gemini-2.0-flash-exp",
    datastore_id=os.environ.get("DATASTORE_ID"),
    model_parameters={"temperature": 0.0})
```
- **What it demonstrates**: with a code executor, events carry `part.executable_code` (the generated source) and `part.code_execution_result` (outcome + output) — inspect both when debugging. `VSearchAgent` events expose `grounding_metadata.grounding_attributions` for source citation.

## Reference Tables

| Use case | Tool | Agent flow |
|---|---|---|
| Weather | weather API (location → conditions) | identify need → call with "London" → format response |
| E-commerce | inventory / order-status / payment APIs | "Is X in stock?" → inventory API → report stock |
| Financial | calculator, stock API, spreadsheet | stock API for price → calculator for profit → format |
| Personal assistant | email-sending API | extract recipient/subject/body → send |
| Coding assistant | code interpreter | run the snippet → analyze output |
| Smart home | smart-light API | map command + target device → call |

| Framework | Tool definition | Runtime |
|---|---|---|
| LangChain | `@tool` decorator on a Python function | `create_tool_calling_agent` + `AgentExecutor` |
| CrewAI | `@tool("Name")` decorator | `Crew(...).kickoff()` |
| Google ADK | pre-built (`google_search`, `BuiltInCodeExecutor`, `VSearchAgent`) or `FunctionTool` | `Runner` + session service |

| | Function calling | Vertex AI Extension |
|---|---|---|
| Execution | manual, by the user/client | automatic, by Vertex AI |
| Controls | whatever you build | enterprise security, data privacy, performance guarantees |
| Examples | any Python function | Code Interpreter, Vertex AI Search, custom |

## Worked Example
**Financial query with two chained tools.** User: *"What's the current price of AAPL and calculate the potential profit if I bought 100 shares at $150?"*

1. The model reads the request against its tool definitions and recognizes two needs: current price (external data) and arithmetic (precision).
2. It emits a call to the stock API with `{"ticker": "AAPL"}`. The framework executes it; the tool returns `178.15` as a float — clean data, no prose.
3. The result returns as an observation. The model now emits a second call, to the calculator, with the arithmetic `(178.15 - 150) * 100`.
4. The calculator returns `2815.0`.
5. The model composes the final answer from both observations.

Had the price lookup failed, the tool raises `ValueError` rather than returning `"not found"` — the agent sees an exception, and the task's explicit failure instruction tells it to report inability rather than invent a number. That combination — clean data on success, a raised error on failure, and a task-level instruction for both — is what makes tool use reliable.

## Key Takeaways
1. Tool Use lets agents interact with external systems and reach dynamic information.
2. Tools are defined with clear descriptions and typed parameters the LLM can understand; the description drives selection.
3. The LLM decides *when* to use a tool and generates a structured call; the framework executes it and returns the result.
4. Think "tool calling", not just "function calling" — an API, a database, or another agent all qualify.
5. Tools should return clean typed data and raise specific errors, not encode failure in strings.
6. LangChain gives `@tool`, `create_tool_calling_agent`, and `AgentExecutor`; ADK ships Google Search, Code Execution, and Vertex AI Search out of the box.
7. Extensions execute automatically under enterprise controls; plain function calls execute on your side.
8. Delegate deterministic logic and precise calculation to a code executor rather than the model.

## Connects To
- **Ch 1 (Prompt Chaining)**: tools are the deterministic work slotted between chain links.
- **Ch 2 (Routing)**: tool selection is routing at subroutine granularity.
- **Ch 10 (Model Context Protocol)**: MCP standardizes tool discovery and invocation across servers.
- **Ch 14 (Knowledge Retrieval / RAG)**: retrieval is a tool; `VSearchAgent` is RAG as a built-in.
- **Ch 15 (Inter-Agent Communication / A2A)**: another agent as a tool.
- **Ch 18 (Guardrails)**: tool execution is where real-world side effects — and real-world risk — enter.
- **Ch 22 (Advanced Prompting)**: ReAct interleaves thought with tool actions.
