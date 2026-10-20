# Chapter II: LLM Architectures and Landscape

## Core Idea
The transformer split into three families — **encoder-only, encoder-decoder, decoder-only** — and the differences extend beyond network design into **learning objectives**, which is what actually shapes model behavior. Choosing the right family (and the right model tier: proprietary / open / open-source) is a design decision, not a default.

## Frameworks Introduced
- **The three transformer categories**:
  - **Encoder-only** — extracts context-aware representations. Text-to-vector for similarity, or paired with a classification head (a "Pooler" layer in Hugging Face). **The fundamental distinction is the absence of Masked Self-Attention**, so the encoder processes the full input simultaneously — ideal for document vector representations that retain all information. Representative: BERT.
  - **Encoder-decoder** — sequence-to-sequence: translation, summarization, and multimodal tasks like caption generation. **Cross-attention** helps the decoder concentrate on the most relevant parts of the encoder's representation during generation. Representative: BART.
  - **Decoder-only** — produces output following provided instructions. Masked self-attention means it can only attend to previous tokens, which "gave rise to the concept of prompting." Representative: the GPT family.
- **The four transformer components**:
  1. **Input Embedding** — high-dimensional vectors capturing semantic features. GPT-3 uses 12,000 dimensions; BERT uses 768. Large because "when you cannot understand meanings and words, you need thousands of values representing them."
  2. **Positional Encoding** — transformers have no built-in sequential processing (unlike RNNs), so vectors with unique per-index values are added to the input embeddings to convey relative or absolute position.
  3. **Self-Attention** — a weighted total of all word embeddings, weights from learned attention scores, implemented via **Query** ("What should I pay attention to?"), **Key** ("What is important to consider?"), and **Value** ("What information do we have?") vectors.
  4. **Multi-head attention** — multiple attention components process the same information, each head learning to focus on different features (verbs, nouns, numerals). Typically 16–32 heads per layer.
- **Causal Language Modeling** — the GPT pre-training objective. Self-supervised on data with **soft ground truth**, eliminating explicit labels: predict the next word, attending only to tokens on the left. Called "causal" because each prediction is purely based on previous steps. Advantages over masked language modeling: it more closely replicates natural writing and speech, and **the model stays effective on real-world texts that contain no masked tokens**.
- **Large Multimodal Model (LMM) architecture** — an **image encoder** extracting visual features (CNN like ResNet, or transformer like ViT) feeding a standard language model that generates text. Training is auto-regressive loss on output tokens, with **image tokens**: photographs segmented into non-overlapping patches, analogous to text tokenization. Image tokens attend to one another; each text token is conditioned on all previously generated image and text tokens.
- **The three connection-module approaches** across LMMs:
  - **BLIP2** — a trainable, lightweight connection module (Q-Former) bridging vision and language; **only the connection module is trained**, which is cost- and time-efficient with robust zero-shot image understanding.
  - **Flamingo** — a **Perceiver Resampler** processes spatiotemporal features from the frozen Vision Encoder and produces a **fixed number of visual tokens**. These condition a **frozen language model** via newly-initialized **cross-attention layers** interleaved into the LM's existing layers — only those layers update. Less efficient (more trainable parameters) but a more sophisticated integration.
  - **LLaVA** — a **simple linear projection matrix W** transforming image features into language embedding tokens dimension-matched to the LM's word embedding space. Deliberately lighter than BLIP2's Q-Former and Flamingo's perceiver + cross-attention.
- **The three model-openness tiers**:
  - **Proprietary** — accessible only through paid APIs or web interfaces (GPT-4, Claude 3 Opus).
  - **Open** — architecture and weights openly available (LLaMA 2, Mixtral 8x7B).
  - **Open-source** — complete pre-training data, training code, evaluation code, **and** weights, enabling researchers to re-create and analyze the model (OLMo by AI2).

## Key Concepts
- **"Attention Is All You Need"**: Google Brain + University of Toronto; state-of-the-art 41.8 on WMT 2014 English-to-French after **just 3.5 days on eight GPUs** — a drastic reduction in training cost. Key innovation: a highly parallelized network structure.
- **Attention mask**: a vector where 1 means process the token normally and 0 means ignore it.
- **BERT's two pre-training objectives**: Masked Language Modeling (predict obscured random tokens) and Next Sentence Prediction (does sentence B logically follow A?).
- **BART's objective**: process an input where parts are randomly masked, alongside an input shifted by one token, and reconstruct the original.
- **Scaled dot-product attention** with masking: masked positions get a large negative score so softmax nullifies them.
- **ConvID in GPT-2**: OpenAI uses distinct linear layers to transpose weights — **not** PyTorch convolutional layers. Unique to OpenAI; other open models use conventional linear layers.
- **minGPT** (Andrej Karpathy): ~300 lines of PyTorch. `model.py` (architecture), `bpe.py` (Byte Pair Encoding tokenization), `trainer.py` (generic training loop), `demo.ipynb` (end-to-end). Light enough to run on a MacBook Air.
- **LLaVA's synthetic dataset**: GPT-4 given *symbolic* representations of images — captions plus bounding-box coordinates from COCO — as prompts to generate 158,000 training samples in three types: question-answer conversations, thorough descriptions, and complex reasoning problems.
- **IDEFICS**: Hugging Face's open Flamingo replica from LLaMA v1 and OpenCLIP, in base and instructed variants at 9B and 80B. Trained partly on **OBELICS**, a new 115-billion-token dataset of 141 million image-text documents with 353 million images.
- **Constitutional AI** (Anthropic): use a language model with clear directives — a *constitution* — to guide the model during training, instead of relying on human feedback, aiming to be helpful, honest, and harmless.
- **LLaMA 2's dual reward models**: distinct **safety** and **helpfulness** reward models assessing response quality, balancing the two.

## Mental Models
- **Learning objective, not just architecture, determines behavior.** Encoder-only sees everything at once because nothing needs masking; decoder-only masks the future to avoid "cheating"; that single difference produces two entirely different classes of model.
- **Embeddings are large because meaning is hard.** The model compares lists of numbers arithmetically instead of understanding words — so the lists must be long.
- **Bigger is not automatically the answer.** "Smaller, more focused models might deliver comparable outcomes while functioning more effectively" — DistilBERT on a local server for similarity may be more appropriate and far cheaper than a proprietary API.
- **Start proprietary, move open later.** Begin with reliable proprietary models during initial development and consider open alternatives once the product has traction, because implementing an open model costs more resources.

## Anti-patterns
- **Using a decoder-only model for embeddings**: encoder-only models retain all information because they see the whole input at once.
- **Reaching for an LLM where DistilBERT suffices**: cost without benefit.
- **Ignoring service downtime in proprietary models**: reliability planning must account for API outages.
- **Confusing GPT-2's `ConvID` with a convolution**: it is a transposed linear layer, OpenAI-specific.
- **Assuming open weights means open source**: only open-source releases include pre-training data, training code, and evaluation code.
- **Deploying without addressing hallucination, bias, privacy, and employment impact**: LLMs can memorize and disclose sensitive information; anonymization and access controls are required.

## Code Examples
Loading a decoder-only model and tracing it layer by layer:
```python
from transformers import AutoModelForCausalLM, AutoTokenizer

OPT = AutoModelForCausalLM.from_pretrained("facebook/opt-1.3b", load_in_8bit=True)
tokenizer = AutoTokenizer.from_pretrained("facebook/opt-1.3b")

inp = "The quick brown fox jumps over the lazy dog"
inp_tokenized = tokenizer(inp, return_tensors="pt")
print(inp_tokenized['input_ids'].size())   # torch.Size([1, 10])
# {'input_ids': tensor([[2, 133, 2119, 6219, 23602, 13855, 81, 5, 22414, 2335]]),
#  'attention_mask': tensor([[1,1,1,1,1,1,1,1,1,1]])}

print(OPT.model)
# OPTModel( (decoder): OPTDecoder(
#   (embed_tokens): Embedding(50272, 2048, padding_idx=1)
#   (embed_positions): OPTLearnedPositionalEmbedding(2050, 2048)
#   (layers): ModuleList( (0-23): 24 x OPTDecoderLayer( (self_attn): OPTAttention(
#       k_proj / v_proj / q_proj / out_proj: Linear8bitLt(2048 -> 2048) ) ... )))
```
Walking the three stages by hand:
```python
# 1. Embeddings: [1, 10] token ids -> [1, 10, 2048] vectors
embedded_input = OPT.model.decoder.embed_tokens(inp_tokenized['input_ids'])

# 2. Positional encoding: built from the attention mask, same shape
embed_pos_input = OPT.model.decoder.embed_positions(inp_tokenized['attention_mask'])

# 3. Self-attention takes the SUM of the two
embed_position_input = embedded_input + embed_pos_input
hidden_states, _, _ = OPT.model.decoder.layers[0].self_attn(embed_position_input)
```
- **What it demonstrates**: the diagram made concrete — self-attention's input is embedding **plus** positional encoding, not a concatenation. In production the model also supplies the attention mask so parts of the input can be disregarded (omitted here for clarity).

Masked self-attention from scratch:
```python
import numpy as np

def self_attention(query, key, value, mask=None):
    # Compute attention scores
    scores = np.dot(query, key.T)

    if mask is not None:
        # Apply mask by setting masked positions to a large negative value
        scores = scores + mask * -1e9

    # Softmax -> attention weights (the -1e9 positions become ~0)
    attention_weights = np.exp(scores) / np.sum(np.exp(scores), axis=-1, keepdims=True)

    # Weighted sum of value vectors
    return np.dot(attention_weights, value)
```

The three families in three lines:
```python
from transformers import AutoModel, pipeline

BART = AutoModel.from_pretrained("facebook/bart-large")   # encoder-decoder
BERT = AutoModel.from_pretrained("bert-base-uncased")     # encoder-only
gpt2 = AutoModel.from_pretrained("gpt2")                  # decoder-only
```
BART's decoder carries an extra `encoder_attn` layer — **cross-attention**, which conditions decoder output on encoder representations. BERT's output goes to a `BertPooler` (dense + Tanh) producing the final representation for classification or similarity. GPT-2 is a decoder block **without** cross-attention.

Using them via pipelines:
```python
summarizer = pipeline("summarization", model="facebook/bart-large-cnn")
summarizer(article_text, min_length=20, max_length=50)

classifier = pipeline("text-classification",
                      model="nlptown/bert-base-multilingual-uncased-sentiment")
classifier("This restaurant is awesome.")   # [{'label': '5 stars', 'score': 0.855}]

generator = pipeline(model="gpt2")
generator("This movie was a very", do_sample=True, top_p=0.95,
          num_return_sequences=4, max_new_tokens=50, return_full_text=False)
```

Loading an open model (same code across LLaMA 2, Falcon, Dolly — only `model_id` changes):
```python
from transformers import AutoModelForCausalLM, AutoTokenizer
import torch

model_id = "meta-llama/Llama-2-7b-chat-hf"      # ~14GB download
# model_id = "tiiuae/falcon-7b-instruct"
# model_id = "databricks/dolly-v2-3b"

tokenizer = AutoTokenizer.from_pretrained(model_id)
model = AutoModelForCausalLM.from_pretrained(
    model_id, trust_remote_code=True, torch_dtype=torch.bfloat16)

prompt = "Translate English to French: Configuration files are easy to use!"
inputs = tokenizer(prompt, return_tensors="pt", return_token_type_ids=False)
outputs = model.generate(**inputs, max_new_tokens=100)
print(tokenizer.batch_decode(outputs, skip_special_tokens=True)[0])
```

Cohere with a built-in web-search connector:
```python
import cohere
co = cohere.Client('<<apiKey>>')

response = co.chat(
    chat_history=[
        {"role": "USER", "message": "Who discovered gravity?"},
        {"role": "CHATBOT", "message": "The man who is widely credited with "
                                       "discovering gravity is Sir Isaac Newton"},
    ],
    message="What year was he born?",
    connectors=[{"id": "web-search"}],   # search before answering
)
```

## Reference Tables

| Category | Masked self-attention | Best for | Example |
|---|---|---|---|
| Encoder-only | absent — sees full input at once | text→vector, similarity, classification | BERT, RoBERTa, DistilBERT |
| Encoder-decoder | in the decoder; plus cross-attention | translation, summarization, captioning | BART, T5 |
| Decoder-only | present | instruction following, generation | GPT family, LLaMA 2 |

| Component | Role |
|---|---|
| Input embedding | semantic feature vectors (GPT-3: 12,000-d; BERT: 768-d) |
| Positional encoding | word order, absent from the attention mechanism itself |
| Query | what to pay attention to |
| Key | what is important to consider |
| Value | what information we have |
| Multi-head | 16–32 heads, each learning different features |

| LMM | Connection module | What trains |
|---|---|---|
| BLIP2 | Q-Former (lightweight, trainable) | connection module only |
| Flamingo | Perceiver Resampler + new cross-attention layers | resampler + inserted cross-attention |
| LLaVA | single linear projection matrix W | W, then W + language model end-to-end |

| Openness tier | What you get | Trade-off |
|---|---|---|
| Proprietary | API/web access; often fine-tuning within usage policy | best performance, no customization, service downtime risk |
| Open | architecture + weights | full customization; you supply implementation and compute |
| Open-source | + pre-training data, training code, evaluation code | full reproducibility and analysis |

| Model (as of April 2024) | Notes |
|---|---|
| **Cohere** | three categories: Command (chat, long context), Rerank (semantic sorting), Embed. Command R: 128K context, RAG- and tool-use-optimized, ten major languages. Free trial key, 100 calls/min |
| **OpenAI GPT-3.5 / GPT-4** | GPT-3.5 cost-effective for basic chat; Turbo faster and cheaper; GPT-4 Turbo accepts text and image input, text output only; training specifics confidential |
| **Anthropic Claude 3** | Haiku / Sonnet / Opus tiers; 200K context (processing up to 1M); Constitutional AI; ASL-3 red teaming; strong at complex instructions and brand voice |
| **Google Gemini** | Ultra / Pro / Nano; multimodal across text, image, audio, video, code; Gemini 1.5 Pro up to 1M context |
| **Meta LLaMA 2** | open license, research + commercial; 2T training tokens; 7B/13B/70B plus LLaMA-Chat; SFT + RLHF with **separate safety and helpfulness reward models** |
| **Falcon (TII)** | Apache 2.0; Falcon-40B beat LLaMA 65B and MPT-7B; Falcon-7B for consumer-hardware fine-tuning; RefinedWeb dataset retains image links and alt text |
| **Dolly 2.0 (Databricks)** | EleutherAI Pythia-12B; `databricks-dolly-15k` — 15,000 prompt/response pairs, CC BY-SA 3.0, real human-generated rather than synthesized |
| **Open Assistant** | 13,000+ volunteers; 600,000+ interactions, 150,000 messages, 10,000 annotated conversation trees; weaker at math and coding |
| **Mistral** | Mistral 7B beats LLaMA 2 13B and LLaMA 1 34B; **Mixtral 8x7B** sparse MoE — 46.7B parameters but only 12.9B per token, 6× faster inference than LLaMA 2 70B, 32k context; Mistral Large/Small proprietary with JSON mode and function calling |

| Industry | LLM application | Example |
|---|---|---|
| Healthcare | diagnostics, digital pathology, drug discovery, personalized devices, admin automation | Med-PaLM, BioMedLM |
| Finance | customer interaction, time-series analysis, macroeconomic insight, market prediction | BloombergGPT |
| Copywriting | websites, blogs, social, product descriptions, SEO; consistent brand voice | Jasper |
| Education | personalized tutoring, adaptive testing, virtual teachers, language practice | Khanmigo |
| Programming | code completion, unit test generation, natural-language-to-code | GitHub Copilot |
| Legal | legal advice, document comprehension, court-case analysis, term identification | — (hallucination minimization is the key goal) |

## Worked Example
**Tracing "The quick brown fox jumps over the lazy dog" through OPT-1.3B** — the chapter's most useful exercise, because it turns the architecture diagram into observable tensor shapes.

1. **Tokenize.** The string becomes `[2, 133, 2119, 6219, 23602, 13855, 81, 5, 22414, 2335]` — shape `[1, 10]` — plus an attention mask of all 1s, meaning process every token.
2. **Inspect the model.** `OPT.model` reveals `OPTDecoder` with `embed_tokens` (vocabulary 50,272 → 2,048 dimensions), `embed_positions`, and 24 identical stacked `OPTDecoderLayer`s. Being decoder-only, everything lives under the `decoder` key.
3. **Embed.** `embed_tokens` maps `[1, 10]` → `[1, 10, 2048]`. Ten token IDs became ten 2,048-dimensional vectors.
4. **Encode position.** `embed_positions(attention_mask)` produces the same `[1, 10, 2048]` shape — a unique vector per position.
5. **Add, then attend.** `embedded_input + embed_pos_input` is the actual input to self-attention. `layers[0].self_attn` reveals the four projections — `k_proj`, `v_proj`, `q_proj`, `out_proj` — each `Linear8bitLt(2048 → 2048)`, and returns `[1, 10, 2048]`.

Two details worth noticing. First, the addition in step 5: positional information is *summed into* the semantic embedding, not concatenated alongside it. Second, `load_in_8bit=True` — the model runs quantized to save GPU memory, which is why the linear layers show as `Linear8bitLt` rather than `Linear`. Everything after layer 0 is 23 more of the same block, plus nonlinearity (ReLU), feedforward, and normalization.

## Key Takeaways
1. The transformer split into encoder-only, encoder-decoder, and decoder-only; the differences are in both design and learning objective.
2. Encoder-only lacks masked self-attention and sees the whole input — best for representations; decoder-only masks the future — best for generation and prompting.
3. Cross-attention is what makes an encoder-decoder: it conditions decoder output on the encoder's representation.
4. Self-attention is Q/K/V — what to attend to, what matters, what information exists — with 16–32 heads each learning different features.
5. Positional encoding is *added* to input embeddings because attention has no inherent notion of order.
6. Causal Language Modeling is self-supervised on soft ground truth and keeps the model effective on real text, unlike masked language modeling.
7. LMMs pair an image encoder with a language model; the connection module differs — Q-Former (BLIP2), Perceiver + cross-attention (Flamingo), linear projection (LLaVA).
8. Instruction-tuned LMMs (LLaVA, MiniGPT-4, InstructBLIP) generalize better to unseen problems; LLaVA built its 158k-sample dataset by prompting GPT-4 with symbolic image representations.
9. Proprietary / open / open-source is a spectrum of access; start proprietary during development and consider open once the product has traction.
10. Smaller focused models can match larger ones for narrow tasks at far lower cost.
11. Real-world deployment risks: hallucination, bias, data privacy (models can memorize and disclose sensitive information), and employment impact requiring balanced human oversight.

## Connects To
- **Ch I (Introduction to LLMs)**: the terminology and history this chapter builds on.
- **Ch III (LLMs in Practice)**: hallucination, bias, and evaluation in depth.
- **Ch VII (RAG)**: embeddings from encoder-only models power retrieval.
- **Ch IX (Agents)**: multimodal financial document analysis uses LMM capabilities.
- **Ch X (Fine-Tuning)**: SFT and RLHF, referenced here for LLaMA 2.
- **Ch XI (Deployment)**: 8-bit loading previewed here; quantization in full there.
