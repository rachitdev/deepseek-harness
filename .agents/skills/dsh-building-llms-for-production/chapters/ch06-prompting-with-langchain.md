# Chapter VI — Prompting with LangChain

## Core Idea

A prompt in production is not a string you type; it is a **template plus a
contract**. LangChain splits the job in two: `PromptTemplate` /
`FewShotPromptTemplate` control what goes *into* the model (with example
selectors deciding dynamically how many demonstrations fit the context
window), and **Output Parsers** define a typed data structure the model must
come *out* with — plus repair parsers that fix violations of that structure.
Once input and output are both structured, an LLM step becomes a callable
function you can chain, and text can be transformed into machine-usable
artifacts such as knowledge graphs.

## Frameworks Introduced

### The prompt-template contract
A `PromptTemplate` needs exactly two things:

1. `input_variables` — list of placeholder names appearing in the template.
2. `template` — the formatted string containing `{placeholder}` slots.

```python
from langchain import LLMChain, PromptTemplate
from langchain.chat_models import ChatOpenAI

llm = ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0)

template = """Answer the question based on the context below. If the
question cannot be answered using the information provided, answer
with "I don't know".
Context: Quantum computing is an emerging field that leverages quantum
mechanics to solve complex problems faster than classical computers.

Question: {query}
Answer: """

prompt_template = PromptTemplate(input_variables=["query"], template=template)
chain = LLMChain(llm=llm, prompt=prompt_template)
chain.run({"query": "What is the main advantage of quantum computing?"})
```

Templates persist to disk and reload, so prompts can be versioned as assets
rather than buried in code:

```python
prompt_template.save("awesome_prompt.json")

from langchain.prompts import load_prompt
loaded_prompt = load_prompt("awesome_prompt.json")
```

### FewShotPromptTemplate anatomy
Six parameters, each with a distinct job:

| Parameter | Job |
|---|---|
| `examples` | static list of demonstration dicts |
| `example_selector` | *alternative* to `examples` — picks them dynamically |
| `example_prompt` | a `PromptTemplate` that renders one example |
| `prefix` | the instruction that precedes all examples |
| `suffix` | the live user input + output indicator |
| `input_variables` | placeholders appearing in `suffix` |
| `example_separator` | string between rendered examples (`"\n\n"`) |

```python
dynamic_prompt = FewShotPromptTemplate(
    examples=examples,
    example_prompt=example_prompt,
    prefix="Identify the habitat of the given animal",
    suffix="Animal: {input}\nHabitat:",
    input_variables=["input"],
    example_separator="\n\n",
)
```

### Three ways to do few-shot in LangChain

| Approach | Best for | Cost |
|---|---|---|
| Alternating Human/AI messages (`AIMessagePromptTemplate`) | chat applications; you literally write the bot's replies | simple, but locked to chat format |
| `FewShotPromptTemplate` with a static `examples` list | full control over example formatting | manual curation; inefficient at many examples |
| Example Selectors | dynamic, scale, context-window discipline | needs a selection strategy (and embeddings/vector store for semantic) |

### Two example selectors
- **`LengthBasedExampleSelector(examples, example_prompt, max_length=...)`** —
  includes fewer examples for long queries, more for short ones, keeping the
  rendered prompt under a token budget. Purely mechanical, no API cost.
- **`SemanticSimilarityExampleSelector.from_examples(examples, embeddings, db, k=1)`** —
  embeds examples into a vector store (Deep Lake in the book) and retrieves the
  *k* nearest to the live input. Supports `.add_example({...})` at runtime, so
  the demonstration pool grows without redeploying.

### The output-parser family

| Parser | Data types | Validation | Verdict |
|---|---|---|---|
| `PydanticOutputParser` | any Pydantic-declarable type (lists, nested, ints) | yes — `@validator` | the default choice; needs a strong model (GPT-4 Turbo class) to emit clean JSON |
| `CommaSeparatedListOutputParser` | flat list of strings | none | zero configuration, but cannot carry reasoning alongside items |
| `StructuredOutputParser` + `ResponseSchema` | text only | none | the oldest parser; "offers no particular advantage" |

### Repair parsers

| Class | Sees | Fixes | Fails on |
|---|---|---|---|
| `OutputFixingParser.from_llm(parser, llm)` | the malformed output only | wrong key names (`reasoning` → `reasons`) | *missing* keys — it invents content without knowing the intent |
| `RetryWithErrorOutputParser.from_llm(parser, llm)` + `.parse_with_prompt(output, model_input)` | output **and** original prompt | missing keys, structural mismatches | costs an extra call with full prompt tokens |

## Key Concepts

- **Dynamic vs static prompts** — three stated advantages of dynamic
  (example-bearing) prompts: *Enhanced Contextual Understanding* (the model
  infers desired style from demonstrations), *Flexibility* (structures can be
  swapped and A/B'd), *Better results* (higher alignment with expectations).
- **`input_variables` vs `partial_variables`** — the former are filled at call
  time via `.format_prompt()`; the latter are bound immediately at template
  construction. Format instructions from a parser are always a partial:
  `partial_variables={"format_instructions": parser.get_format_instructions()}`.
- **`.format()` vs `.format_prompt()`** — `.format()` returns a plain string
  (used with `CommaSeparatedListOutputParser`, no `.to_string()` needed);
  `.format_prompt()` returns a `PromptValue`, which is what
  `RetryWithErrorOutputParser.parse_with_prompt` requires as its second
  argument.
- **Validators do double duty** — a `@validator` can *reject* (raise
  `ValueError` on a word starting with a digit) or *repair* (append a missing
  period to each reason). Both run inside `parser.parse()`.
- **Knowledge Base vs Knowledge Graph** — a KB is any store of structured
  information; a KG is specifically a graph of entities and their relations.
  "Fabio lives in Italy" becomes the triple `<Fabio, lives in, Italy>`.
- **Relation Extraction = NER + Relation Classification**, optionally followed
  by *Entity Linking* (mapping surface forms to canonical entities) and
  *Source Tracking* (recording which document produced each triple).

## Mental Models

- **The parser is the type signature.** Writing the Pydantic class first forces
  you to decide what the LLM step actually returns before you decide how to ask
  for it. `get_format_instructions()` then derives the prompt text from the
  type — the schema is the single source of truth.
- **Example selection is a budget allocator.** Context window is a fixed
  resource; every demonstration bought must earn its tokens. Length-based
  selection spends by size, semantic selection spends by relevance.
- **Repair is an exception path, not a pipeline stage.** Running a fixing
  parser unconditionally doubles the API calls for every request. Wrap the
  happy path in `try`, put the repair in `except`.
- **Alternating Human/AI few-shot is "putting words in the bot's mouth."**
  You are not describing the desired style, you are supplying a prior turn where
  the assistant already exhibited it.

## Anti-patterns

- **Parsing model output by splitting on newlines.** The book's motivating
  example: ask for synonyms of "behavior" and you may get a numbered list, a
  bulleted list, or a preamble sentence. There is no stable format to split on.
- **Using a weak model with `PydanticOutputParser`.** JSON emission is a model
  capability; Davinci/Curie-class models will churn out unparseable output no
  matter how good the format instructions are.
- **Dumping every example into the prompt.** More tokens hurt if the examples
  are poorly chosen or misleading — the failure mode of unfiltered few-shot.
- **Reaching for `OutputFixingParser` on a missing field.** It cannot recover
  intent it never saw. The book's demonstration: given
  `'{"words": ["conduct", "manner"]}'` it produced *one* combined reason instead
  of one reason per word. `RetryWithErrorOutputParser` gets it right precisely
  because it re-reads the original prompt.
- **Choosing `StructuredOutputParser` for new work.** Text-only, no validation,
  superseded by both alternatives.

## Code Examples

### Pydantic parser with validation

```python
from langchain.output_parsers import PydanticOutputParser
from pydantic import BaseModel, Field, validator
from typing import List

class Suggestions(BaseModel):
    words: List[str] = Field(description="list of substitute words based on context")
    reasons: List[str] = Field(description="the reasoning of why this word fits the context")

    @validator('words')
    def not_start_with_number(cls, field):
        for item in field:
            if item[0].isnumeric():
                raise ValueError("The word can not start with numbers!")
        return field

    @validator('reasons')
    def end_with_dot(cls, field):
        for idx, item in enumerate(field):
            if item[-1] != ".":
                field[idx] += "."
        return field

parser = PydanticOutputParser(pydantic_object=Suggestions)
```

### Wiring format instructions into the prompt

```python
prompt_template = PromptTemplate(
    template=template,
    input_variables=["target_word", "context"],
    partial_variables={"format_instructions": parser.get_format_instructions()},
)
chain = LLMChain(llm=model, prompt=prompt_template)
output = chain.run({"target_word": target_word, "context": context})
parser.parse(output)
# Suggestions(words=['conduct', 'manner', 'demeanor', 'comportment'], reasons=[...])
```

### Production error handling

```python
try:
    result = parser.parse(output)
except OutputParserException:
    retry_parser = RetryWithErrorOutputParser.from_llm(parser=parser, llm=model)
    result = retry_parser.parse_with_prompt(output, model_input)
```

### Semantic example selection over a vector store

```python
from langchain.prompts.example_selector import SemanticSimilarityExampleSelector
from langchain.vectorstores import DeepLake
from langchain.embeddings import OpenAIEmbeddings

db = DeepLake(dataset_path=f"hub://{org_id}/langchain_course_fewshot_selector")
embeddings = OpenAIEmbeddings(model="text-embedding-ada-002")

example_selector = SemanticSimilarityExampleSelector.from_examples(
    examples, embeddings, db, k=1
)
similar_prompt = FewShotPromptTemplate(
    example_selector=example_selector,
    example_prompt=example_prompt,
    prefix="Convert the temperature from Celsius to Fahrenheit",
    suffix="Input: {temperature}\nOutput:",
    input_variables=["temperature"],
)
similar_prompt.example_selector.add_example({"input": "50°C", "output": "122°F"})
```

### Knowledge-graph triple extraction

```python
from langchain.indexes.graph import KNOWLEDGE_TRIPLE_EXTRACTION_PROMPT, KG_TRIPLE_DELIMITER
# few-shot prompt emits triples separated by "<|>"; text with no relations returns NONE

triples = parse_triples(response, delimiter=KG_TRIPLE_DELIMITER)

import networkx as nx
from pyvis.network import Network

G = nx.DiGraph()
for subj, pred, obj in triples:
    G.add_edge(subj, obj, label=pred)

net = Network(notebook=True)
net.from_nx(G)
net.toggle_hide_edges_on_drag(True)
net.toggle_physics(False)
net.set_edge_smooth('discrete')
net.show('knowledge_graph.html')
```

## Reference Tables

### Parser selection

| Need | Use |
|---|---|
| A list of strings, nothing more | `CommaSeparatedListOutputParser` |
| Multiple typed fields, or validation/repair of values | `PydanticOutputParser` |
| A single text answer with a named schema | `StructuredOutputParser` (legacy) |
| Recover from a mislabelled key | `OutputFixingParser` |
| Recover from a missing key or wrong cardinality | `RetryWithErrorOutputParser` |

### Example-selector selection

| Constraint | Selector |
|---|---|
| Token budget is the binding constraint | `LengthBasedExampleSelector(max_length=N)` |
| Example relevance varies sharply by query | `SemanticSimilarityExampleSelector` |
| Fewer than ~5 examples, all always relevant | static `examples` list, no selector |

## Worked Example — Bulleted news summarizer with a validated schema

The chapter upgrades the earlier News Article Summarizer so that it *cannot*
return prose. Pipeline: scrape → parse → few-shot prompt → typed output.

1. **Scrape** with `requests` using a browser `User-Agent` header, then extract
   `title` and `text` with `newspaper3k`'s `Article`.
2. **Few-shot prompt** containing two example article-summary pairs, so the
   model has seen the bulleted style before it sees the live article.
3. **Declare the output type**, including a validator that enforces the
   business rule "a summary must have at least three bullets":

```python
class ArticleSummary(BaseModel):
    title: str = Field(description="Title of the article")
    summary: List[str] = Field(description="Bulleted list summary of the article")

    @validator('summary', allow_reuse=True)
    def has_three_or_more_lines(cls, list_of_lines):
        if len(list_of_lines) < 3:
            raise ValueError("Generated summary has less than three bullet points!")
        return list_of_lines

parser = PydanticOutputParser(pydantic_object=ArticleSummary)
```

4. **Bind format instructions as a partial** so every call carries the schema:

```python
prompt_template = PromptTemplate(
    template=template,
    input_variables=["article_title", "article_text"],
    partial_variables={"format_instructions": parser.get_format_instructions()},
)
```

5. **Run and parse.** The output is an `ArticleSummary` object — `.title` and
   `.summary` are directly indexable, so rendering is a loop, not a regex.

Note `allow_reuse=True`: without it, re-executing the cell in a notebook raises
a duplicate-validator error. This is the kind of detail that only shows up when
the same schema is defined twice in one process.

## Key Takeaways

- Structure both ends of the LLM call: a template on the way in, a parser on
  the way out. Only then is the step composable.
- `PydanticOutputParser` is the default; the other two parsers are for trivial
  or legacy cases.
- Format instructions belong in `partial_variables`, derived from the schema
  via `get_format_instructions()` — never hand-written.
- Example selectors turn few-shot from a fixed cost into a managed budget;
  length-based for token discipline, semantic for relevance.
- Repair parsers differ by what they can see. `OutputFixingParser` sees the
  output; `RetryWithErrorOutputParser` sees output *and* prompt, which is why
  only the latter can restore missing structure.
- Put repair in an `except` block to keep API calls (and cost) proportional to
  actual failures.
- Prompt + parser is enough machinery to convert unstructured prose into a
  knowledge graph: extract triples with a few-shot prompt, build with NetworkX,
  visualize with pyvis.

## Connects To

- [Chapter V](ch05-langchain-and-llamaindex.md) — introduces the LangChain
  component model that this chapter puts to work.
- [Chapter IV](ch04-introduction-to-prompting.md) — the prompting techniques
  (few-shot, chain-prompting) that these templates operationalize.
- [Chapter VII](ch07-retrieval-augmented-generation.md) — the same Deep Lake
  vector store used here for example selection becomes the retrieval backbone.
- [Chapter IX](ch09-agents.md) — output parsers are how an agent's tool calls
  are made machine-readable.
