//! Domain-neutral effect lifecycle primitive.

mod application;
mod definition;
mod events;
mod ids;
mod pipeline;

pub use application::{
    EffectApplication, EffectApplicationDecision, EffectApplicationDraft, EffectApplicationInput,
    EffectApplicationRejection, EffectApplicationRejectionView, EffectApplicationView,
    EffectInitializer, EffectSourcePolicy, NoopEffectInitializer,
};
pub use definition::{
    EffectClockPolicy, EffectDefinition, EffectDefinitionError, EffectDefinitionRegistryError,
    EffectDefinitions, EffectKind, EffectRouting,
};
pub use events::{
    EffectAdvance, EffectAdvanceView, EffectExecution, EffectExecutionView, EffectInstance,
    EffectInstanceView, EffectLifecycleEvent, EffectLifecycleEventView,
};
pub use ids::ActiveEffectId;
pub use pipeline::{
    EffectApplicationError, EffectApplyOutcome, EffectInitializationError,
    EffectObjectRemovalPolicy, EffectPipeline,
};
