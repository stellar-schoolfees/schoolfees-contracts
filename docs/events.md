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

## Not implemented yet

- Fee lifecycle events (a record created, settled, expired, cancelled). These
  land with the v0 fee lifecycle and will be documented here in the same commit
  that adds them.
