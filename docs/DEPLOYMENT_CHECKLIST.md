# Deployment checklist — schoolfees contract

The release gate for the contract. Adapted from the Build Arsenal
`RELEASE_RUNBOOK` and `DEPLOYMENT_CHECKLIST_TEMPLATE` to a Soroban contract that
has **no upgrade path**.

**Testnet only.** No mainnet in this phase. The real-pilot checklist below remains
incomplete. A separately authorized synthetic demonstration was deployed and
initialized October 8, 2026; its verified results are in
[TESTNET_DEMONSTRATION.md](TESTNET_DEMONSTRATION.md). That exception does not
satisfy the pilot agreement or business-flow smoke-test requirements below.

Run by the **maintainer**, never by an agent (`AGENTS.md`). The deploy script is
`scripts/deploy-testnet.sh`; it reads the signing identity from
`STELLAR_ACCOUNT` and never reads, prints or stores a secret itself.

## 1. The pilot gate (do not pass this)

- [ ] A real school or tutorial centre has **agreed to try the flow**. No
      agreement exists today. This is the gate in `ROADMAP.md`.
- [ ] A named contact on the school side, who knows which reference belongs to
      which student, and who agreed how they may be named in writing.
- [ ] A short written record of what was agreed, including what ends the pilot
      early. The docs repo's `pilot-playbook.md` has the rules.
- [ ] The pilot's own readiness checklist in
      `schoolfees-docs/src/pilot-readiness.md` is fully ticked. It is mostly
      unticked, and that is the honest state.

## 2. Build and record the artefact

- [ ] Working tree clean, on `main`, CI green (`.github/workflows/contract.yml`).
- [ ] All six checks pass locally:
      `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
      `cargo test` (34 tests), `node --test` (9 checker tests),
      `node scripts/check-errors.mjs`, `stellar contract build`.
- [ ] **Stellar CLI upgraded to v28.1.0 first.** The maintainer's machine has
      27.1.0; CI pins v28.1.0. Comparing the two has never been done
      (docs repo `src/todo-verify.md`).
- [ ] `stellar contract build` complete, and the **wasm hash and size recorded
      here**. For reference, the local build on 2026-10-01 with CLI 27.1.0
      produced `schoolfees.wasm`, 13,794 bytes optimized, hash
      `d842422d5ac2d3c700adce89405cf23c086760e80ba6e7a25811066bc395f279`,
      with 8 exported functions (`admin`, `close_fee`, `create_fee`, `get_fee`,
      `initialize`, `pay`, `refund`, `status`). **Record the hash of the artefact
      you actually deploy, built with v28.1.0** — this line is not a substitute.
- [ ] Deployment record: contract id `TODO(verify)` · wasm hash `TODO(verify)` ·
      date `TODO(verify)` · deploying account `TODO(verify)`.

## 3. Keys and who holds them

Nothing here can be recovered by the contract, so decide before deploying.

- [ ] **Admin key** (`initialize(admin)`): which address, and who holds it. Note
      that in v0 the admin has **no power at all** over fees — no fee function
      reads it. It exists so `admin()` has an answer and so `Initialized` is
      emitted once.
- [ ] **School key**: creates fees, receives every payment, authorises refunds
      and closes fees. It must be held by the school side, not by the
      maintainer. If it is lost, the fee can never be refunded or closed and the
      payments cannot be moved anywhere else.
- [ ] **Payer key**: held by the payer. The contract cannot spend it without
      that payer's signature.
- [ ] All keys are **testnet** keys. Nothing on this machine's mainnet identity
      is used. `.env` and key material are never committed.
- [ ] Agreed: what happens on the day a key is lost. The contract cannot help.

## 4. Token and funding

- [ ] The exact SEP-41 token decided, and its testnet contract address recorded
      here: `TODO(verify)`.
- [ ] Confirmed the **school account holds that token** — a refund is paid from
      the school's own balance, so without tokens it cannot refund anything.
- [ ] Confirmed the **payer account holds enough** of that token for the pilot
      amounts, plus testnet XLM for fees.
- [ ] Accepted, in writing: there is **no whitelist**, the contract uses
      whatever token the school names, and a transfer failure is a host error
      rather than one of the codes in `ERRORS.md`.

## 5. Deploy (testnet only)

- [ ] `stellar contract build` artefact is the one from §2.
- [ ] `STELLAR_ACCOUNT=<identity> PILOT_CONFIRMED=yes ./scripts/deploy-testnet.sh`
      — the script refuses to run without `PILOT_CONFIRMED=yes`.
- [ ] `initialize(admin)` called once and confirmed; a second call must fail
      with `AlreadyInitialized` (2).
- [ ] `VITE_CONTRACT_ID` set from the real deployed id for the app, and the id
      recorded in `schoolfees-docs/src/architecture.md` and the app README. Only
      real values — never an invented address.

## 6. Rollback and forward plan

The contract has **no upgrade path, no pause, no admin override and no
migration**. There is therefore no rollback of a deployed instance: the only
"rollback" is to stop using it.

- [ ] Agreed and written down: the trigger for abandoning an instance (for
      example a wrong total accepted, a payment to a wrong address, or any
      personal data reaching the chain).
- [ ] Agreed: the forward plan is **deploy a new instance** with the same opaque
      references *only if* the references were not the problem — a new instance
      has a new contract id and a new fee-id space, so the school's off-chain
      mapping must be updated by hand.
- [ ] Agreed: who tells the pilot participants, and how, if that happens.
- [ ] Noted: testnet can be reset by Stellar at any time, which erases every
      record. That is not a rollback plan either.

## 7. Post-deploy smoke test (contract side)

Every step recorded with its **real transaction hash**; nothing here is done in
advance or written up from imagination.

- [ ] `create_fee` with a synthetic reference, a small total and a near due
      date. Record the fee id and the transaction hash.
- [ ] `get_fee` and `status` for that fee id: `Open` before the due date.
- [ ] `pay` part of the total: `status` stays `Open`, `paid_total` rises.
- [ ] Pay the remainder: `status` becomes `Paid`.
- [ ] `close_fee`: succeeds; `status` becomes `Closed`; a further `pay` fails
      with `FeeClosed` (10).
- [ ] A second fee, part-paid, then `close_fee` — must fail with
      `CloseNotAllowed` (11).
- [ ] `refund` a payer for part of what they paid, then `close_fee` — the fee
      must become closable once `net == 0`, and the refund must fail if it
      exceeds what that payer still has paid (`RefundExceedsPaid`, 33).
- [ ] One unauthorized call (a different key signing as the school) fails, and
      the failure is a host auth error, not a code in `ERRORS.md`.
- [ ] The app, pointed at the real id, completes create → read → pay → refund →
      close once (that is the app's own smoke test, see the app's
      `docs/DEPLOYMENT_CHECKLIST.md`).

## 8. After the deployment

- [ ] Real contract id and explorer links recorded in the docs repo — real
      only.
- [ ] `schoolfees-docs/src/todo-verify.md` updated: the rows blocked on
      deployment either move to evidence or stay open with the reason.
- [ ] Anything the deployment or the pilot broke is written into the docs repo's
      `limitations.md` and `threat-model.md`.
- [ ] The app's `README.md` status banner updated from "not deployed".
- [ ] Standing reminder: a testnet deployment is **not** a security review, and
      the production boundary in the docs repo's `limitations.md` still applies.
