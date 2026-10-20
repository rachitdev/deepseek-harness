# Chapter IV: Introduction to Prompting

## Core Idea
Prompting is the bridge between humans and AI — "a way to tell an AI agent what we want and how we want it using adapted human language." Prompt engineering is the discipline of translating an idea from conversational language into precise, optimized instructions, and it is inherently **iterative**.

## Frameworks Introduced
- **The seven reasons prompting matters**:
  1. Guides the model toward relevant output, coherent in context and in a specific format.
  2. Increases control and interpretability and reduces potential biases.
  3. Different models respond differently to the same prompt — the right prompt for the specific model generates precise results.
  4. Guides a hallucination-prone model in the right direction, e.g. by asking it to cite correct sources.
  5. Enables experimentation with diverse data types and presentation formats.
  6. Determines what good and bad outcomes look like by incorporating the goal into the prompt.
  7. Improves safety and helps defend against **prompt hacking** (users sending prompts to produce undesired behaviors).
- **The five prompting techniques**:
  1. **Zero-Shot** — no examples. Many tasks are well within LLM capabilities without guides. Caveat: without a description or example, the model chooses the style itself.
  2. **In-Context Learning / Few-Shot** — a small set of relevant examples in the prompt. The model learns from these and applies what it learned to similar questions. Adaptable across tasks with only a handful of samples.
  3. **Role Prompting** — instruct the LLM to assume a specific role or identity, providing context or perspective. The iterative loop: define the role → generate → evaluate and refine.
  4. **Chain Prompting** — link prompts sequentially, each output feeding the next. Steps: identify and extract relevant information from the response → develop a new prompt using it, building on the previous response → continue as necessary.
  5. **Chain of Thought (CoT)** — present examples showcasing the reasoning process, guiding the LLM to explain its logic. Beneficial for arithmetic, common-sense reasoning, and symbolic thinking.
- **The four properties of a good prompt** (from the song-title example):
  - **Precise Directions** — structured as a straightforward request, explicitly stating context ("As a futuristic robot band conductor").
  - **Specificity** — a particular theme and year, with input variables for versatility.
  - **Promoting Creativity** — no restriction on format or style where creativity is wanted.
  - **Concentrated on the Task** — one task only. *"Integrating multiple tasks in one prompt can confuse the model, potentially compromising its effectiveness in each task."*
- **The four structural elements of a well-built few-shot prompt** (from the life-coach example):
  - **Sets a Clear Context in the Prefix** — states the role and the kind of advice expected, framing all responses.
  - **Utilizes Examples** — illustrating role, tone, and style so responses stay consistent.
  - **Distinguishes Between Examples and the Actual Query** — clear separation so the AI knows the format and focuses on the current query.
  - **Includes a Clear Suffix** — a marker for end-of-user-input and start-of-AI-response, maintaining consistent format.
- **The three general rules**: be specific (sufficient context and detail), force conciseness when necessary, and **encourage the model to describe why it is the way it is** — which yields more precise solutions, particularly for complex tasks.

## Key Concepts
- **System prompt vs prompt**: the distinction is **specific to the OpenAI API**, which uses a system prompt to direct the model's behavior; other LLMs operate with a single standard prompt.
- **CoT's two limitations**: (1) it is effective primarily with models of **around 100 billion parameters or more** — smaller models produce nonsensical thought processes, *reducing* accuracy versus traditional prompting; (2) effectiveness varies by task type — strong for arithmetic, common sense, and symbolic reasoning, less meaningful elsewhere.
- **`PromptTemplate`**: simplifies prompts with dynamic inputs; central to chain prompting where a prompt depends on a previous response.
- **`FewShotPromptTemplate`**: composes `examples`, an `example_prompt`, a `prefix`, a `suffix`, `input_variables`, and an `example_separator` into one formatted prompt.
- **Prompt hacking**: sending prompts designed to produce undesired model behavior; good prompting is part of the defense.

## Mental Models
- **A prompt engineer is a translator** — from conversational intent into precise, optimized instruction.
- **One prompt, one task.** Combining tasks degrades performance on each.
- **Iterate; expect multiple adjustments.** "Prompting is inherently a process of refinement."
- **Vagueness is worse than wrong.** "Overly general prompts can lead to correct but irrelevant answers, and in some cases, vague prompts may even result in the generation of false information."

## Anti-patterns
- **The bare vague prompt**: `"Tell me something about {topic}."` — no context, no guidance, no meaningful response.
- **The open-ended follow-up in a chain**: `"Tell me something interesting about {scientist}."` produced correct facts about Einstein's vegetarianism, pacifism, and violin playing — factually true, entirely off-topic from general relativity.
- **The self-contradictory instruction**: `"Tell me something about {genre1}, {genre2}, and {genre3} without giving any specific details."` — asks for information while forbidding information. Ambiguity makes a coherent response impossible.
- **Unspecified criteria**: asking about "some musical genres" with no criteria, then about "uniqueness" with no guidance on which aspect — historical origins, stylistic elements, cultural impact.
- **CoT on a small model**: below ~100B parameters the reasoning is nonsensical and *hurts* accuracy.
- **Multiple tasks in one prompt**: dilutes effectiveness on all of them.

## Code Examples
Setup:
```python
import os
os.environ['OPENAI_API_KEY'] = "<OPENAI_API_KEY>"
```
```bash
pip install -q langchain==0.0.208 openai==0.27.8
```

Zero-shot with a system prompt:
```python
import openai

prompt_system = "You are a helpful assistant whose goal is to help write stories."
prompt = """Continue the following story. Write no more than 50 words.
Once upon a time, in a world where animals could speak, a courageous mouse
named Benjamin decided to"""

response = openai.ChatCompletion.create(
    model="gpt-3.5-turbo",
    messages=[{"role": "system", "content": prompt_system},
              {"role": "user",   "content": prompt}])
print(response.choices[0]['message']['content'])
```

Few-shot via alternating message roles:
```python
prompt_system = "You are a helpful assistant whose goal is to write short poems."
prompt = """Write a short poem about {topic}."""

examples = {
    "nature": "Birdsong fills the air,\nMountains high and valleys deep,\nNature's music sweet.",
    "winter": "Snow blankets the ground,\nSilence is the only sound,\nWinter's beauty found.",
}

response = openai.ChatCompletion.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system",    "content": prompt_system},
        {"role": "user",      "content": prompt.format(topic="nature")},
        {"role": "assistant", "content": examples["nature"]},
        {"role": "user",      "content": prompt.format(topic="winter")},
        {"role": "assistant", "content": examples["winter"]},
        {"role": "user",      "content": prompt.format(topic="summer")},
    ])
# Golden sun up high, / Laughter echoes in the sky, / Summer days fly by.
```
- **What it demonstrates**: the examples set not just the topic pattern but the *form* — the model infers haiku from two examples without being told.

Few-shot via `FewShotPromptTemplate`:
```python
from langchain import PromptTemplate, FewShotPromptTemplate, LLMChain
from langchain.chat_models import ChatOpenAI

llm = ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0)

examples = [
    {"color": "red",   "emotion": "passion"},
    {"color": "blue",  "emotion": "serenity"},
    {"color": "green", "emotion": "tranquility"},
]

example_prompt = PromptTemplate(
    input_variables=["color", "emotion"],
    template="Color: {color}\nEmotion: {emotion}\n")

few_shot_prompt = FewShotPromptTemplate(
    examples=examples,
    example_prompt=example_prompt,
    prefix="Here are some examples of colors and the emotions associated with them:\n\n",
    suffix="\n\nNow, given a new color, identify the emotion associated with it:"
           "\n\nColor: {input}\nEmotion:",
    input_variables=["input"],
    example_separator="\n")

formatted_prompt = few_shot_prompt.format(input="purple")
chain = LLMChain(llm=llm,
                 prompt=PromptTemplate(template=formatted_prompt, input_variables=[]))
chain.run({})    # -> "royalty or luxury"
```

Role prompting with variables:
```python
template = """
As a futuristic robot band conductor, I need you to help me come up with a song title.
What's a cool song title for a song about {theme} in the year {year}?
"""
prompt = PromptTemplate(input_variables=["theme", "year"], template=template)
chain = LLMChain(llm=llm, prompt=prompt)
chain.run({"theme": "interstellar travel", "year": "3030"})
# "Journey to the Stars: 3030"
```

Chain prompting — output of one becomes input of the next:
```python
# Prompt 1
template_question = """What is the name of the famous scientist who developed
the theory of general relativity?
Answer: """
prompt_question = PromptTemplate(template=template_question, input_variables=[])

# Prompt 2 — note how SPECIFIC it is
template_fact = """Provide a brief description of {scientist}'s theory of general relativity.
Answer: """
prompt_fact = PromptTemplate(input_variables=["scientist"], template=template_fact)

chain_question = LLMChain(llm=llm, prompt=prompt_question)
scientist = chain_question.run({}).strip()          # "Albert Einstein"

chain_fact = LLMChain(llm=llm, prompt=prompt_fact)
chain_fact.run({"scientist": scientist})
# "...gravitation that states that the gravitational force between two objects
#  results from the curvature of spacetime caused by the presence of mass and energy..."
```

The complete life-coach prompt — prefix, examples, separator, suffix:
```python
examples = [
    {"query": "What's the secret to happiness?",
     "answer": "Finding balance in life and learning to enjoy the small moments."},
    {"query": "How can I become more productive?",
     "answer": "Try prioritizing tasks, setting goals, and maintaining a healthy work-life balance."},
]

example_prompt = PromptTemplate(
    input_variables=["query", "answer"],
    template="\nUser: {query}\nAI: {answer}\n")

prefix = """The following are excerpts from conversations with an AI life coach.
The assistant provides insightful and practical advice to the users' questions.
Here are some examples:
"""
suffix = """
User: {query}
AI: """

few_shot_prompt_template = FewShotPromptTemplate(
    examples=examples, example_prompt=example_prompt,
    prefix=prefix, suffix=suffix,
    input_variables=["query"], example_separator="\n\n")

chain = LLMChain(llm=llm, prompt=few_shot_prompt_template)
chain.run({"query": "What are some tips for improving communication skills?"})
# "Practice active listening, be mindful of your body language, and be open to
#  constructive feedback."
```

## Reference Tables

| Technique | Examples supplied | Best for |
|---|---|---|
| Zero-shot | none | tasks well within model capability; quickest first try |
| Few-shot / in-context | a small set | complex tasks, specific format, tone, or style |
| Role prompting | none (a persona instead) | shaping perspective, expertise, voice |
| Chain prompting | — | multi-step tasks where each step depends on the last |
| Chain of Thought | reasoning traces | arithmetic, common sense, symbolic reasoning — on models ≥ ~100B |

| Good prompt property | What it does |
|---|---|
| Precise directions | states the request and the context explicitly |
| Specificity | narrows the task with concrete parameters |
| Promoting creativity | leaves format open where variety is wanted |
| Concentrated on the task | one task per prompt |

| Bad prompt | Why it fails |
|---|---|
| "Tell me something about {topic}." | no context or guidance |
| "Tell me something interesting about {scientist}." | correct but off-topic output |
| "…without giving any specific details." | self-contradictory; ambiguity blocks a coherent answer |
| "What are some musical genres?" then asking about "uniqueness" | no criteria specified at either step |

## Worked Example
**The same chain, twice — and why one specific word decides the outcome.**

Both versions ask the same first question: *"What is the name of the famous scientist who developed the theory of general relativity?"* Both extract "Albert Einstein". The chains diverge entirely at the second prompt.

*Version A (bad):* `"Tell me something interesting about {scientist}."`

> "Albert Einstein was a vegetarian and an advocate for animal rights. He was also a pacifist and a socialist, and he was a strong supporter of the civil rights movement. He was also a passionate violinist and a lover of sailing."

Every statement is true. Not one is about general relativity. The chain's *purpose* — establishing the scientist, then explaining the theory — has been lost, and nothing in the output signals that anything went wrong.

*Version B (good):* `"Provide a brief description of {scientist}'s theory of general relativity."`

> "…a theory of gravitation that states that the gravitational force between two objects results from the curvature of spacetime caused by the presence of mass and energy…"

The difference is that Version B **carries the chain's intent forward into the second prompt**. Version A relies on the model to remember why it was asked for the scientist's name — but the second prompt is a fresh call with no memory of the first's purpose, only its *answer*. The extracted value transfers; the intent does not, unless you write it in.

This is the chapter's practical lesson about chain prompting: extracting the right value is only half the job. Each link must restate enough context to stand on its own.

## Key Takeaways
1. Prompting bridges humans and AI; prompt engineering translates intent into precise, optimized instructions.
2. Prompting increases control and interpretability, reduces bias, guides against hallucination, and defends against prompt hacking.
3. Zero-shot suits capable-model tasks; few-shot supplies pattern, tone, and format; role prompting supplies perspective; chain prompting handles multi-step; CoT exposes reasoning.
4. CoT needs models around 100B+ parameters — below that it produces nonsense and reduces accuracy.
5. A good prompt has precise directions, specificity, appropriate creative latitude, and exactly one task.
6. A good few-shot prompt has a context-setting prefix, illustrative examples, clear separation from the query, and a suffix marking the response boundary.
7. In chain prompting, each link must carry the intent forward — extracting the value is not enough.
8. Self-contradictory or criteria-free instructions produce confused output; vague prompts can produce outright false information.
9. Prompting is iterative refinement; expect multiple adjustments.

## Connects To
- **Ch I (Introduction to LLMs)**: prompting as one of the four toolkit elements; few-shot as an emergent ability.
- **Ch III (LLMs in Practice)**: careful prompting as hallucination control; temperature and decoding as complements.
- **Ch V (LangChain & LlamaIndex)**: the frameworks used here, introduced properly.
- **Ch VI (Prompting with LangChain)**: `PromptTemplate`, few-shot templates, example selectors, and output parsers in depth.
- **Ch VII (RAG)**: chains and retrieval combined.
