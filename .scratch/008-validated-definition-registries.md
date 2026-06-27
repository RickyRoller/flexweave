# Add validated ability/effect definition registries and key-aware runtime workflows

## Validation Verdict

Valid, with boundary constraint.

This strengthens Flexweave if scoped as domain-neutral, registry-backed definition-key workflows for abilities and effects. It would muddy the purpose if it moved Studio catalog ownership, generated output ownership, or caller runtime bindings into core.

## Problem

Ability and effect definitions have stable keys and validation, but runtime workflows still tend to pass full definitions at grant/application sites. That creates repeated validation, repeated definition construction, and drift between authored definition metadata and runtime command calls.

Current examples/tests often hand-roll registry lookup and carry definition keys inside caller payloads.

## Evidence

- Flexweave owns mechanics primitives, registries, deterministic stores, and primitive errors, while not owning authored content storage or runtime bindings.
- `Registry` exists, but it is a light lookup wrapper and does not validate duplicate keys or own a compiled definition collection.
- `AbilityDefinition` has a `key` and validation.
- `AbilityStore::grant_with_definition` validates the passed definition, then stores a runtime grant without the definition key.
- Ability activation takes `AbilityCommitTiming` separately, so runtime activation can drift from definition metadata.
- `EffectDefinition` has a `key`.
- `EffectPipeline::apply_with_events` requires a full definition every application and validates it there.
- Active effect instances/events do not retain the originating definition key.
- Signals already model a stronger shape: validated `SignalDefinitions`, duplicate-key rejection, deterministic order, and projected facts carrying the definition key.
- Mechanics tests manually store/use definition keys and rebuild effect definitions at application time.

## What Would Muddy Flexweave

Do not move Studio catalog storage, generation paths, local authoring files, or runtime engine bindings into `core`.

Core should provide domain-neutral registered definitions and key-aware runtime helpers. Studio can generate or feed those definitions, but should remain a separate product surface.

## Proposed Scope

Add validated definition collections:

- `AbilityDefinitions`.
- `EffectDefinitions`.
- Or a shared typed validated definition registry.

Responsibilities:

- Validate definitions once at construction.
- Reject duplicate keys.
- Preserve declaration order.
- Lookup by stable caller-owned key.
- Provide typed errors for missing/duplicate/invalid definitions.

Add key-aware runtime helpers:

```rust
abilities.grant_registered(&ability_definitions, key, grant)
effects.apply_registered_with_events(&effect_definitions, key, input, emit)
```

Carry originating definition keys where useful:

- Granted ability.
- Active ability.
- Effect application.
- Active effect instance.
- Lifecycle events.
- Signal projection inputs/outputs.

## Design Constraints

- Preserve current direct-definition APIs where useful.
- Do not force every caller to use registries for tiny/manual simulations.
- Avoid duplicating Studio validation concepts that are not runtime primitives.
- Definition keys should remain caller-owned strings or a documented key type.

## Acceptance Criteria

- Ability definitions can be registered once, validated once, and looked up by key.
- Effect definitions can be registered once, validated once, and looked up by key.
- Duplicate keys are rejected deterministically.
- Registry order is deterministic and tested.
- Runtime helper grants an ability from a registered definition.
- Runtime helper applies an effect from a registered definition.
- Active/runtime lifecycle facts can include originating definition key where useful.
- Tests show activation/application cannot silently drift from definition metadata.
- Studio ownership boundaries remain documented and unchanged.
