# Reduce owned lifecycle-event cloning in mechanics hot paths

## Validation Verdict

Valid, with scope.

The strongest issue is runtime owned lifecycle event emission from `AbilityStore`, `EffectPipeline`, `MechanicsDriver`, and `EventChannel`. Definition structs also use owned `String`/`Vec`, but much of that is authoring-time. The runtime concern is clone pressure and `Clone` bounds in hot activation, application, ticking, and publication paths.

This strengthens Flexweave because these paths are likely to run per activation, effect application, effect tick, or event publish.

## Problem

Flexweave lifecycle events own caller payloads/tags/costs. Many event paths require `Clone` and clone active state solely to emit facts or retain them.

That makes large payloads expensive and non-`Clone` payloads unusable even when a caller only wants a streaming, non-retained event path.

## Evidence

- `EventChannel::publish` requires `Event: Clone + LifecycleEvent`.
- Retained channels clone each published event into a retained `Vec`.
- Listeners receive `&Event`, so the clone requirement is retention-driven, not listener-driven.
- Even `Drop` retention inherits the compile-time `Clone` bound.
- `EffectPipeline::apply_with_events` requires `Tags: Clone` and `Payload: Clone`.
- Effect application clones input tags/payload and clones active effect instances before storing/emitting.
- Effect ticking clones active effects for advance and periodic events.
- Ability lifecycle event structs own tags, cost, and payload.
- Ability store event APIs emit owned events and non-event wrappers still route through event paths with `Clone` bounds.
- `MechanicsDriver::tick` allocates a fresh `Vec<Event>`; `tick_with` streams but still emits owned events.
- `TagCollection` itself requires `Clone`, and `TagSet`/`Tag` are `Vec`-backed.
- `TagSet::has_tag_with_all_atoms` allocates a temporary vector.

## What Would Muddy Flexweave

Do not prematurely optimize every authoring structure or force all callers into borrowed event lifetimes.

Owned retained facts are useful for diagnostics, replay, tests, and projection. The issue is that owned retention and borrowed streaming are currently coupled too tightly.

## Proposed Scope

Split streaming lifecycle emission from owned retained event construction.

Possible direction:

- Add borrowed lifecycle event views for ability and effect emission.
- Keep existing owned event APIs as compatibility wrappers.
- Add no-event paths that avoid constructing event payloads and avoid unnecessary `Clone` bounds.
- Let `EventChannel` support non-retained publication without `Event: Clone`.
- Only require `Clone` when retaining or materializing owned facts.
- Add reusable-buffer or visitor alternatives for APIs that currently return fresh `Vec`s in mechanics paths.
- Consider a validated/compiled effect definition type so runtime application does not re-scan authoring metadata every call.

## Design Constraints

- Existing owned lifecycle events should remain available.
- Retained event semantics must stay deterministic.
- Borrowed event APIs must not make common call sites unusably complex.
- Avoid adding global allocation strategies or runtime dependencies.

## Acceptance Criteria

- Existing public behavior remains available.
- A hot streaming path can emit ability lifecycle facts without cloning large payloads solely for publication.
- A hot streaming path can emit effect lifecycle facts without cloning large payloads solely for publication.
- Non-retained event publication does not require `Event: Clone`.
- No-event ability/effect methods avoid event-construction clones.
- Tests or compile checks cover non-`Clone` payloads where owned retention is not requested.
- Documentation clearly distinguishes borrowed streaming, owned event materialization, and retained event costs.
