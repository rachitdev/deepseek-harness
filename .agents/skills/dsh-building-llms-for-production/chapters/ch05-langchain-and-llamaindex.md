# Chapter V: Introduction to LangChain & LlamaIndex

## Core Idea
Two frameworks with different centers of gravity: **LangChain** is for dynamic, context-rich interaction — modular, flexible, fast to prototype. **LlamaIndex** is a data framework — processing, structuring, and accessing private or domain-specific data for high-precision RAG. The selection question is *"do you require interactive agents or sophisticated search capabilities for information retrieval?"*

## Frameworks Introduced
- **LangChain's five key features**: abstractions and **LCEL** for composing chains; third-party integrations and partner packages; chains, agents, and retrieval strategies for cognitive architectures; **LangGraph** for robust stateful multi-actor applications; **LangServe** for deploying chains as REST APIs. Plus **LangSmith**, a developer platform for debugging, testing, evaluating, and monitoring.
- **The twelve LangChain components**:
  - **Prompts** — predefined recipes for generating prompts.
  - **Output Parsers** — classes that take LLM output and transform it into a more suitable format.
  - **Retrievers** — accept a string query, return a list of Documents.
  - **Document Loaders** — a `load` method producing Documents (text + metadata) from a configured source.
  - **Text Splitters** — divide a document into smaller chunks; built-in transformers can split, combine, and filter.
  - **Indexes** — data structures organizing and storing data for quick, efficient search.
  - **Embeddings models** — a standard interface across providers (OpenAI, Cohere, Hugging Face).
  - **Vector Stores** — store embedded data and perform vector search.
  - **Agents** — the decision-making components that decide the plan of action.
  - **Chains** — sequences of calls to an LLM, a tool, or a preprocessing step, integrating model, prompt, memory, output parsing, and debugging.
  - **Tool** — a specific function helping the model gather information: Google Search, database queries, Python REPL, other chains.
  - **Memory** — records past interactions, providing context for future ones.
  - **Callbacks** — hooks into stages of the application for logging, monitoring, and streaming.
- **The six LangChain agent types**:
  - **Zero-shot ReAct** — decides tool usage from **descriptions alone**, no usage examples needed.
  - **Structured Input ReAct** — handles tools requiring multiple inputs.
  - **OpenAI Functions Agent** — for function calls with fine-tuned models; works with gpt-3.5-turbo and gpt-4-turbo.
  - **Self-Ask with Search Agent** — factual answers via an "Intermediate Answer" tool.
  - **ReAct Document Store Agent** — combines "Search" and "Lookup" tools for a continuous thought process.
  - **Plan-and-Execute Agents** — formulate a multi-action plan carried out sequentially. Effective for complex or long-running tasks with steady focus on long-term goals; **trade-off: increased latency**.
- **LlamaIndex's five-stage pipeline**:
  1. **Data Connectors (Readers)** — parse and convert data into a Document format (text plus basic metadata), automating collection from APIs, PDFs, SQL databases. Hosted on the open-source **LlamaHub**.
  2. **Nodes** — documents transformed into smaller, more detailed units containing primary content, metadata, and contextual details, via a `NodeParser`.
  3. **Indices** — unstructured data transformed into embeddings capturing semantic meaning. **Summary Index** extracts a summary from each document and saves it with all its nodes — useful when matching small node embeddings to a query isn't straightforward. **Vector Store Index** generates embeddings at construction to identify the top-k most similar nodes.
  4. **Query Engines** — a pipeline combining a **Retriever** and a **Response Synthesizer**: retrieve nodes by query string, send them to the LLM to build a response.
  5. **Routers** — select the most suitable retriever/query engine for a task. Particularly advantageous with multiple data sources — e.g. deciding between a SQL database and a Vector Store for a given query.
- **OpenAI Assistants API's three features**: a **Code Interpreter** to write and run Python safely, **Knowledge Retrieval** to find information, and **Function Calling** to add custom functions or tools.

## Key Concepts
- **`ChatPromptTemplate`** with **`SystemMessagePromptTemplate`** (initial instructions, context, or data) and **`HumanMessagePromptTemplate`** (user messages the model answers).
- **`.to_messages()`**: converts a formatted chat prompt template into a list of message objects — structured management of the conversation so the model comprehends context and roles.
- **Deep Lake**: the book's vector store. Multi-cloud (S3, GCP, Azure, local, in-memory); native compression with lazy NumPy-like indexing; **dataset version control** with commits, branches, and checkouts; built-in dataloaders for PyTorch and TensorFlow; integrations with LangChain, LlamaIndex, Weights & Biases, MMDetection.
- **LlamaIndex defaults**: `gpt-3.5-turbo` for text generation and `text-embedding-ada-002` for embeddings.
- **`persist()`**: saves index data — nodes and embeddings — to disk (default `./storage`), avoiding repetitive processing.
- **`load_summarize_chain`**: a pre-built summarization chain; custom prompt templates can be supplied to tailor it.
- **Vector-store search across modalities**: text (word usage in a large collection), computer vision (visual features), recommendation systems (users and items from interactions and likes).

## Mental Models
- **LangChain talks; LlamaIndex indexes.** LangChain "can directly interact with LLMs and create chains without additional data"; LlamaIndex "mostly uses LLMs in the context of manipulating data — either for indexing or querying."
- **They are complementary, not exclusive.** "Combining elements of LangChain and LlamaIndex can be beneficial for leveraging their distinct strengths."
- **Documents → Nodes → Index → Query Engine** is the LlamaIndex spine; everything else is a variation on it.
- **Prompt as role assignment.** The summarizer works by assigning the model the role of "an advanced AI assistant" with a specific goal — the same mechanism that makes "in French, as a bulleted list" a one-line change.

## Anti-patterns
- **Asking a general QA chain factual questions without retrieval**: "Be mindful of the potential for hallucinations and instances where the models might generate information that is not factual."
- **Rebuilding an index on every run**: check `os.path.exists("./storage")` and load instead.
- **Assuming non-English output matches English quality**: "English is the primary training language for GPT-4. While it is multilingual, the performance quality for languages other than English may differ."
- **Plan-and-Execute agents for latency-sensitive work**: the trade-off is explicitly increased latency.
- **Choosing a framework before characterizing the use case**: interactive agents and sophisticated retrieval pull toward different tools.

## Code Examples
LangChain — chat prompt templates:
```python
from langchain.chat_models import ChatOpenAI
from langchain.prompts.chat import (ChatPromptTemplate,
                                    SystemMessagePromptTemplate,
                                    HumanMessagePromptTemplate)

chat = ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0)

system_message_prompt = SystemMessagePromptTemplate.from_template(
    "You are an assistant that helps users find information about movies.")
human_message_prompt = HumanMessagePromptTemplate.from_template(
    "Find information about the movie {movie_title}.")

chat_prompt = ChatPromptTemplate.from_messages([system_message_prompt,
                                                human_message_prompt])

response = chat(chat_prompt.format_prompt(movie_title="Inception").to_messages())
```

Summarization chain over a PDF:
```python
from langchain.chains.summarize import load_summarize_chain
from langchain.document_loaders import PyPDFLoader

llm = ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0)
summarize_chain = load_summarize_chain(llm)

document = PyPDFLoader(file_path="path/to/your/pdf/file.pdf").load()
summary = summarize_chain(document)
print(summary['output_text'])
```

QA chain with a custom template:
```python
from langchain.prompts import PromptTemplate
from langchain.chains import LLMChain

prompt = PromptTemplate(template="Question: {question}\nAnswer:",
                        input_variables=["question"])
chain = LLMChain(llm=ChatOpenAI(model_name="gpt-3.5-turbo", temperature=0),
                 prompt=prompt)
chain.run("what is the meaning of life?")
```

The News Articles Summarizer — scrape, then prompt:
```python
import requests
from newspaper import Article
from langchain.schema import HumanMessage
from langchain.chat_models import ChatOpenAI

headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) '
                         'AppleWebKit/537.36 (KHTML, like Gecko) '
                         'Chrome/89.0.4389.82 Safari/537.36'}

article_url = "https://www.artificialintelligence-news.com/..."
session = requests.Session()
response = session.get(article_url, headers=headers, timeout=10)

if response.status_code == 200:
    article = Article(article_url)
    article.download()
    article.parse()

template = """You are a very good assistant that summarizes online articles.

Here's the article you want to summarize.

==================
Title: {article_title}

{article_text}
==================

Write a summary of the previous article.
"""
prompt = template.format(article_title=article.title, article_text=article.text)

chat = ChatOpenAI(model_name="gpt-4-turbo", temperature=0)
summary = chat([HumanMessage(content=prompt)])
```
Changing the output shape is a **one-line prompt edit** — bulleted list, or bulleted list in French:
```python
template = """You are an advanced AI assistant that summarizes online articles
into bulleted lists in French.
...
Now, provide a summarized version of the article in a bulleted list format, in French.
"""
```

LlamaIndex — the full pipeline:
```python
# 1. Data connector
from llama_index import download_loader
WikipediaReader = download_loader("WikipediaReader")
documents = WikipediaReader().load_data(
    pages=['Natural Language Processing', 'Artificial Intelligence'])   # 2 documents

# 2. Nodes
from llama_index.node_parser import SimpleNodeParser
parser = SimpleNodeParser.from_defaults(chunk_size=512, chunk_overlap=20)
nodes = parser.get_nodes_from_documents(documents)                      # 48 nodes

# 3. Index, backed by Deep Lake
from llama_index.vector_stores import DeepLakeVectorStore
from llama_index.storage.storage_context import StorageContext
from llama_index import VectorStoreIndex

dataset_path = f"hub://{my_activeloop_org_id}/{my_activeloop_dataset_name}"
vector_store = DeepLakeVectorStore(dataset_path=dataset_path, overwrite=False)
storage_context = StorageContext.from_defaults(vector_store=vector_store)
index = VectorStoreIndex.from_documents(documents, storage_context=storage_context)
# tensors=['text', 'metadata', 'embedding', 'id']; embedding shape (23, 1536)

# 4. Query engine
query_engine = index.as_query_engine()
response = query_engine.query("What does NLP stand for?")
```

Persist and reload instead of rebuilding:
```python
import os.path
from llama_index import VectorStoreIndex, StorageContext, load_index_from_storage

if not os.path.exists("./storage"):
    documents = WikipediaReader().load_data(pages=[...])
    index = VectorStoreIndex.from_documents(documents)
    index.storage_context.persist()          # saves to ./storage by default
else:
    storage_context = StorageContext.from_defaults(persist_dir="./storage")
    index = load_index_from_storage(storage_context)
```

Logging during development:
```python
import logging, sys
logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
logging.getLogger().addHandler(logging.StreamHandler(stream=sys.stdout))
```

## Reference Tables

| | **LangChain** | **LlamaIndex** | **OpenAI Assistants** |
|---|---|---|---|
| **What is it?** | Interact with LLMs — modular and more flexible | Data framework for LLMs — empower RAG | Assistant API — SaaS |
| **Data** | standard formats (CSV, PDF, TXT); mostly focuses on Vector Stores | dedicated loaders (Discord, Slack, Notion…); efficient indexing and retrieval; adds new data points without recomputing all embeddings; improved chunking by linking chunks and using metadata; supports multimodality | 20 files, each up to 512 MB; wide range of file types |
| **LLM interaction** | prompt templates; very flexible chains and modules; multiple prompting, model, and output-parser options; can interact directly without data | mostly uses LLMs to manipulate data — indexing or querying | GPT-3.5 Turbo or GPT-4, plus any fine-tuned model |
| **Optimizations** | N/A | LLM fine-tuning; embedding fine-tuning | N/A |
| **Querying** | retriever functions | advanced techniques — subquestions, HyDE; routing across multiple data sources | threads and messages tracking user conversations |
| **Agents** | LangSmith | LlamaHub | code interpreter, knowledge retriever, custom function call |
| **Documentation** | easy to debug; easy to find concepts and function usage | as of Nov 2023 mostly tutorials and blog posts; harder to debug | great |
| **Pricing** | FREE | FREE | $0.03/code-interpreter session; $0.20/GB/assistant/day + usual LLM usage |

| LlamaIndex index type | Behavior |
|---|---|
| Summary Index | extracts a summary per document and saves it with all its nodes — helps when small node embeddings don't match the query well |
| Vector Store Index | generates embeddings at construction; retrieves top-k most similar nodes; scales with a high-performance vector database |

| Deep Lake feature | Benefit |
|---|---|
| Multi-cloud support | S3, GCP, Azure, local, in-memory |
| Native compression + lazy NumPy-like indexing | efficient slicing, indexing, iteration in native formats |
| Dataset version control | commits, branches, checkouts — collaboration and reproducibility |
| Built-in dataloaders | PyTorch and TensorFlow training at scale |
| Integrations | LangChain, LlamaIndex, Weights & Biases, MMDetection |

## Worked Example
**The News Articles Summarizer, and what changing one sentence does.**

The pipeline is six steps: install (`requests`, `newspaper3k`, `langchain`) → scrape the article with a custom User-Agent header → extract title and text with `newspaper` → preprocess → summarize with GPT-4 → output alongside the title.

The engineering is almost entirely in the last two steps, and they are almost entirely prompt work. The base template assigns a role ("a very good assistant that summarizes online articles"), delimits the content with `==================` fences, interpolates `{article_title}` and `{article_text}`, and states the task. On the Meta AI supercomputer article, it produces a dense paragraph.

Then, without touching a line of the scraping code:

- Change the role to *"an advanced AI assistant that summarizes online articles into bulleted lists"* and the task to *"provide a summarized version of the article in a bulleted list format"* → eight bullets covering RSC's speed claims, timeline, parameter targets, and privacy design.
- Add *"in French"* to both the role and the task → the same eight bullets, in French.

The lesson the chapter draws: "We are directing the model to generate the desired outcome using natural language instructions. This interaction is similar to asking a human assistant to execute a task." The counterweight it immediately adds: **English is GPT-4's primary training language, so quality in other languages may differ** — the French output is fluent, but that is not a guarantee you get for free.

Note also `temperature=0` throughout, "to ensure consistency and eliminate randomness" — a summarizer wants determinism, not creativity.

## Key Takeaways
1. LangChain simplifies development, productionization, and deployment of LLM applications through composable building blocks; the ecosystem adds LangGraph, LangServe, and LangSmith.
2. Its twelve components — prompts, output parsers, retrievers, loaders, splitters, indexes, embeddings, vector stores, agents, chains, tools, memory, callbacks — cover the whole lifecycle.
3. Six agent types differ in how they select tools and whether they plan ahead; Plan-and-Execute trades latency for long-horizon focus.
4. LlamaIndex's pipeline is Data Connectors → Nodes → Indices → Query Engines → Routers.
5. A Query Engine is a Retriever plus a Response Synthesizer; Routers pick the right engine across multiple data sources.
6. Deep Lake supports multi-cloud storage, dataset version control, native compression, and framework dataloaders.
7. Persist indexes to disk and reload rather than rebuilding.
8. LangChain excels at dynamic context-rich interaction and rapid prototyping; LlamaIndex at connecting LLMs to varied data sources with high precision; OpenAI Assistants offers a managed SaaS path with code interpreter, retrieval, and function calling.
9. The frameworks are complementary — combine them to use each one's strengths.
10. Output format, tone, and language are prompt changes, not code changes.

## Connects To
- **Ch IV (Introduction to Prompting)**: the prompting techniques these frameworks operationalize.
- **Ch VI (Prompting with LangChain)**: prompt templates, few-shot selectors, and output parsers in depth.
- **Ch VII (RAG)** and **Ch VIII (Advanced RAG)**: indexes, retrievers, chains, and evaluation built out.
- **Ch IX (Agents)**: the agent types listed here, implemented.
- **Ch III (LLMs in Practice)**: the hallucination warning motivating retrieval over plain QA chains.
