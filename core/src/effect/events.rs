use crate::clock::ClockUnits;
use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::application::{EffectApplication, EffectApplicationRejection};
use super::definition::EffectClockPolicy;
use super::ids::ActiveEffectId;

/// Runtime effect instance owned by `EffectPipeline`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectInstance<Tags, Payload>
where
    Tags: TagCollection,
{
    pub id: ActiveEffectId,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub remaining_units: Option<ClockUnits>,
    pub period: Option<EffectClockPolicy>,
    pub period_elapsed_units: ClockUnits,
    pub tags: Tags,
    pub payload: Payload,
}

impl<Tags, Payload> EffectInstance<Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn has_tag(&self, tag: &Tags::Tag) -> bool {
        self.tags.has_tag(tag)
    }
}

/// Active effect advancement fact for the pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectAdvance<Tags, Payload>
where
    Tags: TagCollection,
{
    pub effect: EffectInstance<Tags, Payload>,
    pub elapsed_units: ClockUnits,
    pub previous_remaining_units: Option<ClockUnits>,
}

/// Effect execution fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecution<Tags, Payload>
where
    Tags: TagCollection,
{
    pub active_effect_id: Option<ActiveEffectId>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
    pub elapsed_units: Option<ClockUnits>,
}

/// Effect pipeline lifecycle events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    ApplicationAccepted(EffectApplication<Tags, Payload>),
    ApplicationRejected(EffectApplicationRejection<Tags, Payload>),
    ActiveCreated(EffectInstance<Tags, Payload>),
    Executed(EffectExecution<Tags, Payload>),
    PeriodicExecuted(EffectExecution<Tags, Payload>),
    Advanced(EffectAdvance<Tags, Payload>),
    Removed(EffectInstance<Tags, Payload>),
    Expired(EffectInstance<Tags, Payload>),
}
