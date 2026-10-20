# Chapter XI — Deployment

## Core Idea

Deployment is where transformer size becomes a bill and a delay. Two resources
bind: **latency** (users read ~250 words/min ≈ 5 tokens/sec, so **100–200 ms per
token** is the near-real-time target) and **memory** (LLaMA 2 70B at fp16 =
130 GB before you serve a single request). Three families of optimization
attack both — **quantization** (fewer bits per weight), **sparsity/pruning**
(fewer weights), and **distillation** (smaller model) — and the chapter shows
that a well-quantized model on a CPU instance can be a legitimate production
target, not a compromise.

## Frameworks Introduced

### The memory formula

```
memory_GB = (parameters × bytes_per_parameter) / 1024³
```

LLaMA 2 70B at Float16 (2 bytes): `(70e9 × 2) / 1024³ = 130.385 GB`.

### Precision ladder

| Type | Bytes | Effect |
|---|---|---|
| Float64 / Float32 | 8 / 4 | original training standard, maximum accuracy |
| Float16 / BFloat16 | 2 | half the memory, slight accuracy cost |
| int8 (`load_in_8bit`) | 1 | quarter of fp32 |
| 4-bit (NF4) | 0.5 | eighth of fp32 |

**Mixed-precision quantization** — allocating higher bit precision to more
important layers — is the standard way to limit accuracy loss.

### Two classical quantization techniques

**Scalar quantization** — treat each dimension independently: find its min and
max across the dataset, split that range into uniform bins (256 for uint8), and
assign each value to a bin.

**Product quantization** — split each vector into *m* sub-vectors, k-means each
sub-vector group into *k* centroids, replace each sub-vector with its nearest
centroid index. Retains more information than scalar quantization, especially
when dimension scales differ. The trade-off: more centroids improve accuracy but
shrink the memory saving, and vice versa.

Both break down on billion-parameter models — larger models pack more
information per parameter, so precision loss costs more accuracy.

### Three post-training quantization methods for LLMs

| Method | Insight | Mechanism |
|---|---|---|
| **LLM.int8()** | activation **outliers** disrupt quantization at scale | keep the outliers in higher precision |
| **GPTQ** | quantize layer by layer | minimize MSE between quantized and full-precision weights for a given input; **mixed int4-fp16** — weights int4, activations fp16, de-quantized on the fly at inference; **requires a calibration dataset** |
| **AWQ** | not all weights matter equally | identify the **0.1–1% salient** weight channels — selected by **activation magnitude**, not weight distribution — and protect them; then scale before quantizing everything to a uniform dtype, avoiding mixed-precision hardware inefficiency |

AWQ's refinement is the notable one: keeping a few channels in FP16 works but
creates mixed-dtype hardware overhead, so they instead *scale* the outlier
channels before uniform quantization — same protection, no dtype split.

### QLoRA's NF4, explained

Pre-trained weights are approximately **zero-centered normal with some standard
deviation σ**. NF4 uses **quantile quantization**, which puts an equal number of
input-tensor values in each bin — optimal precisely for normally distributed
data. QLoRA standardizes weights by scaling σ so the distribution fits the NF4
range exactly. Result: no measured accuracy loss versus BFloat16.

### Pruning taxonomy

| Type | Target | Trade-off |
|---|---|---|
| **Weight — unstructured (magnitude-based)** | individual small-magnitude weights, any sparsity pattern | highest compression rate; needs specialized hardware to realize speedups |
| **Weight — structured** | channels, kernels, whole neurons | memory/energy/latency gains **without specialized hardware**; lower compression rate |
| **Activation** | redundant activations at inference | good fit for transformers; requires dynamic runtime identification |

### Pruning research the chapter cites

- **Network Trimming** — many neurons output zero regardless of input; those are
  redundant. Prune, then retrain from the *original pre-pruning weights*.
- **Network Slimming** — pruning schemes for deep CNNs targeting size, run-time
  memory, and operation count.
- **Wanda** (Pruning by Weights and activations) — for LLMs: prune the weights
  with the smallest **magnitude × corresponding input activation**, assessed
  per-output. **Requires no retraining or weight updates** — the pruned model is
  immediately usable.
- **Structured Pruning of Deep CNNs** — channel-wise, kernel-wise, and
  intra-kernel strided sparsity; particle filtering scores connections by the
  misclassification rate they cause; retrain afterward.
- **The Lottery Ticket Hypothesis** — inside a large randomly initialized
  network there exist subnetworks ("winning tickets") **10–20% of the size**
  that, trained independently *from their original initialization*, match or
  **outperform** the full network, learning faster and reaching higher test
  accuracy (MNIST, CIFAR10).
- **A Fast Post-Training Pruning Framework for Transformers** — removes the
  retraining step entirely, using structured sparsity plus three techniques:
  a lightweight mask search algorithm, mask rearrangement, and mask tuning.
- **SparseGPT** — ≥50% size reduction **one-shot, no retraining**;
  demonstrated on OPT-175B and BLOOM-176B in **under 4.5 hours**, reaching 60%
  unstructured sparsity — over 100 billion weights removed at inference — with
  only a minor perplexity increase.

### Static vs. dynamic quantization

| Approach | When applied | Best for |
|---|---|---|
| **Static** | to weights, before deployment | fixed data distributions |
| **Dynamic** | at runtime, typically to activations based on the data being processed | **transformers** — adapts to varying activation distributions |

## Key Concepts

- **The latency budget, derived.** 250 words/min ≈ 312 tokens/min ≈ 5 tokens/sec
  ≈ **200 ms/token**. Anything faster than reading speed is invisible to the
  user; anything slower is felt.
- **Quantization is rounding, in one analogy.** "10:58 p.m." versus "about
  11 p.m." — less precise, faster to say, easier to use.
- **ONNX** standardizes AI models for interoperability and efficient CPU
  execution via hardware optimizations.
- **Accuracy-driven tuning.** Intel Neural Compressor lets you declare a
  *tolerable loss* (e.g. 1%) and a trial budget, then searches quantization
  configurations until it meets the target — quantization as a constrained
  search, not a one-shot cast.
- **The evaluation metric must match the task.** Summarization → ROUGE;
  translation → BLEU; classification → accuracy; QA → F1 via `squad`. The
  quantizer needs an `eval_fn` returning a number, so picking that number
  correctly is a prerequisite.
- **The model must already be fine-tuned for its task before quantizing.** The
  book uses an OPT-1.3B already fine-tuned on SQuAD.
- **Quantized tokenizers are unchanged.** Quantization does not touch the
  vocabulary, so load the base model's tokenizer.
- **Fixing `min_length == max_length`** for benchmarking guarantees both models
  emit the same token count, making generation times comparable. It also forces
  a non-instruction-tuned model into repetitive loops — an artifact of the
  benchmark, not the quantization.

### Measured results

| Decoding method | Vanilla (s) | Quantized (s) | Speedup |
|---|---|---|---|
| Greedy | 58.09 | 26.85 | 2.2× |
| Beam Search (K=4) | 144.77 | 40.73 | **3.5×** |

Hardware: 4th Gen Intel Xeon Scalable, 8 vCPU (4 cores), 64 GB, batch size 1.

## Mental Models

- **Three levers, one goal.** Quantization shrinks each weight; pruning removes
  weights; distillation replaces the model. They compose.
- **Latency is a user-experience budget, not an engineering score.** 200 ms per
  token is derived from human reading speed, which is why it is the number to
  design against.
- **Outliers are the whole quantization problem.** LLM.int8() protects them by
  precision, AWQ by scaling — both exist because a handful of large-magnitude
  channels dominate the error.
- **The Lottery Ticket reframes pruning.** The small network was always in
  there; pruning finds it rather than damaging the big one.
- **CPU is a real deployment target.** A 3.5× speedup from dynamic quantization
  on Xeon changes the cost equation for workloads that do not need a GPU.

## Anti-patterns

- **Scalar or product quantization on a billion-parameter model.** Adequate for
  small models; significant accuracy loss at LLM scale — which is why GPTQ, AWQ,
  and LLM.int8() exist.
- **Static quantization for transformers.** Activation distributions vary with
  the data; dynamic is the preferred approach.
- **Quantizing without an accuracy gate.** `AccuracyCriterion(tolerable_loss=…)`
  exists so you find out during quantization, not in production.
- **Unstructured pruning without hardware support.** Higher compression on
  paper, no speedup in practice unless the hardware exploits the sparsity
  pattern. Structured pruning gives real gains on ordinary hardware.
- **Leaving cloud instances running.** Stated for both GCP and AWS: shut down
  idle instances.
- **Assuming quantization is free.** "There's a constant trade-off between using
  lower-precision numbers and potentially diminishing the language processing
  capabilities of models."

## Code Examples

### Scalar quantization from scratch

```python
import numpy as np

def scalar_quantisation(dataset):
    ranges = np.vstack((np.min(dataset, axis=0), np.max(dataset, axis=0)))
    starts = ranges[0, :]
    steps = (ranges[1, :] - starts) / 255      # 256 bins for uint8
    return np.uint8((dataset - starts) / steps)
```

### Product quantization

```python
from sklearn.cluster import KMeans
import numpy as np

m, k = 3, 2                       # sub-vectors, centroids
subvectors = array.reshape(-1, m)
kmeans = KMeans(n_clusters=k, random_state=0).fit(subvectors)
quantized_array = kmeans.labels_.reshape(array.shape[0], -1)
# array([[0, 1, 1],
#        [0, 0, 0]], dtype=int32)
```

The motivating failure: scalar-quantizing a two-row array whose rows span
wildly different scales (8–310 vs 0.1–9.7) to 4 bits collapsed the entire second
row to zeros. Product quantization preserved the distinction.

### One-line dynamic quantization

```bash
pip install git+https://github.com/huggingface/optimum-intel.git@v1.11.0
pip install onnx==1.14.1 neural_compressor==2.2.1

optimum-cli inc quantize --model facebook/opt-1.3b --output opt1.3b-quantized
```

Add `--task` if the script cannot infer the model's task.

### Accuracy-constrained quantization

```python
import evaluate
from datasets import load_dataset
from transformers import pipeline
from neural_compressor.config import (AccuracyCriterion, TuningCriterion,
                                      PostTrainingQuantConfig)
from optimum.intel import INCQuantizer

task_evaluator = evaluate.evaluator("question-answering")
eval_dataset = load_dataset("squad", split="validation").select(range(64))
qa_pipeline = pipeline("question-answering", model=model, tokenizer=tokenizer)

def eval_fn(model):
    qa_pipeline.model = model
    metrics = task_evaluator.compute(model_or_pipeline=qa_pipeline,
                                     data=eval_dataset, metric="squad")
    return metrics["f1"]

quantization_config = PostTrainingQuantConfig(
    approach="dynamic",
    accuracy_criterion=AccuracyCriterion(tolerable_loss=0.01),   # ≤1% F1 loss
    tuning_criterion=TuningCriterion(max_trials=10),
)

quantizer = INCQuantizer.from_pretrained(model, eval_fn=eval_fn)
quantizer.quantize(quantization_config=quantization_config,
                   save_directory="opt1.3b-quantized")
```

### Loading and running a quantized model

```python
from transformers import AutoTokenizer
from optimum.intel import INCModelForCausalLM

tokenizer = AutoTokenizer.from_pretrained("facebook/opt-1.3b")   # unchanged
model = INCModelForCausalLM.from_pretrained("./opt1.3b-quantized")

inputs = tokenizer("<PROMPT>", return_tensors="pt")
generation_output = model.generate(**inputs,
                                   return_dict_in_generate=True,
                                   output_scores=True,
                                   min_length=512, max_length=512,
                                   num_beams=1, do_sample=True,
                                   repetition_penalty=1.5)
print(tokenizer.decode(generation_output.sequences[0]))
```

Task-specific loaders: `INCModelForCausalLM`,
`INCModelForSequenceClassification`, `INCModelForQuestionAnswering`.

## Reference Tables

### Optimization tooling

| Library | Role |
|---|---|
| **Hugging Face Optimum** | bridge between `transformers`/`diffusers` and hardware-optimized runtimes |
| **Intel Neural Compressor** | quantization (static/dynamic/aware-training), pruning, knowledge distillation, with automatic accuracy-driven tuning |
| **optimum-intel** | the Optimum ↔ INC integration (`optimum-cli inc quantize`, `INCQuantizer`, `INCModel*`) |
| **BitsAndBytes** | 4-bit/8-bit loading |
| **ONNX Runtime** | INC's path to AMD CPU, ARM CPU, and Nvidia GPU (limited testing) |
| **Intel Extension for PyTorch / CPU Extension for transformers** | further Intel-specific optimization |

### Serving frameworks

| Option | Character |
|---|---|
| **vLLM** | purpose-built LLM serving — optimization, request batching, memory management |
| **TorchServe** | model serving for PyTorch, endpoint creation from multiple sources |
| **FastAPI** | general backend library — full control, build the API yourself |

APIs are the integration point: they let LLMs plug into web and mobile clients
for real-time interaction.

## Worked Example — Deploying a quantized LLM on a cloud CPU

The chapter's end-to-end path, and its point is that the cheapest hardware is
often sufficient once the model is optimized.

1. **Pick an already-fine-tuned model for the target task.** Here,
   `aman-mehra/opt-1.3b-finetune-squad-…` — question answering.
2. **Choose the metric that defines "still good enough."** QA → F1 on SQuAD.
   Wire it into an `eval_fn` that takes a model and returns a number.
3. **Declare the accuracy budget and trial budget.** 1% tolerable loss, 10
   trials, `approach="dynamic"` (transformers).
4. **Quantize.** `INCQuantizer.from_pretrained(model, eval_fn=eval_fn)` then
   `.quantize(...)`. The library searches configurations until it meets the gate.
5. **Benchmark honestly.** Fix `min_length = max_length = 512` so both models
   emit exactly the same token count. Result on 8 vCPU Xeon: greedy
   58.09 s → 26.85 s; beam search K=4 144.77 s → **40.73 s (3.5×)**.
6. **Deploy on GCP Compute Engine:** enable the Compute Engine API, create a
   CPU instance, SSH in, install dependencies, transfer the server code (FastAPI
   or vLLM), launch. The same flow applies to AWS EC2. **Shut the instance down
   when idle** — billing is by resource-hour.

The sequencing matters: fine-tune → choose metric → set accuracy gate →
quantize → benchmark with a controlled token count → deploy. Skipping the gate
means learning about the quality regression from users.

## Key Takeaways

- Target ~100–200 ms per token; that number comes from human reading speed.
- Memory ≈ parameters × bytes per parameter. Cutting bytes per parameter is the
  highest-leverage single change.
- Scalar and product quantization are fine for small models and vector stores;
  LLMs need LLM.int8(), GPTQ, or AWQ, all of which are really about handling
  activation outliers.
- Dynamic quantization suits transformers because activation distributions
  shift with the input.
- Quantize against an explicit accuracy budget with a task-appropriate metric.
- Pruning is the other axis: structured for hardware-independent gains,
  unstructured for maximum compression. Wanda and SparseGPT prune LLMs with no
  retraining; the Lottery Ticket Hypothesis explains why small subnetworks work
  at all.
- CPU deployment is viable — a 3.5× beam-search speedup on Xeon, from a single
  quantization pass.
- Serve through an API: vLLM or TorchServe for LLM-specific features, FastAPI
  for full control.

## Connects To

- [Chapter X](ch10-fine-tuning.md) — QLoRA's NF4 and double quantization, and
  the fine-tuned model that becomes the deployment artifact.
- [Chapter VIII](ch08-advanced-rag.md) — the CPU-inference cost argument and the
  latency/quality balance in production RAG.
- [Chapter II](ch02-llm-architectures-and-landscape.md) — model sizes and the
  open-model landscape that determine the memory bill.
- [Chapter III](ch03-llms-in-practice.md) — decoding methods (greedy, beam
  search) whose costs are measured here.
