# Late-fee policy with a hard cap

**Difficulty:** hard
**Labels:** help wanted, area:contracts

## Problem

Overdue fees carry no consequence on-chain: `status` reports `Overdue`, but a
payment costs the same late as on time. Schools asked for a late fee, and an
unbounded or compounding one would be a trap for payers.

## Scope

- An optional late fee on a fee record, chosen at creation: either a flat amount
  or a percentage of the unpaid balance — pick one and justify it in
  `docs/design/`.
- A hard cap enforced by the contract (for example: never more than 10% of
  `total`), applied once per fee, never compounding.
- The late-fee amount is visible in `get_fee` and in the payment event.
- Out of scope: interest, variable rates, oracles, waivers.

## Acceptance criteria

- [ ] Creation rejects configurations above the documented cap.
- [ ] An overdue payment costs exactly the documented amount; boundary tests at
      the due date and at the cap.
- [ ] New variants, `ERRORS.md` rows and `error_path_*` tests land together.

## Where to start

`src/fee.rs`, `src/types.rs`, and the approved v0 design in
`docs/design/interface-v0.md`.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
