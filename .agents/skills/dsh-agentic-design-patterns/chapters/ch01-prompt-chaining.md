# Chapter 1: Prompt Chaining

## Core Idea
Rather than asking an LLM to solve a complex problem in one monolithic prompt, decompose it into a sequence of focused sub-prompts where each step's output becomes the next step's input — trading a single high-cognitive-load call for a controllable, debuggable pipeline.

## Frameworks Introduced
- **Prompt Chaining (a.k.a. the Pipeline pattern)**: break the original problem into smaller sub-problems, each addressed by a purpose-built prompt; the output of prompt N feeds prompt N+1.
  - When to use: the task is too complex for a single prompt, has multiple distinct processing stages, needs external tools between steps, or the system must maintain state across multi-step reasoning.
  - How:
    1. Decompose the request into ordered sub-tasks with one responsibility each.
    2. Write a focused prompt per sub-task; optionally assign each stage a distinct role ("Market Analyst" → "Trend Analyst" → "Expert Documentation Writer").
    3. Force **structured output** (JSON/XML) at each boundary so the next prompt parses rather than interprets.
    4. Insert deterministic logic between calls — validation, branching, tool invocation.
    5. Feed validated output forward; repeat conditionally on failure.
- **Context Engineering**: the systematic discipline of designing and delivering the complete informational environment to a model *before* token generation.
  - When to use: whenever output quality matters more than prompt phrasing — i.e. for any serious agentic system.
  - How: assemble system prompt + retrieved documents (RAG) + tool outputs + implicit data (user identity, interaction history, environmental state) + memory + structured-output contracts, then build runtime pipelines to fetch/transform that data and feedback loops to improve it.
  - Why it works: "even advanced models underperform when provided with a limited or poorly constructed view of the operational environment." Output quality depends less on model architecture than on context richness.

## Key Concepts
- **Instruction neglect**: the model silently skips parts of an overloaded prompt.
- **Contextual drift**: the model loses track of the initial context as the prompt grows.
- **Error propagation**: an early mistake amplifies through the rest of the response.
- **Sequential decomposition**: splitting a task so each step is simpler and less ambiguous, reducing cognitive load.
- **Structured output contract**: the JSON/XML schema passed between chain steps; the integrity of the whole chain depends on it.
- **LCEL (LangChain Expression Language)**: the `prompt | llm | parser` pipe syntax for composing chains.
- **LangGraph**: extends LangChain's linear chains to stateful, *cyclical* computation — needed for agentic loops.
- **Prompt optimizer** (e.g. Vertex AI prompt optimizer): systematically evaluates responses against sample inputs and metrics to refine prompts/system instructions across models without manual rewriting.

## Mental Models
- Think of a prompt chain as a **computational pipeline**: each function does one operation and hands its result on. Debuggability comes from the seams, not the steps.
- Use **role assignment per stage** when successive steps require different expertise — it narrows the model's search space at each hop.
- Think of **context as a primary component**, not decoration: what the agent knows, when it knows it, and how it uses it.
- Prefer **parallel gather + chained synthesis**: independent data collection fans out concurrently; dependent synthesis and refinement stay sequential.

## Anti-patterns
- **The monolithic prompt**: "analyze this report, summarize it, identify trends with data points, and draft an email" — the model summarizes well and then fails the rest.
- **Passing free-form prose between steps**: ambiguous or badly formatted intermediate output makes the next prompt fail on faulty input. Specify a format.
- **Chaining without validation gates**: no check between calls means errors compound silently.
- **Asking the LLM to do arithmetic mid-chain**: delegate precise calculation to an external calculator tool and feed the result back.
- **Confusing prompt engineering with context engineering**: optimizing phrasing while starving the model of retrieved documents, tool output, and history.

## Code Examples
```python
import os
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser

llm = ChatOpenAI(temperature=0)

# --- Prompt 1: Extract Information ---
prompt_extract = ChatPromptTemplate.from_template(
    "Extract the technical specifications from the following text:\n\n{text_input}"
)

# --- Prompt 2: Transform to JSON ---
prompt_transform = ChatPromptTemplate.from_template(
    "Transform the following specifications into a JSON object with "
    "'cpu', 'memory', and 'storage' as keys:\n\n{specifications}"
)

# --- Build the Chain using LCEL ---
extraction_chain = prompt_extract | llm | StrOutputParser()

full_chain = (
    {"specifications": extraction_chain}
    | prompt_transform
    | llm
    | StrOutputParser()
)

input_text = ("The new laptop model features a 3.5 GHz octa-core processor, "
              "16GB of RAM, and a 1TB NVMe SSD.")
final_result = full_chain.invoke({"text_input": input_text})
print(final_result)
```
- **What it demonstrates**: a two-step chain where `extraction_chain`'s string output is bound to the `specifications` variable of the second prompt. `StrOutputParser()` normalizes the message object to a string so it can be interpolated. Install with `pip install langchain langchain-community langchain-openai langgraph`.

Structured hand-off between steps:
```json
{
  "trends": [
    {
      "trend_name": "AI-Powered Personalization",
      "supporting_data": "73% of consumers prefer brands that use personal information to make shopping experiences more relevant."
    },
    {
      "trend_name": "Sustainable and Ethical Brands",
      "supporting_data": "Sales of products with ESG-related claims grew 28% over five years, vs 20% for products without."
    }
  ]
}
```

## Reference Tables

| Use case | Chain shape |
|---|---|
| Information processing workflow | extract text → summarize → extract entities → query KB → generate report |
| Complex query answering | decompose into sub-questions → retrieve per sub-question → synthesize |
| Data extraction / OCR | extract fields → validate → conditional re-prompt for missing fields → normalize → delegate arithmetic to tool |
| Content generation | ideate 5 topics → select → outline → draft per section (with prior section as context) → revise for coherence |
| Conversational state | parse utterance (intent + entities) → update state → generate response / identify next needed info → repeat |
| Code generation | pseudocode → draft → static analysis / critique → refine → add docs and tests |
| Multimodal reasoning | extract image text → link text to labels → interpret via table |

| Framework | Best for |
|---|---|
| LangChain (LCEL) | linear sequences, foundational abstractions |
| LangGraph | stateful and cyclical computation, agentic loops |
| CrewAI / Google ADK | structured multi-step environments with control flow and state |

## Worked Example
**Market research report → marketing email**, the book's running example, decomposed:

1. *Initial Prompt (Summarization)* — "Summarize the key findings of the following market research report: [text]." Sole focus is summarization, so accuracy at this step is high.
2. *Second Prompt (Trend Identification)* — "Using the summary, identify the top three emerging trends and extract the specific data points that support each trend: [output from step 1]." Constrained, and builds on a validated output. Emit as the JSON above.
3. *Third Prompt (Email Composition)* — "Draft a concise email to the marketing team that outlines the following trends and their supporting data: [output from step 2]."

Contrast with the monolithic version — "analyze this report, summarize findings, identify trends with data points, and draft an email" — which typically produces a decent summary and then degrades: missing data points, a thin email, or both.

**Invoice/OCR variant with a validation gate:** extract fields → *check all required fields present and well-formatted* → if not, re-prompt specifically for the missing/malformed ones with context from the failed attempt → validate again → normalize ("one thousand and fifty" → 1050) → hand arithmetic to a calculator tool → return validated structured data.

## Key Takeaways
1. Prompt Chaining breaks complex tasks into a sequence of smaller, focused steps — occasionally called the Pipeline pattern.
2. Each step is an LLM call or processing logic consuming the previous step's output.
3. Reliability comes from focus per step plus **structured output** at every boundary; free-form hand-offs are the usual failure point.
4. The seams between calls are where you put deterministic logic: validation, conditional branching, tool calls, arithmetic.
5. Assigning each stage a distinct role improves per-step accuracy.
6. Context Engineering is the superset discipline — prompt phrasing is one layer among system prompt, RAG, tool output, memory, and implicit state.
7. LangChain/LangGraph, CrewAI and Google ADK provide the control flow, state, and component integration for these sequences.

## Connects To
- **Ch 3 (Parallelization)**: independent gathering fans out; chaining handles the dependent synthesis steps.
- **Ch 4 (Reflection)**: adds a critique/refine loop on top of a chain step.
- **Ch 5 (Tool Use)**: the deterministic work inserted between chain links.
- **Ch 8 (Memory Management)**: how conversational state accumulates across chained turns.
- **Ch 14 (Knowledge Retrieval / RAG)**: one of the context layers Context Engineering assembles.
- **Ch 22 (Advanced Prompting Techniques)**: structured output, system/role prompting, factored cognition.
