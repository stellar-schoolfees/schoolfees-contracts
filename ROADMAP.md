# Roadmap

What is next, in order. Anything not listed as done is **not implemented**.

## Done (Phase 1 — skeleton)

- [x] Repository structure, `AGENTS.md`, CI (tests, clippy, error-doc sync, wasm build).
- [x] `initialize` / `admin` with `require_auth`, instance-storage TTL extension.
- [x] Error codes 1–2 with a table in `ERRORS.md` and one test each in `src/error_paths.rs`.
- [x] `scripts/check-errors.mjs` + tests, wired into CI.
- [x] `Initialized` event with its layout documented in `docs/events.md`.

## Next — v0 fee lifecycle

Not started. This is the actual product work, and it is deliberately not
invented here: it follows the v0 contract design document for schoolfees. It
will bring:

- The fee record: what is on-chain (opaque reference, amount, due date, status),
  who may create it, and who may settle it.
- Deadline-based TTL helpers in `src/storage.rs`, computed from each record's due
  date plus a safety margin — not a flat constant.
- Lifecycle events, documented in `docs/events.md` as they are added.
- Error variants filling ranges **10–29** (lifecycle & timing) and **30–49**
  (validation & authorization), each with a row in `ERRORS.md` and a test in
  `src/error_paths.rs` in the same commit.
- Tests for the happy path, the full lifecycle, unauthorized callers, and timing
  boundaries.

## Before any pilot

- [ ] The lifecycle above, tested, with the checks in `AGENTS.md` green.
- [ ] First testnet deployment via `scripts/deploy-testnet.sh` — run by the human.
- [ ] Explorer links and the deployed contract id recorded in the docs repo.
  Nothing invented: only real hashes and addresses.

## Pilot gate

Recruitment is running in parallel with the build, and it gates deployment:

> **No testnet deployment until a real school or tutorial centre has agreed to
> try the flow.** Outreach is handled by the maintainer; the repo records only
> what actually happened, after it happens.

A trained pilot uses people outside this project, gives them a task rather than a
tour, and ends with a written record including what did not work. An interested
person is not a completed pilot.

## Later (contributor-sized, not v0)

- [ ] The `schoolfees-docs` repo: architecture, limitations, threat model, pilot playbook.
- [ ] The `schoolfees-app` repo: wallet flow, TESTNET banner, error mapping from `ERRORS.md`.
- [ ] Fuzz or property tests for the record arithmetic.
- [ ] A coverage gate once the test suite is established (no number is set yet on purpose).

## Explicitly out of scope

Mainnet deployment, investor or fundraising material, a TypeScript SDK, keeper or
indexer services, and any feature that the v0 design does not ask for.
