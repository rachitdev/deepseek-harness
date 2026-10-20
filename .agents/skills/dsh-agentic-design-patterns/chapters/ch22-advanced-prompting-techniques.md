# Chapter 22: Advanced Prompting Techniques

## Core Idea
Prompting is a disciplined engineering practice, not asking questions. Its purpose is to reduce the ambiguity inherent in natural language so a probabilistic model's output converges on a single correct intention — and, for agents, becomes reliable enough for automated pipelines.

## Frameworks Introduced
- **The five core prompting principles**:
  - **Clarity and Specificity** — unambiguous, precise instructions. Define the task, output format, and any limitations. Models interpret patterns; multiple interpretations produce unintended responses.
  - **Conciseness** — direct instructions. *What is confusing to the user is likely confusing to the model.* Avoid intricate language and superfluous information.
  - **Using Verbs** — action verbs indicate the expected operation and activate relevant training data. "Summarize the following text" beats "Think about summarizing this." Effective verbs: Act, Analyze, Categorize, Classify, Contrast, Compare, Create, Describe, Define, Evaluate, Extract, Find, Generate, Identify, List, Measure, Organize, Parse, Pick, Predict, Provide, Rank, Recommend, Return, Retrieve, Rewrite, Select, Show, Sort, Summarize, Translate, Write.
  - **Instructions Over Constraints** — positive instructions beat negative constraints. Excessive constraints make the model focus on avoidance rather than the objective. Constraints still have a place for safety and strict formatting.
  - **Experimentation and Iteration** — draft, test, analyze, identify shortcomings, refine. Model variations, configurations (temperature, top-p), and slight phrasing changes yield different results. Document attempts.
- **The three shot levels**:
  - **Zero-shot** — instruction plus input, no examples. Sufficient for tasks the model has seen extensively; the quickest thing to try first.
  - **One-shot** — one input/output example as a template. Useful when the output format or style is specific or uncommon.
  - **Few-shot** — typically 3–5 examples. Effective for specific formats, styles, or nuanced variation: classification, schema-constrained extraction, styled generation.
    - **Example quality and diversity** matter: examples must be accurate, representative, and cover variations and edge cases. Even a small mistake in an example can confuse the model.
    - **Mix up classes in classification examples** — randomize the order so the model learns each class's features independently rather than overfitting the sequence.
    - **Many-shot learning** — with long-context models, optimal performance for complex tasks may need hundreds of in-prompt examples.
- **The four structuring techniques**:
  - **System Prompting** — sets overall context and purpose for the interaction: rules, persona, general behavior, tone, style. Also used for safety and toxicity control.
  - **Role Prompting** — assigns a character/persona ("Act as a travel guide", "You are an expert data analyst"), pulling in that role's knowledge, tone, and communication style; the style within the role can be specified too.
  - **Using Delimiters** — triple backticks, XML tags (`<instruction>`, `<context>`), or markers (`---`) to visually and programmatically separate sections, minimizing misinterpretation.
  - **Structured Output** — request JSON, XML, CSV, or Markdown tables, optionally with a schema or example. *Returning JSON forces the model to create a structure and can limit hallucinations.*
- **Context Engineering** — dynamically providing background information, versus a static system prompt. Layers: system prompts; **external data** (retrieved documents via RAG, tool outputs); and **implicit data** (user identity, interaction history, environmental state). Core principle: output quality depends more on context richness than model architecture. Implicit context raises privacy and ethical data-management challenges, so robust governance is essential — especially in enterprise, healthcare, and finance.
- **The four reasoning techniques**:
  - **Chain of Thought (CoT)** — generate intermediate reasoning steps before the answer.
    - *Zero-Shot CoT*: append "Let's think step by step."
    - *Few-Shot CoT*: supply examples showing input, step-by-step reasoning, and output.
    - Advantages: low effort, works with off-the-shelf LLMs, increases interpretability, improves robustness across model versions. Disadvantage: longer output → higher token usage, cost, and latency.
    - Best practices: **place the final answer after the reasoning steps** (reasoning generation influences subsequent token predictions), and set **temperature to 0** for tasks with a single correct answer.
  - **Self-Consistency** — (1) generate diverse reasoning paths by running the same CoT prompt multiple times at higher temperature; (2) extract the answer from each; (3) majority-vote. Gives a pseudo-probability of correctness. Cost: multiple runs per query.
  - **Step-Back Prompting** — first ask the model for a general principle or concept related to the task, then use that response as context for the specific problem. Activates relevant background knowledge and broader reasoning strategies; less influenced by superficial elements; can mitigate bias.
  - **Tree of Thoughts (ToT)** — explore multiple reasoning paths concurrently in a tree of "thoughts", supporting exploration, backtracking, and recovery from initial errors.
- **The two action techniques**:
  - **Tool Use / Function Calling** — the model interprets tool descriptions, decides whether a tool is needed, and emits structured JSON naming the tool and parameters. **The model does not execute the tool**; an agentic system does and returns the result.
  - **ReAct** — the interleaved Thought → Action → Observation loop, continuing until a Final Answer.
- **Programmatic prompt optimization (DSPy)** — treat prompts as programmatic modules that can be automatically optimized. Requires two components: a **Goldset** (representative high-quality input/output pairs serving as ground truth) and an **Objective Function** (automatically scores output against the golden output). An optimizer (e.g. Bayesian) then applies two strategies:
  - **Few-Shot Example Optimization** — programmatically sample combinations of examples from the goldset to find the set that best guides the model.
  - **Instructional Prompt Optimization** — use an LLM as a meta-model to iteratively mutate and rephrase the instruction text, discovering which phrasing scores highest.

## Key Concepts
- **Automatic Prompt Engineering (APE)**: a meta-model generates candidate prompts from a task description; they're evaluated on sample inputs (BLEU, ROUGE, or human evaluation) and the best selected and refined.
- **Iterative Prompting/Refinement**: the human-driven loop — generic → add emphasis → add product name, specific claims, and target audience.
- **Providing Negative Examples**: showing an undesired output can clarify boundaries, used carefully against the instructions-over-constraints principle.
- **Using Analogies**: "Act as a data chef. Take the raw ingredients (data points) and prepare a summary dish (report) that highlights the key flavors (trends)."
- **Factored Cognition / Decomposition**: deliberately break a complex goal into sub-tasks prompted separately, then combine.
- **Persona Pattern (User Persona)**: describes the *audience* rather than the model's role, tailoring language, complexity, tone, and content.
- **Google Gems**: user-configurable specialized instances of Gemini with a persistent instruction set defining purpose, response style, and knowledge domain — removing the need to re-establish context each query.
- **Meta-prompting**: give an LLM your existing prompt, the task, and the disappointing output, and ask it to suggest improvements. Benefits: accelerated iteration, identification of blind spots, learning opportunity, scalability. Suggestions still need evaluation and testing.
- **"Parse, don't validate" at component boundaries**: encapsulating LLM output in a Pydantic object makes it safe to pass to other functions, APIs, and pipelines.

## Mental Models
- **Prompting converts probabilistic text generation into a deterministic, trustworthy cognitive engine** for an agent — that is the whole point of the structuring techniques.
- **Reasoning techniques are the agent's mind; action techniques are its hands; RAG and Context Engineering are its senses.**
- **Answer after reasoning, always.** Generating the reasoning is what conditions the answer tokens; putting the answer first wastes the whole technique.
- **The prompt is code.** Store it in version-controlled files, use variables instead of hardcoded values, and cover it with automated tests.

## Anti-patterns
- **Vague language and assumptions**: inadequate prompts yield ambiguous, inaccurate responses.
- **Over-constraining**: excessive "do not" instructions redirect the model to avoidance rather than the objective.
- **Poor or homogeneous few-shot examples**: a small mistake in an example propagates; lack of diversity prevents generalization.
- **Ordered class examples in classification**: invites overfitting to the sequence.
- **Answer before reasoning in CoT**: the reasoning no longer influences the answer.
- **Non-zero temperature on single-answer tasks with CoT**: unnecessary variance.
- **Unstructured output feeding a pipeline**: without structured data and programmatic validation, the agent's cognitive components cannot communicate reliably — catastrophic failures in automated workflows.
- **Hardcoding values in application prompts**: use variables so prompts are dynamic and reusable.
- **Not re-testing on model updates**: models change; prompts need revalidation.

## Code Examples
Pydantic as an object-oriented facade over LLM output:
```python
from pydantic import BaseModel, EmailStr, Field, ValidationError
from typing import List, Optional
from datetime import date

class User(BaseModel):
    name: str = Field(..., description="The full name of the user.")
    email: EmailStr = Field(..., description="The user's email address.")
    date_of_birth: Optional[date] = Field(None, description="The user's date of birth.")
    interests: List[str] = Field(default_factory=list,
                                 description="A list of the user's interests.")

llm_output_json = """
{
    "name": "Alice Wonderland",
    "email": "alice.w@example.com",
    "date_of_birth": "1995-07-21",
    "interests": ["Natural Language Processing", "Python Programming", "Gardening"]
}
"""

try:
    # Parses AND validates in a single step
    user_object = User.model_validate_json(llm_output_json)
    print(user_object.name, user_object.email)
    # Pydantic has already coerced 'date_of_birth' to a datetime.date
    print(type(user_object.date_of_birth))
except ValidationError as e:
    print("Failed to validate JSON from LLM.")
    print(e)
```
For XML, use `xmltodict` to convert to a dict, then feed it to a Pydantic model; `Field` aliases map verbose or attribute-heavy XML structures onto clean field names.

Few-shot classification with mixed classes:
```
Classify the sentiment of the following movie reviews as POSITIVE, NEUTRAL, or NEGATIVE:

Review: "The acting was superb and the story was engaging."
Sentiment: POSITIVE

Review: "It was okay, nothing special."
Sentiment: NEUTRAL

Review: "I found the plot confusing and the characters unlikable."
Sentiment: NEGATIVE

Review: "The visuals were stunning, but the dialogue was weak."
Sentiment:
```

Delimiters:
```
<instruction>Summarize the following article, focusing on the main arguments presented
by the author.</instruction>

<article>
[Insert the full text of the article here]
</article>
```

A ReAct trace:
```
User: What is the capital of France and its current population?

Thought: The user is asking for two pieces of information: the capital of France and its
  population. I can use a search tool to find this information. I will start by confirming
  the capital, although I am likely to know it.
Action: Search({"query": "capital of France"})
Observation: Paris
Thought: Okay, the capital is Paris. Now I need to find the population of Paris.
Action: Search({"query": "current population of Paris"})
Observation: The population of Paris was estimated to be around 2.1 million people in 2023.
Thought: I have found both pieces of information. I can now provide the final answer.
Final Answer: The capital of France is Paris, and its estimated population in 2023 was
  around 2.1 million people.
```

Function-call output shape:
```json
{
  "tool_name": "get_current_weather",
  "parameters": { "city": "London" }
}
```

## Reference Tables

| Technique | Examples given | Best for |
|---|---|---|
| Zero-shot | none | tasks well-represented in training; try first |
| One-shot | one | specific or uncommon output format/style |
| Few-shot | 3–5 | format adherence, classification, schema extraction, styled generation |
| Many-shot | up to hundreds | complex intricate patterns, long-context models |

| Structuring element | Purpose |
|---|---|
| System prompt | rules, persona, overall behavior, safety guidelines for the session |
| Role prompt | the model's assumed identity, expertise, tone |
| Persona pattern | the *audience* — tailors complexity, language, and content |
| Delimiters | separate instruction / context / examples / input |
| Structured output | machine-parseable result; limits hallucination |

| Reasoning technique | Mechanism | Cost |
|---|---|---|
| Zero-shot CoT | "Let's think step by step" | longer output |
| Few-shot CoT | examples with reasoning shown | longer prompt + output |
| Self-Consistency | N runs at high temperature, majority vote | N× compute |
| Step-Back | general principle first, then the specific task | 2 calls |
| Tree of Thoughts | branching exploration with backtracking | highest |

| Context Engineering layer | Examples |
|---|---|
| System prompts | "You are a technical writer; your tone must be formal and precise" |
| Retrieved documents | technical specifications pulled from a knowledge base |
| Tool outputs | calendar availability from an API |
| Implicit data | user identity, interaction history, environmental state |

| Best practice | Why |
|---|---|
| Provide examples | the single most effective guidance |
| Design with simplicity | confusing to you = confusing to the model |
| Be specific about output | format, length, style, content |
| Instructions over constraints | avoids avoidance-focused behavior |
| Control max token length | via config or explicit instruction |
| Use variables in prompts | dynamic, reusable, no hardcoding |
| Mix classes in few-shot classification | prevents order overfitting |
| Adapt to model updates | re-test on new versions |
| Experiment with output formats | JSON/XML for non-creative tasks |
| Collaborate with other prompt engineers | different perspectives |
| Document prompt attempts | structured record of prompts, configs, results |
| Save prompts in codebases | separate files, maintainable, version-controlled |
| Automated tests and evaluation | monitor performance and generalization in production |

## Worked Example
**The full spectrum on one task — extracting entities from news articles — and how each layer improves it.**

*Attempt 1, the naive prompt:* "Summarize the main points and list important names and places from this article: [text]". It sometimes misses entities and gets the main topic wrong. Every failure traces to a principle:

- **Clarity/specificity**: "important" is undefined, "names and places" omits organizations.
- **Verbs**: "list" is fine but "summarize the main points" conflates two tasks.
- **Structured output**: prose output cannot be parsed downstream.
- **Delimiters**: the article text runs into the instruction.

*Attempt 2, applying the layers:*
```
<instruction>
Extract the following from the article below and return a single JSON object with keys
"main_topic", "people", "organizations", and "locations".
- main_topic: one sentence naming the article's central subject.
- people / organizations / locations: arrays of proper nouns as they appear in the text.
Return an empty array for any category with no entries.
</instruction>

<article>
[article text]
</article>
```

*Attempt 3, the meta-approach:* rather than guessing further, hand the LLM the prompt itself:

> Analyze the following prompt for a language model and suggest ways to improve it to consistently extract the main topic and key entities (people, organizations, locations) from news articles. The current prompt sometimes misses entities or gets the main topic wrong.

The model can flag ambiguities you overlooked, suggest a persona, or recommend few-shot examples — accelerating iteration and teaching you what makes prompts effective.

*Attempt 4, programmatic optimization:* assemble a **goldset** of articles with correct extractions and an **objective function** scoring entity precision/recall. A DSPy-style optimizer then searches both which few-shot examples to include and how to phrase the instruction, maximizing that score. The prompt is now machine-optimized rather than hand-tuned.

Finally, the output is parsed into a Pydantic model at the boundary — so a malformed extraction raises a `ValidationError` in the pipeline instead of propagating silently into whatever consumes it.

## Key Takeaways
1. Prompting is engineering: clarity, conciseness, action verbs, positive instructions, and iteration are the foundation.
2. Zero-, one-, few-, and many-shot scale the amount of demonstrated behavior; example quality and diversity dominate few-shot success, and classification examples must be order-randomized.
3. System prompts, role prompts, delimiters, and structured output give architectural control over the interaction.
4. Context Engineering supersedes prompt phrasing: system prompt + retrieved documents + tool outputs + implicit data, with governance for the implicit layer.
5. CoT, Self-Consistency, Step-Back, and ToT externalize reasoning; answer after reasoning, temperature 0 for single-answer tasks.
6. Function calling emits a structured call the *system* executes; ReAct interleaves thought, action, and observation.
7. Structured output validated with Pydantic is a necessity for agent automation, not a convenience.
8. Prompts can be optimized by LLMs (APE, meta-prompting) or programmatically against a goldset and objective function (DSPy).
9. Treat prompts as code: variables, separate files, version control, documentation, and automated evaluation.

## Connects To
- **Ch 1 (Prompt Chaining)**: Context Engineering and structured hand-offs.
- **Ch 5 (Tool Use)**: function calling from the prompt side.
- **Ch 14 (Knowledge Retrieval / RAG)**: RAG as a prompting technique.
- **Ch 17 (Reasoning Techniques)**: CoT, ToT, Self-Consistency, ReAct in depth.
- **Ch 4 (Reflection)**: iterative refinement and meta-prompting.
- **Ch 19 (Evaluation and Monitoring)**: goldsets, objective functions, automated prompt tests.
- **Ch 18 (Guardrails)**: system prompts for safety and toxicity control.
