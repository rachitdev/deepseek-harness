# Chapter 16: Resource-Aware Optimization

## Core Idea
Agents should dynamically monitor and manage **computational, temporal, and financial** resources at runtime — deciding *how* to execute, not just *what* to execute — so that output quality is traded against cost, latency, and energy deliberately rather than by accident.

## Frameworks Introduced
- **Resource-Aware Optimization**: make execution decisions to achieve goals within resource budgets or to optimize efficiency. Distinct from planning, which sequences actions; this decides which model, which tool, and how much compute each step warrants.
  - When to use: strict financial budgets for API calls or compute; latency-sensitive applications; resource-constrained hardware (edge devices, limited battery); programmatically balancing quality against operational cost; multi-step workflows where tasks have varying resource requirements.
- **The Router + Critique architecture** — the chapter's core three-role design:
  - **Router Agent** — classifies incoming request complexity and forwards to the most suitable model or tool. Can route on a simple metric (query length) or use an LLM/ML model to analyze nuance. Improvable via **prompt tuning** (crafting the router's prompt) and **fine-tuning** on a dataset of queries and their optimal model choices.
  - **Answering Agents** — identical setup, different models and costs (e.g. Gemini Pro vs Gemini Flash).
  - **Critique Agent** — evaluates responses, serving four functions: **self-correction** (identify errors, prompt refinement), **performance monitoring** (track accuracy and relevance), **signal for RL or fine-tuning** (consistent inadequacy of Flash responses refines the router's logic), and **indirect budget management** (identifying suboptimal routing — simple queries sent to Pro, complex queries sent to Flash).
- **The fallback mechanism**: when a preferred model is unavailable due to overload or throttling, automatically switch to a default or more affordable model — **graceful degradation** maintaining service continuity instead of failing completely.
- **The nine additional optimization techniques** — beyond dynamic model switching:
  - **Adaptive Tool Use and Selection** — choose the most appropriate and efficient tool per sub-task, weighing API cost, latency, and execution time.
  - **Contextual Pruning and Summarization** — minimize prompt token count by summarizing and selectively retaining only the most relevant interaction history.
  - **Proactive Resource Prediction** — forecast future workloads to allocate ahead and prevent bottlenecks.
  - **Cost-Sensitive Exploration** — in multi-agent systems, count *communication* cost alongside computation when choosing collaboration strategies.
  - **Energy-Efficient Deployment** — minimize energy footprint to extend operational time and reduce running costs.
  - **Parallelization and Distributed Computing Awareness** — distribute workloads across machines for throughput.
  - **Learned Resource Allocation Policies** — adapt allocation strategies over time from feedback and performance metrics.
  - **Graceful Degradation and Fallback Mechanisms** — keep functioning at reduced capacity under severe constraint.
  - **Prioritization of Critical Tasks**.

## Key Concepts
- **The quality/resource trade-off**: the fundamental tension the pattern manages — better output costs more time and money.
- **Hierarchical model assignment**: the strong model does the reasoning, the cheap model does the repetitive tool calls.
- **Query classification categories** (OpenAI example): `simple` (direct factual, no reasoning or current events), `reasoning` (logic, math, multi-step inference), `internet_search` (current events, recent data, outside training data).
- **OpenRouter Automated Model Selection** (`"model": "openrouter/auto"`): routes to an optimized model from a curated set based on prompt content; the chosen model's identifier is returned in the response metadata.
- **OpenRouter Sequential Model Fallback** (`"models": [...]`): a hierarchical list; on failure (service unavailability, rate-limiting, content filtering) the request re-routes to the next model until one succeeds or the list is exhausted. Final cost and returned model identifier correspond to the model that succeeded.
- **LiteLLM**: integrates models not directly supported by ADK's default `Agent`.

## Mental Models
- **Pro for planning, Flash for fetching.** The travel-planner example: understanding a complex request and building a coherent multi-step itinerary needs deep reasoning; looking up flight prices, hotel availability, and restaurant reviews are simple repetitive web queries.
- **Route on complexity, not on length — unless you have to.** Query length is a workable first metric; an LLM router analyzing nuance is the real answer.
- **The critic pays for itself indirectly.** It doesn't manage the budget, but by catching mis-routes it improves resource allocation over time.
- **Degrade, don't fail.** A cheaper answer beats no answer when the preferred model is throttled.

## Anti-patterns
- **One model for everything**: either overpaying on simple queries or underperforming on complex ones.
- **No fallback on model unavailability**: overload or throttling becomes an outage.
- **Routing solely on query length**: a short question can be very hard ("Prove the Collatz conjecture"). Use it as a baseline, not a policy.
- **Unbounded context growth**: prompt token count is a direct cost line; prune and summarize.
- **Ignoring communication cost in multi-agent systems**: agent chatter is a resource too.
- **A router that never learns**: without critique feedback, mis-routing persists indefinitely.

## Code Examples
Two agents, same setup, different cost tier:
```python
from google.adk.agents import Agent
# from google.adk.models.lite_llm import LiteLlm   # for models ADK doesn't support directly

gemini_pro_agent = Agent(
    name="GeminiProAgent",
    model="gemini-2.5-pro",
    description="A highly capable agent for complex queries.",
    instruction="You are an expert assistant for complex problem-solving.")

gemini_flash_agent = Agent(
    name="GeminiFlashAgent",
    model="gemini-2.5-flash",
    description="A fast and efficient agent for simple queries.",
    instruction="You are a quick assistant for straightforward questions.")
```

A router agent (simple metric baseline):
```python
from google.adk.agents import BaseAgent
from google.adk.events import Event
from google.adk.agents.invocation_context import InvocationContext

class QueryRouterAgent(BaseAgent):
    name: str = "QueryRouter"
    description: str = "Routes user queries to the appropriate LLM agent based on complexity."

    async def _run_async_impl(self, context: InvocationContext) -> AsyncGenerator[Event, None]:
        user_query = context.current_message.text
        query_length = len(user_query.split())      # simple metric: number of words

        if query_length < 20:
            response = await gemini_flash_agent.run_async(context.current_message)
            yield Event(author=self.name, content=f"Flash Agent processed: {response}")
        else:
            response = await gemini_pro_agent.run_async(context.current_message)
            yield Event(author=self.name, content=f"Pro Agent processed: {response}")
```

The Critic Agent's system prompt:
```
You are the **Critic Agent**, serving as the quality assurance arm of our collaborative
research assistant system. Your primary function is to **meticulously review and challenge**
information from the Researcher Agent, guaranteeing **accuracy, completeness, and unbiased
presentation**.

Your duties encompass:
* **Assessing research findings** for factual correctness, thoroughness, and potential leanings.
* **Identifying any missing data** or inconsistencies in reasoning.
* **Raising critical questions** that could refine or expand the current understanding.
* **Offering constructive suggestions** for enhancement or exploring different angles.
* **Validating that the final output is comprehensive** and balanced.

All criticism must be constructive. Your goal is to fortify the research, not invalidate it.
Structure your feedback clearly, drawing attention to specific points for revision. Your
overarching aim is to ensure the final research product meets the highest possible quality
standards.
```
- A good critic prompt must: establish the role as evaluator, specify the areas for critical focus, emphasize constructive feedback over dismissal, encourage identifying both strengths and weaknesses, and guide how to structure the feedback.

OpenAI — three-way classification routing to three model tiers:
```python
# Step 1: classify
def classify_prompt(prompt: str) -> dict:
    system_message = {"role": "system", "content": (
        "You are a classifier that analyzes user prompts and returns one of three "
        "categories ONLY:\n\n- simple\n- reasoning\n- internet_search\n\n"
        "Rules:\n"
        "- Use 'simple' for direct factual questions that need no reasoning or current events.\n"
        "- Use 'reasoning' for logic, math, or multi-step inference questions.\n"
        "- Use 'internet_search' if the prompt refers to current events, recent data, or "
        "things not in your training data.\n\n"
        'Respond ONLY with JSON like:\n{ "classification": "simple" }')}
    response = client.chat.completions.create(
        model="gpt-4o", messages=[system_message, {"role": "user", "content": prompt}],
        temperature=1)
    return json.loads(response.choices[0].message.content)

# Step 3: pick the model tier from the classification
def generate_response(prompt: str, classification: str, search_results=None):
    if classification == "simple":
        model, full_prompt = "gpt-4o-mini", prompt
    elif classification == "reasoning":
        model, full_prompt = "o4-mini", prompt
    elif classification == "internet_search":
        model = "gpt-4o"
        search_context = "\n".join(
            f"Title: {i.get('title')}\nSnippet: {i.get('snippet')}\nLink: {i.get('link')}"
            for i in (search_results or [])) or "No search results found."
        full_prompt = f"""Use the following web results to answer the user query:
{search_context}
Query: {prompt}"""
    response = client.chat.completions.create(
        model=model, messages=[{"role": "user", "content": full_prompt}], temperature=1)
    return response.choices[0].message.content, model

# Step 4: orchestrate
def handle_prompt(prompt: str) -> dict:
    classification = classify_prompt(prompt)["classification"]
    search_results = google_search(prompt) if classification == "internet_search" else None
    answer, model = generate_response(prompt, classification, search_results)
    return {"classification": classification, "response": answer, "model": model}
```
- **What it demonstrates**: the classifier is forced to emit JSON from a three-value vocabulary, so the routing branch is mechanical. The classification does double duty — it picks the model tier *and* decides whether a web search runs first.

OpenRouter — a unified endpoint with two routing modes:
```python
import requests, json

response = requests.post(
    url="https://openrouter.ai/api/v1/chat/completions",
    headers={"Authorization": "Bearer <OPENROUTER_API_KEY>",
             "HTTP-Referer": "<YOUR_SITE_URL>",   # optional, for rankings
             "X-Title": "<YOUR_SITE_NAME>"},      # optional, for rankings
    data=json.dumps({
        "model": "openai/gpt-4o",
        "messages": [{"role": "user", "content": "What is the meaning of life?"}]}))
```
```json
// Automated model selection — router picks from a curated set by prompt content
{ "model": "openrouter/auto" }

// Sequential fallback — try in order until one succeeds
{ "models": ["anthropic/claude-3.5-sonnet", "gryphe/mythomax-l2-13b"] }
```

## Reference Tables

| Use case | Optimization |
|---|---|
| Cost-optimized LLM usage | large expensive model for complex, small affordable for simple, under a budget constraint |
| Latency-sensitive operations | faster, less comprehensive reasoning path for timely response |
| Energy efficiency | optimize processing on edge devices to conserve battery |
| Fallback for reliability | switch to a backup model when the primary is unavailable |
| Data usage management | summarized retrieval instead of full dataset downloads |
| Adaptive task allocation | agents self-assign by current load or available time |

| Classification | Model tier | Extra step |
|---|---|---|
| `simple` | cheapest (gpt-4o-mini) | none |
| `reasoning` | reasoning model (o4-mini) | none |
| `internet_search` | strong general model (gpt-4o) | Google Custom Search first, results as context |

| Router sophistication | Signal | Trade-off |
|---|---|---|
| Length threshold | word count | trivial to implement; wrong on short-hard queries |
| LLM router | prompt content and nuance | an extra call; improvable by prompt tuning |
| Fine-tuned router | learned from query→optimal-model pairs | needs a labeled dataset; best accuracy and efficiency |

| Technique | Resource saved |
|---|---|
| Dynamic model switching | cost, latency |
| Adaptive tool use and selection | API cost, latency, execution time |
| Contextual pruning and summarization | prompt tokens, inference cost |
| Proactive resource prediction | prevents bottlenecks |
| Cost-sensitive exploration | inter-agent communication cost |
| Energy-efficient deployment | battery, running cost |
| Parallelization / distributed awareness | wall-clock time |
| Learned allocation policies | improves all of the above over time |
| Graceful degradation and fallback | availability |

## Worked Example
**The hierarchical travel planner** — the clearest illustration of why one model per system is wrong.

A user asks for a week in Japan under a budget with specific interests. That request needs genuine reasoning: understand the constraints, decompose into a coherent multi-step itinerary, sequence cities sensibly, respect the budget. That is the **planner** agent, and it runs on **Gemini Pro** — a mis-sequenced itinerary is a bad product.

But once the plan exists, the work it generates is: look up flight prices, check hotel availability, find restaurant reviews. Dozens of calls, each a straightforward web query with a simple extraction. Running those on Pro multiplies cost with no quality gain — they go to **Gemini Flash**.

The financial-analyst variant makes the *temporal* dimension explicit: if the analyst needs a preliminary report immediately, use a fast affordable model to summarize key trends; if they need a highly accurate forecast for a critical investment decision and have budget and time, allocate a powerful, slower, more precise predictive model. **Same agent, same task type, different resource decision — driven by the caller's constraints, not the task's category.**

Layer the fallback on top: if Pro is throttled at the moment the plan is requested, the system does not fail — it degrades to a cheaper model and keeps serving.

## Key Takeaways
1. Agents can manage computational, temporal, and financial resources dynamically; model and execution-path decisions follow real-time constraints and objectives.
2. A multi-agent architecture (answering, routing, critique) makes this modular and scalable.
3. A Router Agent directs by complexity — Flash for simple, Pro for complex — optimizing cost and performance.
4. Route on nuance where possible; improve the router with prompt tuning and fine-tuning on query→model data.
5. A Critique Agent drives self-correction, performance monitoring, router refinement, and indirect budget management.
6. Assign models hierarchically: strong model for planning and reasoning, cheap model for repetitive tool calls.
7. Fallback mechanisms turn model unavailability into graceful degradation rather than outage.
8. Beyond model switching: adaptive tool selection, contextual pruning, proactive prediction, cost-sensitive exploration, energy efficiency, parallelization, learned allocation policies, and graceful degradation.

## Connects To
- **Ch 2 (Routing)**: this is routing with cost as the routing criterion.
- **Ch 4 (Reflection)** and **Ch 19 (Evaluation and Monitoring)**: the Critique Agent's feedback loop.
- **Ch 3 (Parallelization)**: distributed computing awareness as an optimization.
- **Ch 8 (Memory Management)**: contextual pruning and summarization operate on memory.
- **Ch 12 (Exception Handling and Recovery)**: fallback and graceful degradation are shared mechanisms.
- **Ch 20 (Prioritization)**: prioritizing critical tasks under constraint.
- **Ch 9 (Learning and Adaptation)**: learned resource allocation policies.
