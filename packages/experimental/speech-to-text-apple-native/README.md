---
description: "Transcribe browser recordings through a local Apple SpeechAnalyzer MCP process."
kind: "package-reference"
---

# @deepseek-ai/dsh-experimental-speech-to-text-apple-native

English | [中文](README.zh.md)

## Summary

This macOS-only Host provider sends the browser's bounded recording to the configured local `apple-native-mcp` executable for on-device SpeechAnalyzer transcription. It is absent from the provider list when the executable is not installed or the Host is not macOS.

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

The [voice input bundle](../voice-input-bundle/README.md) selects the executable under the DSH home and advertises configured locale hints. Install `apple-native-mcp` at that path, enable Voice Input in Plugins, then select Apple on-device speech and a locale in bundle details. Browser microphone permission still applies. The Host needs macOS SpeechTranscriber availability; the Apple runtime may install language assets on first use, independently of SenseVoice preparation. `command`, `locales`, `providerId`, and `timeoutMs` are plugin config fields.

-----

<a id="understand-the-implementation"></a>
## Understand the implementation

<details>
<summary>Maintainer details — click to expand</summary>

Each recognition call starts a dedicated local MCP process, writes the WAV in an owner-only temporary directory and parses the `transcribe_file` result. Cancellation closes the process and cleanup waits for closure before removing the WAV. The temporary recording is not a Session attachment or a model-facing MCP tool call. Selecting an unavailable provider fails explicitly without fallback.

</details>

-----

<a id="further-exploration"></a>
## Further Exploration

[Voice input subsystem](../../../docs/subsystems/voice-input.md)

-----

<a id="model-experience"></a>
## Model Experience

None, as local recognition returns transient text without changing model requests; only the user's later submission is model-visible.

#### KV Cache effect

No direct effect.

## Known Limitations and Deferred Work

<a id="known-limitations-and-deferred-work"></a>

- The native executable is a separately installed local dependency, not bundled with DSH. Its runtime determines which locales and on-device assets are available; configure only supported locale hints. Each recording starts a process rather than sharing a long-lived connection.

-----

<a id="dev-note"></a>
### Dev Note

<details>
<summary>Maintainer details — click to expand</summary>

None.

</details>
