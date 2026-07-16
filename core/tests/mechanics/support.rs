pub(crate) use crate::common::TestAtom;
pub(crate) use flexweave::{
    AbilityActivation, AbilityCommit, AbilityCommitAction, AbilityCommitActionExecutor, AbilityEnd,
    AbilityGrant, AbilityStore, ActiveAbilityView, ActiveEffectId, Clock, ClockUnits,
    DefinitionRegistryEntry, EffectApplicationDecision, EffectApplicationInput, EffectApply,
    EffectApplyError, EffectApplyOutcome, EffectClockPolicy,
    EffectDefinition as FlexEffectDefinition, EffectKind, EffectLifecycleEvent, EffectPipeline,
    EffectRouting, EventChannel, EventChannelDefinition, EventChannelDefinitionError,
    EventChannelDefinitions, EventChannelError, EventChannelRouteDefinition, EventConnectionHandle,
    EventRetention, FixedStepClock, Grant, LifecycleEvent, LifecycleEventKind, LocalLifecycleEvent,
    MechanicsDriver, MechanicsTick, NoEffectExecutor, ObjectId, ObjectStore, RealtimeClock,
    RealtimeClockAccumulator, Registry, RegistryEntry, Tag, TagSet,
};
pub(crate) use std::sync::{Arc, Mutex};
pub(crate) use std::time::Duration;

pub(crate) fn cooldown_tag() -> Tag<TestAtom> {
    Tag::new([TestAtom::Ability, TestAtom::Variant])
}

pub(crate) fn duration_effect_definition(
    key: &str,
    duration_units: ClockUnits,
) -> FlexEffectDefinition {
    FlexEffectDefinition {
        key: key.to_owned(),
        kind: EffectKind::Duration,
        duration: Some(EffectClockPolicy {
            units: duration_units,
        }),
        period: None,
        routing: EffectRouting::default(),
        payload_schema: (),
    }
}

pub(crate) fn apply_effect<Payload>(
    effects: &mut EffectPipeline<TagSet<TestAtom>, Payload>,
    definition: &FlexEffectDefinition,
    input: EffectApplicationInput<TagSet<TestAtom>, Payload>,
) -> Result<EffectApplyOutcome, EffectApplyError> {
    EffectApply::definition(definition, input).run(effects)
}

pub(crate) fn apply_effect_with_events<Payload, F>(
    effects: &mut EffectPipeline<TagSet<TestAtom>, Payload>,
    definition: &FlexEffectDefinition,
    input: EffectApplicationInput<TagSet<TestAtom>, Payload>,
    emit: F,
) -> Result<EffectApplyOutcome, EffectApplyError>
where
    Payload: Clone,
    F: FnMut(EffectLifecycleEvent<TagSet<TestAtom>, Payload>),
{
    let mut context = ();
    let mut executor = NoEffectExecutor::new().with_owned_events(emit);
    EffectApply::definition(definition, input).run_with_executor(
        effects,
        &mut context,
        &mut executor,
    )
}

pub(crate) fn active_effect_advance_event<Payload>(
    payload: Payload,
    previous_remaining_units: ClockUnits,
    remaining_units: ClockUnits,
) -> LocalLifecycleEvent<TagSet<TestAtom>, Payload>
where
    Payload: Clone,
{
    LocalLifecycleEvent::Effect(EffectLifecycleEvent::Advanced(flexweave::EffectAdvance {
        effect: flexweave::EffectInstance {
            id: ActiveEffectId::new(1),
            definition_key: None,
            source_id: None,
            target_id: ObjectId::new(1),
            remaining_units: Some(remaining_units),
            period: None,
            period_elapsed_units: 0,
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload,
        },
        elapsed_units: previous_remaining_units - remaining_units,
        previous_remaining_units: Some(previous_remaining_units),
    }))
}
