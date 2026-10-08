# schoolfees — contracts

Soroban (Stellar) contract for paying **school fees**: a school records a fee
obligation against an opaque reference, and a payer settles it on testnet with a
transaction anyone can verify. No student names, phone numbers or IDs ever go
on-chain — only opaque references or hashes.

> **Status: v0 fee lifecycle implemented; synthetic testnet demonstration deployed.**
> Deployment and initialization are verified in the [demonstration record](docs/TESTNET_DEMONSTRATION.md).
> No real pilot, audit or browser-wallet business flow has been completed.

Part of the schoolfees project, which is three repositories:
`schoolfees-contracts` (this one), `schoolfees-app`, `schoolfees-docs`.

## What works today

| Function | Behaviour |
|---|---|
| `initialize(admin)` | One-time setup. Records the administrator, requires that administrator's signature. Fails with `AlreadyInitialized` if run twice. |
| `admin()` | Returns the recorded administrator. Fails with `NotInitialized` before setup. |
| `create_fee(school, token, reference, total, due_at)` | Records a fee against an opaque 32-byte reference. Requires the school's signature. Rejects a non-positive total, a due date that is not in the future, and a reference the school already used. Returns the fee id. |
| `pay(fee_id, payer, amount)` | Pays part or all of a fee, moving tokens straight from the payer to the school. Rejects overpayment, zero or negative amounts, and payment on a closed fee. |
| `close_fee(fee_id)` | Closes a fee that has nothing owed or nothing paid. Requires the school's signature. |
| `refund(fee_id, payer, amount)` | Refunds a payer from the school's own balance, capped at what that payer still has paid. Requires the school's signature. |
| `get_fee(fee_id)` | Returns the fee record: school, token, reference, total, due date, paid and refunded totals, closed flag. |
| `status(fee_id)` | Returns `Open`, `Paid`, `Overdue` or `Closed`, derived from the record and the ledger time. |

Every lifecycle call keeps the records it touches alive with a deadline-based TTL
bump: the fee's due date plus a 30-day settlement margin, with a 7-day floor.
The contract never takes custody — payments and refunds move tokens directly
between the payer and the school. Events and error codes are documented in
[docs/events.md](docs/events.md) and [ERRORS.md](ERRORS.md).

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

One gate remains: a real school or tutorial centre must have agreed to try the
flow before anything is deployed (see [ROADMAP.md](ROADMAP.md)).

## Layout

```text
├── src/
│   ├── lib.rs          # #[contract] and #[contractimpl] only; thin
│   ├── fee.rs          # fee lifecycle logic
│   ├── types.rs        # #[contracterror] enum, stored types, events
│   ├── storage.rs      # storage keys, TTL constants, extend_ttl helpers
│   ├── error_paths.rs  # one test per error variant
│   ├── test_helpers.rs # shared test setup
│   └── test.rs         # happy-path and integration tests
├── scripts/
│   ├── check-errors.mjs       # ERRORS.md <-> enum Error sync check (no deps)
│   ├── check-errors.test.mjs  # its tests
│   └── deploy-testnet.sh      # written, run by the human only
├── docs/
│   ├── events.md       # event layouts
│   ├── design/         # approved interface drafts
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
- The Stellar CLI's current stable release is **v28.1.0** per
  developers.stellar.org, and CI pins it with `stellar/stellar-cli@v28.1.0` —
  that is the version this repo's build is verified against. The maintainer's
  machine still has 27.1.0 installed, so upgrade before trusting a local
  `stellar contract build`. Anything version-specific should be checked against
  the Stellar docs, not against another project's lockfile.

## Honest limitations

- Testnet only. Not deployed anywhere, and no real payer has used it.
- No external review or audit. Do not route real money through this.
- No upgrade path and no pause: a deployed instance keeps exactly the behaviour
  described here.
- Tokens sent directly to the contract address cannot be recovered — there is no
  sweep function, because the contract never intends to hold funds.
- Fee records are public: addresses, amounts and due dates are readable by
  anyone. References are opaque and must never contain personal data.
- No reminders, translations or receipt export yet; those belong to the app
  repo, [`schoolfees-app`](https://github.com/stellar-schoolfees/schoolfees-app),
  which is implemented but has never run against a deployed contract or a real
  wallet.
- The docs repo,
  [`schoolfees-docs`](https://github.com/stellar-schoolfees/schoolfees-docs),
  holds the full `limitations.md`;
  the section above is the short version.

## License

MIT — see [LICENSE](LICENSE).
