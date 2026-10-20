# Chapter VIII — Advanced RAG

## Core Idea

Basic RAG retrieves top-k chunks by cosine similarity and hopes for the best.
Advanced RAG treats retrieval as an engineered subsystem: reshape the *query*
before searching (construction, expansion, transformation), reshape the
*results* after searching (reranking, small-to-big, recursive), and — critically —
**measure the retriever and the generator separately**, because a faithful
answer to the wrong context is still a wrong answer. The chapter also settles
the standing question of when to prompt, when to fine-tune, and when to retrieve.

## Frameworks Introduced

### Prompting vs. fine-tuning vs. RAG

The book's organizing axes: **prompt engineering and RAG optimize *context*
(what the model knows right now); fine-tuning optimizes the *LLM* (how the model
needs to act).**

| Approach | Optimizes | Best for | Costs / limits |
|---|---|---|---|
| Prompt engineering | context | simple or well-defined tasks; fastest path | ceilings out on complex tasks |
| Fine-tuning | model behavior | structure, tone, instruction-following — JSON extraction, text-to-SQL, house style | needs a large high-quality labeled dataset; poor at new/rapidly-changing data; poor at out-of-distribution queries; **not** the effective way to add knowledge |
| RAG | context | extensive or current information; evolving datasets | integration complexity (vector DB + retrieval algorithms), data management, retrieval accuracy is make-or-break |
| RAG + fine-tuning | both | specialized domains (medical, financial, legal) needing both a specific tone *and* external knowledge | resource-intensive: repeated training runs plus RAG data upkeep |

Also stated: combining **few-shot with RAG** — retrieving the most relevant
*examples* per query from a curated collection — works for complex tasks where
adapting the prompt alone is insufficient.

### LlamaIndex querying model

| Component | Role |
|---|---|
| **Retriever** | fetches nodes from an index for a query |
| **Query Engine** | processes a query and returns a response object; composes retrievers + response synthesizer |
| **Query Transform** | refines the raw query string before retrieval; works alongside a Retriever and Query Engine |
| **Chat Engine** | conversational variant — a series of queries and responses |

### The three query-side techniques

**Query Construction** — translate a natural-language question into the query
language of the target store. "Movies about aliens in the year 1980" splits into
a semantic part ("aliens" → vector search) and a structured part ("year = 1980"
→ metadata filter or SQL). Implementations: `MetadataFilter` classes with an
auto-retriever for vector stores; text-to-SQL for relational; Cypher for graph.
Text-to-SQL hazards: hallucinated tables/fields, and user misspellings —
mitigated by giving the LLM the accurate schema plus few-shot examples.

**Query Expansion** — add synonyms and related terms when the query is too
specific or too jargon-heavy. "Climate change effects" expands with "global
warming impact", "environmental consequences", "temperature rise implications".
`synonym_expand_policy` on `KnowledgeGraphRAGRetriever`; most effective paired
with Query Transform.

**Query Transformation** — restructure for search performance. "What were
Microsoft's revenues in 2021?" → "Microsoft revenues 2021".

### Retrieval strategies

| Strategy | Mechanism | Best for |
|---|---|---|
| `VectorIndexRetriever` | top-k most similar nodes | precision-critical, domain-specific inquiries |
| `SummaryIndexRetriever` | all nodes related to the query, unranked | broad exploratory searches, general overviews |
| **Sub Question Query Engine** | decomposes a question into sub-questions, answers each, synthesizes | multi-part questions ("before, during, and after") |
| **Recursive retrieval** | navigates a graph of interconnected nodes | hierarchical documents — PDFs with tables, diagrams, cross-references |
| **Small-to-big / Sentence Window** | match on a precise sentence, hand the LLM the surrounding window | layered relationships; query covers only part of the relevant content |
| **Reranking** | re-score and reorder retrieved results, discard low scorers | complex, domain-specific queries in embedding-based systems |

### Reranking mechanics
Documents are organized into batches; the model scores each batch for relevance
to the query; the highest-scoring documents across all batches form the final
response. Cohere's Rerank endpoint is **complementary to, not a replacement
for, a search engine** — it optimizes ordering. The canonical pattern:
retrieve wide (`similarity_top_k=10`), rerank narrow (`top_n=2`).

### The five RAG evaluation metrics

| Metric | Measures | Needs labels? |
|---|---|---|
| **Correctness** | generated answer vs. a reference answer | yes |
| **Faithfulness** | answer's integrity w.r.t. the retrieved context — no fabrication | no |
| **Context Relevancy** | relevance of retrieved context (and answer) to the query | no |
| **Guideline Adherence** | conformance to stylistic, factual, ethical criteria | no (guidelines) |
| **Embedding Semantic Similarity** | embedding-space closeness of generated vs. reference answer | yes |

Plus Google's **SSA** (Sensibleness and Specificity Average) for open-domain
chatbots: sensibleness = contextual coherence, specificity = detailed and direct
rather than vague. Originally human-evaluated.

### Retriever metrics

| Metric | What it rewards |
|---|---|
| **Hit Rate** | a relevant item appearing anywhere in the top results |
| **MRR** (Mean Reciprocal Rank) | putting the single most relevant result at the very top |
| **MAP** (Mean Average Precision) | ranking accuracy averaged across queries — precision after each relevant doc retrieved |
| **NDCG** (Normalized Discounted Cumulative Gain) | relevant documents appearing early; normalized so query sets compare, perfect = 1 |

## Key Concepts

- **Faithfulness ≠ relevance.** A response can mirror the retrieved context
  accurately and still score low on answer relevance — if it is incomplete,
  redundant, or does not address what was asked. High faithfulness does not
  imply high relevance.
- **Golden Context Dataset** — 177 hand-selected user queries paired with the
  single most appropriate source and an ideal expected answer, structured as
  'question' / 'source' pairs. The benchmark for precision evaluation.
- **Synthetic questions are a compromise.** The book is explicit: metrics tied
  to *authentic user experience* should not rest on LLM-generated questions.
  Manually author questions from the user's viewpoint when accuracy matters.
- **Evaluate components, then end-to-end.** Set baselines for chunking logic and
  embedding model, assess each part individually, then the whole — otherwise you
  cannot attribute a change in score to a change in the system.
- **Evaluate with a stronger model than you generate with.** GPT-4 as judge,
  GPT-3.5 as generator: the book's stated practice.
- **Fine-tuning the embedding model** yields a stated 5–10% improvement in
  retrieval metrics (LlamaIndex data). Process: generate synthetic Q&A from
  random documents → fine-tune → optionally evaluate.
- **Fine-tuning the LLM** can reach GPT-4-level performance on GPT-3.5-level
  economics for a specific dataset, and fixes output errors that prompt
  engineering struggles with.

### Four production challenges

1. **Document updates vs. stored vectors** — modifications, additions, and
   deletions must propagate into the vector store or retrieval serves stale data.
   Needs dynamic updating mechanisms.
2. **Chunking granularity** — too large omits essential detail; too small loses
   the larger context. Requires rigorous, data-specific testing.
3. **Diverse representations in latent space** — a paragraph, a table, and an
   image in one embedding space produce conflicts and inconsistent retrieval.
4. **Compliance** — regulated sectors, private documents with restricted access;
   failure means legal exposure, breaches, or misuse.

### Four optimization levers

- **Model selection** — cheap efficient embedding model, capable LLM for
  generation; proprietary (OpenAI, Cohere) or open-source (LLaMA 2, Mistral),
  self-hosted or API.
- **Hybrid retrieval** — keyword + embedding + reranking, balancing latency
  against quality.
- **CPU-based inference** — GPU inference is a major production cost; better
  hardware or optimized inference code compounds savings at scale.
- **Retrieval performance** — networks of interconnected nodes, key-term
  extraction, metadata enrichment, decoupling embeddings from raw text chunks,
  sentence-level focus.

## Mental Models

- **Two knobs, not one.** Context optimization (RAG, prompting) and LLM
  optimization (fine-tuning) are orthogonal axes, and most real systems move
  along both.
- **Retrieval is a funnel with two stages.** Cast wide with cheap semantic
  search, then narrow with an expensive but sharp reranker. Reranking is the
  cost-efficient upgrade because it improves an existing system without
  rebuilding it.
- **Small-to-big inverts the chunking dilemma.** Instead of picking one chunk
  size that must serve both matching and answering, match on the small unit and
  answer from the big one.
- **A metric without a golden dataset is a vibe.** Boolean pass/fail from a
  `FaithfulnessEvaluator` is only as meaningful as the queries fed to it.

## Anti-patterns

- **Fine-tuning to inject knowledge.** Stated plainly: it is not the effective
  method for integrating new information, and it adapts poorly to new or rapidly
  changing data.
- **Judging the system by response quality alone.** Retrieval and generation
  fail differently and must be measured separately (retriever metrics vs.
  response metrics).
- **Trusting a single-query evaluation.** Real applications need batch
  evaluation across many queries — `BatchEvalRunner` with `workers=8` exists for
  this reason.
- **Treating Rerank as a search engine replacement.** It reorders; it does not
  retrieve.
- **Streaming + Sub Question Query Engine.** Incompatible — the streaming flag
  must be deactivated.
- **Embedding-only retrieval for entity lookup.** Hybrid keyword + embedding
  search beats it; keywords supply specificity, embeddings supply context.

## Code Examples

### LlamaIndex index → query engine

```python
from llama_index import SimpleDirectoryReader, ServiceContext, VectorStoreIndex
from llama_index.vector_stores import DeepLakeVectorStore
from llama_index.storage.storage_context import StorageContext

documents = SimpleDirectoryReader("./paul_graham").load_data()

service_context = ServiceContext.from_defaults(chunk_size=512, chunk_overlap=64)
nodes = service_context.node_parser.get_nodes_from_documents(documents)

vector_store = DeepLakeVectorStore(
    dataset_path="hub://genai360/LlamaIndex_paulgraham_essays", overwrite=False)
storage_context = StorageContext.from_defaults(vector_store=vector_store)
storage_context.docstore.add_documents(nodes)
vector_index = VectorStoreIndex(nodes, storage_context=storage_context)

query_engine = vector_index.as_query_engine(streaming=True, similarity_top_k=10)
query_engine.query("What does Paul Graham do?").print_response_stream()
```

### Sub Question Query Engine

```python
from llama_index.tools import QueryEngineTool, ToolMetadata
from llama_index.query_engine import SubQuestionQueryEngine

query_engine = vector_index.as_query_engine(similarity_top_k=10)  # streaming OFF

tools = [QueryEngineTool(
    query_engine=query_engine,
    metadata=ToolMetadata(name="pg_essay",
                          description="Paul Graham essay on What I Worked On"),
)]

engine = SubQuestionQueryEngine.from_defaults(
    query_engine_tools=tools, service_context=service_context, use_async=True)

engine.query("How was Paul Graham's life different before, during, and after YC?")
# Generated 3 sub questions → answered individually → synthesized
```

The `ToolMetadata` description is not decoration: it is how the framework picks
the right tool when several are registered.

### Reranking, standalone and in a query engine

```python
import cohere, os
co = cohere.Client(os.environ['COHERE_API_KEY'])
results = co.rerank(query=query, documents=docs, top_n=3,
                    model='rerank-english-v2.0')
for idx, r in enumerate(results):
    print(idx + 1, r.index, f"{r.relevance_score:.2f}")
# Washington D.C. → 0.99; Northern Mariana Islands → 0.30; capital punishment → 0.27
```

```python
from llama_index.postprocessor.cohere_rerank import CohereRerank

cohere_rerank = CohereRerank(api_key=os.environ['COHERE_API_KEY'], top_n=2)
query_engine = vector_index.as_query_engine(
    similarity_top_k=10,                      # wide retrieval
    node_postprocessors=[cohere_rerank],      # narrow rerank
)
```

The score spread (0.99 vs 0.30) is the argument for reranking in one line:
lexical overlap on "capital" and "United States" ranked three near-useless
passages highly under pure similarity.

### Faithfulness evaluation

```python
from llama_index.evaluation import FaithfulnessEvaluator, RelevancyEvaluator

gpt4 = OpenAI(model="gpt-4-turbo", temperature=0.0)
service_context_gpt4 = ServiceContext.from_defaults(llm=gpt4)

faithfulness_gpt4 = FaithfulnessEvaluator(service_context=service_context_gpt4)
relevancy_gpt4 = RelevancyEvaluator(service_context=service_context_gpt4)

response = query_engine.query("What does Paul Graham do?")
faithfulness_gpt4.evaluate_response(response=response).passing   # -> True
relevancy_gpt4.evaluate_response(query=q, response=response).passing
```

### Batch evaluation

```python
from llama_index.evaluation import BatchEvalRunner

runner = BatchEvalRunner(
    {"faithfulness": faithfulness_gpt4, "relevancy": relevancy_gpt4},
    workers=8,
)
eval_results = await runner.aevaluate_queries(query_engine, queries=queries[:10])

faithfulness_score = sum(r.passing for r in eval_results['faithfulness']) \
                     / len(eval_results['faithfulness'])
```

### Ragas

```python
from ragas.metrics import faithfulness, answer_relevancy, context_precision, context_recall
from ragas.metrics.critique import harmfulness
from ragas.llama_index import evaluate

metrics = [faithfulness, answer_relevancy, context_precision, context_recall, harmfulness]
result = evaluate(query_engine, metrics, eval_questions, eval_answers)
# {'faithfulness': 0.8000, 'answer_relevancy': 0.7634,
#  'context_precision': 0.6000, 'context_recall': 0.8667, 'harmfulness': 0.0000}
```

Read that result as a diagnosis: recall 0.87 with precision 0.60 means the
retriever is *finding* the right context but dragging irrelevant material along —
exactly the condition reranking or contextual compression fixes.

## Reference Tables

### LangSmith and LangChain Hub

| Tool | Purpose |
|---|---|
| **LangChain Hub** | community prompt repository; prompt exploration, versioning, playground; public or org-private |
| **LangSmith** | evaluation and monitoring: metadata, token usage, execution time, response-pattern visualization, custom test environments, execution tracing |

LangChain is for prototyping; LangSmith is where debugging, testing, and
refinement happen.

```python
from langchain import hub
from langchain.prompts.chat import ChatPromptTemplate

prompt = ChatPromptTemplate.from_template("tell me a joke about {topic}")
hub.push(f"{handle}/rag", prompt)     # commit a version
```

### RAG best-practice checklist

| Practice | Payoff |
|---|---|
| Fine-tune the embedding model | 5–10% retrieval metric improvement |
| Fine-tune the LLM | fewer output errors; GPT-4-level results on cheaper models |
| Continuous evaluation | attribute performance changes to specific edits |
| Generative feedback loops | store generated descriptions/embeddings back into the DB to refine it over time |
| Hybrid search | keyword specificity + embedding context |
| Metadata enrichment + node relationships | structured retrieval, better context |

## Worked Example — Building a custom RAG evaluation pipeline

The chapter's most transferable artifact: a self-contained loop that produces
retriever metrics for a corpus you have never evaluated before.

1. **Load and node-parse** at a chosen chunk size:

```python
from llama_index.node_parser import SimpleNodeParser

docs = SimpleDirectoryReader(input_files=["venus_transmission.txt"]).load_data()
node_parser = SimpleNodeParser.from_defaults(chunk_size=512)
nodes = node_parser.get_nodes_from_documents(docs)
vector_index = VectorStoreIndex(nodes)
```

2. **Inspect what retrieval actually returned** — not just the answer:

```python
response_vector = vector_index.as_query_engine().query("What was the first beings…")
response_vector.source_nodes[0].get_text()
response_vector.source_nodes[1].get_text()
```

`source_nodes` is the debugging surface. If the answer is wrong, this tells you
whether retrieval or generation failed.

3. **Synthesize the eval dataset** — two questions per node:

```python
from llama_index.evaluation import generate_question_context_pairs

qa_dataset = generate_question_context_pairs(
    nodes, llm=OpenAI(model="gpt-3.5-turbo"), num_questions_per_chunk=2)
queries = list(qa_dataset.queries.values())
```

4. **Score the retriever**:

```python
from llama_index.evaluation import RetrieverEvaluator

retriever = vector_index.as_retriever(similarity_top_k=2)
retriever_evaluator = RetrieverEvaluator.from_metric_names(
    ["mrr", "hit_rate"], retriever=retriever)
eval_results = await retriever_evaluator.aevaluate_dataset(qa_dataset)
# OpenAI Embedding Retriever → Hit Rate 0.8846, MRR 0.7308
```

Read the gap: Hit Rate 0.88 with MRR 0.73 says the correct chunk is usually
*in* the top-2, but not always *first*. That gap is a reranking opportunity, and
this is the whole point of measuring the two separately.

5. **Then score the responses** with `BatchEvalRunner` (faithfulness, relevancy)
   using GPT-4 as judge, and — for anything user-facing — replace the synthetic
   questions with hand-written ones.

## Key Takeaways

- Prompting and RAG optimize context; fine-tuning optimizes behavior. Use RAG
  for knowledge, fine-tuning for form, both for specialized domains.
- Advanced RAG is query-side work (construction, expansion, transformation) plus
  result-side work (reranking, small-to-big, recursive retrieval).
- Retrieve wide, rerank narrow: `similarity_top_k=10` → `top_n=2` is the
  workhorse pattern, and reranking upgrades an existing system without rebuilding it.
- Measure retriever and generator separately. Hit Rate / MRR / MAP / NDCG for
  retrieval; correctness, faithfulness, context relevancy, guideline adherence,
  semantic similarity for generation.
- Faithfulness and relevance are independent — a grounded answer to the wrong
  question still fails.
- Use a stronger model to judge than to generate, batch your evaluations, and
  hand-write the questions that stand in for real users.
- Production concerns are index freshness, chunk granularity, multi-modal latent
  space, and compliance — none of which are solved by a better prompt.

## Connects To

- [Chapter VII](ch07-retrieval-augmented-generation.md) — the basic RAG pipeline
  this chapter optimizes, and the failure case it diagnoses.
- [Chapter X](ch10-fine-tuning.md) — the fine-tuning half of the
  prompt/fine-tune/RAG decision made here.
- [Chapter III](ch03-llms-in-practice.md) — general LLM evaluation benchmarks,
  of which RAG metrics are the retrieval-specific counterpart.
- [Chapter XI](ch11-deployment.md) — CPU inference, latency/quality balance, and
  the cost pressures behind the optimization section.
