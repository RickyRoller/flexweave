# ADR 0010: Expose Explicit Ability Rollback

## Status

Accepted

## Context

Runtime adapters can fail after beginning an activation but before commitment, such as during setup, task startup, targeting, or other caller-owned execution. Mapping those failures to cancellation would make intentional abandonment and failed activation execution indistinguishable.

## Decision

Flexweave exposes an explicit primitive rollback command. Runtime adapters call cancel when an activation is intentionally abandoned or interrupted, and rollback when activation execution fails before commitment. Rollback removes uncommitted active state, emits `RolledBack`, and does not accept a caller participant. Rolling back a committed activation is an explicit error and leaves active state in place.

## Consequences

Runtime supervisors and diagnostics can distinguish `Canceled` from `RolledBack` while keeping both commands synchronous and primitive. Rollback remains a before-commit concept; after commitment, abnormal termination is cancellation and normal termination is completion. Commit action failure continues to roll back automatically through the commit command.
