use super::support::{application, effect_definition};
use crate::common::TestAtom;
use flexweave::{
    AbilityActivationId, AbilityId, ActiveAbility, ActiveEffectId, EffectApplicationDecision,
    EffectApplicationError, EffectApplicationInput, EffectApply, EffectApplyError,
    EffectApplyOutcome, EffectClockPolicy, EffectKind, EffectLifecycleEvent, EffectPipeline,
    EffectSourcePolicy, NoEffectExecutor, ObjectId, ObjectStore, Tag, TagSet,
};

#[test]
fn checked_effect_application_rejects_invalid_target() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let mut objects = ObjectStore::new();
    let source = objects.create();
    let missing_target = ObjectId::new(9_999);
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();

    assert_eq!(
        {
            let mut context = ();
            let mut executor =
                NoEffectExecutor::new().with_owned_events(|event| events.push(event));
            EffectApply::definition(
                &effect_definition("hit", EffectKind::Instant, None, None),
                EffectApplicationInput::accept(
                    source,
                    missing_target,
                    TagSet::new([Tag::new([TestAtom::Category])]),
                    Payload::Hit,
                ),
            )
            .checked(&objects, EffectSourcePolicy::RequireLiveSource)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
        },
        Err(EffectApplyError::Application(
            EffectApplicationError::InvalidTarget {
                target_id: missing_target,
            }
        ))
    );
    assert!(events.is_empty());
    assert_eq!(pipeline.count(), 0);
}

#[test]
fn checked_effect_application_rejects_invalid_explicit_source() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let mut objects = ObjectStore::new();
    let target = objects.create();
    let missing_source = ObjectId::new(9_999);
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();

    assert_eq!(
        {
            let mut context = ();
            let mut executor =
                NoEffectExecutor::new().with_owned_events(|event| events.push(event));
            EffectApply::definition(
                &effect_definition("hit", EffectKind::Instant, None, None),
                EffectApplicationInput::accept(
                    missing_source,
                    target,
                    TagSet::new([Tag::new([TestAtom::Category])]),
                    Payload::Hit,
                ),
            )
            .checked(&objects, EffectSourcePolicy::RequireLiveSource)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
        },
        Err(EffectApplyError::Application(
            EffectApplicationError::InvalidSource {
                source_id: missing_source,
            }
        ))
    );
    assert!(events.is_empty());
    assert_eq!(pipeline.count(), 0);
}

#[test]
fn checked_effect_application_allows_system_source_when_policy_permits() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let mut objects = ObjectStore::new();
    let target = objects.create();
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();

    let outcome = {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &effect_definition("hit", EffectKind::Instant, None, None),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Hit,
            ),
        )
        .checked(&objects, EffectSourcePolicy::AllowSystemSource)
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
    }
    .unwrap();

    assert_eq!(outcome, EffectApplyOutcome::ExecutedInstant);
    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::Executed(executed),
    ] = events.as_slice()
    else {
        panic!("system-sourced instant effect should be accepted and executed");
    };
    assert_eq!(accepted.source_id, None);
    assert_eq!(executed.source_id, None);

    assert_eq!(
        EffectApply::definition(
            &effect_definition("requires_source", EffectKind::Instant, None, None),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Hit,
            ),
        )
        .checked(&objects, EffectSourcePolicy::RequireLiveSource)
        .run(&mut pipeline),
        Err(EffectApplyError::Application(
            EffectApplicationError::MissingSource
        ))
    );
}

#[test]
fn effect_input_can_derive_source_from_active_ability() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let mut objects = ObjectStore::new();
    let source = objects.create();
    let target = objects.create();
    let active = ActiveAbility {
        activation_id: AbilityActivationId::new(1),
        ability_id: AbilityId::new(1),
        definition_key: None,
        owner_id: source,
        tags: TagSet::new([Tag::new([TestAtom::Ability])]),
        payload: (),
        committed: true,
    };
    let input = EffectApplicationInput::accept_from_active_ability(
        &active,
        target,
        TagSet::new([Tag::new([TestAtom::Category])]),
        Payload::Hit,
    );

    assert_eq!(active.source_id(), source);
    assert_eq!(input.source_id, Some(source));

    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &effect_definition("hit", EffectKind::Instant, None, None),
            input,
        )
        .checked(&objects, EffectSourcePolicy::RequireLiveSource)
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
        .unwrap();
    }

    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::Executed(executed),
    ] = events.as_slice()
    else {
        panic!("active-ability-sourced instant effect should be accepted and executed");
    };
    assert_eq!(accepted.source_id, Some(source));
    assert_eq!(executed.source_id, Some(source));
}

#[test]
fn instant_effect_execution_emits_without_active_storage() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let definition = effect_definition("hit", EffectKind::Instant, None, None);
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();

    let outcome = {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &definition,
            application(Payload::Hit, EffectApplicationDecision::Accept),
        )
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
    }
    .unwrap();

    assert_eq!(outcome, EffectApplyOutcome::ExecutedInstant);
    assert_eq!(pipeline.count(), 0);
    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::Executed(executed),
    ] = events.as_slice()
    else {
        panic!("instant effect should emit accepted then executed");
    };
    assert_eq!(accepted.target_id, ObjectId::new(20));
    assert_eq!(executed.active_effect_id, None);
    assert_eq!(executed.target_id, ObjectId::new(20));
    assert_eq!(executed.payload, Payload::Hit);
}

#[test]
fn rejected_effect_application_leaves_no_active_effect() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let definition = effect_definition(
        "buff",
        EffectKind::Duration,
        Some(EffectClockPolicy { units: 100 }),
        None,
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();

    let outcome = {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &definition,
            application(
                Payload::Buff,
                EffectApplicationDecision::Reject {
                    reason: "blocked".to_owned(),
                },
            ),
        )
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
    }
    .unwrap();

    assert_eq!(outcome, EffectApplyOutcome::Rejected);
    assert_eq!(pipeline.count(), 0);
    let [EffectLifecycleEvent::ApplicationRejected(rejected)] = events.as_slice() else {
        panic!("rejected application should emit only a rejection fact");
    };
    assert_eq!(rejected.reason, "blocked");
    assert_eq!(rejected.application.target_id, ObjectId::new(20));
}

#[test]
fn effect_apply_outcomes_distinguish_rejected_instant_and_active_creation() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
        Buff,
    }

    let instant = effect_definition("hit", EffectKind::Instant, None, None);
    let duration = effect_definition(
        "buff",
        EffectKind::Duration,
        Some(EffectClockPolicy { units: 100 }),
        None,
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    assert_eq!(
        EffectApply::definition(
            &instant,
            application(Payload::Hit, EffectApplicationDecision::Accept)
        )
        .run(&mut pipeline)
        .unwrap(),
        EffectApplyOutcome::ExecutedInstant
    );
    assert_eq!(pipeline.count(), 0);

    assert_eq!(
        EffectApply::definition(
            &duration,
            application(
                Payload::Buff,
                EffectApplicationDecision::Reject {
                    reason: "blocked".to_owned(),
                },
            ),
        )
        .run(&mut pipeline)
        .unwrap(),
        EffectApplyOutcome::Rejected
    );
    assert_eq!(pipeline.count(), 0);

    assert_eq!(
        EffectApply::definition(
            &duration,
            application(Payload::Buff, EffectApplicationDecision::Accept),
        )
        .run(&mut pipeline)
        .unwrap(),
        EffectApplyOutcome::ActiveCreated(ActiveEffectId::new(1))
    );
    assert_eq!(pipeline.count(), 1);
}
