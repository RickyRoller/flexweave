use crate::common::TestAtom;
use flexweave::{
    ActiveEffectId, EffectApplicationDecision, EffectApplicationInput, EffectApplyOutcome,
    EffectClockPolicy, EffectDefinition, EffectKind, EffectRouting, ObjectId, Tag, TagSet,
};

pub(crate) fn effect_definition(
    key: &str,
    kind: EffectKind,
    duration: Option<EffectClockPolicy>,
    period: Option<EffectClockPolicy>,
) -> EffectDefinition {
    EffectDefinition {
        key: key.to_owned(),
        kind,
        duration,
        period,
        routing: EffectRouting::default(),
        payload_schema: (),
    }
}

pub(crate) fn application<Payload>(
    payload: Payload,
    decision: EffectApplicationDecision,
) -> EffectApplicationInput<TagSet<TestAtom>, Payload> {
    EffectApplicationInput {
        source_id: Some(ObjectId::new(10)),
        target_id: ObjectId::new(20),
        tags: TagSet::new([Tag::new([TestAtom::Category, TestAtom::Variant])]),
        payload,
        decision,
    }
}

pub(crate) trait EffectApplyOutcomeTestExt {
    fn active_effect_id(self) -> Option<ActiveEffectId>;
}

impl EffectApplyOutcomeTestExt for EffectApplyOutcome {
    fn active_effect_id(self) -> Option<ActiveEffectId> {
        match self {
            EffectApplyOutcome::ActiveCreated(id) => Some(id),
            EffectApplyOutcome::Rejected | EffectApplyOutcome::ExecutedInstant => None,
        }
    }
}
