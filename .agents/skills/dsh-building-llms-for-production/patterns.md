# Patterns — Building LLMs for Production

Recurring, reusable moves from the book, each in **When / How / Trade-offs** form.

---

## Part 1 — Prompting patterns

### Structured prompt template
**When** any prompt sent more than once, or with variable content.
**How** `PromptTemplate(input_variables=[...], template="...{slot}...")`; save to
JSON/YAML and version it as an asset (`prompt_template.save()` / `load_prompt()`).
**Trade-offs** small upfront overhead; buys reproducibility and the ability to
diff prompt changes.

### Few-shot with a static example set
**When** the task has a consistent style the model must imitate; fewer than ~5
examples, all always relevant.
**How** `FewShotPromptTemplate(examples, example_prompt, prefix, suffix, input_variables, example_separator)`.
**Trade-offs** full formatting control; manual curation; inefficient at scale;
poorly chosen examples actively hurt.

### Few-shot via alternating Human/AI messages
**When** chat applications where you can write the assistant's prior turn.
**How** `SystemMessagePromptTemplate` + `HumanMessagePromptTemplate` +
`AIMessagePromptTemplate` + live human template, composed with
`ChatPromptTemplate.from_messages`.
**Trade-offs** natural in chat; locked to chat format.

### Length-budgeted example selection
**When** query length varies and the context window is the binding constraint.
**How** `LengthBasedExampleSelector(examples, example_prompt, max_length=N)`
passed as `example_selector` instead of `examples`.
**Trade-offs** free (no API calls); ignores relevance.

### Semantic example selection
**When** which examples help depends sharply on the query.
**How** `SemanticSimilarityExampleSelector.from_examples(examples, embeddings, db, k=1)`
over a vector store; grow the pool at runtime with `.add_example()`.
**Trade-offs** costs an embedding call per query and a vector store; adapts
without redeployment.

### Typed output contract
**When** the output feeds code rather than a human.
**How** declare a Pydantic `BaseModel` with `Field(description=...)` and
`@validator`s; wrap in `PydanticOutputParser`; inject
`parser.get_format_instructions()` as a `partial_variables` entry.
**Trade-offs** requires a strong model (GPT-4-Turbo class) to emit clean JSON;
eliminates all downstream string parsing.

### Repair-on-exception
**When** production, where parse failures are occasional rather than routine.
**How**
```python
try:
    result = parser.parse(output)
except OutputParserException:
    result = RetryWithErrorOutputParser.from_llm(parser=parser, llm=model) \
                 .parse_with_prompt(output, model_input)
```
**Trade-offs** keeps API cost proportional to actual failures; adds a code path
that must itself be tested. `OutputFixingParser` is cheaper but cannot recover
missing keys — only `RetryWithError...` sees the original prompt.

---

## Part 2 — RAG patterns

### Baseline RAG pipeline
**When** the model needs knowledge it was not trained on, or knowledge that changes.
**How** load → split → embed → store → retrieve → `RetrievalQA.from_chain_type(llm, chain_type, retriever)`.
**Trade-offs** more moving parts than prompting; buys citations and updatable
knowledge without retraining.

### Structure-preserving chunking
**When** always, unless you have measured that a simpler splitter is fine.
**How** `RecursiveCharacterTextSplitter(chunk_size, chunk_overlap, length_function)`
cascading `["\n\n","\n"," ",""]`; swap `length_function` for a token counter when
the model's limit is the constraint.
**Trade-offs** overlap duplicates tokens; the right chunk size is empirical —
clean the data, sweep sizes, evaluate with real queries.

### Retrieve wide, compress or rerank narrow
**When** retrieval brings back relevant *and* irrelevant text (i.e. always).
**How** either `ContextualCompressionRetriever(base_compressor=LLMChainExtractor.from_llm(llm), base_retriever=retriever)`
or `as_query_engine(similarity_top_k=10, node_postprocessors=[CohereRerank(top_n=2)])`.
**Trade-offs** an extra model call per query; lets the base retriever optimize
for recall while precision is recovered downstream.

### Query-side reshaping
**When** user queries are terse, jargon-heavy, or mix semantic and structured
conditions.
**How** *construction* (natural language → SQL/Cypher/metadata filter),
*expansion* (add synonyms), *transformation* (restructure for search).
**Trade-offs** each adds latency and a place to go wrong; text-to-SQL in
particular hallucinates tables — supply the real schema plus few-shot examples.

### Question decomposition
**When** the question has multiple parts ("before, during, and after").
**How** `SubQuestionQueryEngine.from_defaults(query_engine_tools, service_context, use_async=True)`.
**Trade-offs** one call per sub-question plus synthesis; incompatible with
streaming.

### Small-to-big retrieval
**When** the precise matching unit and the useful answering unit differ in size.
**How** `SentenceWindowNodeParser` (one sentence per node, ±5 sentence window)
plus `MetadataReplacementNodePostProcessor` to swap in the window at retrieval.
**Trade-offs** more nodes to store; resolves the chunk-size dilemma without
picking a compromise size.

### Source-partitioned tools
**When** the corpus spans materially different kinds of content.
**How** one query engine per source, each wrapped in a `QueryEngineTool` with a
`ToolMetadata` description that says what it covers, all handed to one agent.
**Trade-offs** more setup; prevents "uniformly sourcing information from both
data sources regardless of relevance."

### Two-level evaluation
**When** before any RAG change ships.
**How** retriever metrics (`RetrieverEvaluator` → MRR, Hit Rate) *and* response
metrics (`FaithfulnessEvaluator`, `RelevancyEvaluator`, Ragas), run in batch via
`BatchEvalRunner`, judged by a stronger model than the generator.
**Trade-offs** evaluation costs real API spend; without it you cannot attribute
a quality change to a code change. Read the gap between metrics as a diagnosis:
high recall + low precision → rerank; high Hit Rate + low MRR → rerank.

---

## Part 3 — Agent patterns

### ReAct tool loop
**When** the question needs current facts or computation.
**How** `initialize_agent(tools, llm, agent=AgentType.ZERO_SHOT_REACT_DESCRIPTION, verbose=True)`.
**Trade-offs** each hop is an LLM call; `verbose=True` is the only real
debugging surface.

### Descriptive tool registration
**When** any agent with more than one tool.
**How** write the `Tool`/`ToolMetadata` description as routing instructions
("Use a detailed plain text question as input to the tool"); for
`FunctionTool.from_defaults`, write real type annotations and a docstring — they
*become* the description.
**Trade-offs** none; a vague description is a silent routing bug.

### Tool retrieval
**When** the toolbox outgrows what fits comfortably in one prompt.
**How** `SimpleToolNodeMapping.from_objects(tools)` → `ObjectIndex.from_objects(...)`
→ `FnRetrieverOpenAIAgent.from_retriever(obj_index.as_retriever())`.
**Trade-offs** the agent semantic-searches its own tools; adds a retrieval step
before every action.

### Planner/Executor split
**When** long-horizon tasks where a single-step agent loses the thread.
**How** `PlanAndExecute(planner=load_chat_planner(model), executor=load_agent_executor(model, tools))`.
**Trade-offs** more calls and latency; the plan is inspectable and each step is
independently debuggable.

### Autonomous loop with file memory
**When** a task exceeds the context window and must persist intermediate work.
**How** give the agent `WriteFileTool` + `ReadFileTool`, a vector-store
retriever for episodic memory, and a system prompt that states the memory limit
and instructs immediate saving. Always set `max_iterations`.
**Trade-offs** files become the long-term memory; failure modes are loops and
premature "done".

### Enforced deliberation schema
**When** you want an agent to plan and self-criticize before acting.
**How** require a JSON response with `thoughts.{text, reasoning, plan,
criticism, speak}` and `command.{name, args}` — the schema forces reasoning
before the command field is reached. Include an explicit `finish` command and a
cost reminder.
**Trade-offs** more output tokens per step; substantially better step selection.

### Role-play multi-agent
**When** a task naturally splits into two expertises.
**How** CAMEL: assign an AI-user role and an AI-assistant role, add a task
specifier agent to concretize the goal, let them converse.
**Trade-offs** hallucination, conversation drift, role flipping, and termination
are all open problems.

### Three-factor memory retrieval
**When** an agent accumulates history over a long session.
**How** score memories by recency + relevance + importance
(`TimeWeightedVectorStoreRetriever`), with an importance score assigned at write
time and periodic reflection steps that generalize from observations.
**Trade-offs** more bookkeeping than a similarity search; behaves far more like
useful recall.

---

## Part 4 — Fine-tuning patterns

### Adapter fine-tuning
**When** you need behavior change and cannot afford full fine-tuning.
**How** `LoraConfig(r=16, lora_alpha=32, lora_dropout=0.05, bias="none", task_type="CAUSAL_LM")`
+ `SFTTrainer(..., peft_config=lora_config, packing=True)`; freeze the base,
cast 1-D params and the LM head to fp32, enable gradient checkpointing.
**Trade-offs** ~0.2% of parameters trained; verify with a
`print_trainable_parameters` helper before starting a long run.

### Quantized adapter fine-tuning (QLoRA)
**When** the model does not fit in GPU memory at 16-bit.
**How** load with `BitsAndBytesConfig(load_in_4bit=True, bnb_4bit_quant_type='nf4', bnb_4bit_use_double_quant=True, bnb_4bit_compute_dtype=torch.bfloat16)`,
then attach LoRA.
**Trade-offs** 65B on one 48GB GPU with no measured accuracy loss vs bf16.

### One template across all phases
**When** any multi-phase pipeline (SFT → reward → PPO).
**How** define one formatting function (`Question: …\n\nAnswer: …`) and reuse it
in the SFT dataset, the PPO dataset, and at inference.
**Trade-offs** none; skipping it introduces a silent distribution shift.

### Preference alignment
**When** demonstrations are not enough and you have ranked preference data.
**How** SFT → train a reward model on chosen/rejected pairs (DeBERTa-scale is
fine) → PPO with a KL penalty toward the SFT model.
**Trade-offs** expensive, unstable, sensitive to initialization and
hyperparameters; consider DPO (no reward model, no RL), ReST (offline, stable),
or RLAIF (constitution-guided AI feedback) instead.

### Reformat-then-tune
**When** fine-tuning a hosted model (Cohere, OpenAI) on an annotated corpus.
**How** the work is data engineering: convert the corpus into
`{"prompt": ..., "completion": ...}` JSONL, one file per sub-task; keep only the
information-dense fields (abstracts, not titles).
**Trade-offs** most of the effort is preprocessing, not training.

---

## Part 5 — Deployment patterns

### Accuracy-gated quantization
**When** shipping a quantized model.
**How** pick the task metric (F1/ROUGE/BLEU/accuracy), write an `eval_fn`, set
`AccuracyCriterion(tolerable_loss=0.01)` and `TuningCriterion(max_trials=10)`,
then `INCQuantizer.from_pretrained(model, eval_fn=eval_fn).quantize(...)`.
**Trade-offs** the search takes time; you learn about regressions before users do.

### Controlled-token benchmarking
**When** comparing two model variants' speed.
**How** fix `min_length == max_length` so both emit the same token count; report
per-decoding-method timings.
**Trade-offs** forces non-instruction-tuned models into repetition — an artifact
to note, not a result.

### CPU deployment
**When** the workload's latency budget tolerates it (and most do).
**How** dynamic quantization via `optimum-cli inc quantize` or `INCQuantizer`,
serve with FastAPI/vLLM/TorchServe on a Compute Engine or EC2 CPU instance.
**Trade-offs** measured 2.2× (greedy) to 3.5× (beam K=4) speedup on Xeon; shut
instances down when idle.

### Pruning without retraining
**When** model size is the constraint and a retraining budget does not exist.
**How** Wanda (weight magnitude × input activation, per output) or SparseGPT
(one-shot, ≥50% reduction, 60% unstructured sparsity).
**Trade-offs** unstructured sparsity needs hardware support to become speed;
structured pruning gives real gains on ordinary hardware at lower compression.
