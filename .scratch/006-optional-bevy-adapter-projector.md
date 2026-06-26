# Add optional Bevy adapter/projector crate without adding Bevy to core

## Validation Verdict

Valid, with boundary constraint.

Flexweave has the right core shape for a Bevy adapter/projector, but the adapter does not exist today. The fix should be an optional runtime adapter crate/example, not Bevy code inside the `flexweave` core crate.

This strengthens adoption while preserving the runtime-neutral core contract.

## Problem

Bevy consumers currently need to rebuild the same glue:

- `ObjectId` to Bevy `Entity` mapping.
- Bevy time to Flexweave `ClockUnits`.
- Bevy schedule systems that tick registered stores.
- Lifecycle facts and `SignalFact`s to Bevy messages/events.
- Projection over `SignalProjection`.
- Spawn/despawn cleanup coordination.

The Bevy example implemented this manually, which makes every consumer rediscover ordering, projection, and cleanup details.

## Evidence

- The Flexweave Rust workspace currently includes only `core`.
- `core` has no Bevy dependency and should remain domain/runtime agnostic.
- Core exports adapter-relevant primitives: `Clock`, `MechanicsDriver`, `EventChannel`, `SignalProjection`, `ObjectStore`, and object/data/attribute/ability/effect stores.
- Public docs say callers project lifecycle facts into their own runtime model and map clocks through adapters.
- Public docs say Flexweave does not own caller runtime bindings.
- Tests manually create runtime state, hooks, effect application, event vectors, and tick drivers.
- Signal projection is generic but caller-routed.
- Repo search found no Bevy adapter/example in the Flexweave workspace.

## What Would Muddy Flexweave

Adding Bevy as a dependency of `core` would muddy the purpose.

Core should remain:

- Domain-neutral.
- Engine-neutral.
- Dependency-light.
- Suitable for non-Bevy game/simulation runtimes.

The Bevy integration should be a separate crate or example surface.

## Proposed Scope

Add a separate workspace member, for example:

- `adapters/bevy`
- `crates/flexweave-bevy`
- `flexweave-bevy`

Keep `core/Cargo.toml` free of Bevy dependencies.

Adapter responsibilities:

- `ObjectId` to `Entity` mapping resource.
- Helpers for object creation/destruction projection into Bevy.
- Bevy `Time` to `ClockUnits` adapter, ideally using the realtime accumulator issue.
- A plugin/system for ticking registered Flexweave stores.
- Projection from lifecycle facts to Bevy `Message` or current Bevy event APIs.
- Projection from `SignalFact` to Bevy messages/events.
- A small API over `SignalProjection`.

Add a minimal example demonstrating:

- Spawn object and map to Bevy entity.
- Apply effect.
- Tick stores from Bevy time.
- Project lifecycle events/signals.
- Consume projected runtime facts in a Bevy system.

## Design Constraints

- Do not add Bevy to `flexweave` core.
- Target the chosen Bevy version's current buffered message/event APIs.
- Preserve deterministic ordering when projecting facts.
- Keep adapter APIs thin enough that core remains the source of mechanics semantics.

## Acceptance Criteria

- New optional Bevy crate/example exists outside core.
- Core crate dependency graph remains unchanged with respect to Bevy.
- Adapter can map `ObjectId` to Bevy `Entity` deterministically.
- Adapter can tick Flexweave stores from Bevy time.
- Adapter can publish lifecycle facts and signal facts into Bevy runtime messages/events.
- Example covers spawn mapping, effect application, ticking, projection, and consumption.
- Tests or example checks prove projection order is deterministic.

