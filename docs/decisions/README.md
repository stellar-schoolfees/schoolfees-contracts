# Design decisions

One file per decision that a future contributor should not have to re-derive:
`NNNN-short-title.md`. Keep each one short — context, decision, consequences —
and dated.

The convention is a lightweight ADR:

```markdown
# 0001. Short title

Date: YYYY-MM-DD
Status: accepted | superseded by NNNN

## Context
What forced a choice, in two or three sentences.

## Decision
What was chosen, precisely. Pin versions where a library is involved.

## Consequences
What this makes easy, what it makes hard, and what would change our mind.
```

## Recorded so far

- **0001 — Token transfers and OpenZeppelin dependencies** (*accepted*,
  2026-09-30): no OpenZeppelin dependency in v0; token movement via the
  soroban-sdk `TokenClient`. Approved with
  [`docs/design/interface-v0.md`](../design/interface-v0.md).
