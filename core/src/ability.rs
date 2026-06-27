//! Domain-neutral ability primitive.

mod definition;
mod events;
mod hooks;
mod ids;
mod store;

pub use definition::{
    AbilityActivationMode, AbilityCancelPolicy, AbilityCommitTiming, AbilityDefinition,
    AbilityDefinitionError,
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
    AbilityActivationError, AbilityEndResult, AbilityError, AbilityGrantError, AbilityStore, Grant,
    GrantedAbility, RevokedOwnerAbilities,
};
