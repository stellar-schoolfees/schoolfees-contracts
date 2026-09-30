# Per-payer limits for multi-payer fees

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

`pay` lets any payer cover any amount up to what is owed, so one family can
settle another family's obligation. Some schools want a ceiling per payer.

## Scope

- An optional per-payer cap on a fee, set at creation.
- `pay` rejects amounts that would push that payer's recorded total above the
  cap. A refund lowers the payer's total and frees cap room again.
- Out of scope: allow-lists of payer addresses and anything that would put
  identity data on-chain.

## Acceptance criteria

- [ ] Fees without a cap behave exactly as today.
- [ ] Cap enforcement is tested at the boundary and after a partial refund.
- [ ] New variant, `ERRORS.md` row and `error_path_*` test land together.

## Where to start

`src/fee.rs` (payment checks), `src/types.rs` (`Fee`), `src/storage.rs`
(payer records already exist).

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
