//! Domain-neutral effect lifecycle primitive.

mod application;
mod definition;
mod events;
mod ids;
mod pipeline;

pub use application::{
    EffectApplication, EffectApplicationDecision, EffectApplicationInput,
    EffectApplicationRejection,
};
pub use definition::{
    EffectClockPolicy, EffectDefinition, EffectDefinitionError, EffectKind, EffectRouting,
};
pub use events::{EffectAdvance, EffectExecution, EffectInstance, EffectLifecycleEvent};
pub use ids::ActiveEffectId;
pub use pipeline::EffectPipeline;
