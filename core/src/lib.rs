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
#![doc = ""]
#![doc = "The crate is fully safe Rust. Caller-owned hooks and closures carry domain"]
#![doc = "logic at the edges while Flexweave owns the reusable lifecycle shape."]

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
    AbilityActivationAttempt, AbilityActivationCommit, AbilityActivationError, AbilityActivationId,
    AbilityActivationMode, AbilityActivationRejection, AbilityActivationRejectionReason,
    AbilityCancelPolicy, AbilityCommitTiming, AbilityDefinition, AbilityDefinitionError,
    AbilityEndResult, AbilityError, AbilityHooks, AbilityId, AbilityLifecycleEvent, AbilityStore,
    ActiveAbility, CooldownUnits, Grant, GrantedAbility,
};
pub use attribute::{
    Attribute, AttributeChange, AttributeDefaultValue, AttributeDefinition,
    AttributeDefinitionError, AttributeDomain, AttributeMutation, AttributeMutationDecision,
    AttributeMutationHooks, AttributeMutationRejection, AttributeMutationRequest,
    AttributeMutationResult, AttributePolicyDefinition, AttributeValue,
};
pub use clock::{Clock, ClockUnits, FixedStepClock, RealtimeClock};
pub use data_store::DataStore;
pub use derived_attribute::{DerivedAttribute, DerivedChange};
pub use effect::{
    ActiveEffectId, EffectAdvance, EffectApplication, EffectApplicationDecision,
    EffectApplicationInput, EffectApplicationRejection, EffectClockPolicy, EffectDefinition,
    EffectDefinitionError, EffectExecution, EffectInstance, EffectKind, EffectLifecycleEvent,
    EffectObjectRemovalPolicy, EffectPipeline, EffectRouting,
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
