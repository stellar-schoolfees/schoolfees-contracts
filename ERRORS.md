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

## Lifecycle & timing (10–29)

No variants yet. These land with the v0 fee lifecycle, once the contract design
is fixed.

## Validation & authorization (30–49)

No variants yet. These land with the v0 fee lifecycle, once the contract design
is fixed.
