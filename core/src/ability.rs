//! Domain-neutral ability primitive.

mod activation_request;
mod definition;
mod event_sink;
mod events;
mod hooks;
mod ids;
mod indexed_store;
mod lifecycle_transaction;
mod results;
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
pub use results::{
    AbilityBeginError, AbilityCancelOutcome, AbilityCommitError, AbilityCommitOutcome,
    AbilityEndError, AbilityEndOutcome, AbilityError, AbilityGrantError, AbilityRollbackError,
    AbilityRollbackOutcome, RegisteredAbilityActivationError,
};
pub use store::{AbilityStore, Grant, GrantedAbility, RevokedOwnerAbilities};
