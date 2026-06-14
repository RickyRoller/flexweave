use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::definition::AbilityCommitTiming;
use super::ids::{AbilityActivationId, AbilityId, CooldownUnits};

/// Public lifecycle rejection reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityActivationRejectionReason {
    MissingAbility,
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

/// Activation rejection lifecycle fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivationRejection<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub attempt: Option<AbilityActivationAttempt<Tags, Cost, Payload>>,
    pub reason: AbilityActivationRejectionReason,
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
