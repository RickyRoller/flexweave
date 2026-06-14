use crate::ability::AbilityLifecycleEvent;
use crate::attribute::AttributeChange;
use crate::derived_attribute::DerivedChange;
use crate::effect::EffectLifecycleEvent;
use crate::tag::TagCollection;

/// Stable lifecycle fact kinds used by event channel payload contracts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LifecycleEventKind {
    AttributeChanged,
    DerivedAttributeChanged,
    EffectApplicationAccepted,
    EffectApplicationRejected,
    EffectActiveCreated,
    EffectExecuted,
    EffectPeriodicExecuted,
    EffectAdvanced,
    EffectRemoved,
    EffectExpired,
    SignalReinvoked,
    AbilityActivationAttempted,
    AbilityActivationRejected,
    AbilityActivationCommitted,
    AbilityActivationStarted,
    AbilityActivationCanceled,
    AbilityActivationEnded,
}

/// A lifecycle fact that can be routed through a named event channel.
pub trait LifecycleEvent {
    fn lifecycle_event_kind(&self) -> LifecycleEventKind;
}

/// Domain-neutral local lifecycle event wrapper.
#[derive(Clone, Debug, PartialEq)]
pub enum LocalLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    Effect(EffectLifecycleEvent<Tags, Payload>),
    AttributeChanged(AttributeChange),
    DerivedAttributeChanged(DerivedChange),
}

impl<Tags, Payload> LifecycleEvent for LocalLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        match self {
            Self::Effect(event) => event.lifecycle_event_kind(),
            Self::AttributeChanged(_) => LifecycleEventKind::AttributeChanged,
            Self::DerivedAttributeChanged(_) => LifecycleEventKind::DerivedAttributeChanged,
        }
    }
}

impl<Tags, Payload> From<EffectLifecycleEvent<Tags, Payload>> for LocalLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    fn from(value: EffectLifecycleEvent<Tags, Payload>) -> Self {
        Self::Effect(value)
    }
}

impl<Tags, Payload> From<AttributeChange> for LocalLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    fn from(value: AttributeChange) -> Self {
        Self::AttributeChanged(value)
    }
}

impl<Tags, Payload> From<DerivedChange> for LocalLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    fn from(value: DerivedChange) -> Self {
        Self::DerivedAttributeChanged(value)
    }
}

impl<Tags, Payload> LifecycleEvent for EffectLifecycleEvent<Tags, Payload>
where
    Tags: TagCollection,
{
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        match self {
            Self::ApplicationAccepted(_) => LifecycleEventKind::EffectApplicationAccepted,
            Self::ApplicationRejected(_) => LifecycleEventKind::EffectApplicationRejected,
            Self::ActiveCreated(_) => LifecycleEventKind::EffectActiveCreated,
            Self::Executed(_) => LifecycleEventKind::EffectExecuted,
            Self::PeriodicExecuted(_) => LifecycleEventKind::EffectPeriodicExecuted,
            Self::Advanced(_) => LifecycleEventKind::EffectAdvanced,
            Self::Removed(_) => LifecycleEventKind::EffectRemoved,
            Self::Expired(_) => LifecycleEventKind::EffectExpired,
        }
    }
}

impl<Tags, Cost, Payload> LifecycleEvent for AbilityLifecycleEvent<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        match self {
            Self::Attempted(_) => LifecycleEventKind::AbilityActivationAttempted,
            Self::Rejected(_) => LifecycleEventKind::AbilityActivationRejected,
            Self::Committed(_) => LifecycleEventKind::AbilityActivationCommitted,
            Self::Started(_) => LifecycleEventKind::AbilityActivationStarted,
            Self::Canceled(_) => LifecycleEventKind::AbilityActivationCanceled,
            Self::Ended(_) => LifecycleEventKind::AbilityActivationEnded,
        }
    }
}

impl LifecycleEvent for AttributeChange {
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        LifecycleEventKind::AttributeChanged
    }
}

impl LifecycleEvent for DerivedChange {
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        LifecycleEventKind::DerivedAttributeChanged
    }
}
