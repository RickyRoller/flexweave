# Clarify lifecycle events, event channels, signals, and projection responsibilities

## Validation Verdict

Valid.

Flexweave has a coherent implied model, but it is not clear enough in public docs and examples. Some interfaces make routing/channel keys look operational even though core currently treats them as metadata unless caller code wires publication.

This strengthens Flexweave by making the runtime fact/projection model explicit.

## Problem

Consumers can reasonably misunderstand:

- Whether lifecycle events are raw mechanics facts or application events.
- Whether `EventChannel` is a full event bus or a caller-owned transport/retention primitive.
- Whether signal channel keys auto-route anything.
- Whether signals replace lifecycle events or are derived from them.
- Whether signal projection is generic or currently effect-lifecycle based.
- Whether route keys on ability/effect definitions have runtime behavior without caller wiring.

## Evidence

- `EventChannel` validates accepted `LifecycleEventKind`, optionally retains events, and notifies listeners. It does not auto-route from stores.
- Signals are described as projection from mechanics lifecycle facts.
- `SignalDefinition` carries signal kind, lifecycle event kinds, channel key, retention, export, tags, and payload metadata.
- `SignalProjection` currently projects from `EffectLifecycleEvent` and active effect instances.
- `SignalFact` implements `LifecycleEvent` by returning the source lifecycle kind.
- `MechanicsDriver` only returns or streams lifecycle events; it does not project signals or dispatch channels.
- `EffectRouting` exposes lifecycle and signal channel keys, but tests manually publish events/facts into channels.
- Public README/CONTEXT docs are terse; the clearest distinctions are in agent skill docs rather than crate-facing docs.

## What Would Muddy Flexweave

Do not turn core into an application event bus.

Flexweave should own raw mechanics facts, signal projection primitives, and simple caller-owned channels. It should not own a game engine event system, networking bus, UI event model, or observer framework.

## Proposed Scope

Add public documentation and examples that define the model:

- Lifecycle events are raw facts emitted by attributes, abilities, effects, and mechanics ticking.
- `EventChannel` is a typed, caller-owned transport/retention primitive.
- `SignalProjection` creates derived/exportable facts from source lifecycle facts.
- Routing/channel keys are metadata and validation hints unless caller code wires publication.
- Signals do not replace lifecycle facts; they are projected facts intended for export, runtime reactions, or author-defined semantics.

Add examples:

1. Raw lifecycle event flow:
   - Ability/effect emits lifecycle event.
   - Caller publishes it to an `EventChannel`.
   - Consumer drains retained facts or receives listener callback.

2. Projection flow:
   - Effect lifecycle event is emitted.
   - `SignalProjection` projects one or more `SignalFact`s.
   - Caller publishes signal facts into a channel or runtime bus.

3. Routing metadata:
   - Definition declares channel keys.
   - Caller validates keys.
   - Caller explicitly wires publication.

## Design Constraints

- Keep core dependency-free and runtime-neutral.
- Do not imply automatic routing unless it exists.
- If future auto-routing is added, it should be explicit in a separate driver/API.
- Keep examples small and domain-neutral.

## Acceptance Criteria

- `core/README.md` and crate docs explain lifecycle events, event channels, signals, and projection responsibilities.
- Public docs state that channel keys are metadata unless caller code wires publication.
- Examples show raw event flow and signal projection flow.
- Tests or doc tests demonstrate projection and manual channel publication.
- Bevy or other engine integration is referenced only as an adapter concern, not core behavior.
