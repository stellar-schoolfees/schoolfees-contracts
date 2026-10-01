# Testing — schoolfees contract

What the test suite actually covers, how to run it, and — the important half —
what it does not cover. Adapted from the Build Arsenal `TESTING_TEMPLATE` and
the Flowtick testing rules to a Soroban contract.

Everything below was run on 2026-10-01 and the counts are the real ones. Where a
number is not verified it is marked as such.

## 1. The five layers that exist

| Layer | Where | Runs in CI |
|---|---|---|
| Error-path tests — one per `Error` variant, triggering the real return path | `src/error_paths.rs` (11 tests) | yes, `cargo test` |
| Lifecycle and integration tests — happy paths, installments, refunds, closing, timing, TTL, unauthorized callers | `src/test.rs` (23 tests) | yes, `cargo test` |
| Doc-sync tests for the `ERRORS.md` checker | `scripts/check-errors.test.mjs` (9 tests) | yes, `node --test` |
| The `ERRORS.md` ↔ `enum Error` sync check itself | `scripts/check-errors.mjs` | yes |
| Compile, lint and build | `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `stellar contract build` | yes |

**Total: 34 Rust tests** (23 + 11) and 9 checker tests.

### 1.1 Error-path tests (`src/error_paths.rs`, 11)

One per variant, named `error_path_<variant_in_snake_case>`, and each one drives
the real failure through the contract rather than constructing the error value:

`error_path_not_initialized`, `error_path_already_initialized`,
`error_path_fee_not_found`, `error_path_payer_not_found`,
`error_path_fee_closed`, `error_path_close_not_allowed`,
`error_path_due_date_in_past`, `error_path_invalid_amount`,
`error_path_overpayment`, `error_path_duplicate_reference`,
`error_path_refund_exceeds_paid`.

**Rule:** adding an `Error` variant means adding its `ERRORS.md` row, its
`error_path_*` test and its user-facing wording in the same commit. The sync
checker fails CI if the enum and the table drift apart.

### 1.2 Lifecycle and integration tests (`src/test.rs`, 23)

`initialize_records_the_admin`, `initialize_requires_the_admin_signature`,
`initialize_extends_the_instance_ttl`,
`initialize_publishes_an_initialized_event`, `admin_read_extends_the_instance_ttl`,
`create_fee_records_an_open_fee`, `create_fee_requires_the_school_signature`,
`create_fee_assigns_sequential_ids`,
`create_fee_allows_the_same_reference_for_another_school`,
`create_fee_extends_the_fee_and_reference_ttls`,
`pay_records_installments_until_the_fee_is_paid`,
`pay_requires_the_payer_signature`,
`pay_rejects_a_signature_from_someone_other_than_the_payer`,
`pay_extends_the_payer_record_ttl`,
`pay_after_due_date_is_allowed_and_status_becomes_overdue_then_paid`,
`close_fee_with_nothing_paid_marks_it_closed`,
`close_fee_after_full_payment_marks_it_closed`,
`close_fee_requires_the_school_signature`,
`refund_returns_tokens_and_reopens_the_fee`,
`refund_requires_the_school_signature`,
`status_is_open_up_to_the_due_date_and_overdue_after`,
`lifecycle_publishes_documented_events`,
`get_fee_keeps_a_late_record_alive_with_the_floor_ttl`.

Notes that matter:

- Payments and refunds are tested against a **real Stellar Asset Contract**
  registered in the test environment (`src/test_helpers.rs::setup_token`,
  `register_stellar_asset_contract_v2`), not a mock.
- Events are asserted against the layouts documented in `docs/events.md`
  (`lifecycle_publishes_documented_events`,
  `initialize_publishes_an_initialized_event`).
- TTL behaviour is asserted directly, including the floor case after the
  deadline.
- Tests use synthetic references only. No personal data appears in any fixture.

## 2. How to run it

From the repository root (the same commands `AGENTS.md` lists and CI runs):

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
node --test
node scripts/check-errors.mjs     # prints "in sync … 11 variants checked"
stellar contract build
```

`cargo test` needs the `wasm32v1-none` target added only for
`stellar contract build`; the unit tests run on the host toolchain.

## 3. What is NOT tested

Stated plainly, because a green suite is not the same as a covered one. These
are the same gaps recorded in the docs repo's
[`proven-vs-assumed.md`](https://github.com/stellar-schoolfees/schoolfees-docs/blob/main/src/proven-vs-assumed.md),
and three of them already have drafts.

| Not tested | Why it matters | Tracked by |
|---|---|---|
| **Aggregate invariants under arbitrary call sequences.** `fee.paid_total` always equalling the sum of per-payer `paid` records holds by construction (`src/fee.rs::pay` writes both in one call), but no test proves it under random sequences: many payers, interleaved partial refunds, closing attempts between them. | A subtle ordering bug could leave the fee total and the payer records disagreeing, and the derived `remaining` would then be wrong for everyone. | [draft 06](issue-drafts/06-property-based-invariants.md) |
| **Property-based and fuzz testing.** Nothing generates adversarial or random inputs. | The suite checks the paths a human thought of. | [draft 06](issue-drafts/06-property-based-invariants.md) |
| **Real token edge cases.** Tests use a standard Stellar Asset Contract. Nothing covers a token that fails a transfer, returns `false`, is not SEP-41-conformant, has unusual decimals, or is malicious. | The contract trusts any token the school names, so this is the largest untested trust boundary. | no draft — see §4 |
| **The test-code-size floor** ("at least as large as the implementation") is counted by hand. | A rule that is not measured drifts. | [draft 08](issue-drafts/08-coverage-gate.md) |
| **Restoring an archived record.** Nothing exercises archival followed by a restore, because v0 has no restore entrypoint. | A long-idle fee becomes unrecoverable through this contract. | [draft 07](issue-drafts/07-extend-ttl-entrypoint.md) |
| **Deployment itself, on any network.** No `stellar contract deploy` has been run, and CI never deploys (no secrets in CI). | Local tests prove the logic, not the deploy path or the resulting wasm. `scripts/deploy-testnet.sh` has never run. | pilot gate, `ROADMAP.md` |
| **CLI equivalence.** The maintainer's CLI is 27.1.0; CI pins v28.1.0. | The two builds have never been compared. | docs repo `src/todo-verify.md` |
| **Concurrency and load.** Multiple callers in the same ledger, fee counts in the thousands, storage growth and cost. | Not analysed at all. | no draft — see §4 |
| **Anything about identity or off-chain truth.** The contract cannot tell whether a school is a school, or whether a reference means anything. | Untestable on-chain by design; it is a limitation, not a gap in the tests. | docs repo `limitations.md` |

**Nothing in this document claims testing that was not run.** The pre-existing
test suite was not modified by this task.

## 4. Two gaps with no draft, on purpose

- **Real-token edge cases** belong with a token-decision document: the right
  answer depends on whether v0 ever restricts which tokens are accepted, which
  is decision `7` in `docs/design/interface-v0.md` §10 and a pilot-time choice.
  Writing a mock-based test now would test a mock.
- **Concurrency and load** cannot be measured without a deployment. The honest
  position is that nobody has looked, and the docs repo says so.

## 5. Release smoke test

Once a deployment exists (blocked by the pilot gate), the contract-side smoke
test is the one in [`DEPLOYMENT_CHECKLIST.md`](DEPLOYMENT_CHECKLIST.md) §7:
`initialize`, `create_fee`, `pay`, `get_fee`, `status`, `close_fee`, with real
testnet hashes recorded. It has not been run.
