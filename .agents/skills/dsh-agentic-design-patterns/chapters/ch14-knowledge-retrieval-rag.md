# Chapter 14: Knowledge Retrieval (RAG)

## Core Idea
RAG turns an LLM from a **closed-book reasoner into an open-book one**: before generating, the system semantically searches an external knowledge base, augments the prompt with the most relevant chunks, and lets the model answer grounded in verifiable — and citable — data.

## Frameworks Introduced
- **The RAG pipeline**: query → semantic search over an external knowledge base → extract the most pertinent chunks → **augment** the original prompt with them → send the enriched prompt to the LLM → grounded response.
  - When to use: the LLM must answer from specific, up-to-date, or proprietary information absent from its training data — Q&A over internal documents, support bots, anything needing verifiable fact-based responses with citations.
  - Benefits: access to current information; reduced hallucination through grounding; use of specialized internal knowledge; and **citations** pinpointing the exact source, which is what makes responses trustworthy and verifiable.
- **The five core concepts**:
  - **Embeddings** — numerical vector representations of text capturing semantic meaning. Similar meanings sit close in vector space: "cat" at (2, 3), "kitten" at (2.1, 3.1), "car" at (8, 1). Real embeddings use hundreds or thousands of dimensions.
  - **Text similarity** — how alike two texts are, lexically (word overlap) or by meaning. "What is the capital of France?" and "Which city is the capital of France?" share few words but score highly.
  - **Semantic similarity and distance** — purely meaning-based; high similarity means low distance. "A furry feline companion" and "a domestic cat" share only "a" yet sit close in vector space. This is the "smart search" that finds relevant information when wording doesn't match.
  - **Chunking** — breaking large documents into manageable pieces while preserving context. A 50-page manual becomes sections: "Troubleshooting" is a separate chunk from "Installation Guide", so a problem question retrieves only the troubleshooting chunk.
  - **Vector databases** — specialized stores for embeddings, using optimized algorithms like **HNSW (Hierarchical Navigable Small World)** to search millions of vectors fast. *While other techniques search for words, vector databases search for meaning.*
- **Retrieval techniques**: **vector search** (embeddings + semantic distance); **BM25** (older keyword-based algorithm ranking chunks by term frequency, no semantic understanding); **hybrid search** combining BM25's keyword precision with semantic search's contextual understanding — capturing both literal matches and conceptual relevance.
- **GraphRAG** — uses a knowledge graph instead of a vector database, answering complex queries by navigating explicit relationships (edges) between entities (nodes).
  - Strength: synthesizes answers from information fragmented across multiple documents — a common failing of traditional RAG.
  - Use cases: complex financial analysis connecting companies to market events; scientific research discovering gene-disease relationships.
  - Drawback: significant complexity, cost, and expertise to build and maintain a high-quality graph; less flexible; higher latency; effectiveness entirely dependent on graph quality and completeness.
- **Agentic RAG** — adds a reasoning and decision-making layer where an agent acts as **gatekeeper and refiner** of retrieved knowledge, actively interrogating its quality, relevance, and completeness. Four capabilities:
  1. **Reflection and source validation** — analyze metadata, recognize the authoritative source, discard the outdated one.
  2. **Reconciling knowledge conflicts** — identify contradictions between sources and prioritize the more reliable one.
  3. **Multi-step reasoning** — decompose a complex question into sub-queries, search each, synthesize a structured comparative context.
  4. **Identifying knowledge gaps and using external tools** — recognize the internal base has nothing relevant and activate a live web-search API.

## Key Concepts
- **Augmentation**: the step that distinguishes RAG — retrieved snippets are added to the prompt, not summarized separately.
- **`SIMILARITY_TOP_K`**: how many top similar chunks the retriever returns.
- **`VECTOR_DISTANCE_THRESHOLD`**: the maximum semantic distance allowed; more distant results are filtered out.
- **Chunk size / overlap**: `CharacterTextSplitter(chunk_size=500, chunk_overlap=50)` — overlap preserves context across boundaries.
- **Retriever**: the interface over a vector store (`vectorstore.as_retriever()`).
- **Vector DB landscape**: managed (Pinecone, Weaviate); open-source (Chroma DB, Milvus, Qdrant); existing databases augmented with vector search (Redis, Elasticsearch, Postgres via `pgvector`). Core retrieval mechanisms powered by libraries like Meta AI's **FAISS** or Google Research's **ScaNN**.

## Mental Models
- RAG is **consulting a book before answering** — the model looks things up rather than recalling.
- **Retrieval quality caps answer quality.** Irrelevant chunks introduce noise and confuse the LLM; the LLM cannot recover from a bad retrieval.
- **Naive RAG is a pipeline; Agentic RAG is a problem-solving framework.** The difference is a reasoning layer that can reject what it retrieved.
- Choose the retrieval substrate by **what the question needs**: vector search for conceptual similarity, BM25 for exact terms, hybrid for both, a graph when the answer lives in the *relationships* between facts.

## Anti-patterns
- **Feeding whole documents to the LLM**: expensive and unfocused; chunk strategically.
- **Chunking that severs context**: splitting mid-concept means neither chunk answers the question.
- **Assuming one chunk holds the answer**: RAG's primary failure is information **spread across multiple parts of a document or several documents** — the retriever gathers incomplete context and the answer is wrong.
- **Ignoring source authority and recency**: a 2020 blog post retrieved alongside the official 2025 policy, with no agent to prefer the latter.
- **Keyword-only search where meaning matters**: "furry feline companion" will not match "cat".
- **Treating the knowledge base as write-once**: it requires periodic reconciliation to stay current — critical for evolving sources like company wikis.
- **Underestimating the cost**: pre-processing and storing an entire knowledge base is a considerable undertaking, and RAG increases latency, operational cost, and prompt token count.
- **Adding an agentic layer reflexively**: the agent itself becomes a new error source — flawed reasoning can loop uselessly, misinterpret the task, or improperly discard relevant information.

## Code Examples
ADK — search as the simplest retriever:
```python
from google.adk.tools import google_search
from google.adk.agents import Agent

search_agent = Agent(
    name="research_assistant",
    model="gemini-2.0-flash-exp",
    instruction="You help users research topics. When asked, use the Google Search tool",
    tools=[google_search])
```

ADK — Vertex AI RAG Corpus with retrieval tuning:
```python
from google.adk.memory import VertexAiRagMemoryService

RAG_CORPUS_RESOURCE_NAME = "projects/<proj>/locations/us-central1/ragCorpora/<corpus-id>"
SIMILARITY_TOP_K = 5              # how many chunks to retrieve
VECTOR_DISTANCE_THRESHOLD = 0.7   # max semantic distance; further results filtered out

memory_service = VertexAiRagMemoryService(
    rag_corpus=RAG_CORPUS_RESOURCE_NAME,
    similarity_top_k=SIMILARITY_TOP_K,
    vector_distance_threshold=VECTOR_DISTANCE_THRESHOLD)
```

LangChain + LangGraph — a complete RAG pipeline as a two-node graph:
```python
from langchain_community.document_loaders import TextLoader
from langchain_community.embeddings import OpenAIEmbeddings
from langchain_community.vectorstores import Weaviate
from langchain.text_splitter import CharacterTextSplitter
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain_openai import ChatOpenAI
from langgraph.graph import StateGraph, END
from typing import List, TypedDict
import weaviate
from weaviate.embedded import EmbeddedOptions

# --- 1. Data preparation ---
documents = TextLoader('./state_of_the_union.txt').load()
chunks = CharacterTextSplitter(chunk_size=500, chunk_overlap=50).split_documents(documents)

client = weaviate.Client(embedded_options=EmbeddedOptions())
vectorstore = Weaviate.from_documents(client=client, documents=chunks,
                                      embedding=OpenAIEmbeddings(), by_text=False)
retriever = vectorstore.as_retriever()
llm = ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0)

# --- 2. Graph state ---
class RAGGraphState(TypedDict):
    question: str
    documents: List[Document]
    generation: str

# --- 3. Nodes ---
def retrieve_documents_node(state: RAGGraphState) -> RAGGraphState:
    """Retrieves documents based on the user's question."""
    question = state["question"]
    return {"documents": retriever.invoke(question), "question": question, "generation": ""}

def generate_response_node(state: RAGGraphState) -> RAGGraphState:
    """Generates a response using the LLM based on retrieved documents."""
    template = """You are an assistant for question-answering tasks.
Use the following pieces of retrieved context to answer the question.
If you don't know the answer, just say that you don't know.
Use three sentences maximum and keep the answer concise.
Question: {question}
Context: {context}
Answer:
"""
    context = "\n\n".join(doc.page_content for doc in state["documents"])
    rag_chain = ChatPromptTemplate.from_template(template) | llm | StrOutputParser()
    generation = rag_chain.invoke({"context": context, "question": state["question"]})
    return {"question": state["question"], "documents": state["documents"],
            "generation": generation}

# --- 4. Graph ---
workflow = StateGraph(RAGGraphState)
workflow.add_node("retrieve", retrieve_documents_node)
workflow.add_node("generate", generate_response_node)
workflow.set_entry_point("retrieve")
workflow.add_edge("retrieve", "generate")
workflow.add_edge("generate", END)
app = workflow.compile()

for s in app.stream({"question": "What did the president say about Justice Breyer"}):
    print(s)
```
- **What it demonstrates**: the prompt template carries three constraints worth copying — *use the retrieved context*, *say you don't know if you don't know*, and a length cap. The first grounds, the second is the anti-hallucination clause, the third keeps the answer usable.

## Reference Tables

| Retrieval technique | Matches on | Strength | Weakness |
|---|---|---|---|
| Vector search | embedding proximity | conceptual relevance, paraphrase-tolerant | can miss exact terms, IDs, codes |
| BM25 | term frequency | keyword precision, cheap | no semantic understanding |
| Hybrid | both | robust: literal + conceptual | more moving parts |
| Graph (GraphRAG) | explicit entity relationships | synthesizes across fragmented sources | build/maintain cost, latency, graph-quality dependent |

| | Naive RAG | Agentic RAG | GraphRAG |
|---|---|---|---|
| Retrieval | one vector search | agent picks tools, may search repeatedly | graph traversal over entities/edges |
| Validation | none | source authority, recency, conflict resolution | structural |
| Complex questions | single-shot | decomposed into sub-queries and synthesized | navigates relationships |
| Cost / latency | lowest | higher | highest |
| New failure mode | irrelevant chunks | agent loops, misinterprets, discards good data | incomplete graph |

| Application | What RAG retrieves |
|---|---|
| Enterprise search & Q&A | HR policies, technical manuals, product specs |
| Customer support / helpdesk | product manuals, FAQs, support tickets |
| Personalized recommendation | content semantically related to preferences and history |
| News summarization | recent articles from real-time feeds |

## Worked Example
**The four Agentic RAG scenarios** — each one a case where naive RAG returns something plausible and wrong.

1. *Source validation.* "What is our company's policy on remote work?" Naive RAG pulls a 2020 blog post **alongside** the official 2025 policy document and hands both to the LLM, which blends them. The agent inspects document metadata, recognizes the 2025 policy as the current authoritative source, and **discards the blog post before** the LLM sees it.

2. *Conflict reconciliation.* "What was Project Alpha's Q1 budget?" Retrieval returns an initial proposal saying €50,000 and a finalized financial report saying €65,000. Naive RAG passes both; the LLM picks one, hedges, or averages. The agent identifies the contradiction, prioritizes the financial report as more reliable, and supplies the verified figure.

3. *Multi-step reasoning.* "How do our product's features and pricing compare to Competitor X's?" A single similarity search cannot serve this — the answer needs four distinct retrievals. The agent decomposes it into our features, our pricing, Competitor X's features, Competitor X's pricing, runs each, and assembles a **structured comparative context** before generation.

4. *Knowledge gap + tool use.* "What was the market's immediate reaction to our product launched yesterday?" The internal base is updated weekly, so it holds nothing. Naive RAG retrieves the nearest-but-irrelevant chunks and the LLM answers from them. The agent recognizes the gap, activates a live web-search API, and answers from fresh news and social sentiment.

The through-line: in all four, the retrieval step *succeeded* by its own metric — it returned the nearest chunks. What was missing was a component willing to judge whether those chunks were the right answer.

## Key Takeaways
1. RAG lets LLMs access external, up-to-date, specific information; the process is Retrieval + Augmentation of the prompt.
2. It overcomes static training data, reduces hallucination, integrates domain-specific knowledge, and enables **attributable answers with citations**.
3. Embeddings, semantic similarity, chunking, and vector databases are the foundational machinery; vector DBs search meaning, keyword search searches words.
4. Combine BM25 and vector search (hybrid) for both literal precision and conceptual relevance.
5. RAG's chief failure is information fragmented across chunks or documents; retrieval quality caps answer quality.
6. GraphRAG navigates explicit relationships to synthesize across fragmented sources, at much higher build and maintenance cost.
7. Agentic RAG adds a reasoning layer that validates sources, reconciles conflicts, decomposes complex queries, and reaches for external tools when the base falls short.
8. Both advanced variants trade complexity, latency, and cost for depth and trustworthiness — and the agent itself can become a new error source.
9. The knowledge base needs ongoing reconciliation; it is infrastructure, not a one-time import.

## Connects To
- **Ch 5 (Tool Use)**: retrieval is a tool; Agentic RAG reaches for other tools on a knowledge gap.
- **Ch 8 (Memory Management)**: RAG is the retrieval mechanism behind semantic long-term memory.
- **Ch 2 (Routing)**: embedding-based routing reuses the same vector machinery.
- **Ch 4 (Reflection)**: Agentic RAG's source validation is reflection applied to retrieved context.
- **Ch 6 (Planning)**: Deep Research is iterative retrieval under a plan.
- **Ch 9 (Learning and Adaptation)**: Knowledge Base Learning agents store proven solutions in a RAG store.
- **Ch 19 (Evaluation and Monitoring)**: citations make grounded answers auditable.
