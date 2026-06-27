use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::definition::AbilityCommitTiming;
use super::ids::{AbilityActivationId, AbilityId, CooldownUnits};

/// Public lifecycle rejection reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityActivationRejectionReason {
    MissingAbility,
    InvalidOwner,
    OwnerMismatch,
    OnCooldown,
    Hook,
}

/// Activation attempt lifecycle fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivationAttempt<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub ability_id: AbilityId,
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub cost: Option<Cost>,
    pub payload: Payload,
}

/// Borrowed activation attempt lifecycle fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivationAttemptView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub ability_id: AbilityId,
    pub owner_id: ObjectId,
    pub tags: &'event Tags,
    pub cost: Option<&'event Cost>,
    pub payload: &'event Payload,
}

impl<'event, Tags, Cost, Payload> AbilityActivationAttemptView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_attempt(&self) -> AbilityActivationAttempt<Tags, Cost, Payload>
    where
        Cost: Clone,
        Payload: Clone,
    {
        AbilityActivationAttempt {
            ability_id: self.ability_id,
            owner_id: self.owner_id,
            tags: self.tags.clone(),
            cost: self.cost.cloned(),
            payload: self.payload.clone(),
        }
    }
}

/// Activation rejection lifecycle fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivationRejection<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub attempt: Option<AbilityActivationAttempt<Tags, Cost, Payload>>,
    pub reason: AbilityActivationRejectionReason,
}

/// Borrowed activation rejection lifecycle fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivationRejectionView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub attempt: Option<AbilityActivationAttemptView<'event, Tags, Cost, Payload>>,
    pub reason: AbilityActivationRejectionReason,
}

impl<'event, Tags, Cost, Payload> AbilityActivationRejectionView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_rejection(&self) -> AbilityActivationRejection<Tags, Cost, Payload>
    where
        Cost: Clone,
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
pub struct AbilityActivationCommit<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub attempt: AbilityActivationAttempt<Tags, Cost, Payload>,
    pub cooldown_units: Option<CooldownUnits>,
}

/// Borrowed ability commit lifecycle fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivationCommitView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub attempt: AbilityActivationAttemptView<'event, Tags, Cost, Payload>,
    pub cooldown_units: Option<CooldownUnits>,
}

impl<'event, Tags, Cost, Payload> AbilityActivationCommitView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_commit(&self) -> AbilityActivationCommit<Tags, Cost, Payload>
    where
        Cost: Clone,
        Payload: Clone,
    {
        AbilityActivationCommit {
            attempt: self.attempt.to_owned_attempt(),
            cooldown_units: self.cooldown_units,
        }
    }
}

/// Active ability execution state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveAbility<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub activation_id: AbilityActivationId,
    pub ability_id: AbilityId,
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub cost: Option<Cost>,
    pub payload: Payload,
    pub commit_timing: AbilityCommitTiming,
    pub committed: bool,
}

impl<Tags, Cost, Payload> ActiveAbility<Tags, Cost, Payload>
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
pub struct ActiveAbilityView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub activation_id: AbilityActivationId,
    pub ability_id: AbilityId,
    pub owner_id: ObjectId,
    pub tags: &'event Tags,
    pub cost: Option<&'event Cost>,
    pub payload: &'event Payload,
    pub commit_timing: AbilityCommitTiming,
    pub committed: bool,
}

impl<'event, Tags, Cost, Payload> ActiveAbilityView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_active(&self) -> ActiveAbility<Tags, Cost, Payload>
    where
        Cost: Clone,
        Payload: Clone,
    {
        ActiveAbility {
            activation_id: self.activation_id,
            ability_id: self.ability_id,
            owner_id: self.owner_id,
            tags: self.tags.clone(),
            cost: self.cost.cloned(),
            payload: self.payload.clone(),
            commit_timing: self.commit_timing,
            committed: self.committed,
        }
    }

    #[must_use]
    pub fn attempt_view(&self) -> AbilityActivationAttemptView<'event, Tags, Cost, Payload> {
        AbilityActivationAttemptView {
            ability_id: self.ability_id,
            owner_id: self.owner_id,
            tags: self.tags,
            cost: self.cost,
            payload: self.payload,
        }
    }

    /// Returns the domain-neutral source object for effects derived from this activation.
    #[must_use]
    pub fn source_id(&self) -> ObjectId {
        self.owner_id
    }
}

impl<'event, Tags, Cost, Payload> From<&'event ActiveAbility<Tags, Cost, Payload>>
    for ActiveAbilityView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    fn from(value: &'event ActiveAbility<Tags, Cost, Payload>) -> Self {
        Self {
            activation_id: value.activation_id,
            ability_id: value.ability_id,
            owner_id: value.owner_id,
            tags: &value.tags,
            cost: value.cost.as_ref(),
            payload: &value.payload,
            commit_timing: value.commit_timing,
            committed: value.committed,
        }
    }
}

/// Ability lifecycle events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityLifecycleEvent<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    Attempted(AbilityActivationAttempt<Tags, Cost, Payload>),
    Rejected(AbilityActivationRejection<Tags, Cost, Payload>),
    Committed(AbilityActivationCommit<Tags, Cost, Payload>),
    Started(ActiveAbility<Tags, Cost, Payload>),
    Canceled(ActiveAbility<Tags, Cost, Payload>),
    Ended(ActiveAbility<Tags, Cost, Payload>),
}

/// Borrowed ability lifecycle event for hot streaming paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityLifecycleEventView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    Attempted(AbilityActivationAttemptView<'event, Tags, Cost, Payload>),
    Rejected(AbilityActivationRejectionView<'event, Tags, Cost, Payload>),
    Committed(AbilityActivationCommitView<'event, Tags, Cost, Payload>),
    Started(ActiveAbilityView<'event, Tags, Cost, Payload>),
    Canceled(ActiveAbilityView<'event, Tags, Cost, Payload>),
    Ended(ActiveAbilityView<'event, Tags, Cost, Payload>),
}

impl<'event, Tags, Cost, Payload> AbilityLifecycleEventView<'event, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_event(&self) -> AbilityLifecycleEvent<Tags, Cost, Payload>
    where
        Cost: Clone,
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
