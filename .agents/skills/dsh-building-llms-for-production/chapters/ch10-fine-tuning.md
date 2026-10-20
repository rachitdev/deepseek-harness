# Chapter X — Fine-Tuning

## Core Idea

Pre-training gives a model language; it does not give it **instruction
following**. Fine-tuning resumes training on task-specific data so the model
adjusts its internal representations toward a target behavior — but touching
all parameters of a billion-parameter model is prohibitive. The chapter's
answer is a ladder of increasingly cheap and increasingly aligned techniques:
**LoRA** (train 0.2% of parameters), **QLoRA** (4-bit base weights, so 65B fits
on one 48GB GPU), **SFT** (learn from demonstrations), and **RLHF** (learn from
human *rankings*), plus its lighter successors DPO, ReST, and RLAIF.

## Frameworks Introduced

### The fine-tuning technique ladder

| Technique | What it changes | Cost | Use when |
|---|---|---|---|
| **Standard fine-tuning** | all parameters | extensive compute | rarely justified for LLMs |
| **LoRA** | small low-rank update matrices added to (typically) attention layers | consumer-grade GPU | the default efficient method |
| **QLoRA** | LoRA on top of a frozen 4-bit-quantized base | 65B params on a 48GB GPU at 16-bit-equivalent quality | largest models, tightest hardware |
| **SFT** | weights, via demonstration data (prompt → response) | moderate | instruction tuning from a good dataset |
| **RLHF** | weights, via a learned reward model + PPO | expensive, unstable | large high-quality preference data, alignment matters |

**Instruction fine-tuning** (popularized by OpenAI's InstructGPT, 2022) is the
goal these serve: teach the model to read a prompt as an *instruction* rather
than as text to auto-complete.

### LoRA's two critical features

1. **Preservation of pretrained weights** — the base weights are frozen, which
   mitigates **catastrophic forgetting**.
2. **Efficient rank-decomposition** — small update matrices are added alongside
   existing weights; only those are trained. Typically inserted into the
   **attention layers**.

Key hyperparameters: `r` (matrix dimension — lower = fewer trainable params),
`lora_alpha` (scaling factor), `lora_dropout`, `bias` (`none` / `all` /
`lora_only`), `task_type`.

### QLoRA's three memory tricks

- **4-bit NormalFloat (NF4)** — a data type designed for normally distributed
  weights.
- **Double quantization** — quantize the quantization constants themselves.
- **Paged optimizers** — absorb memory spikes.

Gradients backpropagate *through* the frozen 4-bit base into the Low-Rank
Adapters. The Guanaco models are the reference results.

### The RLHF training pipeline

Text generation framed as RL: the **language model is the agent**, its possible
outputs are the **action space**, and the **reward** measures alignment with
context and user intent.

1. *(Optional)* **Instruction fine-tune the LLM** — recommended, makes the RL
   phase converge faster.
2. **RLHF dataset creation** — the LLM generates multiple completions per
   instruction.
3. **Collect human feedback** — evaluators rank completions by preference,
   judging completeness, relevancy, accuracy, toxicity, and bias. Rankings
   convert to scores.
4. **Train a reward model** on those rankings — it learns to mirror the
   labelers' judgment.
5. **Fine-tune with RL (PPO)** — the LLM generates for a random instruction, the
   reward model scores it, PPO adjusts parameters toward higher scores, while a
   **KL divergence penalty** keeps the tuned model close to the original token
   distribution.

The reward model is fundamentally a **classifier**: given a pair of
interactions, pick the better one. It need not match the base model's size —
DeBERTa and RoBERTa work well, though a model closer to the base has more
relevant knowledge.

### Three alternatives to RLHF

| Method | Core move | Advantage over RLHF |
|---|---|---|
| **DPO** (Direct Preference Optimization) | analytically converts the reward function into the optimal RL policy, so the objective collapses to **binary cross-entropy over the reference model** | no separate reward model, no RL machinery |
| **ReST** (Google DeepMind, Reinforced Self-Training) | two alternating phases: **Grow** (generate predictions, expand the dataset) and **Improve** (rank/filter with a reward model, fine-tune offline) | reuses one Grow across many Improve steps; policy quality not capped by the original dataset; phase separation exposes data-quality and reward-hacking issues; few hyperparameters, stable |
| **RLAIF** (Anthropic) | an **AI Feedback Model** governed by a human-written **constitution** replaces human rankers | objective, scalable; a Google study found RLAIF and RLHF nearly equally preferred over plain SFT |

### RLHF vs. SFT — the open question

The book does not pretend this is settled. **RLHF aligns better given a
sufficiently large, high-quality dataset**, but is more expensive and
time-consuming, and RL remains unstable: results are highly sensitive to initial
parameters and hyperparameters, it falls into local optima, and the loss
diverges repeatedly, requiring restarts. Meanwhile the **LIMA** paper ("Less Is
More for Alignment") argues a carefully curated 1,000-sample dataset can replace
RLHF via plain SFT.

## Key Concepts

- **Trainable-parameter arithmetic.** OPT-1.3B with LoRA `r=16`:
  **3,145,728 trainable / 1,318,903,808 total = 0.238%**. That ratio is the
  whole argument.
- **Mixed-precision stability trick.** Freeze the base, then cast 1-dimensional
  parameters (LayerNorm) and the LM head to fp32 while the rest stays bf16.
  1-D params are cheap and numerically sensitive.
- **`gradient_checkpointing_enable()`** — trades recompute for memory by storing
  fewer activations.
- **`ConstantLengthDataset`** (TRL) — packs samples to a fixed `seq_length`
  (up to 2048; smaller saves memory) with `infinite=True` so the iterator
  restarts when data runs out but training steps remain. Pair it with
  `packing=True` on the `SFTTrainer`. Reset with
  `train_dataset.start_iteration = 0` after manually consuming the iterator.
- **A formatting function teaches the template.** `prepare_sample_text` renders
  every row as `Question: …\n\nAnswer: …` so the model learns that a prompt
  starting with the question keyword is completed by an answer. The *same*
  template must be used at inference and in every later RLHF phase.
- **Merging is a separate step.** After training, load the base model and wrap
  it with `PeftModel.from_pretrained(model, checkpoint_path)` to get a
  standalone model. Inspecting it shows `lora_A` / `lora_B` / `lora_dropout`
  `ModuleDict`s grafted onto `v_proj` and friends inside each `OPTDecoderLayer`.
- **Quantization is lossy.** "There's a constant trade-off between using
  lower-precision numbers and potentially diminishing the language processing
  capabilities of models." Generally acceptable — but acknowledge it.
- **Higher learning rate + increased weight decay** can improve fine-tuning
  performance; `bf16=True` reduces memory.
- **Dataset quality is the ceiling.** "The quality of a model's output is
  directly tied to the quality of the data used for training."

## Mental Models

- **LoRA is a diff, not a rewrite.** You are learning a small patch to apply
  over frozen weights, which is why forgetting is limited and why you can keep
  many task-specific adapters against one base.
- **SFT teaches form; RLHF teaches preference.** Demonstrations show the model
  what a good answer *looks* like; rankings show it which of two plausible
  answers people actually *want*.
- **The KL penalty is a leash.** Without it, PPO will happily drift the model
  into reward-hacking gibberish that scores well and reads badly.
- **DPO removes a moving part.** RLHF's fragility comes from having two models
  and an RL optimizer in the loop; DPO's contribution is proving you can drop
  both and keep the objective.
- **The RLHF tutorial is three trainings, not one.** `SFTTrainer` →
  `RewardTrainer` → `PPOTrainer`, each with its own dataset and, in the book's
  case, its own pinned library versions.

## Anti-patterns

- **Full fine-tuning an LLM by default.** Effective but resource-heavy; "less
  valuable" than LoRA for most work.
- **`max_iterations`-free, budget-free cloud GPU runs.** The book repeats the
  warning twice: check billing regularly, shut instances down. Their RLHF run
  used an 8×A100 at **$8.80/hour**.
- **Changing the prompt template between phases.** SFT, reward modeling, and PPO
  must all use `Question: …\n\nAnswer:` or the model sees a distribution shift.
- **Assuming RLHF is strictly better than SFT.** It is an open question; RL
  instability is real and LIMA is a serious counter-argument.
- **Skipping the optional instruction-tuning step before RLHF.** It is optional
  but recommended precisely because it makes RL converge faster.
- **Using a proprietary trial API key in production.** Cohere's Trial key is
  rate-limited and explicitly for experimentation.

## Code Examples

### LoRA config and training arguments

```python
from peft import LoraConfig

lora_config = LoraConfig(
    r=16,
    lora_alpha=32,
    lora_dropout=0.05,
    bias="none",
    task_type="CAUSAL_LM",
)
```

```python
from transformers import TrainingArguments

training_args = TrainingArguments(
    output_dir="./OPT-fine_tuned-LIMA-CPU",
    dataloader_drop_last=True,
    evaluation_strategy="epoch",
    save_strategy="epoch",
    num_train_epochs=10,
    logging_steps=5,
    per_device_train_batch_size=8,
    per_device_eval_batch_size=8,
    learning_rate=1e-4,
    lr_scheduler_type="cosine",
    warmup_steps=10,
    gradient_accumulation_steps=1,
    bf16=True,
    weight_decay=0.05,
    run_name="OPT-fine_tuned-LIMA-CPU",
    report_to="wandb",
)
```

### Preparing the base model for adapter training

```python
import torch, torch.nn as nn
from transformers import AutoModelForCausalLM

model = AutoModelForCausalLM.from_pretrained("facebook/opt-1.3b",
                                             torch_dtype=torch.bfloat16)

for param in model.parameters():
    param.requires_grad = False            # freeze base; train adapters
    if param.ndim == 1:
        param.data = param.data.to(torch.float32)   # LayerNorm etc. in fp32

model.gradient_checkpointing_enable()
model.enable_input_require_grads()

class CastOutputToFloat(nn.Sequential):
    def forward(self, x): return super().forward(x).to(torch.float32)

model.lm_head = CastOutputToFloat(model.lm_head)
```

### Dataset packing and the SFT trainer

```python
from trl.trainer import ConstantLengthDataset
from trl import SFTTrainer

def prepare_sample_text(example):
    """Prepare the text from a sample of the dataset."""
    return f"Question: {example['question'].text()}\n\nAnswer: {example['answer'].text()}"

train_dataset = ConstantLengthDataset(
    tokenizer, ds, formatting_func=prepare_sample_text,
    infinite=True, seq_length=1024)

trainer = SFTTrainer(
    model=model, args=training_args,
    train_dataset=train_dataset, eval_dataset=eval_dataset,
    peft_config=lora_config,
    packing=True,
)
trainer.train()
```

### Counting what you actually train

```python
def print_trainable_parameters(model):
    trainable, total = 0, 0
    for _, param in model.named_parameters():
        total += param.numel()
        if param.requires_grad:
            trainable += param.numel()
    print(f"trainable params: {trainable} || all params: {total} "
          f"|| trainable%: {100 * trainable / total}")

# trainable params: 3145728 || all params: 1318903808 || trainable%: 0.2385
```

Run this on every fine-tune. If the percentage is not tiny, LoRA is not
attached where you think it is.

### Merging adapters back

```python
from peft import PeftModel

model = AutoModelForCausalLM.from_pretrained(
    "facebook/opt-1.3b", return_dict=True, torch_dtype=torch.bfloat16)
model = PeftModel.from_pretrained(model, "./OPT-fine_tuned-LIMA-CPU/<checkpoint>/")
model.eval()
```

### QLoRA 4-bit loading

```python
from transformers import AutoModelForCausalLM, BitsAndBytesConfig
import torch

model = AutoModelForCausalLM.from_pretrained(
    model_name_or_path='/name/or/path/to/your/model',
    load_in_4bit=True,
    device_map='auto',
    torch_dtype=torch.bfloat16,
    quantization_config=BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_compute_dtype=torch.bfloat16,
        bnb_4bit_use_double_quant=True,
        bnb_4bit_quant_type='nf4',
    ),
)
```

### Reward-model dataset shape

The Anthropic **helpfulness/harmless (hh)** dataset has exactly two tensors:
`chosen` and `rejected`. A reward-model training example is always a *pair* —
the model learns to score the chosen one higher. Tokenizer:
`microsoft/deberta-v3-base`.

> The book flags that this dataset contains offensive language, and that this is
> the point: the model is being taught not to reproduce it.

### PPO-phase dataset

```python
class MyDataset(Dataset):
    def __getitem__(self, idx):
        query = "Question: " + self.ds.input[idx].text() + "\n\nAnswer: "
        tokenized = tokenizer(query, truncation=True, max_length=400,
                              padding='max_length', return_tensors="pt")
        return {"query": query, "input_ids": tokenized["input_ids"][0]}

def collator(data):
    return dict((key, [d[key] for d in data]) for key in data[0])
```

Both raw text (`query`) and token ids are kept — PPO needs both. The RL phase
needs **no labels at all**: the reward model supplies the signal, so any prompt
source works (here, only the `input` column of Alpaca-OrcaChat).

### Inference with beam search

```python
generation_output = model.generate(
    **inputs, return_dict_in_generate=True, output_scores=True,
    max_new_tokens=128, num_beams=4, do_sample=True, top_k=10, temperature=0.6)
print(tokenizer.decode(generation_output['sequences'][0]))
```

## Reference Tables

### Tooling

| Library | Role |
|---|---|
| **PEFT** | LoRA, Prefix Tuning, P-tuning — adapt models without touching all parameters |
| **TRL** | `SFTTrainer`, `RewardTrainer`, `PPOTrainer`, `ConstantLengthDataset` |
| **BitsAndBytes** | 4-bit / 8-bit quantized loading |
| **Lit-GPT** (LightningAI) | LoRA without manual architecture edits; Vicuna, Pythia, Falcon; per-matrix configs and adjustable precision |
| **Deep Lake** | dataset streaming — no download, no full load into memory |
| **Weights & Biases** | run tracking (`report_to="wandb"`), prompt engineering, hyperparameter sweeps |

### Precision ladder

| Format | Bytes/weight | Note |
|---|---|---|
| fp32 | 4 | original training standard |
| bf16 | 2 | `bf16=True` during fine-tuning |
| int8 | 1 | `load_in_8bit` |
| 4-bit NF4 | 0.5 | QLoRA; `bnb_4bit_quant_type='nf4'` |

### The RLHF tutorial's three phases

| Phase | Trainer | Model | Dataset |
|---|---|---|---|
| 1. Supervised fine-tuning | `SFTTrainer` + QLoRA | OPT-1.3B | OpenOrca (1M of 4M points) |
| 2. Reward modeling | `RewardTrainer` | DeBERTa-v3-base (300M) | Anthropic hh (chosen/rejected pairs) |
| 3. RL alignment | `PPOTrainer` | SFT model + LoRA | Alpaca-OrcaChat (prompts only) |

Each phase pins different library versions — the book supplies a separate
`pip install` line per step, and that is not incidental.

## Worked Example — Fine-tuning Cohere for medical NER

The chapter's proprietary-model example is the clearest demonstration of *what
fine-tuning buys you over prompting*, because it shows the base and tuned
outputs side by side on the same task.

**Task.** Extract diseases, chemicals, and their relations from PubMed
abstracts — turning unstructured prose into structured records, so that "all
chemicals influencing disease X" becomes a query instead of a literature review.

**Dataset.** BC5CDR (BioCreative V Chemical Disease Relation): 1,500 manually
annotated PubMed papers, split 500/500/500 train/validation/test.

**Baseline — prompting `command` zero-shot:**

```python
import cohere
co = cohere.Client("<API_KEY>")

prompt = """The following article contains technical terms including
diseases, drugs and chemicals. Create a list only of the diseases mentioned:

<abstract text>

List of extracted diseases:"""

response = co.generate(model='command', prompt=prompt,
                       max_tokens=200, temperature=0.750)
```

On a glaucoma abstract the base model returned two diseases — correct, but
shallow, and only after the prompt spelled out the task.

**Fine-tuning data format.** Cohere accepts CSV, JSONL, or plain text. JSONL:

```json
{"prompt": "This is the first prompt", "completion": "This is the first completion"}
{"prompt": "This is the second prompt", "completion": "This is the second completion"}
```

Preprocessing uses only the `abstract` passages, not the `title` passages —
titles are short and carry little detail. Each entry's `entities` list carries
`text` and `type` (`Chemical` / `Disease`), plus `relations`.

**Three tuned models** were built the same way: disease extraction, chemical
extraction, and relation extraction. Across all three the custom model produced
markedly richer, better-structured extractions than the base `command` model on
identical inputs.

The transferable lesson is the shape of the work: the *prompting* effort went
into describing the task; the *fine-tuning* effort went into reformatting an
existing annotated corpus into prompt/completion pairs. Fine-tuning is largely a
data-engineering job.

## Key Takeaways

- Pre-training teaches language; fine-tuning teaches instruction-following.
- LoRA freezes the base and trains low-rank update matrices in the attention
  layers — ~0.2% of parameters, no catastrophic forgetting, consumer GPUs.
- QLoRA adds 4-bit NF4 base weights, double quantization, and paged optimizers:
  65B parameters on a 48GB GPU at full-16-bit quality.
- SFT learns from demonstrations; RLHF learns from rankings through a reward
  model and PPO, with a KL penalty holding it near the original distribution.
- The reward model is a pairwise classifier and can be far smaller than the base.
- RLHF vs. SFT is unresolved: RLHF aligns better with enough good data, but is
  expensive and unstable; LIMA argues 1,000 curated samples via SFT can suffice.
- DPO removes the reward model and the RL loop; ReST decouples generation from
  improvement; RLAIF replaces human rankers with a constitution-guided AI.
- Keep one prompt template across every phase, count your trainable parameters,
  and watch the GPU bill.

## Connects To

- [Chapter XI](ch11-deployment.md) — quantization in depth, and serving the
  fine-tuned model.
- [Chapter VIII](ch08-advanced-rag.md) — when to fine-tune versus retrieve, and
  fine-tuning the embedding model rather than the LLM.
- [Chapter III](ch03-llms-in-practice.md) — RLHF and Constitutional AI as
  hallucination and alignment controls, and the evaluation benchmarks used to
  verify a fine-tune.
- [Chapter II](ch02-llm-architectures-and-landscape.md) — OPT, LLaMA, and the
  open-model landscape these techniques are applied to.
