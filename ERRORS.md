# schoolfees Contract — Error Codes

Every failure mode has a defined code. No silent fallback.

> Guardrail: no error codes are invented here. Every row matches one variant of
> `enum Error` in `src/types.rs`. Every variant has a test in
> `src/error_paths.rs` that triggers the real return path.
> `scripts/check-errors.mjs` runs in CI and fails if this file and the enum
> drift apart.

## Categories

| Range | Category |
|---|---|
| 1–9 | Initialization & lookup |
| 10–29 | Lifecycle & timing |
| 30–49 | Validation & authorization |

## Initialization & lookup (1–9)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 1 | `NotInitialized` | `admin`, and every later read of setup state | The contract is called before `initialize` has run. | "This contract is not set up yet." | Ask the administrator to run setup. |
| 2 | `AlreadyInitialized` | `initialize` | `initialize` is called when an admin is already recorded. | "This contract is already set up." | No action needed. |
| 3 | `FeeNotFound` | `pay`, `close_fee`, `refund`, `get_fee`, `status` | No fee record exists for that id. | "We couldn't find that fee. Check the reference with the school." | Ask the school to confirm the fee reference. |
| 4 | `PayerNotFound` | `refund` | The school tried to refund an address with no payment record on this fee. | "That address has no payment on this fee, so there is nothing to refund." | Check the payer address and the fee. |

## Lifecycle & timing (10–29)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 10 | `FeeClosed` | `pay`, `close_fee`, `refund` | The fee has already been closed. | "This fee is closed — no more payments or refunds can be made." | Contact the school if you think this is wrong. |
| 11 | `CloseNotAllowed` | `close_fee` | The fee is partially paid: something is owed and something has been paid. | "This fee still has part of a payment on it. Refund the remaining payments before closing." | Refund the outstanding payments, then close the fee. |
| 12 | `DueDateInPast` | `create_fee` | The due date chosen is not in the future. | "The due date must be in the future." | Choose a new due date. |

## Validation & authorization (30–49)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 30 | `InvalidAmount` | `create_fee`, `pay`, `refund` | The total or the amount was zero or negative. | "Enter an amount greater than zero." | Correct the amount and try again. |
| 31 | `Overpayment` | `pay` | The payment is larger than the amount still owed. | "That is more than the amount still owed." | Pay the remaining balance or less. |
| 32 | `DuplicateReference` | `create_fee` | This school already has a fee with this reference. | "A fee with this reference already exists for this school." | Use a new reference, or open the existing fee. |
| 33 | `RefundExceedsPaid` | `refund` | The refund is larger than what this payer still has paid. | "You can refund at most what this payer still has paid." | Lower the refund amount. |
