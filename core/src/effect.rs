//! Domain-neutral effect lifecycle primitive.

mod application;
mod definition;
mod events;
mod ids;
mod pipeline;

pub use application::{
    EffectApplication, EffectApplicationDecision, EffectApplicationInput,
    EffectApplicationRejection, EffectApplicationRejectionView, EffectApplicationView,
    EffectSourcePolicy,
};
pub use definition::{
    EffectClockPolicy, EffectDefinition, EffectDefinitionError, EffectKind, EffectRouting,
};
pub use events::{
    EffectAdvance, EffectAdvanceView, EffectExecution, EffectExecutionView, EffectInstance,
    EffectInstanceView, EffectLifecycleEvent, EffectLifecycleEventView,
};
pub use ids::ActiveEffectId;
pub use pipeline::{EffectApplicationError, EffectObjectRemovalPolicy, EffectPipeline};
