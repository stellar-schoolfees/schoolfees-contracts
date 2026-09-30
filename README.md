# schoolfees — contracts

Soroban (Stellar) contract for paying **school fees**: a school records a fee
obligation against an opaque reference, and a payer settles it on testnet with a
transaction anyone can verify. No student names, phone numbers or IDs ever go
on-chain — only opaque references or hashes.

> **Status: skeleton (testnet only, not deployed).**
> This repo currently contains the contract's structure and its initialization
> surface: `initialize` and `admin`, two error codes, one event, and the CI that
> keeps error codes honest. The fee lifecycle itself lands with the v0 design
> (see [ROADMAP.md](ROADMAP.md)). Nothing here has been deployed, audited, or
> used by a real payer.

Part of the schoolfees project, which is three repositories:
`schoolfees-contracts` (this one), `schoolfees-app`, `schoolfees-docs`.

## What works today

| Function | Behaviour |
|---|---|
| `initialize(admin)` | One-time setup. Records the administrator, requires that administrator's signature. Fails with `AlreadyInitialized` if run twice. |
| `admin()` | Returns the recorded administrator. Fails with `NotInitialized` before setup. |

Both calls keep the contract's instance storage alive with a TTL bump. Events and
error codes are documented in [docs/events.md](docs/events.md) and
[ERRORS.md](ERRORS.md).

## Quickstart

```bash
rustup target add wasm32v1-none   # Rust 1.84.0 or higher

cargo test                        # contract tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
node --test                       # tests for the ERRORS.md checker
node scripts/check-errors.mjs     # fails if ERRORS.md drifts from enum Error
stellar contract build            # produces target/wasm32v1-none/release/schoolfees.wasm
```

Deployment is done by a human, never by an agent:

```bash
STELLAR_ACCOUNT=dev ./scripts/deploy-testnet.sh   # prints the contract id
```

Two gates must be cleared first: the v0 fee lifecycle must exist, and a real
school or tutorial centre must have agreed to try the flow (see
[ROADMAP.md](ROADMAP.md)).

## Layout

```text
├── src/
│   ├── lib.rs          # #[contract] and #[contractimpl] only; thin
│   ├── types.rs        # #[contracterror] enum, #[contractevent] types
│   ├── storage.rs      # storage keys, TTL constants, extend_ttl helpers
│   ├── error_paths.rs  # one test per error variant
│   └── test.rs         # happy-path and integration tests
├── scripts/
│   ├── check-errors.mjs       # ERRORS.md <-> enum Error sync check (no deps)
│   ├── check-errors.test.mjs  # its tests
│   └── deploy-testnet.sh      # written, run by the human only
├── docs/
│   ├── events.md       # event layouts
│   ├── decisions/      # design decisions (see the README there)
│   └── issue-drafts/   # drafts for contributors (never created on GitHub for you)
├── ERRORS.md           # error table, checked against the code in CI
├── AGENTS.md           # rules for AI agents working in this repo
└── ROADMAP.md          # what is next, and what is deliberately not built
```

## Toolchain notes

- Rust stable, `wasm32v1-none` target, `soroban-sdk = "28"`.
- On Windows without an MSVC linker, the default toolchain is
  `stable-x86_64-pc-windows-gnu` with MinGW-w64 `gcc`/`ld`. `rust-toolchain.toml`
  intentionally pins only the target, not the channel; see the comment in that
  file.
- The Stellar CLI's latest stable release is **v28.1.0** per
  developers.stellar.org. Anything version-specific in this repo should be
  checked against those docs, not against another project's lockfile.
- CI installs the CLI with `stellar/stellar-cli@v28.1.0`.

## Honest limitations

- Testnet only. Not deployed anywhere.
- No fee record exists yet: the contract cannot take a payment, record a due
  date, or prove anything to a school.
- No external review. Do not route real money through this.
- The docs repo holds the full `limitations.md` once it exists; this section is
  the short version.

## License

MIT — see [LICENSE](LICENSE).
