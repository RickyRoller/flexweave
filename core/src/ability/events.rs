use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::definition::AbilityCommitTiming;
use super::ids::{AbilityActivationId, AbilityId};

/// Public lifecycle rejection reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityActivationRejectionReason {
    MissingAbility,
    InvalidOwner,
    OwnerMismatch,
    Blocked,
    Hook,
}

/// Activation attempt lifecycle fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivationAttempt<Tags, Payload>
where
    Tags: TagCollection,
{
    pub ability_id: AbilityId,
    pub definition_key: Option<String>,
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
}

/// Borrowed activation attempt lifecycle fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivationAttemptView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub ability_id: AbilityId,
    pub definition_key: Option<&'event str>,
    pub owner_id: ObjectId,
    pub tags: &'event Tags,
    pub payload: &'event Payload,
}

impl<'event, Tags, Payload> AbilityActivationAttemptView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_attempt(&self) -> AbilityActivationAttempt<Tags, Payload>
    where
        Payload: Clone,
    {
        AbilityActivationAttempt {
            ability_id: self.ability_id,
            definition_key: self.definition_key.map(str::to_owned),
            owner_id: self.owner_id,
            tags: self.tags.clone(),
            payload: self.payload.clone(),
        }
    }
}

/// Activation rejection lifecycle fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivationRejection<Tags, Payload>
where
    Tags: TagCollection,
{
    pub attempt: Option<AbilityActivationAttempt<Tags, Payload>>,
    pub reason: AbilityActivationRejectionReason,
}

/// Borrowed activation rejection lifecycle fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivationRejectionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub attempt: Option<AbilityActivationAttemptView<'event, Tags, Payload>>,
    pub reason: AbilityActivationRejectionReason,
}

impl<'event, Tags, Payload> AbilityActivationRejectionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_rejection(&self) -> AbilityActivationRejection<Tags, Payload>
    where
        Payload: Clone,
    {
        AbilityActivationRejection {
            attempt: self
                .attempt
                .as_ref()
                .map(AbilityActivationAttemptView::to_owned_attempt),
            reason: self.reason,
        }
    }
}

/// Ability commit lifecycle fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivationCommit<Tags, Payload>
where
    Tags: TagCollection,
{
    pub attempt: AbilityActivationAttempt<Tags, Payload>,
}

/// Borrowed ability commit lifecycle fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivationCommitView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub attempt: AbilityActivationAttemptView<'event, Tags, Payload>,
}

impl<'event, Tags, Payload> AbilityActivationCommitView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_commit(&self) -> AbilityActivationCommit<Tags, Payload>
    where
        Payload: Clone,
    {
        AbilityActivationCommit {
            attempt: self.attempt.to_owned_attempt(),
        }
    }
}

/// Active ability execution state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveAbility<Tags, Payload>
where
    Tags: TagCollection,
{
    pub activation_id: AbilityActivationId,
    pub ability_id: AbilityId,
    pub definition_key: Option<String>,
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
    pub commit_timing: AbilityCommitTiming,
    pub committed: bool,
}

impl<Tags, Payload> ActiveAbility<Tags, Payload>
where
    Tags: TagCollection,
{
    /// Returns the domain-neutral source object for effects derived from this activation.
    #[must_use]
    pub fn source_id(&self) -> ObjectId {
        self.owner_id
    }
}

/// Borrowed active ability execution state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveAbilityView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub activation_id: AbilityActivationId,
    pub ability_id: AbilityId,
    pub definition_key: Option<&'event str>,
    pub owner_id: ObjectId,
    pub tags: &'event Tags,
    pub payload: &'event Payload,
    pub commit_timing: AbilityCommitTiming,
    pub committed: bool,
}

impl<'event, Tags, Payload> ActiveAbilityView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_active(&self) -> ActiveAbility<Tags, Payload>
    where
        Payload: Clone,
    {
        ActiveAbility {
            activation_id: self.activation_id,
            ability_id: self.ability_id,
            definition_key: self.definition_key.map(str::to_owned),
            owner_id: self.owner_id,
            tags: self.tags.clone(),
            payload: self.payload.clone(),
            commit_timing: self.commit_timing,
            committed: self.committed,
        }
    }

    #[must_use]
    pub fn attempt_view(&self) -> AbilityActivationAttemptView<'event, Tags, Payload> {
        AbilityActivationAttemptView {
            ability_id: self.ability_id,
            definition_key: self.definition_key,
            owner_id: self.owner_id,
            tags: self.tags,
            payload: self.payload,
        }
    }

    /// Returns the domain-neutral source object for effects derived from this activation.
    #[must_use]
    pub fn source_id(&self) -> ObjectId {
        self.owner_id
    }
}

impl<'event, Tags, Payload> From<&'event ActiveAbility<Tags, Payload>>
    for ActiveAbilityView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    fn from(value: &'event ActiveAbility<Tags, Payload>) -> Self {
        Self {
            activation_id: value.activation_id,
            ability_id: value.ability_id,
            definition_key: value.definition_key.as_deref(),
            owner_id: value.owner_id,
            tags: &value.tags,
            payload: &value.payload,
            commit_timing: value.commit_timing,
            committed: value.committed,
        }
    }
}

/// Ability lifecycle events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    Attempted(AbilityActivationAttempt<Tags, Payload>),
    Rejected(AbilityActivationRejection<Tags, Payload>),
    Committed(AbilityActivationCommit<Tags, Payload>),
    Started(ActiveAbility<Tags, Payload>),
    Canceled(ActiveAbility<Tags, Payload>),
    Ended(ActiveAbility<Tags, Payload>),
}

/// Borrowed ability lifecycle event for hot streaming paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityLifecycleEventView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    Attempted(AbilityActivationAttemptView<'event, Tags, Payload>),
    Rejected(AbilityActivationRejectionView<'event, Tags, Payload>),
    Committed(AbilityActivationCommitView<'event, Tags, Payload>),
    Started(ActiveAbilityView<'event, Tags, Payload>),
    Canceled(ActiveAbilityView<'event, Tags, Payload>),
    Ended(ActiveAbilityView<'event, Tags, Payload>),
}

impl<'event, Tags, Payload> AbilityLifecycleEventView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_event(&self) -> AbilityLifecycleEvent<Tags, Payload>
    where
        Payload: Clone,
    {
        match self {
            Self::Attempted(attempt) => {
                AbilityLifecycleEvent::Attempted(attempt.to_owned_attempt())
            }
            Self::Rejected(rejection) => {
                AbilityLifecycleEvent::Rejected(rejection.to_owned_rejection())
            }
            Self::Committed(commit) => AbilityLifecycleEvent::Committed(commit.to_owned_commit()),
            Self::Started(active) => AbilityLifecycleEvent::Started(active.to_owned_active()),
            Self::Canceled(active) => AbilityLifecycleEvent::Canceled(active.to_owned_active()),
            Self::Ended(active) => AbilityLifecycleEvent::Ended(active.to_owned_active()),
        }
    }
}
