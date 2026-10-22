---
description: "The Laya decision provider runs a cached local typed-decision checkpoint only after the primary decision provider fails, then releases its memory after an idle timeout."
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-laya

English | [中文](README.zh.md)

## Summary

`dsh-decision-laya` answers [`ctx.decision`](../decision/README.md) questions with the local Apache-2.0 Laya model. Choose it as a configured fallback when Jev is the primary provider and decision work must continue through a TypeSafe outage or quota failure. The package starts no model on a healthy primary path: its provider launches the Python worker only after the decision service invokes Laya, and the worker exits after its idle timeout. A cold fallback pays model-load latency and requires cached weights, while warm calls avoid network inference cost.

## Table of Contents

- [Use this package](#use-this-package)
- [Understand the implementation](#understand-the-implementation)
- [Further Exploration](#further-exploration)
- [Model Experience](#model-experience)
- [Known Limitations and Deferred Work](#known-limitations-and-deferred-work)
- [Dev Note](#dev-note)

-----

<a id="use-this-package"></a>
## Use this package

Mount Laya beside the decision service, set `provider: jev`, and set `fallbackProvider: laya`; the service calls Laya only after Jev cannot answer.

### When to choose it

Choose this provider when the host can run PyTorch locally and tolerate cold-start latency in exchange for an offline fallback with no per-call API charge. Avoid it as a general text model: Laya returns only `choice`, `score`, and truth-probability answers. For decision tasks, the default checkpoint is the repository's `typed-decisions` subfolder rather than the weaker root checkpoint.

### Minimal configuration

```yaml
- name: '@deepseek-ai/dsh-decision'
  config:
    provider: jev
    fallbackProvider: laya
- name: '@deepseek-ai/dsh-decision-jev'
- name: '@deepseek-ai/dsh-decision-laya'
```

| Field | Default | Meaning |
|---|---|---|
| `pythonPath` | `~/.dsh/laya-env/bin/python`, else `python3` | Python interpreter containing `laya`, PyTorch, Transformers, and Safetensors |
| `workerScriptPath` | `~/.dsh/bin/laya-worker.py` | On-demand HTTP worker executable |
| `host` / `port` | `127.0.0.1` / `58085` | Worker listen address |
| `idleTimeoutSeconds` | `300` | Exit delay after the last decision evaluation |
| `startupTimeoutMs` | `60000` | Maximum cold-start wait |
| `healthTimeoutMs` | `1000` | Timeout for each readiness probe |
| `startupPollIntervalMs` | `500` | Delay between readiness probes |
| `baseURL` | derived from `host` and `port` | Worker API base URL |
| `model` | `convaiinnovations/laya` | Cached local path or Hugging Face repository |
| `modelSubfolder` | `typed-decisions` | Checkpoint specialized for typed decisions |

The generated [configuration catalog](../../../docs/config-catalog.md#deepseek-aidsh-decision-laya) is the exhaustive source for accepted fields.

-----

<a id="understand-the-implementation"></a>
## Understand the implementation

<details>
<summary>Implementation internals — click to expand</summary>

The TypeScript provider first checks the worker health endpoint. If no worker answers, one in-process startup lock launches the configured Python executable and polls until the checkpoint is ready. The worker binds its loopback port before loading weights, so concurrent DSH processes do not each load a model after a startup race. Health probes do not reset the idle timer; only a decision evaluation does. The worker loads cached Hugging Face files with `local_files_only`, answers the same System One question format as Jev, and terminates after inactivity.

| File | Responsibility |
|---|---|
| [`src/provider.ts`](src/provider.ts) | Provider translation, worker activation, readiness, and HTTP evaluation |
| [`src/worker.py`](src/worker.py) | Cached Laya loading, local HTTP service, and idle shutdown. Repository source only: the published tarball omits it, so a deployment supplies the script through `workerScriptPath` |
| [`src/index.ts`](src/index.ts) | Cordis configuration and provider registration |

</details>

-----

<a id="further-exploration"></a>
## Further Exploration

- [`@deepseek-ai/dsh-decision`](../decision/README.md) — primary/fallback provider selection and question types.
- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.md) — the primary TypeSafe Jev provider.
- [Decision subsystem](../../../docs/subsystems/decision.md) — capability semantics and errors.
- [Laya model card](https://huggingface.co/convaiinnovations/laya) — upstream checkpoints, benchmarks, and license.

-----

<a id="model-experience"></a>
## Model Experience

None, as this provider runs outside the conversation model context and returns typed decisions only.

#### KV Cache effect

No effect; Laya uses an independent encoder and does not read or invalidate the conversation model's KV cache.

## Known Limitations and Deferred Work

<a id="known-limitations-and-deferred-work"></a>

- **Cold activation is intentionally slow.** The model is absent from memory on the healthy Jev path; the first fallback must load the checkpoint before answering.
- **Weights must already be cached.** The worker uses local-only Hugging Face resolution so an upstream outage cannot make failover depend on a download.
- **Calibration is checkpoint-specific.** Thresholds tuned for Jev must be revalidated against Laya before safety-sensitive consumers rely on equivalent numeric confidence.
- **The worker is host-local.** This package does not manage a remote Laya deployment or distribute Python dependencies.

<a id="dev-note"></a>
### Dev Note

<details>
<summary>Working context for maintainers — click to expand</summary>

None.

</details>
