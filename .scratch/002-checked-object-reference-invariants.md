# Make object-reference invariants harder to bypass

## Validation Verdict

Valid, with scope correction.

Flexweave should enforce or strongly guide domain-neutral object-reference invariants. It should not own game-specific alive/dead rules beyond object membership unless that state is modeled through a generic Flexweave primitive.

This strengthens Flexweave because object identity is already a core primitive. References between abilities, effects, sources, owners, and targets should not be easy to make invalid by accident.

## Problem

Several runtime operations accept raw `ObjectId`s and trust caller convention:

- Ability grants accept any owner id.
- Ability activation does not take an expected actor/source id.
- Effect applications accept any source/target ids.
- Signal projection exports whatever effect source/target ids were stored.

The result is that invalid ids, stale ids, owner mismatches, and source mismatches can become accepted runtime state unless every caller manually validates them.

## Evidence

- `ObjectStore` has domain-neutral live object validation through `exists`: `core/src/identity.rs`.
- `query::require_object` exposes object validation as a helper: `core/src/query.rs`.
- `Grant` accepts a raw `ObjectId` owner: `core/src/ability/store.rs`.
- `AbilityStore::grant` copies the owner id directly and `grant_with_definition` only validates authoring metadata: `core/src/ability/store.rs`.
- `begin_activation_with_events` checks missing ability, cooldown, and hooks, but does not validate a requested actor/source id: `core/src/ability/store.rs`.
- Default `AbilityHooks::can_activate` returns `Ok(())`, so validation is optional: `core/src/ability/hooks.rs`.
- `EffectApplicationInput` accepts raw `source_id` and `target_id`: `core/src/effect/application.rs`.
- `EffectPipeline::apply_with_events` validates only the effect definition before storing/emitting source and target ids: `core/src/effect/pipeline.rs`.
- Signal projection copies effect source and target ids into exported facts: `core/src/signal/projection.rs`.

## What Would Muddy Flexweave

Do not encode caller-domain rules such as:

- Health zero means not alive.
- A stunned unit cannot activate.
- A target is hostile, friendly, visible, reachable, or in range.
- A gameplay object can or cannot receive a particular effect.

Those remain hook or caller logic.

Flexweave should focus on domain-neutral reference validity:

- Is this `ObjectId` live in the supplied object store?
- Does this ability belong to this owner?
- Is this effect source either absent by explicit policy or live?
- Is this effect target live?
- Does an active ability-derived effect use the active ability owner as source?

## Proposed Scope

Add checked paths while preserving low-level unchecked APIs.

Candidate APIs:

```rust
impl<Tags, Cost, Payload> AbilityStore<Tags, Cost, Payload> {
    pub fn grant_checked(
        &mut self,
        objects: &ObjectStore,
        grant: Grant<Tags, Cost, Payload>,
    ) -> Result<AbilityId, AbilityGrantError>;

    pub fn begin_activation_for_with_events(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        /* existing args */
    ) -> Result<AbilityActivationId, AbilityActivationError<_>>;
}
```

For effects:

```rust
impl<Tags, Payload> EffectPipeline<Tags, Payload> {
    pub fn apply_checked_with_events(
        &mut self,
        objects: &ObjectStore,
        definition: &EffectDefinition<_>,
        input: EffectApplicationInput<Tags, Payload>,
        emit: impl FnMut(EffectLifecycleEvent<Tags, Payload>),
    ) -> Result<EffectApplicationOutcome, EffectApplicationError>;
}
```

Add convenience builders that derive source from `ActiveAbility`:

```rust
impl<Tags, Cost, Payload> ActiveAbility<Tags, Cost, Payload> {
    pub fn source_id(&self) -> ObjectId;
}
```

or an effect input helper:

```rust
EffectApplicationInput::from_active_ability(&active, target_id, tags, payload)
```

## Design Constraints

- `source_id: None` must remain valid for environmental/system effects, but that policy should be explicit.
- Existing raw methods can remain documented as unchecked/low-level paths.
- Checked methods should reject invalid ids before hooks that assume valid references.
- Error types should distinguish invalid source, invalid target, invalid owner, and owner mismatch.

## Acceptance Criteria

- Tests reject granting an ability to an invalid or missing owner id.
- Tests reject activation when the expected owner does not match the ability owner.
- Tests reject effect application with an invalid/missing target.
- Tests reject effect application with an invalid/missing explicit source.
- Tests allow `source_id: None` when the selected policy permits system/environment effects.
- Tests cover a helper that derives effect source from an active ability.
- Public docs route common examples through checked paths and label raw methods as unchecked.
