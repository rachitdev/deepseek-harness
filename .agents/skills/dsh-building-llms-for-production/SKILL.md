---
name: dsh-building-llms-for-production
description: "Knowledge base from \"Building LLMs for Production: Enhancing LLM Abilities and Reliability with Prompting, Fine-Tuning, and RAG\" by Louis-François Bouchard & Louie Peters (Towards AI, 2024). Use when building production LLM systems — prompt templates and output parsers, RAG with LangChain or LlamaIndex, chunking and reranking, RAG evaluation metrics, agents, LoRA/QLoRA/RLHF fine-tuning, or quantization, pruning and deployment."
---

<!-- argument-hint: [topic, technique, or chapter number] -->

# Building LLMs for Production
**Authors**: Louis-François Bouchard & Louie Peters (Towards AI) | **Pages**: ~467 | **Chapters**: 11 | **Generated**: 2026-08-22

## How to Use This Skill

- **Without arguments** — the Core Frameworks below are the working reference
- **With a topic** — ask about `chunking`, `reranking`, `LoRA`, `quantization`,
  `output parsers`, or anything in the Topic Index; I read the relevant chapter
- **With a chapter** — ask for `ch08`; I load that chapter file
- **Browse** — ask "what chapters do you have?" for the full index

When you ask about something not covered in Core Frameworks, I read the relevant
chapter file before answering.

---

## Core Frameworks & Mental Models

### The three levers, and the axis that separates them

**Prompting and RAG optimize *context* — what the model knows right now.
Fine-tuning optimizes the *LLM* — how the model needs to act.**

| Symptom | Lever |
|---|---|
| Capable but inconsistently shaped output | **Prompting** + output parsers |
| Missing or stale knowledge | **RAG** |
| Won't follow instructions, adopt a format, or match a domain voice | **Fine-tuning** |
| Specialized domain needing both | **RAG + fine-tuning** |

Fine-tuning is explicitly **not** the effective way to add knowledge: it adapts
poorly to new or rapidly changing data.

### The decision rules that matter most

- **Structure both ends of every LLM call.** A template on the way in, a parser
  on the way out. Only then is the step composable — and only then can it be
  tested.
- **The parser is the type signature.** Write the Pydantic class first;
  `get_format_instructions()` derives the prompt text from it, so the schema is
  the single source of truth. Never hand-write format instructions.
- **Repair belongs in an `except` block.** Running a fixing parser
  unconditionally doubles API calls for every request.
- **`OutputFixingParser` sees the output; `RetryWithErrorOutputParser` sees the
  output *and* the prompt.** That is why only the latter can restore a missing
  key rather than inventing one.
- **Bigger context windows do not obsolete RAG.** Feeding everything costs
  latency, *accuracy* when facts are scattered, and money on large corpora.
- **Chunk size has no default.** Clean the data, sweep sizes, run real queries,
  evaluate. Time-consuming and not skippable.
- **Recall at retrieval, precision at compression.** Retrieve wide
  (`similarity_top_k=10`), then rerank narrow (`top_n=2`) or compress with
  `LLMChainExtractor`. Reranking upgrades an existing system without rebuilding it.
- **Grounding reduces hallucination; it does not eliminate it.** Given a context
  about kernel features and asked "Is Linux free?", GPT-3 still answers from
  parametric memory. Short vague queries defeat semantic search — and "do not
  invent stuff" in the prompt is not an enforcement mechanism.
- **Measure retriever and generator separately.** They fail differently. Hit
  Rate 0.88 with MRR 0.73 means the right chunk is in the top-2 but not first —
  a reranking opportunity you cannot see from response quality alone.
- **Faithfulness ≠ relevance.** A response can mirror the context perfectly and
  still fail to answer the question.
- **Judge with a stronger model than you generate with.** GPT-4 evaluates,
  GPT-3.5 generates.
- **Synthetic eval questions are a compromise.** For metrics meant to reflect
  real user experience, hand-write the questions.
- **An agent is a reasoning engine, not a content generator.** As a reasoning
  engine it gathers, processes, and synthesizes — tools required, temperature 0.
  As a generator it draws on training patterns — no tools, higher temperature.
- **Tool descriptions are the routing table.** In LlamaIndex,
  `FunctionTool.from_defaults()` builds the description from type annotations
  and the docstring — those are functional, not cosmetic.
- **Debug an agent by interviewing it.** Ask what tools it has, what arguments
  they take, what those mean, and what each is for. The answers expose prompt
  flaws.
- **Bound every autonomous loop.** `max_iterations=None` means "may go on
  forever if stuck." Give the agent an explicit `finish` command.
- **Specialization beat generality.** AutoGPT's community pivoted from
  general-purpose to task-specific agents because breadth made it ineffective.
- **LoRA is a diff, not a rewrite.** Frozen base weights mean limited
  catastrophic forgetting, ~0.2% trainable parameters, and many adapters over
  one base. Verify the percentage before every long run.
- **One prompt template across every fine-tuning phase.** SFT, reward modeling,
  and PPO must all use the same `Question: …\n\nAnswer:` shape.
- **RLHF vs SFT is unresolved.** RLHF aligns better given large high-quality
  preference data, but is expensive and unstable — sensitive to initialization,
  prone to local optima, loss diverges and needs restarts. LIMA argues 1,000
  curated SFT samples can suffice.
- **Quantize against an explicit accuracy budget.** Declare a tolerable loss and
  a task-appropriate metric; let the tuner search. Otherwise you discover the
  regression in production.
- **CPU deployment is real.** Dynamic quantization gave 3.5× on beam search on a
  4-core Xeon. Target 100–200 ms/token — the number comes from human reading speed.

### Reading metric gaps as diagnoses

| Pattern | Diagnosis | Fix |
|---|---|---|
| High context recall, low context precision | irrelevant material dragged along | rerank / compress |
| High Hit Rate, low MRR | right chunk not ranked first | rerank |
| High faithfulness, low answer relevancy | grounded but off-question | query transformation |
| Both retrieval metrics low | retrieval itself failing | chunking, embedding model, hybrid search |

### Numbers worth remembering

| Quantity | Value |
|---|---|
| Latency target | 100–200 ms/token |
| Memory | `params × bytes / 1024³` — LLaMA 2 70B fp16 = 130 GB |
| LoRA trainable share | ~0.24% (OPT-1.3B, r=16) |
| QLoRA reach | 65B on a 48GB GPU, no measured accuracy loss vs bf16 |
| AWQ salient weights | 0.1–1% of channels |
| SparseGPT | ≥50% one-shot, no retraining, OPT-175B in <4.5 h |
| Quantization speedup | 2.2× greedy, 3.5× beam K=4 (Xeon, 4 cores) |
| Embedding fine-tune gain | 5–10% retrieval metrics |
| Deep Memory gain | ~25% expected chunks returned (one metric) |

---

## Chapter Index

| # | Chapter | Covers |
|---|---|---|
| I | [Introduction to LLMs](chapters/ch01-introduction-to-llms.md) | march of 9s, four-part toolkit, Chinchilla scaling, emergent abilities, context-window techniques, model timeline |
| II | [LLM Architectures and Landscape](chapters/ch02-llm-architectures-and-landscape.md) | encoder/decoder/encoder-decoder, Q/K/V attention, OPT-1.3B trace, LMM connection modules, proprietary vs open vs open-source |
| III | [LLMs in Practice](chapters/ch03-llms-in-practice.md) | four causes of hallucination, Constitutional AI, perplexity, GLUE/SuperGLUE/BIG-bench/HELM/FLASK, five decoding methods, temperature |
| IV | [Introduction to Prompting](chapters/ch04-introduction-to-prompting.md) | seven reasons prompting works, five techniques, four properties of a good prompt, bad-prompt practices, chain prompting |
| V | [LangChain & LlamaIndex](chapters/ch05-langchain-and-llamaindex.md) | LangChain's 12 components, 6 agent types, LlamaIndex's 5-stage pipeline, Deep Lake, comparison with OpenAI Assistants |
| VI | [Prompting with LangChain](chapters/ch06-prompting-with-langchain.md) | PromptTemplate, FewShotPromptTemplate, example selectors, output parsers, repair parsers, knowledge graphs |
| VII | [Retrieval-Augmented Generation](chapters/ch07-retrieval-augmented-generation.md) | indexes and retrievers, document loaders, six text splitters, embeddings, Deep Lake, chains, contextual compression |
| VIII | [Advanced RAG](chapters/ch08-advanced-rag.md) | prompting vs fine-tuning vs RAG, LlamaIndex query engines, query construction/expansion/transformation, reranking, small-to-big, RAG metrics, Ragas, LangSmith |
| IX | [Agents](chapters/ch09-agents.md) | ReAct loop, Action vs Plan-and-Execute, AutoGPT, BabyAGI, CAMEL, Generative Agents, LlamaIndex agents, OpenAI Assistants, Deep Memory |
| X | [Fine-Tuning](chapters/ch10-fine-tuning.md) | LoRA, QLoRA/NF4, SFT with TRL, Cohere medical NER, RLHF pipeline, DPO, ReST, RLAIF |
| XI | [Deployment](chapters/ch11-deployment.md) | latency and memory budgets, scalar/product quantization, LLM.int8()/GPTQ/AWQ, pruning taxonomy, Lottery Ticket, SparseGPT, Intel Neural Compressor, CPU deployment |

---

## Topic Index

**Prompting** — templates (Ch 6) · few-shot (Ch 4, 6) · chain-of-thought (Ch 4, 8) ·
example selectors (Ch 6) · chain prompting (Ch 4) · bad-prompt practices (Ch 4)

**Output structure** — PydanticOutputParser (Ch 6) · validators (Ch 6) ·
format instructions as partials (Ch 6) · OutputFixingParser vs
RetryWithErrorOutputParser (Ch 6) · knowledge-graph triples (Ch 6)

**RAG** — pipeline (Ch 7) · document loaders (Ch 7) · text splitters (Ch 7) ·
chunk size and overlap (Ch 7, 8) · embeddings: OpenAI / HuggingFace / Cohere (Ch 7) ·
Deep Lake (Ch 5, 7, 9) · chain types stuff/map_reduce/refine (Ch 7) ·
contextual compression (Ch 7) · reranking (Ch 8) · query construction, expansion,
transformation (Ch 8) · sub-question decomposition (Ch 8) · recursive and
small-to-big retrieval (Ch 8) · hybrid search (Ch 8)

**Evaluation** — retriever metrics MRR/Hit Rate/MAP/NDCG (Ch 8) · response metrics
(Ch 8) · Ragas (Ch 8) · FaithfulnessEvaluator / RelevancyEvaluator (Ch 8) ·
BatchEvalRunner (Ch 8) · Golden Context Dataset (Ch 8) ·
generate_question_context_pairs (Ch 8) · perplexity and benchmarks (Ch 3) ·
LangSmith (Ch 8)

**Agents** — ReAct (Ch 9) · Action vs Plan-and-Execute (Ch 9) · tools and
descriptions (Ch 9) · AutoGPT prompt anatomy (Ch 9) · BabyAGI sub-agents (Ch 9) ·
CAMEL role-play (Ch 9) · Generative Agents memory (Ch 9) · ObjectIndex tool
retrieval (Ch 9) · OpenAI Assistants API (Ch 5, 9) · Hugging Face Inference API (Ch 9) ·
Deep Memory (Ch 9)

**Fine-tuning** — instruction tuning (Ch 10) · LoRA config (Ch 10) · QLoRA / NF4
(Ch 10, 11) · PEFT and TRL (Ch 10) · ConstantLengthDataset (Ch 10) · merging
adapters (Ch 10) · reward models (Ch 10) · RLHF/PPO/KL penalty (Ch 10) · DPO,
ReST, RLAIF (Ch 10) · Cohere fine-tuning (Ch 10)

**Deployment** — latency budget (Ch 11) · memory formula (Ch 11) · scalar and
product quantization (Ch 11) · LLM.int8, GPTQ, AWQ (Ch 11) · static vs dynamic
(Ch 11) · Intel Neural Compressor (Ch 11) · pruning: unstructured, structured,
activation (Ch 11) · Wanda, SparseGPT, Lottery Ticket (Ch 11) · ONNX (Ch 11) ·
vLLM / TorchServe / FastAPI (Ch 11) · GCP and EC2 CPU deployment (Ch 11)

---

## Supporting Files

- **[glossary.md](glossary.md)** — ~170 terms, alphabetical, each tagged with its chapter
- **[patterns.md](patterns.md)** — reusable patterns in When / How / Trade-offs form, grouped by prompting, RAG, agents, fine-tuning, deployment
- **[cheatsheet.md](cheatsheet.md)** — decision tables, numbers, selection matrices, metric-gap diagnoses, tells and smells, cost levers, pipeline templates

---

## Scope & Limits

- Covers the book as published (Towards AI, 2024). Library versions are pinned to
  that era — `langchain==0.0.208`, `llama-index==0.9.14`, `openai==0.27.8` /
  `1.3.8`, `trl==0.5.0`/`0.6.0`. APIs have moved since; treat code as
  *architecture*, not copy-paste. Notably, LangChain moved experimental classes
  from `langchain.experimental` to `langchain_experimental` in August 2023.
- Model references (GPT-3.5-turbo, GPT-4-Turbo, OPT-1.3B, LLaMA 2) are the
  book's; the reasoning transfers, the model names do not.
- Notebooks referenced as "at towardsai.net/book" are not reproduced here —
  chapters carry the code inline.
- Some examples depend on paid services (OpenAI, Cohere, Activeloop, Lambda
  Labs GPUs at ~$8.80/h). Cost warnings from the book are preserved in the
  chapters.
