# A public TTL-extend entrypoint

**Difficulty:** medium
**Labels:** good first issue, area:contracts

## Problem

Records are only kept alive when a fee function is called. A fee that is ignored
for a long stretch can archive even though it is still owed, and there is no way
for anyone to pay the small cost of keeping it available.

## Scope

- A permissionless entrypoint, for example `extend_fee(fee_id)`, that tops up
  the fee, payer and reference entries with the same deadline-based policy in
  `src/storage.rs`.
- Works on live entries only; restoring already archived entries is out of scope
  and must be documented as such.
- Out of scope: automatic restoration, keeper bots, changing any money fields.

## Acceptance criteria

- [ ] The call extends the records without changing balances, totals or status.
- [ ] TTL tests mirror the existing horizon and floor tests in `src/test.rs`.
- [ ] `README.md` and `docs/events.md` are updated if the event surface changes.

## Where to start

`src/storage.rs` (`record_ttl_target`, `extend_record_ttl`), `src/fee.rs`,
`src/lib.rs`.

## How to test

```bash
cargo test
stellar contract build
```
