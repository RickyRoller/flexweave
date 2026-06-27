use crate::effect::ActiveEffectId;
use crate::identity::ObjectId;
use crate::lifecycle::{LifecycleEvent, LifecycleEventKind};
use crate::tag::TagSet;

use super::definition::{SignalExportPolicy, SignalKind, SignalRetentionPolicy};

/// Why a removed Signal was emitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalRemovalReason {
    Removed,
    Expired,
}

/// Derived Signal fact emitted by `SignalProjection`.
///
/// `SignalFact` implements [`LifecycleEvent`] with its source lifecycle kind so
/// caller-owned channels can validate the same source fact kind after
/// projection. The `channel_key` is metadata copied from the matched definition;
/// publishing remains caller-owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalFact<Atom, SignalPayload, SourcePayload> {
    pub key: String,
    pub signal_kind: SignalKind,
    pub channel_key: String,
    pub category: String,
    pub retention: SignalRetentionPolicy,
    pub export: SignalExportPolicy,
    pub source_lifecycle_event_kind: LifecycleEventKind,
    pub source_definition_key: Option<String>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub owner_id: Option<ObjectId>,
    pub active_effect_id: Option<ActiveEffectId>,
    pub clock_units: Option<u64>,
    pub removal_reason: Option<SignalRemovalReason>,
    pub tags: TagSet<Atom>,
    pub signal_payload: SignalPayload,
    pub source_payload: Option<SourcePayload>,
}

impl<Atom, SignalPayload, SourcePayload> LifecycleEvent
    for SignalFact<Atom, SignalPayload, SourcePayload>
{
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        self.source_lifecycle_event_kind
    }
}
