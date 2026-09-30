# Contributing

Thanks for looking. This is a small, testnet-only project; the most useful
contributions are the issues drafted in [docs/issue-drafts](docs/issue-drafts)
and small, well-tested changes.

## Setup

```bash
rustup target add wasm32v1-none
cargo build
```

Read [AGENTS.md](AGENTS.md) before changing anything: it is the short version of
the rules this repo is held to (testnet only, no secrets, honest docs, no
invented APIs). It applies to humans as much as to AI agents.

## Before you open a pull request

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
node --test
node scripts/check-errors.mjs
stellar contract build
```

CI runs the same checks, plus the wasm build.

## The error-code workflow

Every failure mode has a code, and the code, the table and the test move
together. When you add a variant:

1. Add it to `enum Error` in `src/types.rs`, inside the right range (1–9
   initialization & lookup, 10–29 lifecycle & timing, 30–49 validation &
   authorization). Codes are never renumbered.
2. Add a row to `ERRORS.md` with the user-facing message. That column is the app's
   source of truth; the app does not invent its own wording.
3. Add `error_path_<variant>` to `src/error_paths.rs` that triggers the real
   return path.
4. Run `node scripts/check-errors.mjs` — it fails if the code and the table
   disagree.

## Rules of thumb

- Tests first, or at least alongside. Test code should be at least as large as the
  implementation, and that ratio is a floor rather than a target to pad with
  trivial tests.
- No `unwrap()`/`expect()` outside tests.
- No personal data on-chain, in tests, or in fixtures. Use synthetic references.
- Small commits with clear messages. Do not rewrite history.
- Do not add a dependency without saying why in the pull request, and check that
  the crate is maintained first.
- Never commit `.env` files, secret keys or seed phrases. If you see one in a
  diff, say so instead of merging.

## Issues

Issues are drafted in [docs/issue-drafts](docs/issue-drafts) using the template in
`AGENTS.md`; the maintainer posts them. Please do not open an issue asking for
mainnet support — this phase is testnet only by design.
