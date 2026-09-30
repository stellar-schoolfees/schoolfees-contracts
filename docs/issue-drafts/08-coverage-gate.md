# Add a coverage gate for the contract tests

**Difficulty:** medium
**Labels:** help wanted, area:ci

## Problem

`AGENTS.md` states a floor — "test code that is at least as large as
implementation code" — and the architecture page in `schoolfees-docs` notes that
it is checked by hand, not automated. Nothing stops a change from lowering
coverage, and a reviewer has to compare line counts manually to see the ratio.
No target number is set yet, on purpose: it should be chosen from the current
measured baseline, not guessed.

## Scope

Add a coverage measurement for the Rust contract tests and a CI check that fails
when coverage drops below the chosen threshold. Keep the existing CI steps
unchanged in name and order where possible.

Out of scope: coverage for the Node checker scripts, coverage for
`schoolfees-app` (another repo), and any change to the contract code itself.

## Acceptance criteria

- [ ] Coverage is measured in CI with a documented command (for example
      `cargo llvm-cov --summary-only`), pinned to a version, and its output is
      visible in the job log.
- [ ] A baseline number is measured first and written into the workflow as the
      fail threshold, with a comment saying it came from the baseline.
- [ ] Lowering the threshold requires editing the workflow, so a drop is visible
      in review.
- [ ] The check fails on a deliberately deleted test (verified locally before
      pushing).
- [ ] `AGENTS.md` is updated to point at the automated check instead of the
      hand-checked sentence in the docs repo.

## Where to start

`.github/workflows/contract.yml` (add a step after `cargo test`), and
`AGENTS.md` ("Contract rules"). Read the existing job first: it must keep
running `cargo fmt`, `cargo clippy`, `cargo test`, `node --test`,
`node scripts/check-errors.mjs` and `stellar contract build`.

## How to test

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
node --test
node scripts/check-errors.mjs
stellar contract build
```

Then run the new coverage command locally, note the number, and confirm CI
reports the same one.
