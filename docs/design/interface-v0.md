# schoolfees contract — v0 fee lifecycle: interface draft

Status: **approved 2026-09-30; implemented.** The interface below is what
`src/` builds; §9 maps the rules to their tests.
Date: 2026-09-30 · Scope: testnet-only v0 of the `schoolfees` contract.
Related: [decision 0001](../decisions/0001-openzeppelin-and-token-dependencies.md)
(dependencies), [ROADMAP.md](../../ROADMAP.md), [AGENTS.md](../../AGENTS.md).

## 1. Summary

A fee is one record: a school, a token, an opaque reference, a total, and a due
date. Payers pay in as many installments as they like, straight into the
school's token balance — **the contract never holds funds**. The school closes
the record when nothing is owed or nothing was paid, and can refund a payer,
from its own balance, up to what that payer paid. Anyone can read a fee and its
derived status.

v0 adds six functions (`create_fee`, `pay`, `close_fee`, `refund`, `get_fee`,
`status`), three persistent record types, four events, and nine new error codes.
`initialize` / `admin` from Phase 1 stay exactly as they are.

Not in v0 (see §11): enforced installment schedules, late fees, per-payer
limits, sibling/bundle discounts, paginated listings, property-based
invariants, a public TTL-extend entrypoint — and no upgradeability, pausing,
school registry, or token issuance.

## 2. Roles and trust

| Actor | Is | Authority |
|---|---|---|
| **School** | The address that created the fee. Any address can create one — there is no registry in v0. | `require_auth` on `create_fee`, `close_fee`, `refund`. |
| **Payer** | Whoever settles a fee. Anyone, including the school itself. | `require_auth` on `pay` (the same signature authorizes the token transfer out of the payer's balance). |
| **Admin** | Recorded by `initialize` (Phase 1). | **None in v0.** No fee function consults it. |
| **Reader** | Any caller. | None; `get_fee` and `status` are public. |

- The school picks the token when creating the fee. Payers must verify school,
  token, reference and amount against the school out-of-band; the contract
  makes no claim about the school's identity or the token's value.
- No custody: payments and refunds are direct transfers between the payer's and
  the school's token balances, executed by the token contract.

## 3. On-chain data

### 3.1 `Fee` — persistent, key `DataKey::Fee(fee_id)`

| Field | Type | Meaning |
|---|---|---|
| `id` | `u64` | Fee id, starting at 1, never reused. |
| `school` | `Address` | Receives payments; authorized creation. |
| `token` | `Address` | SEP-41 token contract (works with the Stellar Asset Contract). |
| `reference` | `BytesN<32>` | Opaque 32-byte reference. Never personal data. |
| `total` | `i128` | Amount owed; rejects `<= 0`. |
| `due_at` | `u64` | Unix seconds, same clock as `env.ledger().timestamp()`. |
| `paid_total` | `i128` | Sum of every payer's payments. |
| `refunded_total` | `i128` | Sum of every payer's refunds. |
| `closed` | `bool` | Terminal flag, set by `close_fee`. |

Derived values (never stored): `net = paid_total - refunded_total`,
`remaining = total - net`.

### 3.2 `PayerRecord` — persistent, key `DataKey::Payer(fee_id, payer)`

| Field | Type | Meaning |
|---|---|---|
| `paid` | `i128` | Everything this payer paid. |
| `refunded` | `i128` | Everything this payer got back. |

### 3.3 Reference index — persistent, key `DataKey::Reference(school, reference)`

| Field | Type | Meaning |
|---|---|---|
| `fee_id` | `u64` | Enforces one fee per `(school, reference)`, forever. |

### 3.4 Instance storage

| Key | Value | Meaning |
|---|---|---|
| `DataKey::Admin` | `Address` | Existing Phase 1 value. |
| `DataKey::NextFeeId` | `u64` | Next fee id to hand out (first id is 1). |

### 3.5 Privacy (non-negotiable)

- `reference` is the only free-form on-chain field: **opaque only**. The app
  hashes an off-chain identifier (e.g. SHA-256 of the school's internal id)
  before calling; the contract cannot verify opacity, so this is a client
  responsibility and is stated in the app rules.
- Never on-chain: names, phone numbers, emails, student or member ids, or
  anything about children — in arguments, structs, events, errors, or test
  fixtures (tests use synthetic references).
- Addresses are public keys and are inherently on-chain. Amounts, due dates
  and payment history are **publicly readable** — the pilot material must tell
  schools that fee records are transparent on testnet.

## 4. Storage keys and TTL policy

Keys: `DataKey::{Admin, NextFeeId}` (instance), `DataKey::{Fee(u64), Payer(u64, Address), Reference(Address, BytesN<32>)}` (persistent).

**Instance storage** keeps the Phase 1 policy unchanged
(`INSTANCE_TTL_THRESHOLD` 7 days, `INSTANCE_TTL_EXTEND_TO` 30 days).

**Records** use a deadline-based policy computed from the fee's real deadline
plus a safety margin (per `AGENTS.md`):

| Constant | Value | Meaning |
|---|---|---|
| `SECONDS_PER_LEDGER` | `5` | Same 5-second assumption as `DAY_IN_LEDGERS`. |
| `SETTLEMENT_MARGIN_SECONDS` | `2_592_000` (30 days) | How long the record must outlive `due_at`. |
| `MIN_TTL_LEDGERS` | `7 * DAY_IN_LEDGERS` | Floor when the deadline has passed. |

Helper (`src/storage.rs`), called on every read and write of a fee, payer or
reference entry, using that fee's `due_at`:

```text
deadline       = due_at + SETTLEMENT_MARGIN_SECONDS
remaining_secs = max(deadline - now, 0)
wanted         = remaining_secs / SECONDS_PER_LEDGER
target         = clamp(wanted, MIN_TTL_LEDGERS, env.storage().max_ttl())
persistent().extend_ttl(key, target / 2, target)
```

Semantics: top the entry up to "deadline + margin" whenever less than half of
that horizon remains, capped by the network maximum TTL (`max_ttl()`, verified
in SDK 28). A floor keeps entries alive after the deadline; a fee that is never
touched again archives naturally after roughly `due_at + 30 days` — no restore
UX is built in v0, and every normal call (create/pay/close/refund/get/status)
re-extends the entries it touches.

Notes: reads extend too, so `get_fee`/`status` submitted as transactions keep
records alive (simulations do not persist extensions). Converting seconds to
ledgers is the same approximation the existing `DAY_IN_LEDGERS` already uses.

## 5. Status and state machine

`FeeStatus` is **derived, never stored** — only `closed` is stored:

| Order | Condition | Status |
|---|---|---|
| 1 | `closed` | `Closed` |
| 2 | `net >= total` | `Paid` |
| 3 | `now > due_at` | `Overdue` |
| 4 | otherwise | `Open` |

Transitions: `create_fee` starts every fee `Open`. `pay` raises `net`
(may reach `Paid`; `Overdue -> Paid` is allowed; overpaying is rejected).
`refund` lowers `net` while the fee is not closed (`Paid` can fall back to
`Open`/`Overdue`). `close_fee` sets `closed` when `net == 0` (nothing paid) or
`net == total` (nothing owed); `Closed` is terminal.

Invariants (asserted in tests): `0 <= net <= total` after every call;
`fee.paid_total` equals the sum of payer `paid` values; `fee.refunded_total`
equals the sum of payer `refunded` values; every payer's `refunded <= paid`;
`closed` implies `net` is `0` or `total`, and nothing changes a closed fee's
totals.

## 6. Functions

Unchanged from Phase 1: `initialize(admin)` and `admin()` (see README).
New in v0:

**`create_fee(env, school: Address, token: Address, reference: BytesN<32>, total: i128, due_at: u64) -> Result<u64, Error>`**
- Auth: `school.require_auth()`.
- Checks: `total > 0` (`InvalidAmount`); `due_at > now` (`DueDateInPast`);
  `(school, reference)` unused (`DuplicateReference`).
- Effects: allocate `fee_id` from `NextFeeId`; store `Fee` and reference index;
  extend TTLs; emit `FeeCreated`; return `fee_id`.

**`pay(env, fee_id: u64, payer: Address, amount: i128) -> Result<(), Error>`**
- Auth: `payer.require_auth()`.
- Checks: fee exists (`FeeNotFound`); not closed (`FeeClosed`); `amount > 0`
  (`InvalidAmount`); `amount <= remaining` (`Overpayment`).
- Effects: `payer` record `paid += amount`; fee `paid_total += amount`; token
  `transfer(payer -> school, amount)`; extend TTLs; emit `FeePaid`.
- Notes: payments after `due_at` are allowed (overdue is informative). A token
  transfer failure (e.g. insufficient balance) surfaces as a host error, not a
  contract code.

**`close_fee(env, fee_id: u64) -> Result<(), Error>`**
- Auth: `fee.school.require_auth()`.
- Checks: fee exists (`FeeNotFound`); not closed (`FeeClosed`); `net == 0` or
  `net == total` (`CloseNotAllowed`).
- Effects: `closed = true`; extend; emit `FeeClosed`.

**`refund(env, fee_id: u64, payer: Address, amount: i128) -> Result<(), Error>`**
- Auth: `fee.school.require_auth()`.
- Checks: fee exists (`FeeNotFound`); not closed (`FeeClosed`); `amount > 0`
  (`InvalidAmount`); payer record exists (`PayerNotFound`);
  `amount <= payer.paid - payer.refunded` (`RefundExceedsPaid`).
- Effects: payer record `refunded += amount`; fee `refunded_total += amount`;
  token `transfer(school -> payer, amount)`; extend; emit `FeeRefunded`.
- Notes: paid out of the school's own balance (never from held funds). A
  partially-paid fee can be refunded down to zero and then closed.

**`get_fee(env, fee_id: u64) -> Result<Fee, Error>`** — no auth; extends the
fee's TTL; `FeeNotFound`. Returns the raw `Fee`; clients compute
`remaining = total - paid_total + refunded_total`.

**`status(env, fee_id: u64) -> Result<FeeStatus, Error>`** — no auth; extends
the fee's TTL; derived per §5; `FeeNotFound`.

No function loops or returns lists; every lookup is keyed. Money arithmetic is
checked `i128`.

## 7. Events

| Event | Emitted by | Topics | Data |
|---|---|---|---|
| `FeeCreated` | `create_fee` | `fee_id` | school, token, reference, total, due_at |
| `FeePaid` | `pay` | `fee_id` | payer, amount |
| `FeeClosed` | `close_fee` | `fee_id` | — |
| `FeeRefunded` | `refund` | `fee_id` | payer, amount |

(`Initialized` unchanged.) Layouts go into `docs/events.md` in the same commit
that adds them; indexers can filter by event name plus the `fee_id` topic.

## 8. Errors (draft enum)

| Code | Variant | Raised by | Trigger |
|---:|---|---|---|
| 3 | `FeeNotFound` | fee functions, `get_fee`, `status` | No record for `fee_id`. |
| 4 | `PayerNotFound` | `refund` | No payer record for `(fee_id, payer)`. |
| 10 | `FeeClosed` | `pay`, `close_fee`, `refund` | Operation on a closed fee. |
| 11 | `CloseNotAllowed` | `close_fee` | Partially paid: neither nothing owed nor nothing paid. |
| 12 | `DueDateInPast` | `create_fee` | `due_at <= now`. |
| 30 | `InvalidAmount` | `create_fee`, `pay`, `refund` | `total <= 0` or `amount <= 0`. |
| 31 | `Overpayment` | `pay` | `amount > remaining`. |
| 32 | `DuplicateReference` | `create_fee` | `(school, reference)` already has a fee. |
| 33 | `RefundExceedsPaid` | `refund` | `amount > payer.paid - payer.refunded`. |

Existing codes 1 (`NotInitialized`) and 2 (`AlreadyInitialized`) are untouched.
Auth failures surface as host auth errors — no variant. `ERRORS.md` rows
(including user-facing wording, the app's source of truth) are written from
this enum during implementation.

## 9. Test plan (implementation)

- `src/error_paths.rs`: one `error_path_<variant>` test per new variant (9 new
  tests; 11 total with the two Phase 1 variants).
- Happy path: create -> two installments -> `Paid` -> close -> `Closed`.
- Installments summing to `total`; overpayment rejection; duplicate reference
  (same school rejects, other school with the same reference is fine).
- Unauthorized callers for every `require_auth`; missing fee; missing payer.
- Closing: zero-paid allowed; partially paid rejects; fully paid allowed;
  double-close rejects; closed fee rejects pay and refund.
- Refunds: cap enforcement across multiple partial refunds; a refund on a
  `Paid` fee flips it back to `Open`/`Overdue`; refund of the whole payment
  allows closing a formerly partial fee.
- Timing: `Overdue` derivation and boundary via `set_timestamp`; payment after
  `due_at` allowed.
- TTL: fee/payer/reference entries top up toward `due_at + 30 days`; floor
  applies after the deadline (ledger-sequence manipulation, same pattern as
  the Phase 1 instance tests).
- Token: a real Stellar Asset Contract in tests
  (`register_stellar_asset_contract_v2` + `StellarAssetClient::mint`).
- Events: all four asserted against `docs/events.md` layouts.
- Size floor: test code at least as large as implementation; no `unwrap` /
  `expect` outside tests.

## 10. Open questions and assumptions for sign-off

Each item has a proposed default; approving the draft means approving all of
them. Say which to change.

1. **Refunds on closed fees** — proposed: not allowed; `Closed` is terminal, so
   closed records are settled forever. Alternative: allow (then a closed fee
   can show `remaining > 0`, which muddies "closed = settled").
2. **`due_at` already in the past at creation** — proposed: reject
   (`DueDateInPast`). Alternative: allow, fee starts `Overdue`.
3. **Payments after the due date** — proposed: allowed (`Overdue` is
   informative, not a lock). Alternative: reject.
4. **Closing a partially-paid fee** — proposed: keep the v3 rule (`net == 0`
   or `net == total`); a school refunds to zero first if it needs to close.
   Alternative: allow closing with an outstanding balance.
5. **Reference uniqueness** — proposed: one fee per `(school, reference)`,
   forever (no deletion exists).
6. **Who can create fees** — proposed: any address; no registry, no admin
   approval. Alternative: admin allow-list (adds admin powers the v3 design
   does not describe).
7. **Which tokens** — proposed: school-chosen SEP-41 contracts, no whitelist;
   transfer failures surface when someone pays or is refunded.
8. **Status storage** — proposed: derived per §5; only `closed` stored.
   Alternative: store the enum (redundant state to keep in sync).
9. **Fee ids** — proposed: start at 1, never reused.
10. **TTL numbers** — proposed: 30-day settlement margin, 7-day floor,
    half-horizon threshold, capped by `max_ttl()` (§4).
11. **`initialize` gate** — proposed: fee functions do **not** require
    `initialize`; admin is unused in v0. Alternative: gate `create_fee` on it.
12. **Events** — proposed: names and topics in §7 (only `fee_id` as a topic).
13. Minor defaults (accepted unless objected): `pay`/`close_fee`/`refund`
    return `()`; `get_fee` returns raw aggregates and clients compute
    `remaining`; the school may pay its own fee; records are immutable except
    for payments/refunds/close (no update or cancel functions); tokens sent
    directly to the contract address are unrecoverable (documented as a
    limitation).

## 11. Deliberately not in v0

Recorded in `ROADMAP.md` and one issue draft each (`docs/issue-drafts/`) during
implementation: enforced installment schedules; late-fee policy with a hard
cap; multi-payer per-payer limits; sibling/bundle discounts; paginated school
listing; property-based paid-total invariants; a public extend-TTL entrypoint.

## 12. After sign-off

Implementation order: `types.rs` -> `storage.rs` -> `lib.rs` (stays thin) ->
`error_paths.rs` -> `test.rs`; then `ERRORS.md`, `docs/events.md`, `README.md`,
`ROADMAP.md` and the issue drafts; then all six checks from `AGENTS.md`. Stop
at Checkpoint B: the maintainer pushes, and **nothing is deployed** (pilot
gate).
