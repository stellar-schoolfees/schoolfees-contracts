# Enforce installment schedules for a fee

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

A fee has one total and one due date, and `pay` accepts any amount up to the
remaining balance. A school that splits a term into named installments cannot
express that on-chain: a payer can send everything on the last day and still
look settled.

## Scope

- An optional schedule on a fee: an ordered list of `(due_at, amount)` tranches
  that must sum to `total`, stored with the fee.
- `pay` applies each payment to the earliest open tranche and rejects payments
  for a tranche after its due date. Whether a small grace window is needed is a
  design decision that must be recorded in `docs/design/` before the code lands.
- Out of scope: late fees (see the separate draft), discounts, reminders.

## Acceptance criteria

- [ ] Schedules that do not sum to `total` are rejected at creation.
- [ ] Payments are applied to tranches in order; boundary tests at each due
      date.
- [ ] New error variants sit in the documented ranges, with an `ERRORS.md` row
      and an `error_path_*` test each.
- [ ] `docs/events.md` documents any new event in the same commit as the code.

## Where to start

`src/fee.rs` (payment logic), `src/types.rs` (`Fee` and errors), `src/storage.rs`
(keys and TTL for schedule entries), and the approved v0 design in
`docs/design/interface-v0.md`.

## How to test

```bash
cargo test
node --test
node scripts/check-errors.mjs
stellar contract build
```
