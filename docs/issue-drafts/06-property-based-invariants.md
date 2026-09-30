# Property-based tests for paid-total invariants

**Difficulty:** medium
**Labels:** good first issue, area:contracts

## Problem

The arithmetic invariants — net paid stays between 0 and `total`, the per-payer
sums match the fee aggregates, refunds never exceed what was paid — are checked
by example tests only.

## Scope

- Property tests driving random sequences of create/pay/refund/close calls and
  asserting the invariants after every step. Use a maintained property-testing
  crate, pinned exactly, and say why it was chosen.
- Wire them into `cargo test` or a separate CI step with a stated time budget.
- Out of scope: fuzzing the token contract, changing production behaviour.

## Acceptance criteria

- [ ] Invariants are asserted after every generated call.
- [ ] A failing sequence is printed so it can become an example test.
- [ ] CI runs the property suite, or the draft explains why it stays local.

## Where to start

`src/test.rs` and `src/test_helpers.rs` for the existing patterns; the approved
v0 design lists the invariants in `docs/design/interface-v0.md` §5.

## How to test

```bash
cargo test
```
