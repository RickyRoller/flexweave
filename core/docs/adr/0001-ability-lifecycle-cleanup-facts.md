# ADR 0001: Split Ability Cleanup Lifecycle Facts

## Status

Superseded by [ADR 0003](./0003-synchronous-ability-core-boundary.md).

## Context

Ability lifecycle facts are caller-visible mechanics facts. The `Canceled` fact
previously covered explicit cancellation, owner cleanup, and rollback cleanup.
Only explicit cancellation runs `AbilityHooks::on_cancel`; owner cleanup and
rollback cleanup remove active state without caller-owned cancel behavior.

## Decision

Ability lifecycle facts distinguish hook-backed cancellation from cleanup:

- `Canceled` means `on_cancel` completed and active ability state was removed.
- `Revoked` means active ability state was removed during owner revocation or
  destruction.
- `RolledBack` means active ability state was removed after activation startup
  or instant helper execution failed before normal completion.

## Consequences

- Consumers can route cancellation behavior separately from cleanup facts.
- Owner cleanup remains infallible and does not require caller hooks.
- Instant rollback can preserve the original execute, commit, or end error
  without invoking `on_cancel`.
