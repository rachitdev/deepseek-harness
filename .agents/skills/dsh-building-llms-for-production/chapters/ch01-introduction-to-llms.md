# Chapter I: Introduction to LLMs

## Core Idea
LLMs are next-token predictors whose scale produces **emergent abilities**, but production use is limited by hallucination, missing domain knowledge, and context limits. The book's answer is a four-part toolkit: **Prompt Engineering, RAG, Fine-Tuning, and custom UI/UX** — the "march of 9s" toward reliability.

## Frameworks Introduced
- **The "march of 9s"** — the journey from a basic LLM prompt to sufficient accuracy, reliability, and observability for a target copilot use case. Borrowed from self-driving car development: gradual improvement measured in nines (99.9%…) toward human-level performance. LLMs are generally deployed as **copilots with a human fully in the loop**, not fully automated, precisely because of this gap.
- **The four-part developer toolkit**: 1) Prompt Engineering, 2) Retrieval-Augmented Generation, 3) Fine-Tuning, 4) Custom UI/UX.
- **The four reasons to love RAG**:
  1. Reducing hallucinations by limiting the LLM to answer from existing chosen data.
  2. Helping with explainability, error checking, and copyright by clearly referencing sources for each comment.
  3. Giving private, specific, or more up-to-date data to the LLM.
  4. Not relying on black-box training/fine-tuning for what the model knows and has memorized.
- **Scaling Laws (Chinchilla)** — performance depends on: **N** (parameters, the ability to learn complex patterns), **D** (training dataset size in tokens), and **FLOPs** (compute). The rule: **for a model with X parameters, optimal training uses roughly X × 20 tokens** — Chinchilla's 70B parameters on 1.4T tokens. Despite being smaller, Chinchilla outperformed larger models using less memory and compute.
- **Emergent abilities** — skills appearing unexpectedly as model size grows, not explicitly taught. On a scaling curve, performance looks near-random until a threshold, then improves sharply — a **phase transition**, not extrapolable from smaller systems. Evaluated on BIG-Bench, TruthfulQA, MMLU, and WiC.
- **The four techniques for expanding the context window**:
  1. **ALiBi Positional Encoding** (Attention with Linear Biases) — replaces Positional Sinusoidal Encoding, which struggles to infer larger context lengths. Allows training in smaller contexts then fine-tuning in bigger ones.
  2. **Sparse Attention** — focuses attention scores on a subset of tokens, reducing complexity to linear in *n*.
  3. **FlashAttention** — restructures the attention calculation for GPU efficiency by dividing input matrices into blocks, optimizing GPU memory utilization.
  4. **Multi-Query Attention (MQA)** — aggregates weights across all attention heads during linear projection of Key and Value matrices, reducing key/value decoder cache memory.

## Key Concepts
- **Autoregressive generation**: the model generates the next token based on the tokens already generated, in a loop.
- **The Transformer**: introduced in "Attention Is All You Need", originally an encoder-decoder framework. The encoder processes input into a high-dimensional representation; the decoder transforms it back into readable text. **Components can be used jointly or independently** — some models use only the encoder (text → vector), others only the decoder (the backbone of LLMs).
- **RNN limitation**: RNNs maintain internal state but forget older information over long sequences, caused by the **vanishing gradient problem** — gradients shrink as they propagate back through timesteps, so weights for early inputs barely change.
- **Tokenization**: model-dependent; models ship as a **pair of pre-trained tokenizer and associated weights**. Byte-Pair Encoding splits "coloring" into "color" + "ing" — more tokens per text, but **dramatically fewer entries in the dictionary**, and the model learns that "ing" indicates present tense.
- **Embeddings**: initially random vectors per token; training adjusts them so tokens with similar meanings end up with similar vectors ("happy" near "joyful").
- **Context size**: the maximum tokens processable in a single request; directly affects performance and outcomes.
- **Attention complexity**: **O(n²d + nd²)** where *n* is context length and *d* is embedding size — from linear projections creating Q/K/V matrices and their multiplication. Growth is quadratic in context length.
- **Hallucination as a feature**: a direct result of the next-token-prediction training goal — to some extent it enables "creative" answers. The problem is that **it is difficult for an LLM to know when it is answering from memorized facts versus imagination**, which creates hard-to-identify errors in LLM-assisted workflows.
- **Bias**: the inclination to favor outputs based on training data. "Who is a nurse?" → "*She* is a healthcare professional…" demonstrates gender bias.
- **LongNet's dilated attention**: attention field increases exponentially as the gap between tokens widens, inversely reducing attention calculations with distance — **linear computational complexity**, targeting a 1-billion-token context.

## Mental Models
- **RAG before fine-tuning.** For adding specialized knowledge quickly and efficiently, RAG is usually the better first step: more control over the information used, and a quicker, more transparent, more manageable experimentation phase.
- **CoT works because tokens are processing bandwidth.** "Each token in a language model has a limited 'processing bandwidth' or 'thinking capacity'. The LLM needs these tokens to figure things out."
- **Prompting is delegation.** Over time, prompting will resemble the skills needed for effective communication and delegation to human colleagues.
- **Scale changes kind, not just degree.** Pattern recognition improves quantitatively *and changes qualitatively* as models scale.
- **Subword tokenization trades token count for vocabulary size** — more tokens per document, far fewer dictionary entries, and morphological structure comes free.

## Anti-patterns
- **Fully automating a task LLMs can only copilot**: the accuracy, reliability, and observability gap is real.
- **Reaching for fine-tuning to add knowledge**: RAG gives more control, faster iteration, and transparency.
- **Vague prompts**: "Write about dog food" vs "Write a 500-word informative article about the dietary needs of adult Golden Retrievers."
- **Publishing model output unreviewed**: the book's own example produces a factually inverted safety claim ("all yoga poses are suitable for everyone") that review corrects to "not all".
- **Assuming a bigger model is better-trained**: Chinchilla showed most large models were undertrained for their parameter count.
- **Committing API keys**: use `.env` plus `dotenv`, especially for open-source projects and shared code.

## Code Examples
Environment setup — the book's pinned package set:
```
deeplake==3.6.19      openai==0.27.8        tiktoken==0.4.0
transformers==4.32.0  torch==2.0.1          numpy==1.23.5
deepspeed==0.10.1     trl==0.7.1            peft==0.5.0
wandb==0.15.8         bitsandbytes==0.41.1  accelerate==0.22.0
tqdm==4.66.1          neural_compressor==2.2.1
onnx==1.14.1          pandas==2.0.3         scipy==1.11.2
```
```bash
python -m venv my_venv_name
source my_venv_name/bin/activate
pip install deeplake            # or: pip install deeplake==3.6.5
```

Basic translation via the OpenAI API:
```python
from dotenv import load_dotenv
load_dotenv()
import openai

english_text = "Hello, how are you?"

response = openai.ChatCompletion.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content":
         f'''Translate the following English text to French: "{english_text}"'''}
    ],
)
print(response['choices'][0]['message']['content'])
# Bonjour, comment ça va?
```

Few-shot learning through the message roles:
```python
prompt = """
Describe the following movie using emojis.

{movie}: """

examples = [
    {"input": "Titanic",    "output": "🚢💔🌊🧊🥶"},
    {"input": "The Matrix", "output": "🕶️💊🖥️🥋"},
]
movie = "Toy Story"

response = openai.ChatCompletion.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system",    "content": "You are a helpful assistant."},
        {"role": "user",      "content": prompt.format(movie=examples[0]["input"])},
        {"role": "assistant", "content": examples[0]["output"]},
        {"role": "user",      "content": prompt.format(movie=examples[1]["input"])},
        {"role": "assistant", "content": examples[1]["output"]},
        {"role": "user",      "content": prompt.format(movie=movie)},
    ],
)
```
- **What it demonstrates**: few-shot examples delivered as alternating `user`/`assistant` turns rather than concatenated into one prompt string — the model reads them as prior conversation, which is a stronger pattern signal.

Tokenization comparison on *"The child's coloring book."*:
```
whitespace only : ["The", "child's", "coloring", "book."]
+ punctuation   : ["The", "child", "'", "s", "coloring", "book", "."]
Byte-Pair       : ["The", "child", "'", "s", "color", "ing", "book", "."]
```

## Reference Tables

| Concept | Definition |
|---|---|
| Language modeling | Learning the probability distribution of words in a language from a corpus; predicting the next token |
| Tokenization | Breaking input text into tokens; model-dependent; ships paired with the model weights |
| Embeddings | Numerical vectors giving each token a meaning-capturing representation |
| Pre-training | Training on a huge general corpus (The Pile, CommonCrawl) to learn language comprehension |
| Fine-tuning | Further training on a smaller task- or domain-specific dataset |
| Prediction | Producing a probability distribution over next tokens and selecting by score, in a loop |
| Context size | Maximum tokens processable in a single request |

| Emergent-ability benchmark | What it tests |
|---|---|
| **BIG-Bench** | 200+ benchmarks: arithmetic, IPA transliteration, word unscrambling. GPT-3/LaMDA start near zero and jump above random at scale |
| **TruthfulQA** | Truthfulness via generation and multiple-choice; Gopher at largest size exceeds random by >20% |
| **MMLU** | World knowledge and problem-solving across 57 tasks; smaller models don't beat random guessing on average |
| **WiC (Word in Context)** | Whether a target word shares meaning across two contexts; Chinchilla fails one-shot even at scale, PaLM passes when scaled much larger |

| Context-window optimization | Effect |
|---|---|
| ALiBi | scalable positional encoding; train small, fine-tune large |
| Sparse Attention | linear complexity in *n* |
| FlashAttention | block-wise GPU-efficient attention |
| FlashAttention-2 | ~2× faster, up to 230 TFLOPs on A100; more matmul FLOPs (non-matmul are 16× more expensive), better parallelism across batch/head/sequence, less warp synchronization, head dims to 256, MQA support |
| MQA | shared K/V projections, smaller decoder cache |
| LongNet dilated attention | linear complexity, targeting 1B tokens |

| Year | Model | Significance |
|---|---|---|
| 2018 | GPT-1 | generative decoder-only transformer; unsupervised pretraining + supervised fine-tuning |
| 2019 | GPT-2 | 1.5B parameters; unified format for input, output, task |
| 2020 | GPT-3 | 175B; introduced in-context learning |
| 2021 | Codex | GPT-3 fine-tuned on GitHub code |
| 2021 | LaMDA | 137B, dialog applications |
| 2021 | Gopher | 280B, near-human MMLU; biases and misinformation issues |
| 2022 | InstructGPT | RLHF for instruction-following and safety |
| 2022 | Chinchilla | 70B, compute-optimal per scaling laws |
| 2022 | PaLM | 540B, exceptional few-shot, Pathways distributed computation |
| 2022 | ChatGPT | conversational AI on GPT-3.5/4 |
| 2023 | LLaMA | 7B–65B; broke the closed-weights tradition (noncommercial, then commercial with LLaMA 2) |
| 2023 | GPT-4 | multimodal inputs |
| 2024 | Gemini 1.5 | Mixture-of-Experts, multimodal, up to 1M-token context |
| 2024 | Gemma | 2B and 7B, pre-trained and instruction-tuned, from the Gemini training run |
| 2024 | Claude 3 Opus | 200K context, exceptional recall regardless of position |
| 2024 | Mistral 8x22B | best open-source option; still below GPT-4/Claude |
| 2024 | Infinite Attention | speculated basis of Gemini 1.5 Pro; plug-and-play unbounded context |

| NLP milestone | Contribution |
|---|---|
| 1954 Bag of Words | word counts; no order or context |
| 1972 TF-IDF | weights rare words higher; still no context |
| 2013 Word2Vec | high-dimensional embeddings capturing semantics |
| 1997/2014 RNN, LSTM, BiRNN, Encoder-Decoder | sequence handling, long-term dependencies, context |
| 2017 Transformer | attention; alignment between input and output |
| 2018 BERT | bidirectional transformer, global attention |
| 2018–2020 GPT → GPT-3 | autoregressive scaling on WebText then Common Crawl |
| 2019 CTRL / Transformer-XL / ALBERT / RoBERTa / XLM / XLNet / PEGASUS / DistilBERT / XLM-R / BART / ConvBERT | control codes; reused hidden states; parameter reduction + Sentence Order Prediction; dynamic MLM + BPE; multilingual; generalized autoregressive; gap sentence generation; distillation retaining >95%; multilingual RoBERTa; corrupt-and-reconstruct; convolution blocks |
| 2020 Funnel / Reformer / T5 / Longformer / ProphetNet / ELECTRA | sequence compression; LSH attention + axial encoding; text-to-text; sparse attention for long docs; future n-gram prediction; replaced token detection |
| 2021 Switch Transformers | sparsely-activated Mixture of Experts |

## Worked Example
**Why RAG is the recommended first move, worked through the book's own reasoning.**

Start with the failure modes the chapter enumerates: GPT-4 lacks domain-specific knowledge; struggles with large data volumes; misreads new or technical terms; hallucinates; and sometimes simply fails to use available data effectively.

Consider each against the three interventions:

- *Fine-tuning* can teach a format ("turn sentences into SQL", "always answer in JSON") and can instill domain knowledge — but slowly, opaquely, and with no way to inspect *what* the model now believes or to cite a source.
- *Prompting* is instant and cheap but cannot add information the model never had.
- *RAG* directly addresses four of the five: it supplies domain-specific and up-to-date data, it constrains answers to chosen sources so hallucination drops, it makes every claim traceable for explainability and copyright, and it removes the reliance on black-box memorization.

Hence the book's recommendation: **use fine-tuning for behavior and format, RAG for knowledge** — and reach for RAG first because "you have more control over the information the model uses to generate responses, making the experimentation phase quicker, more transparent, and easier to manage."

The complementary insight is *why* hallucination cannot simply be trained away: it is "a direct result of the model training goal of next token prediction" and "to some extent a feature that allows creative model answers." The model has no internal marker distinguishing recall from invention. Grounding it in retrieved text supplies that marker externally.

## Key Takeaways
1. LLMs are trained on next-token prediction; scale produces emergent abilities not explicitly taught.
2. Production reality is copilot use with a human in the loop; the "march of 9s" is the gap to close.
3. The toolkit is prompt engineering, RAG, fine-tuning, and custom UI/UX — with RAG usually the best first step for adding knowledge.
4. Hallucination is structural, a consequence of the training objective; the model cannot tell recall from invention. Grounding supplies that distinction.
5. Chinchilla scaling: roughly 20 tokens per parameter; a smaller well-trained model can beat a larger undertrained one.
6. Transformers replaced RNNs because attention avoids the vanishing-gradient loss of long-range dependencies.
7. Subword (BPE) tokenization increases token count but shrinks vocabulary and captures morphology.
8. Attention is O(n²d + nd²) in context length; ALiBi, sparse attention, FlashAttention(-2), MQA, and dilated attention (LongNet) attack that limit.
9. Few-shot examples work best delivered as alternating user/assistant turns.
10. Emergent abilities arrive as phase transitions and bring emergent risks — bias amplification, toxicity, memorization — mitigable by "helpful, harmless, honest" training and deduplication.

## Connects To
- **Ch II (LLM Architectures)**: the transformer internals sketched here in full.
- **Ch III (LLMs in Practice)**: hallucination, bias, evaluation, and output control in depth.
- **Ch IV (Introduction to Prompting)**: the prompting techniques introduced here.
- **Ch VII (RAG)** and **Ch VIII (Advanced RAG)**: the toolkit's centerpiece.
- **Ch X (Fine-Tuning)**: when fine-tuning is the right tool instead.
- **Ch XI (Deployment)**: quantization and pruning against the compute cost described here.
