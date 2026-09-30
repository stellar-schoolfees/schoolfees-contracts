# Sibling/bundle discounts

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

A school that gives a sibling discount must invent a smaller `total`, and the
chain keeps no record of why the amount differs from the standard fee. That
makes the record hard to audit later.

## Scope

- Optional discount fields on a fee at creation: the pre-discount total and the
  discount amount, with the contract enforcing
  `pre_discount_total - discount == total`.
- `get_fee` returns them; the payment flow is unchanged.
- Out of scope: detecting siblings (family data never goes on-chain),
  multi-fee bundle logic, refunding a discount.

## Acceptance criteria

- [ ] Creation validates the arithmetic and rejects non-positive values.
- [ ] A discounted fee's full lifecycle is tested.
- [ ] Docs (`docs/events.md`, `ERRORS.md`) move in the same commit as the code.

## Where to start

`src/types.rs`, `src/fee.rs`, and the approved v0 design in
`docs/design/interface-v0.md`.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
