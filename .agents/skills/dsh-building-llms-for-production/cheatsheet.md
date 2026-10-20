# Cheatsheet — Building LLMs for Production

## The three-lever decision

| Symptom | Lever | Chapter |
|---|---|---|
| Model can do the task but not reliably in the right shape | **Prompting** (+ output parsers) | 4, 6 |
| Model lacks knowledge, or the knowledge changes | **RAG** | 7, 8 |
| Model won't follow instructions, adopt a format, or match a domain tone | **Fine-tuning** | 10 |
| Specialized domain needing both current knowledge and a house voice | **RAG + fine-tuning** | 8 |

Axis to remember: prompting and RAG optimize **context** (what the model knows
now); fine-tuning optimizes the **LLM** (how it acts).

---

## Numbers and defaults

| Quantity | Value | Source |
|---|---|---|
| Near-real-time latency target | **100–200 ms/token** (human reads ~5 tokens/s) | Ch 11 |
| Memory estimate | `params × bytes_per_param / 1024³` — LLaMA 2 70B fp16 = **130 GB** | Ch 11 |
| LoRA trainable share (OPT-1.3B, r=16) | **3.1M / 1.32B = 0.24%** | Ch 10 |
| QLoRA reach | **65B params on a 48GB GPU**, bf16-equivalent quality | Ch 10 |
| AWQ salient weights | **0.1–1%** of channels | Ch 11 |
| SparseGPT | **≥50% one-shot**, 60% unstructured sparsity, OPT-175B in **<4.5 h** | Ch 11 |
| Lottery tickets | winning subnetworks at **10–20%** of full size | Ch 11 |
| Quantization speedup (Xeon, beam K=4) | **3.5×** (144.77 s → 40.73 s) | Ch 11 |
| Embedding-model fine-tune gain | **5–10%** retrieval metrics | Ch 8 |
| Deep Memory gain | **~25%** expected chunks returned (one metric only) | Ch 9 |
| Deep Memory minimum corpus | **~100 chunks** | Ch 9 |
| Golden Context Dataset | **177** curated query/source pairs | Ch 8 |
| LIMA | **1,000** curated samples may replace RLHF | Ch 10 |
| RLHF tutorial GPU cost | 8×A100 at **$8.80/hour** | Ch 10 |
| Assistants API parallel tools | up to **128** | Ch 9 |
| `text-embedding-ada-002` dims | **1536** | Ch 7 |
| Cohere `embed-multilingual-v2.0` dims | **768** (dot product) | Ch 7 |
| Whisper training data | **680,000 hours** multilingual | Ch 7 |
| Common chunk settings | 1000 chars / 0–100 overlap (book's tutorials) | Ch 7, 9 |
| Common retrieval settings | `similarity_top_k=10` → rerank `top_n=2`; or `k=3` | Ch 8 |

---

## Precision ladder

| Format | Bytes/weight | Flag |
|---|---|---|
| fp32 | 4 | default |
| bf16 | 2 | `bf16=True`, `torch_dtype=torch.bfloat16` |
| int8 | 1 | `load_in_8bit=True` |
| NF4 | 0.5 | `load_in_4bit=True, bnb_4bit_quant_type='nf4'` |

---

## Text splitter selection

| Constraint | Splitter |
|---|---|
| Default | `RecursiveCharacterTextSplitter` |
| Speed, structure irrelevant | `CharacterTextSplitter` |
| Exact model-token budgets | `TokenTextSplitter` (needs tiktoken) |
| Sentence integrity | `NLTKTextSplitter` / `SpacyTextSplitter` |
| Markdown documents | `MarkdownTextSplitter` |

## Chain type selection

| Situation | `chain_type` |
|---|---|
| Few short documents | `stuff` |
| Many documents, parallelizable | `map_reduce` |
| Maximum precision, sequential budget available | `refine` |

## Output parser selection

| Need | Parser |
|---|---|
| Flat list of strings | `CommaSeparatedListOutputParser` |
| Typed fields, validation, repair | `PydanticOutputParser` |
| Legacy text-only schema | `StructuredOutputParser` |
| Fix a mislabelled key | `OutputFixingParser` |
| Fix a missing key / wrong cardinality | `RetryWithErrorOutputParser` |

## Example selector selection

| Constraint | Selector |
|---|---|
| Token budget binds | `LengthBasedExampleSelector` |
| Relevance varies by query | `SemanticSimilarityExampleSelector` |
| <5 examples, always relevant | static `examples` list |

---

## Metrics by what you are measuring

**Retriever** — Hit Rate (relevant item in top-k) · MRR (best result first) ·
MAP (ranking precision across queries) · NDCG (relevance-weighted, normalized).

**Response** — Correctness (needs labels) · Faithfulness (grounded in context) ·
Context Relevancy · Guideline Adherence · Embedding Semantic Similarity (needs
labels) · SSA (sensible + specific).

**Ragas** — faithfulness · answer_relevancy · context_precision · context_recall ·
harmfulness.

**General LLM** — Perplexity · GLUE / SuperGLUE / BIG-bench / HELM / FLASK.

**Task-specific for quantization gates** — QA → F1 (squad) · summarization →
ROUGE · translation → BLEU · classification → accuracy.

### Reading metric gaps as diagnoses

| Pattern | Diagnosis | Fix |
|---|---|---|
| High recall, low precision | retriever drags in irrelevant material | rerank or contextual compression |
| High Hit Rate, low MRR | right chunk retrieved but not ranked first | rerank |
| High faithfulness, low answer relevancy | grounded but doesn't answer the question | query transformation, better prompt |
| Both low | retrieval is failing | chunking, embedding model, hybrid search |

---

## Tells and smells

- **Splitting model output on newlines** → you need an output parser.
- **`get_format_instructions()` hand-copied into the template** → make it a
  `partial_variables` entry.
- **`OutputFixingParser` invented content** → the key was missing, not
  misnamed; switch to `RetryWithErrorOutputParser`.
- **Agent answers from memory when a tool exists** → weak tool description, or
  the tool isn't registered.
- **Agent never stops** → no `max_iterations`, no `finish` command.
- **Task list growing faster than it's consumed** (BabyAGI) → bound the
  iterations.
- **`trainable%` is not ~0.2%** → LoRA isn't attached where you think.
- **Model repeats itself for exactly N tokens** → `min_length == max_length` on a
  non-instruction-tuned base; a benchmark artifact.
- **RAG answers a question the context doesn't cover** → short vague query
  defeated semantic search; the model fell back on parametric knowledge.
- **Quantized model quality dropped and you found out in production** → no
  `AccuracyCriterion` gate.
- **Streaming broke when you added sub-questions** → `SubQuestionQueryEngine` is
  incompatible with `streaming=True`.

---

## Cost and latency levers, cheapest first

1. **Prompt better** — few-shot, CoT; no infrastructure.
2. **Cache and template prompts** — no repeated engineering.
3. **Cheap embedding model + capable generation model** — asymmetric spend.
4. **Length-based example selection** — spend context only where it pays.
5. **Rerank instead of raising top-k blindly** — improves quality without
   enlarging the prompt.
6. **Dynamic quantization** — 2–3.5× on CPU, one command.
7. **CPU inference instead of GPU** — the largest single production saving.
8. **Pruning (Wanda / SparseGPT)** — no retraining required.
9. **Fine-tune a smaller model** — GPT-4-level results on GPT-3.5-class
   economics for one task.
10. **RLHF** — last, and only with large high-quality preference data.

---

## Framework roles at a glance

| Framework | Use it for |
|---|---|
| **LangChain** | chains, agents, memory, prompt/parsers, AutoGPT/BabyAGI/Plan-and-Execute |
| **LlamaIndex** | RAG depth — query engines, transformations, reranking, recursive & small-to-big retrieval, evaluation |
| **LangSmith** | debugging, tracing, monitoring, token/time metrics |
| **LangChain Hub** | prompt versioning and sharing |
| **Deep Lake** | multi-modal vector store, dataset streaming, versioning, Deep Memory |
| **TRL** | SFTTrainer, RewardTrainer, PPOTrainer |
| **PEFT** | LoRA, Prefix Tuning, P-tuning |
| **BitsAndBytes** | 4/8-bit loading |
| **Optimum + Intel Neural Compressor** | quantization, pruning, distillation with accuracy-driven tuning |
| **vLLM / TorchServe / FastAPI** | serving |
| **Ragas / DeepEval** | community RAG evaluation |
| **Weights & Biases** | run tracking, sweeps |

---

## Pipeline templates

**RAG**
```
load (TextLoader/PyPDFLoader/SeleniumURLLoader)
  → split (RecursiveCharacterTextSplitter)
  → embed (OpenAIEmbeddings) → store (DeepLake)
  → retrieve (as_retriever + rerank/compress)
  → generate (RetrievalQA, chain_type=stuff|map_reduce|refine)
  → evaluate (RetrieverEvaluator + BatchEvalRunner)
```

**Fine-tune**
```
curate data → format one template → LoRA/QLoRA config
  → SFTTrainer → check trainable% → train
  → merge with PeftModel → evaluate → (optionally) reward model → PPO
```

**Deploy**
```
task-fine-tuned model → pick metric → eval_fn
  → AccuracyCriterion + TuningCriterion → INCQuantizer.quantize()
  → benchmark with fixed token count → serve (FastAPI/vLLM) on CPU instance
  → shut down idle instances
```

**Agent**
```
define tools (+ descriptions) → pick Action vs Plan-and-Execute
  → attach memory (vector store / file tools)
  → set max_iterations + a finish command
  → verbose=True and interview the agent about its tools to debug
```
