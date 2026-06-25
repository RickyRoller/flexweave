# Flexweave-Backed Mechanic Contract

Use this contract to decide whether authoring is actually complete.

## Meaning

A Flexweave-backed mechanic uses Flexweave Core for the runtime primitive that
owns the relevant lifecycle or state. Local data files, constants, and wrapper
functions are not proof that gameplay is Flexweave-backed.

## Discovery

Before editing runtime behavior, read the `FLEXWEAVE.md` Core adoption map and
inspect the named files. Classify the mechanic's path through these primitives:

- Object identity.
- Attributes.
- Abilities and cooldowns.
- Effects and lifecycle.
- Tags and queries.
- Mechanics ticking and events.

For each primitive the mechanic needs, use the repo's existing Flexweave-backed
seam when one exists. If the repo currently uses a manual system for that
responsibility, preserve it unless the user asked for migration, and call out the
partial adoption gap in `FLEXWEAVE.md` and the final response.

## Completion Criteria

The mechanic is complete only when all of these are true:

- Flexweave Core is installed in the owning runtime crate.
- Runtime behavior flows through the repo's Flexweave-backed seam for every
  adopted primitive it claims to use.
- Bounded runtime state such as health/resources includes both current and
  maximum values in the repo's Flexweave-backed attribute/data seam, unless the
  integration map documents a different adopted shape.
- Ability-backed mechanics perform their primary gameplay command through the
  Flexweave ability activation executor or hook path. A no-op executor followed
  by separate lifecycle-event dispatch is not enough for damage, effect
  application, resource spending, or target mutation.
- UI projections, death/despawn, status changes, and other observable reactions
  consume emitted Core facts/events from attributes, abilities, effects, or
  mechanics ticking. Polling current store values is not proof of event-backed
  integration, except for initial render or fallback display.
- Existing compile/check commands for the runtime pass.
- Gameplay-facing tests or scenarios exercise the mechanic behavior.
- `FLEXWEAVE.md` records new mechanics, runtime entry points, Core primitives,
  and any partial adoption gaps.

Use existing compile/check commands for runtime confidence. Gameplay tests should
cover behavior such as activation, cooldown, attribute mutation, effect
application, targeting, ticking, or emitted mechanics events.

## Core Primitives

Map the mechanic to the smallest Core primitive that owns the lifecycle:

- `ObjectId` and `ObjectStore` for stable runtime identity.
- `DataStore`, `Attribute`, and `DerivedAttribute` for attached or computed
  state.
- `TagSet`, `TagCollection`, and `TagSetQuery` for grouping and queries.
- `AbilityStore`, `AbilityDefinition`, and ability lifecycle events for
  activation and cooldowns.
- `EffectPipeline`, `EffectDefinition`, and effect lifecycle events for
  application, duration, ticking, and removal.
- `MechanicsDriver`, `Clock`, and `ClockUnits` for deterministic ticking.
- `EventChannel`, lifecycle events, signals, and registries for reusable event
  and definition flow.
