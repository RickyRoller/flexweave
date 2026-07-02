# ADR 0008: Ability Commit Facts Carry Active State

## Status

Accepted

## Context

Runtime adapters supervise ability tasks by `AbilityActivationId`. Most ability lifecycle facts already carry active activation state, but the previous committed fact carried only the original activation attempt, forcing consumers to correlate the commit fact back to active state indirectly.

## Decision

Ability committed facts carry active activation state with the committed flag set. The owned and borrowed event variants should align with the other active lifecycle facts: `Committed(ActiveAbility)` and `Committed(ActiveAbilityView)`.

## Consequences

Runtime supervisors can process `Committed`, `Ended`, `Canceled`, `Revoked`, and `RolledBack` using the same activation identity path. The separate `AbilityActivationCommit` fact type is no longer needed unless a future API needs additional commit-specific data.
