//! Domain-neutral effect lifecycle primitive.

mod application;
mod definition;
mod events;
mod ids;
mod operation;
mod pipeline;

pub use application::{
    DiscardEffectLifecycleEvents, EffectActionExecutor, EffectApplication,
    EffectApplicationDecision, EffectApplicationDraft, EffectApplicationInput,
    EffectApplicationRejection, EffectApplicationRejectionView, EffectApplicationView,
    EffectExecutionAction, EffectExecutor, EffectInitializer, EffectLifecycleSink,
    EffectSourcePolicy, NoEffectExecutor, NoopEffectInitializer, OwnedEffectLifecycleEvents,
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
pub use operation::{
    EffectApply, EffectApplyError, EffectRemove, EffectRemoveForObject, EffectTick,
};
pub use pipeline::{
    EffectApplicationError, EffectApplyOutcome, EffectObjectRemovalPolicy, EffectPipeline,
};
