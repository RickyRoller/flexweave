//! Domain-neutral ability primitive.

mod activation_request;
mod definition;
mod events;
mod hooks;
mod ids;
mod store;

pub use definition::{
    AbilityDefinition, AbilityDefinitionError, AbilityDefinitionRegistryError, AbilityDefinitions,
};
pub use events::{
    AbilityActivationAttempt, AbilityActivationAttemptView, AbilityActivationRejection,
    AbilityActivationRejectionReason, AbilityActivationRejectionView, AbilityLifecycleEvent,
    AbilityLifecycleEventView, ActiveAbility, ActiveAbilityView,
};
pub use hooks::{
    AbilityActivationDecision, AbilityActivationGate, AbilityCommitAction, AllowActivation,
    NoCommitAction,
};
pub use ids::{AbilityActivationId, AbilityId};
pub use store::{
    AbilityBeginError, AbilityCancelOutcome, AbilityCommitError, AbilityCommitOutcome,
    AbilityEndError, AbilityEndOutcome, AbilityError, AbilityGrantError, AbilityRollbackError,
    AbilityRollbackOutcome, AbilityStore, Grant, GrantedAbility, RegisteredAbilityActivationError,
    RevokedOwnerAbilities,
};
