# Security — schoolfees contract

What this contract's security depends on, what it deliberately does not do, and
what has never been checked. Adapted from the Build Arsenal security templates
(`SECURITY_TEMPLATE`, `SECURITY_CHECKLIST`, `CRYPTO_SECURITY`) to a Soroban
contract.

**The threat walk-through is not duplicated here.** The single source of truth
for threats is
[`schoolfees-docs/src/threat-model.md`](https://github.com/stellar-schoolfees/schoolfees-docs/blob/main/src/threat-model.md)
(a STRIDE walk-through with "not applicable, because…" entries). This page is
the requirements list that model is checked against.

Status: **synthetic testnet demonstration deployed; no audit or real pilot.**
See the [deployment record](TESTNET_DEMONSTRATION.md). Nothing on this page is a
claim that the contract is safe to use with real money. See
[`../README.md`](../README.md) and the docs repo's `limitations.md`.

## 1. Authorization

Every function that acts for an address calls `require_auth` for that address.
Auth failures are host errors, so they make the transaction fail and there is no
error code for them.

| Function | Who must sign | Where |
|---|---|---|
| `initialize(admin)` | the address being recorded as admin | `src/lib.rs::initialize` |
| `create_fee(school, …)` | the school the fee is created for | `src/fee.rs::create_fee` |
| `pay(fee_id, payer, amount)` | the payer whose tokens move | `src/fee.rs::pay` |
| `close_fee(fee_id)` | the fee's recorded school | `src/fee.rs::close_fee` |
| `refund(fee_id, payer, amount)` | the fee's recorded school | `src/fee.rs::refund` |
| `admin()` | nobody — read-only | `src/lib.rs::admin` |
| `get_fee`, `status` | nobody — read-only, public by design | `src/fee.rs` |

Rules:

- The auth check happens **before** any state is read or written where it can
  (`create_fee` signs first; `close_fee` and `refund` load the fee, then check
  the school's signature, so the school address comes from storage and not from
  the caller).
- There is **no admin override**. `initialize` records an admin address and
  `admin()` returns it; no fee function reads it. Compromising the admin key
  cannot move funds, change a fee, or pause anything.
- There is **no `require_auth` on `get_fee`/`status`**, because fee records are
  public on a public ledger and the payer must be able to read what they are
  paying. Do not treat a fee record as private.

## 2. Custody

**The contract never holds funds.** `pay` calls the token contract's
`transfer(payer → school)` and `refund` calls `transfer(school → payer)`, both in
the same invocation as the record update. There is no pooled balance, no escrow,
no fee, no sweep function and no way for the contract to hold a balance it could
send anywhere. Tokens sent directly to the contract address are unrecoverable —
documented as a limitation.

## 3. Token trust

- The school chooses **any SEP-41 token**. There is **no whitelist**, and this is
  deliberate: v0 has no registry or identity layer to base a whitelist on
  (`docs/design/interface-v0.md` §10 default 7).
- The contract therefore trusts a contract it did not choose and cannot
  validate. A malicious, broken or unfunded token makes `pay` or `refund` fail;
  the fee's own storage is unaffected, but the fee cannot be settled.
- A transfer failure surfaces as a **host error, not one of the codes in
  `ERRORS.md`**. The app reports the transaction hash rather than inventing
  wording for it.
- Do not add a token allow-list without recording the decision in
  `docs/decisions/` — it changes who can create a fee.

## 4. Input validation

All amounts are `i128` and every check runs before any write:

| Rule | Enforced by | Error |
|---|---|---|
| `total > 0` | `src/fee.rs::create_fee` | `InvalidAmount` (30) |
| `due_at > now` | `src/fee.rs::create_fee` | `DueDateInPast` (12) |
| `(school, reference)` unused | `src/fee.rs::create_fee` | `DuplicateReference` (32) |
| `amount > 0` | `src/fee.rs::pay`, `::refund` | `InvalidAmount` (30) |
| `amount <= remaining` | `src/fee.rs::pay` | `Overpayment` (31) |
| `amount <= payer.paid - payer.refunded` | `src/fee.rs::refund` | `RefundExceedsPaid` (33) |
| fee exists | every fee function | `FeeNotFound` (3) |
| fee not closed | `pay`, `close_fee`, `refund` | `FeeClosed` (10) |
| payer record exists | `refund` | `PayerNotFound` (4) |
| close allowed when `net == 0` or `net == total` | `close_fee` | `CloseNotAllowed` (11) |

Other input rules:

- **No loops, no lists, no unbounded input.** Every function does keyed lookups
  and returns; there is no array, map or vector argument anywhere, so there is
  no unbounded iteration to exhaust.
- **Checked arithmetic.** The release profile builds with `overflow-checks =
  true`; `src/fee.rs::add` is a checked add that traps rather than wrapping, and
  the fee id counter is a checked add. Overflow is documented as unreachable
  because every update is bounded by `total` (at most `i128::MAX`).
- **Failed checks return an error; they never return a success value.** No
  function returns `false` to signal a failed check.
- Validating the reference's *contents* is **not possible on-chain**: it is
  opaque `BytesN<32>`. Keeping it free of personal data is a client
  responsibility, enforced only by the app's input rule and warning.

## 5. Storage, TTL and archival risk

- Storage layout and constants are in `src/storage.rs`; the policy is described
  in `docs/design/interface-v0.md` §4.
- Growing data is one persistent entry per record (`DataKey::Fee`,
  `DataKey::Payer`, `DataKey::Reference`); instance storage holds only the admin
  address and the fee-id counter.
- Every read and write extends the TTL of the entries it touches, computed from
  the fee's **real deadline** (`due_at + 30-day margin`, with a 7-day floor and
  a cap of `max_ttl()`), never from a single flat constant.
- **Residual risk, accepted:** a fee nobody touches can archive after roughly
  `due_at + 30 days`. v0 has no restore entrypoint and no restore UI, so
  recovery is a manual step. This is tracked by
  [issue draft 07](issue-drafts/07-extend-ttl-entrypoint.md) and the app's
  [draft 08](https://github.com/stellar-schoolfees/schoolfees-app/blob/main/docs/issue-drafts/08-restore-archived-fee-record.md).
- **Note:** extending on read only persists when the read is submitted as a
  transaction. A *simulated* read (which is exactly what the app does) does not
  persist the extension. Browsing a fee in the app therefore does not keep it
  alive.

## 6. Dependency review

| Dependency | Version | Why | Reviewed |
|---|---|---|---|
| `soroban-sdk` | `"28"` (28.0.0 resolved in the committed `Cargo.lock`) | the only way to write a Soroban contract | pinned to the major version; `Cargo.lock` fixes the exact patch |
| `soroban-sdk` (dev, `testutils`) | same | test harness | same |

- **No OpenZeppelin Stellar crates are used**, and the reason is recorded in
  [`docs/decisions/0001-openzeppelin-and-token-dependencies.md`](decisions/0001-openzeppelin-and-token-dependencies.md).
  `AGENTS.md` prefers audited libraries over hand-written token, access-control
  or governance code; v0 has none of those three, so `soroban_sdk::token` is
  used directly instead.
- No new dependency may be added without saying why and checking the crate's
  maintainer and recent releases first (`AGENTS.md`).
- **`cargo audit` is not installed on the maintainer's machine, so no
  vulnerability scan of this dependency tree has been run.** Nothing was
  installed to change that. What *is* checked in CI: `cargo fmt`, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, the `ERRORS.md` sync check and
  `stellar contract build`.

## 7. Out of scope, and honest limits

- **No independent review or audit.** Nobody outside this project has reviewed
  the contract.
- **No key management.** If a school's or payer's signing key is stolen, the
  thief can sign as them. The contract cannot help; the link between an address
  and a real person is off-chain and unverified.
- **No front-running or ordering analysis.** Nothing in v0 depends on ordering
  for safety, but none has been studied.
- **The token contract's own security** is not this project's to analyse.
- **No upgrade path, no pause, no admin recovery.** A deployed instance keeps
  exactly the behaviour in the docs repo's `architecture.md`. The only remedy
  for a bug is to stop using the instance and deploy a new one.
- **No restore flow for archived entries** (see §5).
- **Social engineering is the most likely real attack**: because anyone can
  create a fee and there is no registry, a convincing fake fee can borrow a real
  school's credibility. Only out-of-band verification by the payer helps, and
  the app cannot do it for them.
- **Rate limiting, blocklists and moderation do not exist** and are deliberate
  boundaries, not pending work (`ROADMAP.md`).

## 8. Before deploying

[`docs/DEPLOYMENT_CHECKLIST.md`](DEPLOYMENT_CHECKLIST.md) is the gate. The first
item is a pilot agreement in hand, and the whole checklist is testnet-only.
