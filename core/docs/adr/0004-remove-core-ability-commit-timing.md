# ADR 0004: Remove Core Ability Commit Timing

## Status

Accepted

## Context

Flexweave previously modeled `AbilityCommitTiming` so activations could commit automatically on start, on end, or through an explicit manual command. Adding a fallible synchronous commit transaction means automatic commit timing would force the same commit participant into begin, commit, and end command APIs, making the primitive surface harder to reason about.

## Decision

Flexweave core removes ability commit timing as command behavior and definition metadata. Ability commitment is always explicit: the runtime calls begin, commit, end, or cancel at the moments it chooses. If a runtime wants on-start, on-end, or instant behavior, it can compose those synchronous core commands in its own adapter API or carry orchestration hints in consumer-owned definition data.

## Consequences

Core has one commit path and one transactional boundary for caller-owned commit consequences. `AbilityDefinition` no longer carries `commit_timing`. Active ability state keeps a primitive committed flag so core can enforce commit, end, and rollback command invariants. Runtime adapters own orchestration policy and may add convenience helpers later without expanding the primitive core API.
