---
description: "Records a persistence type transition and its compatibility acknowledgement."
kind: persistence-change
---

# 2026-10-22-delegation-tool-name

English | [中文](2026-10-22-delegation-tool-name.zh.md)

## Summary

Adds a `subagent-delegation` message source that names the child session, a summary, and the tool whose delegation pin refused that child's request. The whole source arm is new: no released session format carries it, and it is absent from the 2026-10-13 baseline.

## Table of Contents

- [Declaration](#declaration)
- [Compatibility](#compatibility)
- [Verification](#verification)
- [Dev Note](#dev-note)

<a id="declaration"></a>
## Declaration

```yaml persistence-change
schemaVersion: 1
id: 2026-10-22-delegation-tool-name
baseline: false
changes:
  - root: "event:agent/inbox/spliced"
    previous: "2026-10-13-user-rpc-principal"
    after: "88992ac20b18c962fb8c70a8f0226d9d2cb403abedfa90e92fb726c50e19fd80"
    decision: same-version
  - root: "event:developer/message"
    previous: "2026-10-13-user-rpc-principal"
    after: "2ca77a7c10dcdb65df3ecca451c1923d60209950ebca9e5f2aab189d7ef731e2"
    decision: same-version
  - root: "event:session/title-llm-request"
    previous: "2026-10-13-user-rpc-principal"
    after: "6689b4bc09d4e8877b2e71b0e8e35b34e258faafafbfb3c2a6b6787df9dac25e"
    decision: same-version
  - root: "event:user/message"
    previous: "2026-10-13-user-rpc-principal"
    after: "ccb9500b8d3eb89682d4dd62122cded55cb9bfce21339787138ce9db166e0fb4"
    decision: same-version
```

<a id="compatibility"></a>
## Compatibility

No existing record can contain this source kind, so both reader directions are safe. The shipped reader needs only a non-empty `kind`, and a consumer that switches on a message source kind falls through its default arm for a kind it does not know; the notice itself renders from `summary`. This change adds a source kind rather than a field on a released one, so it introduces no absent-field case and keeps the header version the same.

<a id="verification"></a>
## Verification

pnpm exec vitest run packages/subagent/subagent/tests/delegation-notice.spec.ts: 7 tests passed. `git tag --contains HEAD` prints nothing. A literal diff between this record's schema snapshot and the 2026-10-13 record's shows `subagent-delegation` as the only new literal across the four affected roots.

<a id="dev-note"></a>
## Dev Note

None.
