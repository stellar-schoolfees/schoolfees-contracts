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

## Done (engineering standards — 2026-10-01)

The contract was measured against the Build Arsenal (crypto profile, risk: HIGH)
and the Flowtick engineering playbook. The gap map is
`schoolfees-docs/docs/arsenal-gap-map.md`; the audits are in
`schoolfees-docs/docs/audits/`.

- [x] `docs/SECURITY.md` — authorization per function, no custody, token trust, input validation, TTL and archival risk, dependency review, and what is explicitly out of scope. Links the docs repo's threat model rather than copying it.
- [x] `docs/TESTING.md` — the real layers with exact counts and test names, the error-path rule, and what is **not** tested (aggregate invariants, real token edge cases, fuzzing, deployment).
- [x] `docs/DEPLOYMENT_CHECKLIST.md` — the release gate, starting with the pilot agreement; key holders, token choice, the wasm hash record, the no-upgrade-path rollback plan, and a post-deploy smoke test. Local build observed 2026-10-01: 13,794 bytes optimized, hash `d842422d…`, 8 exported functions, built with CLI 27.1.0.
- [x] `docs/ARCHITECTURE.md` — a module map and a pointer to the single architecture page, with no duplicated description.
- [x] `AGENTS.md` — Source of truth list, Flowtick collaboration rules, and the conventional-commit/staging rule. `CONTRIBUTING.md` gained the Git discipline for outside contributors.
- [x] No new contract issues were found that are not already tracked by drafts 06, 07 and 08.

## Done (companion repos)

- [x] `schoolfees-docs`: architecture, limitations, threat model, pilot playbook, link checker and docs CI.
- [x] `schoolfees-app`: wallet flow, TESTNET banner, error mapping from `ERRORS.md`, web CI.

## Next — publish and deploy

- [x] Push the v0 fee lifecycle and get CI green on GitHub.
- [ ] **Blocked on the pilot gate:** the first testnet deployment via `scripts/deploy-testnet.sh` — run by the human, and only after the pilot gate below is cleared. This is a maintainer step, not contributor work, so it deliberately has no issue draft.
- [ ] **Blocked on that deployment:** record the real contract id and explorer links in the docs repo. Only real hashes and addresses; nothing invented.

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
- [ ] A coverage gate once the test suite is established (no number is set yet on purpose) — [draft](docs/issue-drafts/08-coverage-gate.md).

## Explicitly out of scope

Mainnet deployment, investor or fundraising material, a TypeScript SDK, keeper or
indexer services, and any feature that the v0 design does not ask for.

Three of the limitations in the docs repo are deliberate boundaries rather than
pending work, and no draft tracks them:

- **Editing or cancelling a fee** — a fee is immutable except for payments,
  refunds and the one-way `closed` flag (interface §10 defaults 4–5).
- **Rate limiting or blocklisting fee creation** — there is no registry and no
  identity layer in v0, and any address may create a fee (§10 default 6).
- **Upgrade or pause machinery** — the recorded admin has no power over fees,
  and there is no upgrade path (§10 default 11).
