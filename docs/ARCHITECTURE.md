# Architecture — schoolfees contract

**This page does not describe the system.** The architecture of `schoolfees` is
documented once, in the docs repo, and it is the source of truth:

- [`schoolfees-docs/src/architecture.md`](https://github.com/stellar-schoolfees/schoolfees-docs/blob/main/src/architecture.md)
  — system position, runtime topology, the stored types, the lifecycle, trust
  boundaries and the design invariants, every claim pointing at a file, function
  or test.
- [`docs/design/interface-v0.md`](design/interface-v0.md) — the approved
  interface: roles, storage, TTL policy, state machine, functions, events,
  errors, the test plan and the thirteen recorded defaults.
- [`schoolfees-docs/src/threat-model.md`](https://github.com/stellar-schoolfees/schoolfees-docs/blob/main/src/threat-model.md)
  — the STRIDE walk-through.

What follows is only what those pages do not carry: how this repository is
laid out, and where its decisions live.

## Module map

| File | Owns | Must not contain |
|---|---|---|
| `src/lib.rs` | `#[contract]` / `#[contractimpl]` and thin delegating calls to `fee::*` | logic, storage access, arithmetic |
| `src/fee.rs` | the whole fee lifecycle: validation, storage writes, token transfers, events | new types or keys (they live in the modules below) |
| `src/types.rs` | `enum Error` with its numbered ranges, `#[contracttype]` stored types, `#[contractevent]` events | logic |
| `src/storage.rs` | `DataKey`, TTL constants, `extend_instance_ttl` / `extend_record_ttl`, `record_ttl_target` | business rules |
| `src/error_paths.rs` | exactly one `error_path_<variant>` test per `Error` variant | production code |
| `src/test.rs` | lifecycle and integration tests | fixtures another module needs (those are in `test_helpers.rs`) |
| `src/test_helpers.rs` | shared test setup, including the real Stellar Asset Contract | production code |
| `ERRORS.md` | one row per variant, including the user-facing wording every other repo quotes | — |

`#![no_std]`. `soroban-sdk` types only.

## Request path

A call arrives at the `#[contractimpl]` wrapper in `src/lib.rs`, which delegates
straight to `src/fee.rs` (or, for `initialize`/`admin`, handles it locally).
`fee.rs` validates input, loads or writes records through `src/storage.rs` keys,
extends TTLs, calls the token contract where value moves, and publishes the
event from `src/types.rs`. Errors never return a success value: they return
`Err(Error::…)`, which the host turns into a failed invocation.

## Decisions and deferrals

| Decision | Record |
|---|---|
| Dependencies: `soroban_sdk::token` directly, no OpenZeppelin crates | [`docs/decisions/0001-openzeppelin-and-token-dependencies.md`](decisions/0001-openzeppelin-and-token-dependencies.md) |
| The v0 interface and its thirteen defaults (refunds on closed fees, payments after the due date, who may create a fee, token choice, TTL numbers, and so on) | [`docs/design/interface-v0.md`](design/interface-v0.md) §10, all approved |
| What is deliberately **not** in v0, each with its own issue draft | `ROADMAP.md` |
| Boundaries treated as permanent rather than pending (no fee editing, no rate limiting, no upgrade or pause machinery) | `ROADMAP.md`, "Explicitly out of scope" |
| Deferring the choice of token and the key holders to pilot time | [`DEPLOYMENT_CHECKLIST.md`](DEPLOYMENT_CHECKLIST.md) §3–§4 |

A new decision is recorded in `docs/decisions/` as it is made, not only when it
is obvious. A deferred decision is recorded wherever it will resurface — an
issue draft, the roadmap, or the deployment checklist — never left implicit.
