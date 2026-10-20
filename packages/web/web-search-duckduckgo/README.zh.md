---
description: "面向 ctx.web 的 DuckDuckGo 搜索提供方：一个无需密钥的 Lite HTML 后端，把自然结果解析为 seam 规范化的 WebSearchResult。"
kind: "package-reference"
---

# @deepseek-ai/dsh-web-search-duckduckgo

[English](README.md) | 中文

<a id="summary"></a>
## 概述

基于 [DuckDuckGo](https://duckduckgo.com) 的 `WebSearchProvider`，面向 [web 能力 seam](../web/README.zh.md)（`ctx.web`）。它抓取免费的 DuckDuckGo Lite HTML 结果页（`GET https://lite.duckduckgo.com/lite/?q=<query>`），把自然结果表解析为 seam 规范化的 `WebSearchResult`。它不需要 API 密钥，因此可作为零成本后端，也可在未配置付费凭据时作为后备。它向 `ctx.web` 注册提供方，既不拥有 `ctx.web`，也不注册面向模型的工具；该工具属于 `@deepseek-ai/dsh-tool-web`。

## 目录

- [概述](#summary)
- [配置](#config)
- [映射](#mapping)
- [开发备注](#dev-note)
- [模型体验](#model-experience)
- [已知限制与延期工作](#known-limitations-and-deferred-work)

-----

<a id="config"></a>
## 配置

| 键 | 默认值 | 含义 |
|---|---|---|
| `baseURL` | `https://lite.duckduckgo.com/lite/` | 提供 Lite HTML 页面的端点基址；查询字符串附加其后。无法解析的值使该提供方不可用。 |
| `region` | （未设置） | 作为 `kl` 参数发送的地区过滤（如 `us-en`）。未设置则不发送。存在时必须是 `xx-yy` 形式的语言-地区标签。 |

```yaml
- id: web-search-duckduckgo
  name: '@deepseek-ai/dsh-web-search-duckduckgo'
```

<a id="mapping"></a>
## 映射

DuckDuckGo Lite 把每条自然结果作为锚点放在同一页面上（`result-link`／`result-snippet` class），每条结果映射为一个 `WebSearchSource`：`url` ← 解开 `/l/?uddg=` 重定向包装后的锚点 href，`title` ← 链接文本，`snippet` ← 该行的摘要文本（无摘要的行映射时不带摘要）。没有链接、没有可解码的 http(s) 目标 URL 或标题为空的行没有可移植内容，会被丢弃。DuckDuckGo 不返回生成答案，因此省略 `content`；Lite 在服务端排序，因此请求的 `maxResults` 由 seam 的截断强制执行，而非本提供方。失败以 `WebError` 呈现：网络故障、HTTP 错误和超大响应体为 `WEB_PROVIDER_ERROR`；被中止的请求为 `WEB_ABORTED`；DuckDuckGo 的 202 异常页（机器人限流）为一条指明限流的 `WEB_PROVIDER_ERROR` 消息。HTTP 重定向在联系 `Location` 目标之前即被拒绝。

<a id="dev-note"></a>
## 开发备注

该提供方抓取的是非官方 HTML 布局而非调用有文档的 API，这既是它能在没有凭据的情况下发布的原因，也是它不是生产默认选项的原因。让解析规则与限流检测路径持续受 fixture 覆盖，才能使布局变化表现为测试失败，而不是静默返回空结果集。

<a id="model-experience"></a>
## 模型体验

间接地，通过 [`dsh-tool-web`](../tool-web/README.zh.md)，它保留本提供方的 URL、标题与摘要，或在消费者的错误包装下保留其确切的 `DuckDuckGo search aborted`、`DuckDuckGo search request failed: <error>`、`DuckDuckGo API error (HTTP <status>)` 与 `DuckDuckGo returned an unprocessable response body: <error>` 失败信息；页面外壳与提供方私有标记不会进入上下文。

#### KV 缓存影响

无直接失效；由具名消费者拥有任何请求前缀变化。

<a id="known-limitations-and-deferred-work"></a>
## 已知限制与延期工作

- **抓取契约** —— Lite 的 HTML 布局是非官方的，可能在没有通知的情况下变化；布局变化表现为空结果或解析漂移，而不是 API 错误。官方付费 API（Exa、Perplexity）仍是生产部署的受支持后端。
- **无发布日期** —— Lite 不暴露每条结果的时间戳，因此永远不会映射 `publishedAt`。
- **机器人限流** —— 自动化请求模式可能收到异常页；该提供方将其报告为限流错误而不重试。
