# 0001. Token transfers and OpenZeppelin dependencies

Date: 2026-09-30
Status: accepted — approved with the v0 interface draft
([`docs/design/interface-v0.md`](../design/interface-v0.md))

## Context

`AGENTS.md` says to prefer audited OpenZeppelin Stellar crates over hand-written
token, access-control or governance code, to evaluate before use, to pin exact
versions, and to record the decision here.

The v0 fee lifecycle needs two things: move an existing SEP-41 token between a
payer and a school, and authorize those two parties. It does **not** issue a
token, and it has no single owner, role model, pause, upgrade path or
governance requirement. The repo resolves `soroban-sdk 28.0.0` (see
`Cargo.lock`) and builds with the Stellar CLI 28.1.0 in CI.

## Evaluation (checked 2026-09-30)

| Crate (latest stable) | What it provides | Verdict for v0 |
|---|---|---|
| `stellar-tokens` 0.7.2 | Fungible, non-fungible, RWA and vault **token implementations**. | Not applicable. v0 consumes an existing token by calling the token contract itself; there is no token logic to implement. |
| `stellar-access` 0.7.2 | `Ownable`, role-based access control, role transfer. | Not applicable. There is no contract-level owner or role: each fee names its own school, and authority is enforced per call with `Address::require_auth`. A single-owner admin would add a privileged actor the design deliberately does not have. |
| `stellar-contract-utils` 0.7.2 | Pausable, Upgradeable, crypto/math helpers, fee abstraction. | Not applicable. v0 has no pause, no upgrade path, no fee abstraction, and only needs plain checked `i128` arithmetic. |

Two blocking facts on top of "not applicable":

- **Version conflict.** All three 0.7.2 crates declare `soroban-sdk ^26.1.0`
  (checked via the crates.io dependency API). This repo resolves
  `soroban-sdk 28.0.0`; Cargo cannot satisfy both majors in one build, so
  adopting any of them today would mean downgrading the SDK and CLI toolchain.
- **The 0.8.0 line is not eligible.** The newest `0.8.0-rc.*` pre-releases
  (rc.3, 2026-06-16) are published "for audit purposes" and state they are not
  yet audited and not ready for production. Under the "prefer audited" rule
  they are not usable for the pilot.

What the SDK already provides (verified against the `soroban-sdk 28.0.0`
source on this machine):

- `soroban_sdk::token::TokenClient` — a generated client for the SEP-41 token
  interface, which includes the Stellar Asset Contract (`token::Client` is the
  deprecated alias). `transfer(from, to, amount)` is the only token call v0
  needs.
- `Address::require_auth` — the authorization mechanism for school and payer.

## Decision

- **v0 adds no OpenZeppelin dependency and no new crates at all.**
- Token movement goes through `soroban_sdk::token::TokenClient` against the
  fee's token address: payer → school on `pay`, school → payer on `refund`.
- Authorization is `Address::require_auth` per actor: school on
  `create_fee`/`close_fee`/`refund`, payer on `pay`.
- `soroban-sdk` stays at `"28"` in `Cargo.toml` with 28.0.0 resolved in the
  committed `Cargo.lock`. No manifest change.
- Re-evaluate this decision when an **audited** OpenZeppelin line supports
  `soroban-sdk` 28 (or when the project deliberately moves to a compatible
  SDK major), and/or before adding token issuance, upgradeability,
  pausability or fee abstraction.

## Consequences

- Easy: minimal dependency surface and no resolution conflict with the SDK;
  the whole money path is one audited cross-contract `transfer` on each side
  plus per-call auth, all covered by our own tests.
- Hard / accepted: no OpenZeppelin guardrails to lean on; if the project later
  grows an owner, roles, upgrades or its own token, this decision must be
  revisited before that code is hand-written.
- What would change our mind: an audited SDK-28-compatible OpenZeppelin
  release with a module that maps to a real v0 need; a security review
  recommending their components; or a deliberate SDK downgrade for other
  reasons.
