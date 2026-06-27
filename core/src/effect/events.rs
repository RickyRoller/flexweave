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

/// Borrowed view of a runtime effect instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectInstanceView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub id: ActiveEffectId,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub remaining_units: Option<ClockUnits>,
    pub period: Option<EffectClockPolicy>,
    pub period_elapsed_units: ClockUnits,
    pub tags: &'event Tags,
    pub payload: &'event Payload,
}

impl<'event, Tags, Payload> EffectInstanceView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn has_tag(&self, tag: &Tags::Tag) -> bool {
        self.tags.has_tag(tag)
    }

    #[must_use]
    pub fn to_owned_instance(&self) -> EffectInstance<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectInstance {
            id: self.id,
            source_id: self.source_id,
            target_id: self.target_id,
            remaining_units: self.remaining_units,
            period: self.period,
            period_elapsed_units: self.period_elapsed_units,
            tags: self.tags.clone(),
            payload: self.payload.clone(),
        }
    }
}

impl<'event, Tags, Payload> From<&'event EffectInstance<Tags, Payload>>
    for EffectInstanceView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    fn from(value: &'event EffectInstance<Tags, Payload>) -> Self {
        Self {
            id: value.id,
            source_id: value.source_id,
            target_id: value.target_id,
            remaining_units: value.remaining_units,
            period: value.period,
            period_elapsed_units: value.period_elapsed_units,
            tags: &value.tags,
            payload: &value.payload,
        }
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

/// Borrowed active effect advancement fact for streaming emission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectAdvanceView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub effect: EffectInstanceView<'event, Tags, Payload>,
    pub elapsed_units: ClockUnits,
    pub previous_remaining_units: Option<ClockUnits>,
}

impl<'event, Tags, Payload> EffectAdvanceView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_advance(&self) -> EffectAdvance<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectAdvance {
            effect: self.effect.to_owned_instance(),
            elapsed_units: self.elapsed_units,
            previous_remaining_units: self.previous_remaining_units,
        }
    }
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

/// Borrowed effect execution fact for streaming emission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectExecutionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub active_effect_id: Option<ActiveEffectId>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: &'event Tags,
    pub payload: &'event Payload,
    pub elapsed_units: Option<ClockUnits>,
}

impl<'event, Tags, Payload> EffectExecutionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_execution(&self) -> EffectExecution<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectExecution {
            active_effect_id: self.active_effect_id,
            source_id: self.source_id,
            target_id: self.target_id,
            tags: self.tags.clone(),
            payload: self.payload.clone(),
            elapsed_units: self.elapsed_units,
        }
    }
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

/// Borrowed effect pipeline lifecycle event for hot streaming paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectLifecycleEventView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    ApplicationAccepted(super::application::EffectApplicationView<'event, Tags, Payload>),
    ApplicationRejected(super::application::EffectApplicationRejectionView<'event, Tags, Payload>),
    ActiveCreated(EffectInstanceView<'event, Tags, Payload>),
    Executed(EffectExecutionView<'event, Tags, Payload>),
    PeriodicExecuted(EffectExecutionView<'event, Tags, Payload>),
    Advanced(EffectAdvanceView<'event, Tags, Payload>),
    Removed(EffectInstanceView<'event, Tags, Payload>),
    Expired(EffectInstanceView<'event, Tags, Payload>),
}

impl<'event, Tags, Payload> EffectLifecycleEventView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_event(&self) -> EffectLifecycleEvent<Tags, Payload>
    where
        Payload: Clone,
    {
        match self {
            Self::ApplicationAccepted(application) => {
                EffectLifecycleEvent::ApplicationAccepted(application.to_owned_application())
            }
            Self::ApplicationRejected(rejection) => {
                EffectLifecycleEvent::ApplicationRejected(rejection.to_owned_rejection())
            }
            Self::ActiveCreated(effect) => {
                EffectLifecycleEvent::ActiveCreated(effect.to_owned_instance())
            }
            Self::Executed(execution) => {
                EffectLifecycleEvent::Executed(execution.to_owned_execution())
            }
            Self::PeriodicExecuted(execution) => {
                EffectLifecycleEvent::PeriodicExecuted(execution.to_owned_execution())
            }
            Self::Advanced(advance) => EffectLifecycleEvent::Advanced(advance.to_owned_advance()),
            Self::Removed(effect) => EffectLifecycleEvent::Removed(effect.to_owned_instance()),
            Self::Expired(effect) => EffectLifecycleEvent::Expired(effect.to_owned_instance()),
        }
    }
}
