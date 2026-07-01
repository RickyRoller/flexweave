#![deny(unsafe_code)]
#![doc = "Domain-agnostic mechanics primitives for game and simulation runtimes."]
#![doc = ""]
#![doc = "Flexweave provides reusable building blocks for object identity, attached"]
#![doc = "data, attributes, derived attributes, tags, queries, abilities, active"]
#![doc = "effects, definition registries, caller-defined clock units, and mechanics"]
#![doc = "ticking. It intentionally contains no game-specific nouns: callers decide"]
#![doc = "what an object, ability, effect, tag, clock, or payload means in their own"]
#![doc = "domain."]
#![doc = ""]
#![doc = "`ObjectId`, `AbilityId`, `AbilityActivationId`, and `ActiveEffectId` are"]
#![doc = "newtyped `u64` values allocated in deterministic creation order where"]
#![doc = "Flexweave owns allocation."]
#![doc = "Stores and queries preserve deterministic iteration where ordering is part"]
#![doc = "of the public contract."]
#![doc = "When an `ObjectStore` is available, prefer checked runtime paths such as"]
#![doc = "`AbilityStore::grant_checked`, `AbilityStore::begin_activation_for_owner_with_events`,"]
#![doc = "and `EffectPipeline::apply_checked_with_events`; the raw grant, activation,"]
#![doc = "and application methods are low-level paths that trust caller-managed"]
#![doc = "object-reference invariants."]
#![doc = ""]
#![doc = "State-changing ability and effect commands return explicit primitive"]
#![doc = "outcome enums so callers can distinguish command results without"]
#![doc = "inspecting emitted lifecycle facts."]
#![doc = ""]
#![doc = "The crate is fully safe Rust. Caller-owned hooks and closures carry domain"]
#![doc = "logic at the edges while Flexweave owns the reusable lifecycle shape."]
#![doc = ""]
#![doc = "## Lifecycle Facts, Channels, and Signals"]
#![doc = ""]
#![doc = "Lifecycle events are raw mechanics facts emitted by primitive operations:"]
#![doc = "attribute changes, derived attribute changes, ability activation, effect"]
#![doc = "application, effect execution, and mechanics ticking. They are not"]
#![doc = "application events, engine events, network messages, or UI commands until"]
#![doc = "caller code maps them into that runtime model."]
#![doc = ""]
#![doc = "`EventChannel` is a small caller-owned transport and retention primitive."]
#![doc = "It validates a published fact's `LifecycleEventKind`, optionally retains"]
#![doc = "published facts, and invokes listeners in registration order. Stores and"]
#![doc = "drivers do not discover channels or publish into them automatically; callers"]
#![doc = "wire publication explicitly from emitted facts."]
#![doc = ""]
#![doc = "`SignalProjection` derives `SignalFact` values from source lifecycle facts."]
#![doc = "The current projection surface is effect-lifecycle based, with reinvocation"]
#![doc = "support for active effect instances. Signals do not replace lifecycle facts:"]
#![doc = "they are derived/exportable facts for caller-defined reactions, adapters, or"]
#![doc = "author semantics. Projecting a signal does not publish it to a channel."]
#![doc = ""]
#![doc = "Channel keys on definitions, including effect routing keys and signal"]
#![doc = "channel keys, are metadata and validation hints. They have runtime behavior"]
#![doc = "only when caller code validates them against channel definitions and wires"]
#![doc = "publication to the selected `EventChannel` or external bus."]
#![doc = ""]
#![doc = "Raw lifecycle publication is explicit:"]
#![doc = ""]
#![doc = "```rust"]
#![doc = "use flexweave::{"]
#![doc = "    AttributeChange, EventChannel, EventChannelDefinition, EventRetention,"]
#![doc = "    LifecycleEventKind, ObjectId,"]
#![doc = "};"]
#![doc = ""]
#![doc = "let definition = EventChannelDefinition::new("]
#![doc = "    \"attributes/changes\","]
#![doc = "    [LifecycleEventKind::AttributeChanged],"]
#![doc = ")"]
#![doc = ".unwrap();"]
#![doc = "let mut channel = EventChannel::with_retention(definition, EventRetention::Retain);"]
#![doc = "let fact = AttributeChange {"]
#![doc = "    id: ObjectId::new(1),"]
#![doc = "    previous: Some(10.0),"]
#![doc = "    requested: 12.0,"]
#![doc = "    current: 12.0,"]
#![doc = "};"]
#![doc = ""]
#![doc = "assert!(channel.retained().is_empty());"]
#![doc = "channel.publish(fact).unwrap();"]
#![doc = ""]
#![doc = "let retained = channel.drain_retained();"]
#![doc = "assert_eq!(retained[0].current, 12.0);"]
#![doc = "```"]
#![doc = ""]
#![doc = "Signal projection and signal publication are also explicit steps:"]
#![doc = ""]
#![doc = "```rust"]
#![doc = "use flexweave::{"]
#![doc = "    EffectExecution, EffectLifecycleEvent, EventChannel, EventChannelDefinition,"]
#![doc = "    EventRetention, LifecycleEventKind, ObjectId, SignalDefinition, SignalDefinitions,"]
#![doc = "    SignalExportPolicy, SignalFact, SignalKind, SignalProjection, SignalRetentionPolicy,"]
#![doc = "    SignalTagMatch, Tag, TagSet,"]
#![doc = "};"]
#![doc = ""]
#![doc = "#[derive(Clone, Eq, PartialEq)]"]
#![doc = "enum Atom {"]
#![doc = "    Impact,"]
#![doc = "}"]
#![doc = ""]
#![doc = "let definitions = SignalDefinitions::new([SignalDefinition {"]
#![doc = "    key: \"impact\".to_owned(),"]
#![doc = "    signal_kind: SignalKind::Executed,"]
#![doc = "    lifecycle_event_kinds: vec![LifecycleEventKind::EffectExecuted],"]
#![doc = "    tag_match: SignalTagMatch::Any,"]
#![doc = "    payload_schema: \"impact.v1\".to_owned(),"]
#![doc = "    signal_payload: \"exportable impact\","]
#![doc = "    channel_key: \"signals/effects\".to_owned(),"]
#![doc = "    category: \"runtime\".to_owned(),"]
#![doc = "    retention: SignalRetentionPolicy::Retain,"]
#![doc = "    export: SignalExportPolicy::Export,"]
#![doc = "    debug_label: \"Impact\".to_owned(),"]
#![doc = "    description: \"An effect execution projected for adapters\".to_owned(),"]
#![doc = "}])"]
#![doc = ".unwrap();"]
#![doc = "definitions.validate_channels(&[\"signals/effects\"]).unwrap();"]
#![doc = "let projection = SignalProjection::new(definitions);"]
#![doc = ""]
#![doc = "let event = EffectLifecycleEvent::Executed(EffectExecution {"]
#![doc = "    active_effect_id: None,"]
#![doc = "    definition_key: Some(\"effects/impact\".to_owned()),"]
#![doc = "    source_id: Some(ObjectId::new(1)),"]
#![doc = "    target_id: ObjectId::new(2),"]
#![doc = "    tags: TagSet::new([Tag::new([Atom::Impact])]),"]
#![doc = "    payload: \"source payload\","]
#![doc = "    elapsed_units: None,"]
#![doc = "});"]
#![doc = "let facts = projection.project_effect_event(&event);"]
#![doc = ""]
#![doc = "let channel_definition = EventChannelDefinition::new("]
#![doc = "    \"signals/effects\","]
#![doc = "    [LifecycleEventKind::EffectExecuted],"]
#![doc = ")"]
#![doc = ".unwrap();"]
#![doc = "let mut channel: EventChannel<SignalFact<Atom, &str, &str>> ="]
#![doc = "    EventChannel::with_retention(channel_definition, EventRetention::Retain);"]
#![doc = ""]
#![doc = "assert!(channel.retained().is_empty());"]
#![doc = "for fact in facts {"]
#![doc = "    channel.publish(fact).unwrap();"]
#![doc = "}"]
#![doc = ""]
#![doc = "let retained = channel.drain_retained();"]
#![doc = "assert_eq!(retained[0].key, \"impact\");"]
#![doc = "assert_eq!("]
#![doc = "    retained[0].source_lifecycle_event_kind,"]
#![doc = "    LifecycleEventKind::EffectExecuted"]
#![doc = ");"]
#![doc = "```"]

pub mod ability;
pub mod attribute;
pub mod clock;
pub mod data_store;
pub mod derived_attribute;
pub mod effect;
pub mod errors;
pub mod identity;
pub mod lifecycle;
pub mod mechanics;
pub mod object_lifecycle;
pub(crate) mod object_map;
pub mod query;
pub mod registry;
pub mod signal;
pub mod tag;

pub use ability::{
    AbilityActivationAttempt, AbilityActivationAttemptView, AbilityActivationCommit,
    AbilityActivationCommitView, AbilityActivationDecision, AbilityActivationError,
    AbilityActivationId, AbilityActivationRejection, AbilityActivationRejectionReason,
    AbilityActivationRejectionView, AbilityCancelOutcome, AbilityCommitOutcome,
    AbilityCommitTiming, AbilityDefinition, AbilityDefinitionError, AbilityDefinitionRegistryError,
    AbilityDefinitions, AbilityEndOutcome, AbilityEndOutcomeResult, AbilityError,
    AbilityGrantError, AbilityHookPhase, AbilityHooks, AbilityId, AbilityLifecycleEvent,
    AbilityLifecycleEventView, AbilityStore, ActiveAbility, ActiveAbilityView, Grant,
    GrantedAbility, RegisteredAbilityActivationError, RevokedOwnerAbilities,
};
pub use attribute::{
    Attribute, AttributeChange, AttributeDefaultValue, AttributeDefinition,
    AttributeDefinitionError, AttributeDomain, AttributeMutation, AttributeMutationDecision,
    AttributeMutationHooks, AttributeMutationRejection, AttributeMutationRequest,
    AttributeMutationResult, AttributePolicyDefinition, AttributeValue,
};
pub use clock::{Clock, ClockUnits, FixedStepClock, RealtimeClock, RealtimeClockAccumulator};
pub use data_store::DataStore;
pub use derived_attribute::{DerivedAttribute, DerivedChange};
pub use effect::{
    ActiveEffectId, EffectAdvance, EffectAdvanceView, EffectApplication, EffectApplicationDecision,
    EffectApplicationDraft, EffectApplicationError, EffectApplicationInput,
    EffectApplicationRejection, EffectApplicationRejectionView, EffectApplicationView,
    EffectApplyOutcome, EffectClockPolicy, EffectDefinition, EffectDefinitionError,
    EffectDefinitionRegistryError, EffectDefinitions, EffectExecution, EffectExecutionView,
    EffectInitializationError, EffectInitializer, EffectInstance, EffectInstanceView, EffectKind,
    EffectLifecycleEvent, EffectLifecycleEventView, EffectObjectRemovalPolicy, EffectPipeline,
    EffectRouting, EffectSourcePolicy, NoopEffectInitializer,
};
pub use errors::CoreError;
pub use identity::{INVALID_OBJECT_ID, ObjectId, ObjectStore};
pub use lifecycle::{
    EventChannel, EventChannelDefinition, EventChannelDefinitionError, EventChannelDefinitions,
    EventChannelError, EventChannelRouteDefinition, EventConnectionHandle, EventRetention,
    LifecycleEvent, LifecycleEventKind, LocalLifecycleEvent, ScopedEventConnection,
};
pub use mechanics::{MechanicsDriver, MechanicsStore};
pub use object_lifecycle::{ObjectDestructionDriver, ObjectLifecycleStore};
pub use registry::{DefinitionRegistryEntry, Registry, RegistryEntry};
pub use signal::{
    SignalDefinition, SignalDefinitionError, SignalDefinitions, SignalExportPolicy, SignalFact,
    SignalKind, SignalProjection, SignalRemovalReason, SignalRetentionPolicy, SignalTagMatch,
};
pub use tag::{Tag, TagCollection, TagSet, TagSetQuery};
