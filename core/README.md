# Flexweave

Flexweave is the Rust crate for deterministic mechanics primitives. The
crate package name is `flexweave`.

Use Flexweave when a caller needs a domain-neutral foundation for objects, attached
data, attributes, tags, abilities, effects, signals, lifecycle clocks, and
explicit primitive errors.

## Primitive Model

An Object is a stable mechanics handle. Flexweave allocates Object ids in
deterministic creation order, and Object stores preserve deterministic
iteration order.

Attached data adds caller-owned meaning to an Object id. A Data store holds one
attached value type and reports primitive errors for invalid Object ids or
missing required data.

Attributes expose signed numeric channels on Objects. Derived attributes are
calculated from caller-owned state. Attribute changes report previous and
current values without assigning product meaning to either value.

Tags attach deterministic labels to Objects and support repeatable tag queries.
Queries preserve Object iteration order so identical inputs produce identical
selection order.

Abilities describe activation lifecycle, grants, active state, explicit
commitment, cancellation, rollback, revocation, and completion. Caller-owned
synchronous gates decide whether active state may be created, and caller-owned
commit actions apply point-of-no-return consequences during explicit commit.
Use `AbilityGrant::new(...).checked(...)` and
`AbilityActivation::new(...).for_owner(...).run_with_executor(...)` in common
runtime flows so ability owners are validated against live objects and expected
owners before activation gates run.

Effects describe application, execution, active lifetime, advancement,
removal, and expiration. Active effect instances carry runtime effect state for
a finite or indefinite lifetime. Use
`EffectApply::definition(...).checked(...)` with an explicit
`EffectSourcePolicy` when an `ObjectStore` is available; unchecked
`EffectApply` commands are reserved for callers that intentionally manage
object-reference validity themselves.

State-changing runtime commands return explicit primitive outcomes. Ability
commit distinguishes committed from already committed, end requires committed
active state, cancel distinguishes canceled from missing activation, and
rollback removes only uncommitted active state. Effect application
distinguishes rejected applications, accepted instant executions, and active
effect creation. Ability gate and commit-action failures are returned through
command errors without requiring callers to inspect lifecycle events. Cooldowns
and costs are modeled by caller-owned effects, tags, attributes, gates, and
commit actions rather than stored inside abilities.

Lifecycle events are raw mechanics facts emitted by attributes, derived
attributes, abilities, effects, and mechanics ticking. They describe what the
Flexweave primitive did. They are not application events, engine events, UI
events, network messages, or persisted audit records until caller code maps
them into that model.
Ability cancellation facts mean caller-owned cancel behavior ran. Ability
revocation and rollback facts are cleanup facts for owner removal and failed
activation helpers.

Event channels are typed, caller-owned transport and retention primitives. An
`EventChannel` validates the published `LifecycleEventKind`, optionally retains
published facts, and notifies subscribed listeners in deterministic order. It
does not subscribe to stores, discover definitions, or auto-route emitted facts.
Callers publish facts into channels from ability command callbacks, pipeline callbacks, or
`MechanicsTick::new(...).run_streaming(...)`.

Signals are derived facts created by `SignalProjection` from source lifecycle
facts. The current projection surface is effect-lifecycle based, including
reinvocation for active effect instances. Signals do not replace lifecycle
events; they are projected facts intended for export, runtime reactions, or
author-defined semantics.

Channel keys on ability, effect, and signal definitions are metadata and
validation hints unless caller code wires publication. Definition validation can
prove that keys are known, but runtime behavior appears only when the caller
chooses an `EventChannel` or adapter and publishes the fact.

Validated `AbilityDefinitions` and `EffectDefinitions` are caller-constructed
runtime bundles. A caller can build one bundle per zone, encounter, content
pack, or session, validate duplicate keys at that composition point, and pass
the active bundle to registered runtime helpers. Flexweave does not own catalog
loading or require one central definition registry for an entire game.

Lifecycle events have two runtime shapes. Borrowed event views stream through
callbacks without cloning caller-owned payloads for publication. Owned lifecycle
events remain available for retained facts, diagnostics, replay, tests, and any
caller API that needs events to outlive the callback. Retained event channels
store owned events; borrowed publication is limited to drop-only channels.

Clock units are opaque `u64` mechanics units. Callers map their own clocks into
those units through fixed-step or real-time adapters. `RealtimeClock` is a
stateless flooring conversion for one-shot durations; use
`RealtimeClockAccumulator` when repeated frame deltas need to preserve
fractional clock units across ticks.

## Event and Signal Flow

Raw lifecycle event publication:

```rust
use flexweave::{
    AttributeChange, EventChannel, EventChannelDefinition, EventRetention,
    LifecycleEventKind, ObjectId,
};

let definition = EventChannelDefinition::new(
    "attributes/changes",
    [LifecycleEventKind::AttributeChanged],
)
.unwrap();
let mut channel = EventChannel::with_retention(definition, EventRetention::Retain);

let event = AttributeChange {
    id: ObjectId::new(1),
    previous: Some(10.0),
    requested: 12.0,
    current: 12.0,
};

// Publication is caller-owned.
channel.publish(event).unwrap();

let retained = channel.drain_retained();
assert_eq!(retained[0].current, 12.0);
```

Signal projection and signal publication:

```rust
use flexweave::{
    EffectExecution, EffectLifecycleEvent, EventChannel, EventChannelDefinition,
    EventRetention, LifecycleEventKind, ObjectId, SignalDefinition,
    SignalDefinitions, SignalExportPolicy, SignalFact, SignalKind,
    SignalProjection, SignalRetentionPolicy, SignalTagMatch, Tag, TagSet,
};

#[derive(Clone, Eq, PartialEq)]
enum Atom {
    Impact,
}

let definitions = SignalDefinitions::new([SignalDefinition {
    key: "impact".to_owned(),
    signal_kind: SignalKind::Executed,
    lifecycle_event_kinds: vec![LifecycleEventKind::EffectExecuted],
    tag_match: SignalTagMatch::Any,
    payload_schema: "impact.v1".to_owned(),
    signal_payload: "exportable impact",
    channel_key: "signals/effects".to_owned(),
    category: "runtime".to_owned(),
    retention: SignalRetentionPolicy::Retain,
    export: SignalExportPolicy::Export,
    debug_label: "Impact".to_owned(),
    description: "An effect execution projected for adapters".to_owned(),
}])
.unwrap();
definitions.validate_channels(&["signals/effects"]).unwrap();
let projection = SignalProjection::new(definitions);

let event = EffectLifecycleEvent::Executed(EffectExecution {
    active_effect_id: None,
    source_id: Some(ObjectId::new(1)),
    target_id: ObjectId::new(2),
    tags: TagSet::new([Tag::new([Atom::Impact])]),
    payload: "source payload",
    elapsed_units: None,
});
let facts = projection.project_effect_event(&event);

let channel_definition = EventChannelDefinition::new(
    "signals/effects",
    [LifecycleEventKind::EffectExecuted],
)
.unwrap();
let mut channel: EventChannel<SignalFact<Atom, &str, &str>> =
    EventChannel::with_retention(channel_definition, EventRetention::Retain);

assert!(channel.retained().is_empty());
for fact in facts {
    channel.publish(fact).unwrap();
}

assert_eq!(channel.drain_retained()[0].key, "impact");
```

Routing metadata flow:

1. A definition declares lifecycle or signal channel keys.
2. Caller-owned validation checks those keys against known channel definitions.
3. Caller code explicitly publishes emitted lifecycle facts or projected signal
   facts into the selected channel or runtime adapter.

## Determinism

Flexweave avoids unordered public iteration where ordering is part of the contract.
Object ids, Data store scans, tag queries, and mechanics-store queries are
designed to produce repeatable results from identical inputs.

## Commands

```bash
cargo build -p flexweave
cargo test -p flexweave
cargo clippy -p flexweave --all-targets -- -D warnings
cargo doc -p flexweave --no-deps
```

## Boundary

Flexweave owns object identity, attached data, attributes, derived attributes, tags,
queries, abilities, effects, registries, signals, caller-defined clock units,
deterministic mechanics stores, and primitive errors.

Flexweave does not own authored content storage, generated output paths, design UI,
caller runtime bindings, or engine event systems. Engine integrations belong in
adapters that translate Flexweave lifecycle facts and Signals into the caller's
runtime model.
