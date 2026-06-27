# Index hot Flexweave stores without changing deterministic iteration

## Validation Verdict

Valid, with scope.

This strengthens Flexweave if private indexes sit behind the existing ordered storage and preserve all public deterministic iteration guarantees. It would muddy Flexweave if it became an unordered query engine or added `Hash`/`Ord` constraints to generic tag APIs.

## Problem

Flexweave deliberately uses ordered `Vec` storage to preserve deterministic iteration. That is a good public contract, but several hot lookup/removal paths are linear:

- Object existence and object-keyed maps.
- Ability id and active activation id lookup.
- Effect id, target, and tag lookup.
- Removal by id.

As state grows, callers pay for deterministic order with repeated scans, even when an internal index could preserve the same observable order.

## Evidence

- Determinism is a core purpose in README and crate docs.
- `ObjectMap` is a `Vec<(ObjectId, T)>`; `put`, `contains`, `get`, `replace_existing`, and `remove` scan linearly.
- `ObjectMap` backs `DataStore`, `Attribute`, and derived-attribute tracking.
- `ObjectStore` keeps ordered ids in a `Vec`; `exists` uses `contains`, while `iter` exposes creation/registration order.
- Tests assert object creation/registration order.
- `AbilityStore` uses ordered vectors for granted and active abilities, with linear id/activation lookup and removal.
- `ids_with_tag` promises deterministic grant order.
- `EffectPipeline` uses a vector; `get`, `remove_with_events`, `has_tag`, and `visit_target` scan linearly.
- Effect ticking promises deterministic instance order.
- `TagSet` is intentionally generic and vector-backed; `TagCollection` does not require `Hash` or `Ord`.

## What Would Muddy Flexweave

Do not replace deterministic ordered storage with unordered iteration.

Do not add `Hash` or `Ord` bounds to existing generic tag APIs.

Do not make query result order dependent on hash map iteration.

## Proposed Scope

Add private indexes while retaining ordered vectors as canonical iteration order.

Candidate indexes:

- `ObjectMap`: `ObjectId -> index`.
- `ObjectStore`: `ObjectId -> index` or membership set, plus ordered ids vector.
- `AbilityStore`: `AbilityId -> index`, `AbilityActivationId -> index`.
- Optional owner-ordered ability lookup for `ids_with_tag`, preserving grant order.
- `EffectPipeline`: `ActiveEffectId -> index`.
- Optional target-ordered effect lists for `visit_target` and target queries, preserving application order.

Avoid indexing `TagSet` unless introduced as an opt-in specialized type. Existing tag APIs should stay generic.

## Design Constraints

- Public iteration order remains canonical and deterministic.
- Indexes are implementation details unless intentionally exposed.
- Mutations must update indexes atomically with storage changes.
- Removal from vectors must repair affected indexes.
- Tests should exercise overwrite, remove, cancel, expire, and repeated iteration.

## Acceptance Criteria

- `ObjectStore::iter` order is unchanged.
- Query result order is unchanged.
- `AbilityStore` grant order and active activation order are unchanged.
- `EffectPipeline` application/tick order is unchanged.
- Lookup paths for object, ability, activation, and active effect ids no longer require full linear scans.
- Regression tests cover index consistency after:
  - Object map overwrite.
  - Object removal.
  - Ability grant and lookup.
  - Ability activation cancel/end.
  - Effect application, explicit removal, and expiration.
  - Repeated deterministic iteration.
- Existing tag tests pass without new `Hash`/`Ord` bounds.
