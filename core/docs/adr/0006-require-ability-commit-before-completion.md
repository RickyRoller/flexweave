# ADR 0006: Require Ability Commit Before Completion

## Status

Accepted

## Context

With explicit commitment, an uncommitted active activation represents work that has started but has not crossed its point of no return. Allowing normal completion of an uncommitted activation would make `Ended` overlap with cancellation and require consumers to infer whether an ability actually committed.

## Decision

Flexweave only ends committed activations. Calling end for an uncommitted active activation returns an explicit uncommitted-activation result or error and does not remove active state or emit `Ended`. Runtime code should call cancel when a started activation is abandoned before commit.

## Consequences

`Ended` means normal completion after commitment. `Canceled` means explicit abnormal termination before or after commitment without normal completion. Abilities with no caller-owned commit consequences can still use a no-op commit action before ending.
