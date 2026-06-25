# Flexweave Primitives Overview

Use this as a quick map when choosing which Flexweave primitive should own a
mechanic responsibility. Keep game-specific rules, numbers, targeting, and payload
meaning in the consumer runtime.

## Identity

- `ObjectId`: stable domain-neutral handle for a game object, actor, projectile,
  or other runtime participant.
- `ObjectStore`: deterministic allocator and live-id list. Use when Flexweave
  should own mechanics identity independent of engine/entity ids.

## Attached State

- `DataStore<T>`: object-keyed typed data. Use for non-numeric attached state,
  such as owner links, actor kind, spawned entity handles, or mechanic metadata.
- `Attribute`: object-keyed numeric state with mutation events and hooks. Use
  for health, resource pools, charges, damageable values, and other numbers that
  need controlled mutation or event-backed projections.
- `DerivedAttribute`: read-only numeric projection with refresh events. Use for
  computed values such as Max health, Max resource pool, or other numbers that are
  the result of a calculation.

## Classification And Targeting

- `Tag` and `TagSet`: grouped labels for abilities, effects, objects, or facts.
  Use when mechanics need stable categories such as `ability.attack` or
  `effect.slow`.
- `TagSetQuery`: exact tag inclusion/exclusion query.

## Abilities

- `AbilityDefinition`: authorable shape for an ability, including activation
  mode, lifecycle routing, cancellation, and payload schema notes.
- `AbilityStore`: grants abilities, enforces cooldowns, manages active
  activations, and emits ability lifecycle events. Use for actions a character
  can activate, especially when readiness or cooldown matters.
- `AbilityHooks`: caller-owned checks and cost/cooldown commit hooks. Use for
  runtime rules such as "can cast", spend validation, or activation policy.
- `ActiveAbility`: activation snapshot passed to execution. Use the activation
  executor for the ability's primary gameplay command, then let lifecycle events
  project what happened.

## Effects

- `EffectDefinition`: authorable shape for instant, duration, periodic, or
  indefinite effects.
- `EffectPipeline`: applies effects, owns active duration/periodic instances,
  ticks them, removes them, and emits effect lifecycle events. Use for status
  effects, timed modifiers, periodic damage, and instant executions modeled as
  effects.
- `EffectApplicationInput`: caller-selected application attempt. Use to route
  source, target, tags, payload, and accept/reject decisions into the pipeline.
- `EffectInstance`: active effect state. Read it for projections such as
  movement multipliers while the effect is live.

## Time And Ticking

- `Clock`, `ClockUnits`, `FixedStepClock`, and `RealtimeClock`: convert game
  time into deterministic Flexweave units. Use before ticking cooldowns or effects.
- `MechanicsStore`: trait for stores that advance by elapsed units.
- `MechanicsDriver`: ticks several Flexweave stores together and collects events. Use
  when a runtime wants one shared mechanics tick path.

## Events And Projections

- `EventChannel`: typed lifecycle event channel with optional retention and
  listeners. Use when UI, diagnostics, or follow-on mechanics
  need to consume emitted Flexweave facts.
- `LifecycleEventKind`, `LifecycleEvent`, and `LocalLifecycleEvent`: common
  lifecycle fact classifications for ability, effect, attribute, and signal
  events.
- `SignalDefinition`, `SignalFact`, and `SignalProjection`: reusable signal
  projection from lifecycle facts. Use when mechanics need exported or retained
  facts beyond a local event channel.

## Definitions And Lookup

- `Registry<Entry>`: deterministic lookup over caller-owned definition records.
  Use when a runtime has a local catalog of abilities, effects, tags, or other
  authored definitions.
- `DefinitionRegistryEntry`: builds a Flexweave definition from a registry entry. Use
  when data/catalog records should materialize runtime definitions on demand.
