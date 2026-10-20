---
description: "Laya decision 提供方仅在主要 decision 提供方失败后运行缓存的本地 typed-decision 检查点，并在空闲超时后释放内存。"
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-laya

[English](README.md) | 中文

## 概述

`dsh-decision-laya` 使用本地 Apache-2.0 Laya 模型回答 [`ctx.decision`](../decision/README.zh.md) 问题。当 Jev 是主要提供方，并且 decision 工作必须在 TypeSafe 中断或配额故障期间继续时，请把它配置为后备。主要路径健康时，该包不会启动模型：只有 decision 服务调用 Laya 后，提供方才启动 Python worker，worker 在空闲超时后退出。冷后备需要支付模型加载延迟并要求权重已缓存，热调用则不产生网络推理费用。

## 目录

- [使用该包](#use-this-package)
- [理解实现](#understand-the-implementation)
- [延伸阅读](#further-exploration)
- [模型体验](#model-experience)
- [已知限制与延期工作](#known-limitations-and-deferred-work)
- [开发备注](#dev-note)

-----

<a id="use-this-package"></a>
## 使用该包

将 Laya 与 decision 服务一起挂载，设置 `provider: jev` 和 `fallbackProvider: laya`；只有 Jev 无法回答时，服务才调用 Laya。

### 何时选择

当主机可以在本地运行 PyTorch，并且可以接受冷启动延迟以换取无单次 API 费用的离线后备时，请选择该提供方。不要把它当作通用文本模型：Laya 只返回 `choice`、`score` 和真值概率答案。对于 decision 任务，默认检查点是仓库的 `typed-decisions` 子目录，而不是能力较弱的根检查点。

### 最小配置

```yaml
- name: '@deepseek-ai/dsh-decision'
  config:
    provider: jev
    fallbackProvider: laya
- name: '@deepseek-ai/dsh-decision-jev'
- name: '@deepseek-ai/dsh-decision-laya'
```

| 字段 | 默认值 | 含义 |
|---|---|---|
| `pythonPath` | `~/.dsh/laya-env/bin/python`，否则 `python3` | 包含 `laya`、PyTorch、Transformers 和 Safetensors 的 Python 解释器 |
| `workerScriptPath` | `~/.dsh/bin/laya-worker.py` | 按需 HTTP worker 可执行文件 |
| `host` / `port` | `127.0.0.1` / `58085` | Worker 监听地址 |
| `idleTimeoutSeconds` | `300` | 最后一次 decision 评估后的退出延迟 |
| `startupTimeoutMs` | `60000` | 最大冷启动等待时间 |
| `healthTimeoutMs` | `1000` | 每次就绪探测的超时时间 |
| `startupPollIntervalMs` | `500` | 就绪探测之间的延迟 |
| `baseURL` | 从 `host` 和 `port` 派生 | Worker API 基础 URL |
| `model` | `convaiinnovations/laya` | 缓存的本地路径或 Hugging Face 仓库 |
| `modelSubfolder` | `typed-decisions` | 针对 typed decisions 的检查点 |

生成的[配置目录](../../../docs/config-catalog.zh.md#deepseek-aidsh-decision-laya)是所有可接受字段的完整来源。

-----

<a id="understand-the-implementation"></a>
## 理解实现

<details>
<summary>实现内部细节——点击展开</summary>

TypeScript 提供方先检查 worker 健康端点。如果没有 worker 响应，一个进程内启动锁会启动配置的 Python 可执行文件，并轮询直到检查点就绪。Worker 在加载权重前绑定回环端口，因此并发 DSH 进程不会在启动竞争后各自加载一个模型。健康探测不会重置空闲计时器；只有 decision 评估会重置。Worker 以 `local_files_only` 加载缓存的 Hugging Face 文件，回答与 Jev 相同的 System One 问题格式，并在空闲后终止。

| 文件 | 责任 |
|---|---|
| [`src/provider.ts`](src/provider.ts) | 提供方转换、worker 激活、就绪检查和 HTTP 评估 |
| [`src/worker.py`](src/worker.py) | 缓存的 Laya 加载、本地 HTTP 服务和空闲关闭 |
| [`src/index.ts`](src/index.ts) | Cordis 配置与提供方注册 |

</details>

-----

<a id="further-exploration"></a>
## 延伸阅读

- [`@deepseek-ai/dsh-decision`](../decision/README.zh.md) — 主要/后备提供方选择与问题类型。
- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.zh.md) — 主要 TypeSafe Jev 提供方。
- [Decision 子系统](../../../docs/subsystems/decision.zh.md) — 能力语义与错误。
- [Laya 模型卡](https://huggingface.co/convaiinnovations/laya) — 上游检查点、基准和许可证。

-----

<a id="model-experience"></a>
## 模型体验

无，因为该提供方在对话模型上下文之外运行，并且只返回带类型的 decision。

#### KV Cache 影响

无影响；Laya 使用独立编码器，不读取或使对话模型的 KV cache 失效。

## 已知限制与延期工作

<a id="known-limitations-and-deferred-work"></a>

- **冷激活有意保持较慢。** Jev 健康路径不会把模型驻留在内存中；第一次后备必须先加载检查点再回答。
- **权重必须已经缓存。** Worker 使用仅本地 Hugging Face 解析，因此上游故障不会让 failover 依赖下载。
- **校准取决于检查点。** 在安全敏感的消费方依赖同等数值置信度之前，必须针对 Laya 重新验证为 Jev 调整的阈值。
- **Worker 仅在主机本地运行。** 该包不管理远程 Laya 部署，也不分发 Python 依赖。

<a id="dev-note"></a>
### 开发备注

<details>
<summary>维护者工作上下文——点击展开</summary>

无。

</details>
