# Chapter III: LLMs in Practice

## Core Idea
Off-the-shelf foundation models are limited by **hallucination and bias** — structural consequences of learning word distributions rather than truth. Managing them in production means measuring performance properly (perplexity, GLUE/SuperGLUE/BIG-bench/HELM/FLASK) and controlling generation (decoding methods, temperature, penalties).

## Frameworks Introduced
- **The four causes of hallucination**:
  1. Training datasets lacking the knowledge needed to answer specific questions.
  2. No mechanism to verify factual accuracy of outputs.
  3. Training data containing fictional and subjective content — opinions, beliefs.
  4. **LLMs aren't designed to express uncertainty or admit a lack of information.** Faced with insufficient data, the model generates the most probable text sequence — and *"most probable" does not mean "true."* The primary aim is to learn a language's word distribution patterns, not to distinguish true from false statements.
- **The four hallucination-reduction strategies**: adjust text-generation parameters, improve training data quality, carefully craft prompts, and **employ retriever architectures** to anchor responses in specific documents.
- **The retriever architecture (RAG), in three steps**:
  1. On receiving a question, generate an embedding representation of it.
  2. Use that embedding for semantic search over a document database (comparing embeddings, computing similarity scores).
  3. The LLM uses top-ranked retrieved texts as context — **carefully extracting the answer from the context paragraphs and writing nothing that cannot be inferred from them.**
- **Constitutional AI** (Anthropic) — a two-stage bias-reduction framework: first the model is trained to evaluate and adjust its responses using a set of established principles and a limited number of examples; then reinforcement learning where the model uses **AI-generated feedback derived from those principles** to choose the most suitable response, reducing reliance on human feedback. Uses self-supervision so the AI adapts to its guiding principles without direct human oversight, plus constrained optimization guaranteeing helpfulness within the constitution's parameters.
- **Intrinsic vs extrinsic metrics**:
  - **Intrinsic** — directly related to the training objective. The canonical example is **perplexity**.
  - **Extrinsic** — performance across downstream tasks, unconnected to the training objective. GLUE, SuperGLUE, BIG-bench, HELM, FLASK.
- **The five decoding methods** — all balancing greediness against exploration:
  - **Greedy Search** — always the highest-probability token. Computationally efficient, but repetitive or suboptimal because it prioritizes the immediate token over long-run quality.
  - **Sampling** — random selection guided by the probability distribution. More varied and diverse, sometimes less coherent.
  - **Beam Search** — keep the top N candidates for a limited number of steps, then emit the beam with the highest overall joint probability. More coherent but slow, and can **miss high-probability words preceded by a lower-probability word**.
  - **Top-K Sampling** — restrict the pool to the K most probable words. Diversity with relevance and control.
  - **Top-p (Nucleus) Sampling** — the smallest group of tokens whose combined probability exceeds threshold P. Excludes rare tokens precisely; the challenge is the **unpredictably varying shortlist size**.
- **The four fine-tuning techniques**: Full Fine-Tuning (all parameters; effective but computationally demanding), **LoRA** (low-rank approximations on downstream layers — far fewer trained parameters, lower GPU memory and cost; **QLoRA** adds parameter quantization), **SFT** (supervised on demonstration data, prompts, and responses; can implement instruction tuning), and **RLHF** (incremental alignment to human feedback over iterations; related: DPO, RLAIF).

## Key Concepts
- **Objective/loss function vs evaluation metric**: the loss function is applied during training, must be **differentiable and smooth** for gradient descent. Evaluation metrics measure performance in human-understandable terms, are **not** used in training, and therefore need not be differentiable. LLMs use **cross-entropy loss** — causal language modeling is effectively a classification problem over the vocabulary.
- **Perplexity**: how much uncertainty the model has when assigning probabilities to word sequences. **Lower is better.** Computed as the reciprocal of the length-normalized (geometric-mean) sentence probability.
- **Temperature mechanics**: logits are divided by the temperature *before* softmax. **High (>1)** → logits shrink → more uniform softmax → more diverse and innovative output with more errors. **Low (<1)** → logits grow → more concentrated softmax → more accurate, conservative, less diverse. **=1** → no scaling, the underlying distribution preserved.
- **Frequency vs presence penalty**: the frequency penalty reduces the probability of reusing *repeatedly occurring* tokens; the presence penalty prevents repeating *any* token already present, regardless of frequency.
- **Stop sequences**: designated character sequences terminating generation, controlling output length and structure.
- **Instruction fine-tuning**: makes the model interpret input as an instruction rather than text to continue. A plain LLM given "What is the capital of France?" may respond "What is the capital of Italy?" — continuing a list of geography exercises. An instruction-tuned model answers "Paris."
- **Sources of bias**: "the data, the annotation process, the input representations, the models, and the research methodology" — not just the training corpus.

## Mental Models
- **The model optimizes plausibility, not truth.** The Dolly 2.0 example is exact: GPT-3.5 correctly knows Databricks is a data company and therefore generates a plausible-sounding data-migration tool — the hallucination is *coherent with everything it knows*, which is why it is convincing.
- **Perplexity is "how surprised was the model."** Better model → higher assigned probabilities → lower perplexity.
- **Temperature is a diversity/correctness knob**, and it operates on logits before softmax, not on the output.
- **Greedy is locally optimal, globally not.** A slightly lower-probability token now can unlock much higher-probability tokens later.
- **Fine-tuning teaches behavior; RAG supplies knowledge.** Instruction tuning in particular is about *how to interpret the prompt*, not what facts to know.

## Anti-patterns
- **Expecting an LLM to say "I don't know"**: it isn't designed to express uncertainty; it will produce the most probable continuation instead.
- **Accepting confident output as correct**: hallucinations are convincing precisely because they are internally consistent.
- **Using accuracy as the only evaluation metric**: HELM exists because that is insufficient — calibration, robustness, fairness, bias, toxicity, and efficiency all matter.
- **Greedy decoding for open-ended generation**: repetitive and suboptimal.
- **High temperature for factual tasks**: diversity at the cost of correctness.
- **Comparing raw sentence probabilities across different lengths**: longer sentences always score lower; normalize by the geometric mean.
- **Full fine-tuning when LoRA would do**: full fine-tuning demands considerable computational power for often marginal gain.
- **Treating bias as a data-only problem**: it also enters through annotation, input representations, model design, and research methodology.

## Code Examples
Perplexity, computed directly:
```python
import numpy as np

# "A red fox."  -> P(a) * P(red|a) * P(fox|a red) * P(.|a red fox)
probabilities = np.array([0.4, 0.27, 0.55, 0.79])
sentence_probability = probabilities.prod()                              # 0.0469
sentence_probability_normalized = sentence_probability ** (1 / len(probabilities))  # 0.465
perplexity = 1 / sentence_probability_normalized
print(perplexity)   # 2.1485556947850033

# After further training, next-word probabilities rise -> perplexity falls
probabilities = np.array([0.7, 0.5, 0.6, 0.9])
sentence_probability = probabilities.prod()
sentence_probability_normalized = sentence_probability ** (1 / len(probabilities))
print(1 / sentence_probability_normalized)   # 1.516647134682679  -> lower, better
```
The formulas:
```
P(W)       = product of per-token conditional probabilities
Pnorm(W)   = P(W) ^ (1/n)        # geometric mean, length-normalized
PP(W)      = 1 / Pnorm(W) = (1 / P(W)) ^ (1/n)
```

## Reference Tables

| Decoding method | Mechanism | Strength | Weakness |
|---|---|---|---|
| Greedy Search | always the top token | fast, cheap | repetitive, locally optimal |
| Sampling | random per probability distribution | varied, diverse | less coherent |
| Beam Search | keep top N beams, pick highest joint probability | more coherent | slow; misses good words behind a low-probability one |
| Top-K | restrict to K most probable | diversity + relevance + control | K is fixed regardless of distribution shape |
| Top-p (Nucleus) | smallest set whose cumulative probability > P | excludes rare tokens precisely | shortlist size varies unpredictably |

| Parameter | Effect |
|---|---|
| Temperature | divides logits before softmax; low = deterministic/accurate, high = diverse/creative, 1 = neutral |
| Stop sequences | terminate generation on appearance; control length and structure |
| Frequency penalty | reduces reuse of *repeatedly* occurring tokens |
| Presence penalty | prevents repeating *any* already-present token |
| Top-p | cumulative probability threshold for the candidate pool |

| Benchmark | Structure | Distinctive feature |
|---|---|---|
| **GLUE** | 9 English sentence-understanding tasks in 3 categories — Single-Sentence (CoLA grammaticality, SST-2 sentiment), Similarity & Paraphrase (MRPC, QQP, STS-B), Inference (RTE entailment, QNLI, WNLI pronoun reference) | overall score = average across all nine |
| **SuperGLUE** | 8 tasks: Boolean QA, textual entailment, coreference resolution, common-sense reading comprehension, word-sense disambiguation | keeps GLUE's hardest tasks, adds harder ones; **provides human baselines** per metric |
| **BIG-bench** | 204+ tasks across topics and languages; JSON tasks (output/target comparison) and programmatic tasks (Python assessing generation and conditional log probabilities) | a **"living benchmark"** accepting new task submissions under peer review; open-source |
| **HELM** | broad scenario coverage with explicit acknowledgment of incompleteness; 30 models evaluated | **multi-metric**: accuracy, calibration, robustness, fairness, bias, toxicity, efficiency — plus a standardized few-shot adaptation process |
| **FLASK** | 12 instance-wise skill sets: logical correctness, logical efficiency, factuality, common-sense understanding, comprehension, insightfulness, completeness, metacognition, readability, conciseness, harmlessness | fine-grained, skill-set-level diagnosis rather than a single score |

| Fine-tuning technique | What it adjusts | Cost |
|---|---|---|
| Full Fine-Tuning | all parameters | highest — considerable computational power |
| LoRA | low-rank approximations on downstream layers | far fewer parameters, lower GPU memory and expense |
| QLoRA | LoRA + parameter quantization | lowest memory |
| SFT | supervised on demonstration data, prompts, responses | moderate; also implements instruction tuning |
| RLHF | incremental alignment to human feedback | highest complexity; related: DPO, RLAIF |

## Worked Example
**Anatomy of a hallucination: the Dolly 2.0 question.**

Databricks announced Dolly 2.0 — an open-source 12-billion-parameter instruction-tuned model — on April 12th, 2023. GPT-3.5's training data ends in 2022, so it has no information about it whatsoever. Asked *"Does Databricks' Dolly contribute value to the open-source community?"*, it responds:

> "Databricks Dolly is a creation of Databricks, aimed at assisting users in transferring data between platforms, especially from on-premise data centers to cloud platforms like AWS or Azure. Although Databricks Dolly is **not open-source**, it potentially offers benefits to the open-source community…"

Trace what happened. The model correctly identifies Databricks as the maker — that much is in training data. Everything after that is *generated from the distribution of what Databricks products are typically like*: a data-migration tool moving data to AWS or Azure. It is a perfectly reasonable description of a hypothetical Databricks product. It is also wrong in every particular, including the flatly inverted claim that Dolly is not open-source.

Three properties make this the model failure mode that matters most in production:

1. **It is confident.** No hedging, no uncertainty marker — because the model has no mechanism for expressing either.
2. **It is plausible.** A reader who does not already know the answer has no signal that anything is wrong. This is precisely why the book says these errors are "difficult to identify" in LLM-assisted workflows.
3. **It is structural.** Not a bug in GPT-3.5 — "all similar LLMs, like Bard or LLaMA, also exhibit this behavior." The objective is to learn word distributions, so the model produces the most probable continuation, and *most probable ≠ true*.

Which is why the recommended fix is architectural rather than a prompt tweak: a retriever anchors the answer in an actual document about Dolly, and the instruction to "extract the answer from the context paragraphs and not write anything that cannot be inferred from them" removes the space where invention happens.

## Key Takeaways
1. Hallucinations arise because LLMs learn word distributions, cannot verify facts, and cannot express uncertainty — "most probable" is not "true."
2. Hallucinated content is dangerous precisely because it is coherent and confident; it looks like the model's correct output.
3. Mitigate with generation parameters, better training data, careful prompting, and above all retriever architectures that ground answers in documents.
4. Bias enters through data, annotation, input representations, models, and research methodology — not just the corpus.
5. Constitutional AI trains the model against explicit principles and uses AI-generated feedback in RL, reducing reliance on human labeling.
6. Loss functions must be differentiable and drive training; evaluation metrics need not be and measure human-understandable performance.
7. Perplexity is the intrinsic metric: geometric-mean-normalized inverse sentence probability; lower is better.
8. GLUE → SuperGLUE → BIG-bench → HELM → FLASK trace an arc from single-score accuracy toward multi-metric, skill-set-level, living evaluation.
9. Decoding is a greediness/exploration trade-off across Greedy, Sampling, Beam, Top-K, and Top-p.
10. Temperature scales logits before softmax; stop sequences bound structure; frequency and presence penalties control repetition differently.
11. Pretraining gives general language ability; fine-tuning gives domain expertise; instruction fine-tuning makes the model treat input as commands.
12. LoRA/QLoRA make fine-tuning affordable; SFT and RLHF differ in whether alignment is one-shot supervised or iterative from feedback.

## Connects To
- **Ch I (Introduction to LLMs)**: hallucination introduced; the RAG-first argument.
- **Ch II (Architectures)**: cross-entropy loss over the vocabulary follows from causal language modeling.
- **Ch IV (Introduction to Prompting)**: careful prompt crafting as a hallucination control.
- **Ch VII (RAG)** and **Ch VIII (Advanced RAG)**: the retriever architecture built out, plus RAG-specific evaluation metrics.
- **Ch X (Fine-Tuning)**: LoRA, SFT, and RLHF implemented end to end.
