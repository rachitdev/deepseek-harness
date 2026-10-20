# Agent Note: Decision prompts as request-only inputs

Status: implemented

English | [中文](2026-09-24-decision-request-only-inputs.zh.md)

## Problem

`dsh-decision-llm` and the live failover in `dsh-decision-consumers` sent their auxiliary prompts as durable user messages with producer source kinds `dsh-decision-llm` and `dsh-decision-consumers`. Those kinds joined `MessageSourceMap`, so the persistence inventory recorded two new user-source variants on `user/message`, `developer/message`, `agent/inbox/spliced`, and `session/title-llm-request`. `verify-persistence-changes` rejected the change as breaking against the accepted Session format 4 baseline, although neither prompt is ever written to a Session.

## Decision

- Both prompts are `RequestUserInput` values (`{ role: 'user', content }`, no `id`, no `source`), the type `dsh-llm` provides for user input used by one request only.
- Neither package declares a `MessageSourceMap` member, so the Session format 4 inventory is unchanged and `verify-persistence-changes` matches the accepted history.
- The decision prompt is built and discarded inside the provider call. The failover probe text is a bounded projection of the durable `user/message` event, so every probe input stays reconstructable from the Session log.

## Alternatives considered

**Keep the source kinds and bump the Session format to 5.** A format bump signals a durable change that does not exist and would force a migration for every stored Session.

**Keep the source kinds and acknowledge the change as non-breaking.** The inventory would record variants that no writer produces, and the acknowledgement would misstate the Session format.

## Testing

`pnpm exec vitest run packages/decision` passes 250 tests; `pnpm run verify-persistence-changes` reports 62 roots matching 6 history records; `pnpm exec tsc -b tsconfig.host.json` passes.

## Consequences

A future decision producer that must persist its prompt declares its own `MessageSourceMap` kind together with a Session format change. Request-only prompts from either package never appear in a Session log, `dsh sessions` output, or the persistence inventory.
