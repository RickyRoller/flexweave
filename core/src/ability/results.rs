use std::fmt;

use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::definition::AbilityDefinitionRegistryError;
use super::events::ActiveAbility;
use super::ids::AbilityId;

/// Ability begin errors, including caller-owned blocking and gate failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityBeginError<GateError, BlockReason = ()> {
    Ability(AbilityError),
    Blocked(BlockReason),
    Gate(GateError),
}

impl<GateError, BlockReason> fmt::Display for AbilityBeginError<GateError, BlockReason>
where
    GateError: fmt::Display,
    BlockReason: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ability(error) => write!(formatter, "ability activation failed: {error}"),
            Self::Blocked(reason) => write!(formatter, "ability activation blocked: {reason:?}"),
            Self::Gate(error) => write!(formatter, "ability activation gate failed: {error}"),
        }
    }
}

impl<GateError, BlockReason> std::error::Error for AbilityBeginError<GateError, BlockReason>
where
    GateError: std::error::Error + 'static,
    BlockReason: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ability(error) => Some(error),
            Self::Blocked(_) => None,
            Self::Gate(error) => Some(error),
        }
    }
}

/// Ability commit errors, including caller-owned action failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityCommitError<CommitError> {
    Ability(AbilityError),
    Action(CommitError),
}

impl<CommitError> fmt::Display for AbilityCommitError<CommitError>
where
    CommitError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ability(error) => write!(formatter, "ability commit failed: {error}"),
            Self::Action(error) => write!(formatter, "ability commit action failed: {error}"),
        }
    }
}

impl<CommitError> std::error::Error for AbilityCommitError<CommitError>
where
    CommitError: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ability(error) => Some(error),
            Self::Action(error) => Some(error),
        }
    }
}

/// Ability end command errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityEndError {
    MissingActivation,
    UncommittedActivation,
}

impl fmt::Display for AbilityEndError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingActivation => "missing ability activation",
            Self::UncommittedActivation => "uncommitted ability activation",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityEndError {}

/// Ability rollback command errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityRollbackError {
    MissingActivation,
    AlreadyCommitted,
}

impl fmt::Display for AbilityRollbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingActivation => "missing ability activation",
            Self::AlreadyCommitted => "ability activation is already committed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityRollbackError {}

/// Outcome of a commit command for an active ability activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityCommitOutcome {
    Committed,
    AlreadyCommitted,
}

/// Outcome of an end command for an active ability activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityEndOutcome<Tags, Payload>
where
    Tags: TagCollection,
{
    Ended(ActiveAbility<Tags, Payload>),
}

/// Outcome of a cancel command for an active ability activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityCancelOutcome<Tags, Payload>
where
    Tags: TagCollection,
{
    Canceled(ActiveAbility<Tags, Payload>),
    MissingActivation,
}

/// Outcome of a rollback command for an active ability activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityRollbackOutcome<Tags, Payload>
where
    Tags: TagCollection,
{
    RolledBack(ActiveAbility<Tags, Payload>),
}

/// Store-level ability errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityError {
    MissingAbility,
    MissingActivation,
    InvalidOwner {
        owner_id: ObjectId,
    },
    OwnerMismatch {
        expected_owner_id: ObjectId,
        actual_owner_id: ObjectId,
    },
}

impl fmt::Display for AbilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingAbility => "missing ability",
            Self::MissingActivation => "missing ability activation",
            Self::InvalidOwner { .. } => "invalid ability owner",
            Self::OwnerMismatch { .. } => "ability owner mismatch",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityError {}

/// Registered activation errors for key-aware ability workflows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisteredAbilityActivationError<GateError, BlockReason = ()> {
    MissingGrantedDefinitionKey { ability_id: AbilityId },
    Definition(AbilityDefinitionRegistryError),
    Activation(AbilityBeginError<GateError, BlockReason>),
}

impl<GateError, BlockReason> fmt::Display
    for RegisteredAbilityActivationError<GateError, BlockReason>
where
    GateError: fmt::Display,
    BlockReason: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGrantedDefinitionKey { ability_id } => write!(
                formatter,
                "ability `{ability_id}` was not granted from a registered definition"
            ),
            Self::Definition(error) => {
                write!(formatter, "registered ability activation failed: {error}")
            }
            Self::Activation(error) => {
                write!(formatter, "registered ability activation failed: {error}")
            }
        }
    }
}

impl<GateError, BlockReason> std::error::Error
    for RegisteredAbilityActivationError<GateError, BlockReason>
where
    GateError: std::error::Error + 'static,
    BlockReason: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingGrantedDefinitionKey { .. } => None,
            Self::Definition(error) => Some(error),
            Self::Activation(error) => Some(error),
        }
    }
}

/// Ability grant validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityGrantError {
    InvalidOwner { owner_id: ObjectId },
}

impl fmt::Display for AbilityGrantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOwner { .. } => formatter.write_str("invalid ability grant owner"),
        }
    }
}

impl std::error::Error for AbilityGrantError {}
