# Paginated school listing

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

Fees can only be looked up by id or by reference. A school dashboard cannot list
its fees, which is the first screen `schoolfees-app` will need.

## Scope

- A per-school index written on `create_fee` and a read function like
  `list_school_fees(school, cursor, limit)` returning at most a fixed number of
  records per call.
- Pagination must be bounded: a documented maximum `limit`, no unbounded loops.
- Out of scope: cross-school search, filtering by payer, indexer services.

## Acceptance criteria

- [ ] Listing is bounded and tested with more fees than fit one page.
- [ ] Index entries get TTL extensions from their fee's deadline, like the fee
      record itself.
- [ ] The pagination scheme is recorded in `docs/design/` before the code lands.

## Where to start

`src/storage.rs` (index key and TTL), `src/fee.rs`, `src/lib.rs`.

## How to test

```bash
cargo test
stellar contract build
```
