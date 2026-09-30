# Roadmap

What is next, in order. Anything not listed as done is **not implemented**.

## Done (Phase 1 — skeleton)

- [x] Repository structure, `AGENTS.md`, CI (tests, clippy, error-doc sync, wasm build).
- [x] `initialize` / `admin` with `require_auth`, instance-storage TTL extension.
- [x] Error codes 1–2 with a table in `ERRORS.md` and one test each in `src/error_paths.rs`.
- [x] `scripts/check-errors.mjs` + tests, wired into CI.
- [x] `Initialized` event with its layout documented in `docs/events.md`.

## Done (Phase 2 — v0 fee lifecycle)

Approved design: [docs/design/interface-v0.md](docs/design/interface-v0.md).

- [x] `create_fee`, `pay`, `close_fee`, `refund`, `get_fee`, `status`, with per-fee school/payer authorization.
- [x] Fee, payer and reference records in persistent storage, and no custody: every payment moves tokens straight from the payer to the school.
- [x] TTL computed from each fee's real deadline plus a 30-day settlement margin, with a 7-day floor.
- [x] Error codes 3–4 (lookup), 10–12 (lifecycle & timing) and 30–33 (validation), each with an `ERRORS.md` row and a test in `src/error_paths.rs`.
- [x] `FeeCreated`, `FeePaid`, `FeeRefunded` and `FeeClosed` events documented in `docs/events.md`.
- [x] Tests for the happy path, installments, refunds, closing, timing boundaries, TTL and unauthorized callers.
- [x] The dependency decision (no OpenZeppelin crates in v0) recorded in [docs/decisions/0001-openzeppelin-and-token-dependencies.md](docs/decisions/0001-openzeppelin-and-token-dependencies.md).

## Next — publish and deploy

- [ ] Push the v0 fee lifecycle and get CI green on GitHub.
- [ ] First testnet deployment via `scripts/deploy-testnet.sh` — run by the human, and only after the pilot gate below is cleared.
- [ ] Explorer links and the deployed contract id recorded in the docs repo. Nothing invented: only real hashes and addresses.

## Pilot gate

Recruitment is running in parallel with the build, and it gates deployment:

> **No testnet deployment until a real school or tutorial centre has agreed to
> try the flow.** Outreach is handled by the maintainer; the repo records only
> what actually happened, after it happens.

A trained pilot uses people outside this project, gives them a task rather than a
tour, and ends with a written record including what did not work. An interested
person is not a completed pilot.

## Later (contributor-sized, not v0)

The v0 design deliberately leaves these out. Each has a draft issue in
[docs/issue-drafts](docs/issue-drafts):

- [ ] Enforced installment schedules — [draft](docs/issue-drafts/01-enforced-installment-schedules.md).
- [ ] A late-fee policy with a hard cap — [draft](docs/issue-drafts/02-late-fee-policy.md).
- [ ] Per-payer limits for multi-payer fees — [draft](docs/issue-drafts/03-per-payer-limits.md).
- [ ] Sibling/bundle discounts — [draft](docs/issue-drafts/04-sibling-discounts.md).
- [ ] Paginated school listing — [draft](docs/issue-drafts/05-paginated-school-listing.md).
- [ ] Property-based paid-total invariants — [draft](docs/issue-drafts/06-property-based-invariants.md).
- [ ] A public TTL-extend entrypoint — [draft](docs/issue-drafts/07-extend-ttl-entrypoint.md).
- [ ] The `schoolfees-docs` repo: architecture, limitations, threat model, pilot playbook.
- [ ] The `schoolfees-app` repo: wallet flow, TESTNET banner, error mapping from `ERRORS.md`.
- [ ] A coverage gate once the test suite is established (no number is set yet on purpose).

## Explicitly out of scope

Mainnet deployment, investor or fundraising material, a TypeScript SDK, keeper or
indexer services, and any feature that the v0 design does not ask for.
