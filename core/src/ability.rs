//! Domain-neutral ability primitive.

mod definition;
mod events;
mod hooks;
mod ids;
mod store;

pub use definition::{
    AbilityActivationMode, AbilityCancelPolicy, AbilityCommitTiming, AbilityDefinition,
    AbilityDefinitionError, AbilityDefinitionRegistryError, AbilityDefinitions,
};
pub use events::{
    AbilityActivationAttempt, AbilityActivationAttemptView, AbilityActivationCommit,
    AbilityActivationCommitView, AbilityActivationRejection, AbilityActivationRejectionReason,
    AbilityActivationRejectionView, AbilityLifecycleEvent, AbilityLifecycleEventView,
    ActiveAbility, ActiveAbilityView,
};
pub use hooks::AbilityHooks;
pub use ids::{AbilityActivationId, AbilityId, CooldownUnits};
pub use store::{
    AbilityActivationError, AbilityCancelOutcome, AbilityCommitOutcome, AbilityEndOutcome,
    AbilityEndOutcomeResult, AbilityError, AbilityGrantError, AbilityHookPhase, AbilityStore,
    Grant, GrantedAbility, RegisteredAbilityActivationError, RevokedOwnerAbilities,
};
