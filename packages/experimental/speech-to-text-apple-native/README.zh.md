---
description: "通过本地 Apple SpeechAnalyzer MCP 进程转写浏览器录音。"
kind: "package-reference"
---

# @deepseek-ai/dsh-experimental-speech-to-text-apple-native

[English](README.md) | 中文

## 概述

此 macOS Host Provider 将浏览器的有界录音交给配置的本地 `apple-native-mcp` 可执行文件，使用设备端 SpeechAnalyzer 转写。可执行文件未安装或 Host 不是 macOS 时，识别服务列表不显示此 Provider。

## 目录

- [使用此包](#use-this-package)
- [理解实现](#understand-the-implementation)
- [进一步探索](#further-exploration)
- [模型体验](#model-experience)
- [已知限制与后续工作](#known-limitations-and-deferred-work)
- [开发备注](#dev-note)

-----

<a id="use-this-package"></a>
## 使用此包

[语音输入 Bundle](../voice-input-bundle/README.zh.md) 指定 DSH 主目录下的可执行文件并公布配置的语言区域。将 `apple-native-mcp` 安装到该路径，在“插件”中启用语音输入，再在 Bundle 详情中选择 Apple 设备端语音及语言区域。浏览器仍需麦克风权限。Host 需要 macOS SpeechTranscriber 可用；Apple 运行时首次使用时可能独立安装语言资源，不属于 SenseVoice 准备流程。`command`、`locales`、`providerId` 与 `timeoutMs` 是插件配置字段。

-----

<a id="understand-the-implementation"></a>
## 理解实现

<details>
<summary>维护者信息 — 点击展开</summary>

每次识别启动一个独立的本地 MCP 进程，把 WAV 写入仅所有者可访问的临时目录，并解析 `transcribe_file` 结果。取消会关闭进程，等待关闭后清理 WAV。临时录音不是 Session 附件，也不会调用面向模型的 MCP 工具。选择不可用 Provider 时明确失败，不会回退。

</details>

-----

<a id="further-exploration"></a>
## 进一步探索

[语音输入子系统](../../../docs/subsystems/voice-input.zh.md)

-----

<a id="model-experience"></a>
## 模型体验

无。识别不会将音频或文字发送给 Agent；只有用户之后提交的内容对模型可见。

#### KV 缓存影响

无直接影响。

## 已知限制与后续工作

<a id="known-limitations-and-deferred-work"></a>

- 原生可执行文件是单独安装的本地依赖，不包含在 DSH 内。可用语言区域及设备端资源取决于其运行时；仅配置受支持的语言区域。每次录音都会启动进程，不会复用长连接。

-----

<a id="dev-note"></a>
### 开发备注

<details>
<summary>维护者信息 — 点击展开</summary>

无。

</details>
