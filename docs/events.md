# Contract events

Only events the code actually emits are documented here. Anything not listed is
**not implemented yet**.

Events are defined with `#[contractevent]` in `src/types.rs` and published with
`EventName { .. }.publish(&env)`. The SDK puts the event's name into the topics
as a `Symbol`, followed by any fields marked `#[topic]`; other fields land in the
data map.

## `Initialized`

Emitted once, by `initialize`, after the administrator is recorded.

| | |
|---|---|
| Topics | `Symbol("initialized")`, then the admin `Address` |
| Data | empty map (the event has no non-topic fields) |
| Emitted by | `initialize` in `src/lib.rs` |
| Asserted in | `initialize_publishes_an_initialized_event` in `src/test.rs` |

A downstream indexer can therefore filter by the `initialized` topic and read the
administrator from the second topic. The administrator is an address, never a
name — see the privacy rules in [AGENTS.md](../AGENTS.md).

## `FeeCreated`

Emitted once per fee, by `create_fee`.

| | |
|---|---|
| Topics | `Symbol("fee_created")`, then `fee_id` (`u64`) |
| Data | map with `school` (`Address`), `token` (`Address`), `reference` (`BytesN<32>`), `total` (`i128`), `due_at` (`u64`) |
| Emitted by | `create_fee` in `src/fee.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

## `FeePaid`

Emitted on every successful payment, by `pay`. The token contract emits its own
`transfer` event in the same invocation; the filter used in tests keeps only
this contract's events.

| | |
|---|---|
| Topics | `Symbol("fee_paid")`, then `fee_id` (`u64`) |
| Data | map with `payer` (`Address`) and `amount` (`i128`) |
| Emitted by | `pay` in `src/fee.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

## `FeeRefunded`

Emitted on every successful refund, by `refund`.

| | |
|---|---|
| Topics | `Symbol("fee_refunded")`, then `fee_id` (`u64`) |
| Data | map with `payer` (`Address`) and `amount` (`i128`) |
| Emitted by | `refund` in `src/fee.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

## `FeeClosed`

Emitted once, by `close_fee`, after the fee is closed.

| | |
|---|---|
| Topics | `Symbol("fee_closed")`, then `fee_id` (`u64`) |
| Data | empty map (the event has no non-topic fields) |
| Emitted by | `close_fee` in `src/fee.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

## Not implemented yet

- No other events exist yet. New events are documented here in the same commit
  that adds them to `src/types.rs`.
