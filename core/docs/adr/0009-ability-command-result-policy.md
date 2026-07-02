# ADR 0009: Ability Command Result Policy

## Status

Accepted

## Context

Ability lifecycle commands need to distinguish caller-owned transaction failures, invalid command states, idempotent cleanup misses, and successful primitive transitions. Treating every non-transition as either an error or an outcome makes some consumer flows awkward.

## Decision

Begin, commit, and end use `Result` for invalid command states and caller-owned failures. Commit may return an already-committed success outcome for idempotency. Cancel remains an infallible cleanup command that returns either canceled active state or missing activation. Owner revocation remains infallible cleanup.

## Consequences

Missing activation is an error for commit and end, where it usually indicates stale runtime state. Ending an uncommitted activation is an error and leaves active state in place. Missing activation is an outcome for cancel, where repeated cleanup is expected. Commit action failure is an error after Flexweave removes active state and emits `RolledBack`.
