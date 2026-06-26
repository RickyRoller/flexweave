# Add domain-neutral object destruction and object-keyed cleanup APIs

## Validation Verdict

Valid, with scope correction.

Flexweave should own domain-neutral object liveness and cleanup of Flexweave-owned object-keyed state. It should not own product-specific death, despawn, persistence, runtime entity removal, or gameplay policy.

This strengthens Flexweave because object identity, object-keyed data, attributes, abilities, effects, and deterministic queries are already part of the crate purpose. Without a destruction path, every caller must remember which Flexweave stores can retain stale `ObjectId`s.

## Problem

`ObjectStore` calls its ids live and exposes `exists`, `iter`, and `count`, but it has no `destroy` or `remove` operation. Public object-keyed stores also do not provide a coordinated cleanup path.

The practical outcome is that a consumer can remove an object from its runtime, while Flexweave continues to retain:

- Attached data in `DataStore`.
- Attribute values in `Attribute`.
- Derived attribute tracking/cache entries.
- Granted and active abilities owned by the object.
- Active effects sourced from or targeting the object.
- Query-visible ids if the object remains in `ObjectStore`.

## Evidence

- `ObjectStore` creates/registers ids, checks existence, iterates, and counts, but exposes no removal path: `core/src/identity.rs`.
- `ObjectMap` already has internal `remove`, but public stores generally do not expose object cleanup: `core/src/object_map.rs`.
- `DataStore` attaches values and has no public detach/remove operation: `core/src/data_store.rs`.
- `Attribute` is object-keyed and emits value-change facts, but not object cleanup facts: `core/src/attribute.rs`.
- `DerivedAttribute` can remove private cache entries when calculation returns `None`, but has no explicit object untrack path: `core/src/derived_attribute.rs`.
- `AbilityStore` stores granted and active abilities by `owner_id`, but cannot revoke by owner: `core/src/ability/store.rs`.
- `EffectPipeline` stores source and target ids, but only removes by `ActiveEffectId`: `core/src/effect/pipeline.rs`.
- Queries iterate current `ObjectStore` ids, so object destruction would naturally affect query results once ids can be removed: `core/src/query.rs`.

## What Would Muddy Flexweave

Do not make `ObjectStore` a game world.

Flexweave should not decide:

- Whether health zero means death.
- Whether effects from a removed source should persist for gameplay reasons.
- Whether a Bevy entity, server actor, save record, or authored asset should be removed.
- How consumer runtime resources outside Flexweave are destroyed.

Those are caller-owned semantics. Flexweave should only provide domain-neutral liveness and cleanup primitives.

## Proposed Scope

Add object destruction and cleanup APIs that preserve deterministic order and do not reuse ids.

Candidate surface:

```rust
impl ObjectStore {
    pub fn destroy(&mut self, id: ObjectId) -> Result<ObjectId, CoreError>;
}
```

Add cleanup APIs for object-keyed stores:

- `DataStore::detach(id)` or `remove(id)`.
- `Attribute::detach(id)` or `remove(id)`.
- `DerivedAttribute::untrack(id)` or `remove_cached(id)`.
- `AbilityStore::revoke_owner_with_events(owner_id, emit)` and/or no-event variant.
- `EffectPipeline::remove_for_object_with_events(id, policy, emit)`.

Consider a lightweight cleanup driver:

```rust
pub trait ObjectLifecycleStore<Event> {
    fn remove_object(&mut self, id: ObjectId, emit: &mut dyn FnMut(Event));
}
```

This mirrors the existing mechanics ticking driver idea without creating a monolithic world.

## Design Constraints

- Destroyed ids must not be reused.
- Remaining object iteration order must be preserved.
- Cleanup must not require game-specific state.
- Ability/effect cleanup should emit existing lifecycle facts where state is removed, such as activation canceled or effect removed.
- Existing unchecked/manual cleanup paths can remain for compatibility.

## Acceptance Criteria

- `ObjectStore::destroy` rejects invalid and missing ids.
- Destroyed ids disappear from `ObjectStore::exists`, `iter`, and query helpers.
- Object-keyed stores expose explicit cleanup for a removed object.
- Ability cleanup can revoke grants and cancel active activations for an owner.
- Effect cleanup can remove effects whose source or target matches an object, with documented policy.
- Tests prove destroyed ids do not remain query-visible.
- Tests prove cleanup removes retained Flexweave state from registered stores.
- Tests prove ids remain deterministic and are not reused.

