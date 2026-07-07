mod common;

use common::TestAtom;
use flexweave::{
    AbilityActivationId, AbilityId, ActiveAbility, ActiveEffectId, EffectActionExecutor,
    EffectApplicationDecision, EffectApplicationDraft, EffectApplicationError,
    EffectApplicationInput, EffectApply, EffectApplyError, EffectApplyOutcome, EffectClockPolicy,
    EffectDefinition, EffectDefinitionError, EffectDefinitionRegistryError, EffectDefinitions,
    EffectExecutionView, EffectInitializationError, EffectInitializer, EffectKind,
    EffectLifecycleEvent, EffectLifecycleEventView, EffectPipeline, EffectRouting,
    EffectSourcePolicy, EffectTick, EventChannel, EventChannelDefinition, EventRetention,
    LifecycleEventKind, ObjectId, ObjectStore, Tag, TagSet,
};

#[test]
fn effect_pipeline_creates_advances_expires_and_visits_by_target() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Increase,
        More,
    }

    let source = ObjectId::new(7);
    let target = ObjectId::new(42);
    let other_target = ObjectId::new(99);
    let enhancement = Tag::new([TestAtom::Category, TestAtom::Variant]);
    let more = Tag::new([TestAtom::Category, TestAtom::Family]);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    let first = effects
        .apply(
            &effect_definition(
                "short",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 1000 }),
                None,
            ),
            EffectApplicationInput {
                source_id: Some(source),
                target_id: target,
                tags: TagSet::new([enhancement.clone()]),
                payload: Payload::Increase,
                decision: EffectApplicationDecision::Accept,
            },
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    let second = effects
        .apply(
            &effect_definition(
                "long",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 2000 }),
                None,
            ),
            EffectApplicationInput {
                source_id: None,
                target_id: target,
                tags: TagSet::new([more.clone()]),
                payload: Payload::More,
                decision: EffectApplicationDecision::Accept,
            },
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    let third = effects
        .apply(
            &effect_definition(
                "other",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 1000 }),
                None,
            ),
            EffectApplicationInput {
                source_id: None,
                target_id: other_target,
                tags: TagSet::new([enhancement.clone()]),
                payload: Payload::Increase,
                decision: EffectApplicationDecision::Accept,
            },
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");

    assert_eq!(
        (first, second, third),
        (
            ActiveEffectId::new(1),
            ActiveEffectId::new(2),
            ActiveEffectId::new(3),
        )
    );
    assert_eq!(effects.count(), 3);
    assert!(effects.has_tag(target, &enhancement));
    assert!(!effects.has_tag(target, &Tag::new([TestAtom::Group])));

    let mut visited = Vec::new();
    effects.visit_target(target, |effect| {
        visited.push((effect.id, effect.remaining_units))
    });
    assert_eq!(
        visited,
        vec![
            (ActiveEffectId::new(1), Some(1000)),
            (ActiveEffectId::new(2), Some(2000)),
        ]
    );

    let mut events = Vec::new();
    effects.tick_with_events(999, |event| events.push(event));
    assert_eq!(events.len(), 3);
    assert_eq!(
        effects.get(first).map(|effect| effect.remaining_units),
        Some(Some(1))
    );
    assert_eq!(
        effects.get(second).map(|effect| effect.remaining_units),
        Some(Some(1001))
    );

    events.clear();
    effects.tick_with_events(1, |event| events.push(event));
    assert!(matches!(events[0], EffectLifecycleEvent::Advanced(_)));
    assert!(matches!(events[1], EffectLifecycleEvent::Expired(_)));
    assert!(matches!(events[2], EffectLifecycleEvent::Advanced(_)));
    assert!(matches!(events[3], EffectLifecycleEvent::Advanced(_)));
    assert!(matches!(events[4], EffectLifecycleEvent::Expired(_)));
    assert_eq!(effects.count(), 1);
    assert_eq!(
        effects.get(second).map(|effect| effect.remaining_units),
        Some(Some(1000))
    );
    assert!(!effects.has_tag(target, &enhancement));
    assert!(effects.has_tag(target, &more));
}

#[test]
fn effect_initializer_can_adjust_payload_and_duration_from_context() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    struct Runtime {
        amount_bonus: i32,
        duration_multiplier: u64,
    }

    struct Initializer;

    impl EffectInitializer<Runtime, TagSet<TestAtom>, Payload> for Initializer {
        type Error = &'static str;

        fn initialize(
            &mut self,
            context: &mut Runtime,
            draft: EffectApplicationDraft<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            draft.payload.amount += context.amount_bonus;
            let Some(duration) = *draft.duration else {
                return Err("missing-duration");
            };
            *draft.duration = Some(EffectClockPolicy::new(
                duration.units * context.duration_multiplier,
            ));
            Ok(())
        }
    }

    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut runtime = Runtime {
        amount_bonus: 5,
        duration_multiplier: 2,
    };
    let mut initializer = Initializer;
    let mut events = Vec::new();

    let outcome = pipeline
        .apply_initialized_with_events(
            &EffectDefinition::duration("buff", 100, ()),
            EffectApplicationInput::accept(
                Some(ObjectId::new(1)),
                ObjectId::new(2),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload { amount: 10 },
            ),
            &mut runtime,
            &mut initializer,
            |event| events.push(event),
        )
        .unwrap();

    assert_eq!(
        outcome,
        EffectApplyOutcome::ActiveCreated(ActiveEffectId::new(1))
    );
    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::ActiveCreated(created),
    ] = events.as_slice()
    else {
        panic!("initialized duration effect should emit accepted and active-created events");
    };
    assert_eq!(accepted.payload.amount, 15);
    assert_eq!(created.payload.amount, 15);
    assert_eq!(created.remaining_units, Some(200));
}

#[test]
fn instant_effect_action_runs_before_executed_fact() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Runtime {
        applied: Vec<(ObjectId, i32)>,
    }

    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut runtime = Runtime {
        applied: Vec::new(),
    };
    let mut action = |context: &mut Runtime,
                      execution: EffectExecutionView<'_, TagSet<TestAtom>, Payload>|
     -> Result<(), &'static str> {
        assert_eq!(execution.active_effect_id, None);
        assert_eq!(execution.definition_key, Some("hit"));
        assert_eq!(execution.elapsed_units, None);
        context
            .applied
            .push((execution.target_id, execution.payload.amount));
        Ok(())
    };
    let mut events = Vec::new();

    let outcome = {
        let mut executor =
            EffectActionExecutor::new(&mut action).with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &EffectDefinition::instant("hit", ()),
            EffectApplicationInput::accept(
                Some(ObjectId::new(1)),
                ObjectId::new(2),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload { amount: 7 },
            ),
        )
        .run_with_executor(&mut pipeline, &mut runtime, &mut executor)
        .unwrap()
    };

    assert_eq!(outcome, EffectApplyOutcome::ExecutedInstant);
    assert_eq!(runtime.applied, vec![(ObjectId::new(2), 7)]);
    let [
        EffectLifecycleEvent::ApplicationAccepted(_),
        EffectLifecycleEvent::Executed(executed),
    ] = events.as_slice()
    else {
        panic!("successful action should emit accepted and executed facts");
    };
    assert_eq!(executed.payload.amount, 7);
    assert_eq!(pipeline.count(), 0);
}

#[test]
fn failed_instant_effect_action_suppresses_executed_fact() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Runtime {
        attempts: usize,
    }

    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut runtime = Runtime { attempts: 0 };
    let mut action = |context: &mut Runtime,
                      execution: EffectExecutionView<'_, TagSet<TestAtom>, Payload>|
     -> Result<(), &'static str> {
        context.attempts += 1;
        assert_eq!(execution.payload.amount, 13);
        Err("runtime rejected effect")
    };
    let mut events = Vec::new();

    let error = {
        let mut executor =
            EffectActionExecutor::new(&mut action).with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &EffectDefinition::instant("hit", ()),
            EffectApplicationInput::accept(
                Some(ObjectId::new(1)),
                ObjectId::new(2),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload { amount: 13 },
            ),
        )
        .run_with_executor(&mut pipeline, &mut runtime, &mut executor)
        .unwrap_err()
    };

    assert_eq!(
        error,
        EffectApplyError::Execution("runtime rejected effect")
    );
    assert_eq!(runtime.attempts, 1);
    let [EffectLifecycleEvent::ApplicationAccepted(_)] = events.as_slice() else {
        panic!("failed action should only emit accepted application fact");
    };
    assert_eq!(pipeline.count(), 0);
}

#[test]
fn initialized_instant_effect_action_sees_initialized_payload() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    struct Runtime {
        bonus: i32,
        applied_amounts: Vec<i32>,
    }

    struct Initializer;

    impl EffectInitializer<Runtime, TagSet<TestAtom>, Payload> for Initializer {
        type Error = &'static str;

        fn initialize(
            &mut self,
            context: &mut Runtime,
            draft: EffectApplicationDraft<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            draft.payload.amount += context.bonus;
            Ok(())
        }
    }

    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut runtime = Runtime {
        bonus: 4,
        applied_amounts: Vec::new(),
    };
    let mut initializer = Initializer;
    let mut action = |context: &mut Runtime,
                      execution: EffectExecutionView<'_, TagSet<TestAtom>, Payload>|
     -> Result<(), &'static str> {
        context.applied_amounts.push(execution.payload.amount);
        Ok(())
    };
    let mut events = Vec::new();

    {
        let mut executor =
            EffectActionExecutor::new(&mut action).with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &EffectDefinition::instant("hit", ()),
            EffectApplicationInput::accept(
                Some(ObjectId::new(1)),
                ObjectId::new(2),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload { amount: 6 },
            ),
        )
        .initialized(&mut initializer)
        .run_with_executor(&mut pipeline, &mut runtime, &mut executor)
        .unwrap();
    }

    assert_eq!(runtime.applied_amounts, vec![10]);
    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::Executed(executed),
    ] = events.as_slice()
    else {
        panic!("initialized instant action should emit accepted and executed facts");
    };
    assert_eq!(accepted.payload.amount, 10);
    assert_eq!(executed.payload.amount, 10);
}

#[test]
fn effect_initializer_revalidates_runtime_clock_shape() {
    struct Initializer;

    impl EffectInitializer<(), TagSet<TestAtom>, ()> for Initializer {
        type Error = &'static str;

        fn initialize(
            &mut self,
            _context: &mut (),
            mut draft: EffectApplicationDraft<'_, TagSet<TestAtom>, ()>,
        ) -> Result<(), Self::Error> {
            draft.set_period_units(Some(10));
            Ok(())
        }
    }

    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, ()>::new();
    let mut initializer = Initializer;

    let error = pipeline
        .apply_initialized(
            &EffectDefinition::duration("buff", 100, ()),
            EffectApplicationInput::accept(
                Some(ObjectId::new(1)),
                ObjectId::new(2),
                TagSet::new([Tag::new([TestAtom::Category])]),
                (),
            ),
            &mut (),
            &mut initializer,
        )
        .unwrap_err();

    assert_eq!(
        error,
        EffectInitializationError::Definition(EffectDefinitionError::PeriodNotAllowed {
            key: "buff".to_owned(),
        })
    );
}

#[test]
fn effect_pipeline_removes_effects_with_distinct_lifecycle_fact() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let effect_id = effects
        .apply(
            &effect_definition(
                "buff",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 1000 }),
                None,
            ),
            EffectApplicationInput {
                source_id: None,
                target_id: ObjectId::new(1),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Buff,
                decision: EffectApplicationDecision::Accept,
            },
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    let mut events = Vec::new();

    let removed = effects
        .remove_with_events(effect_id, |event| events.push(event))
        .unwrap();

    assert_eq!(removed.id, effect_id);
    assert_eq!(effects.count(), 0);
    let [EffectLifecycleEvent::Removed(removed_event)] = events.as_slice() else {
        panic!("manual removal should emit a removed event");
    };
    assert_eq!(removed_event.id, effect_id);
}

#[test]
fn effect_indexes_survive_removal_expiration_and_keep_application_order() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        First,
        Second,
        Third,
        Fourth,
    }

    let target = ObjectId::new(42);
    let other_target = ObjectId::new(77);
    let enhancement = Tag::new([TestAtom::Category, TestAtom::Variant]);
    let family = Tag::new([TestAtom::Category, TestAtom::Family]);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    let first = effects
        .apply(
            &effect_definition(
                "first",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 100 }),
                None,
            ),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([enhancement.clone()]),
                Payload::First,
            ),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    let second = effects
        .apply(
            &effect_definition(
                "second",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 200 }),
                None,
            ),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([family.clone()]),
                Payload::Second,
            ),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    let third = effects
        .apply(
            &effect_definition(
                "third",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 300 }),
                None,
            ),
            EffectApplicationInput::accept(
                None,
                other_target,
                TagSet::new([enhancement.clone()]),
                Payload::Third,
            ),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    let fourth = effects
        .apply(
            &effect_definition(
                "fourth",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 400 }),
                None,
            ),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([enhancement.clone()]),
                Payload::Fourth,
            ),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");

    let mut target_order = Vec::new();
    effects.visit_target(target, |effect| target_order.push(effect.id));
    assert_eq!(target_order, vec![first, second, fourth]);

    let removed = effects.remove_with_events(second, |_| {}).unwrap();
    assert_eq!(removed.id, second);
    assert!(effects.get(second).is_none());
    assert_eq!(effects.get(third).unwrap().id, third);
    assert_eq!(effects.get(fourth).unwrap().id, fourth);

    target_order.clear();
    effects.visit_target(target, |effect| target_order.push(effect.id));
    assert_eq!(target_order, vec![first, fourth]);
    assert!(effects.has_tag(target, &enhancement));
    assert!(!effects.has_tag(target, &family));

    effects.tick_with_events(100, |_| {});
    assert!(effects.get(first).is_none());
    assert_eq!(effects.get(third).unwrap().id, third);
    assert_eq!(effects.get(fourth).unwrap().id, fourth);

    target_order.clear();
    effects.visit_target(target, |effect| target_order.push(effect.id));
    assert_eq!(target_order, vec![fourth]);
}

#[test]
fn active_effect_ids_are_typed_value_objects_and_pipeline_uses_them() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let id = ActiveEffectId::new(24);
    assert_eq!(id.get(), 24);
    assert_eq!(ActiveEffectId::from(24).get(), 24);
    assert_eq!(u64::from(id), 24);
    assert_eq!(id.to_string(), "24");

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let effect_id = effects
        .apply(
            &effect_definition(
                "typed",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 1000 }),
                None,
            ),
            EffectApplicationInput::accept(
                None,
                ObjectId::new(5),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Buff,
            ),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");

    assert_eq!(effect_id, ActiveEffectId::new(1));
    assert_eq!(effect_id.get(), 1);
    assert_eq!(effects.get(effect_id).unwrap().id, effect_id);
    let removed = effects.remove_with_events(effect_id, |_| {}).unwrap();
    assert_eq!(removed.id, effect_id);

    let mut default_effects = EffectPipeline::<TagSet<TestAtom>, Payload>::default();
    let default_effect_id = default_effects
        .apply(
            &effect_definition(
                "default-typed",
                EffectKind::Duration,
                Some(EffectClockPolicy { units: 1000 }),
                None,
            ),
            EffectApplicationInput::accept(
                None,
                ObjectId::new(6),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Buff,
            ),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");

    assert_eq!(default_effect_id, ActiveEffectId::new(1));
    assert_ne!(default_effect_id, ActiveEffectId::INVALID);
}

#[test]
fn effect_definitions_validate_authoring_shape() {
    assert_eq!(
        effect_definition("duration", EffectKind::Duration, None, None)
            .validate()
            .unwrap_err(),
        EffectDefinitionError::DurationRequired {
            key: "duration".to_owned(),
        }
    );
    assert_eq!(
        effect_definition(
            "periodic",
            EffectKind::Periodic,
            Some(EffectClockPolicy { units: 100 }),
            None,
        )
        .validate()
        .unwrap_err(),
        EffectDefinitionError::PeriodRequired {
            key: "periodic".to_owned(),
        }
    );
    assert_eq!(
        effect_definition(
            "instant",
            EffectKind::Instant,
            Some(EffectClockPolicy { units: 1 }),
            None,
        )
        .validate()
        .unwrap_err(),
        EffectDefinitionError::DurationNotAllowed {
            key: "instant".to_owned(),
        }
    );
    let mut routed = effect_definition("routed", EffectKind::Instant, None, None);
    routed.routing.requires_lifecycle_channel = true;
    assert_eq!(
        routed.validate().unwrap_err(),
        EffectDefinitionError::MissingLifecycleChannelKey {
            key: "routed".to_owned(),
        }
    );
}

#[test]
fn effect_definitions_validate_lookup_and_preserve_declaration_order() {
    let short = effect_definition(
        "short",
        EffectKind::Duration,
        Some(EffectClockPolicy { units: 100 }),
        None,
    );
    let pulse = effect_definition(
        "pulse",
        EffectKind::Periodic,
        Some(EffectClockPolicy { units: 300 }),
        Some(EffectClockPolicy { units: 50 }),
    );

    let definitions = EffectDefinitions::new([short.clone(), pulse.clone()]).unwrap();

    assert_eq!(definitions.definitions()[0].key, "short");
    assert_eq!(definitions.definitions()[1].key, "pulse");
    assert_eq!(
        definitions.require("pulse").unwrap().period,
        Some(EffectClockPolicy { units: 50 })
    );
    assert_eq!(
        definitions.require("missing").unwrap_err(),
        EffectDefinitionRegistryError::MissingDefinition {
            key: "missing".to_owned(),
        }
    );
    assert_eq!(
        EffectDefinitions::new([short.clone(), short]).unwrap_err(),
        EffectDefinitionRegistryError::DuplicateKey {
            key: "short".to_owned(),
        }
    );
    assert_eq!(
        EffectDefinitions::new([effect_definition("", EffectKind::Instant, None, None)])
            .unwrap_err(),
        EffectDefinitionRegistryError::InvalidDefinition {
            error: EffectDefinitionError::EmptyKey,
        }
    );
}

#[test]
fn apply_registered_uses_definition_duration_and_carries_definition_key() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let definitions =
        EffectDefinitions::new([EffectDefinition::duration("buff", 200, ())]).unwrap();
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();

    let active_id = pipeline
        .apply_registered_with_events(
            &definitions,
            "buff",
            application(Payload::Buff, EffectApplicationDecision::Accept),
            |event| events.push(event),
        )
        .unwrap()
        .active_effect_id()
        .expect("registered duration effect should create an active effect");

    assert_eq!(
        pipeline.get(active_id).unwrap().definition_key.as_deref(),
        Some("buff")
    );
    assert_eq!(pipeline.get(active_id).unwrap().remaining_units, Some(200));
    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::ActiveCreated(created),
    ] = events.as_slice()
    else {
        panic!("registered duration effect should emit accepted and active-created events");
    };
    assert_eq!(accepted.definition_key.as_deref(), Some("buff"));
    assert_eq!(created.definition_key.as_deref(), Some("buff"));
    assert_eq!(
        pipeline
            .apply_registered(
                &definitions,
                "missing",
                application(Payload::Buff, EffectApplicationDecision::Accept),
            )
            .unwrap_err(),
        EffectDefinitionRegistryError::MissingDefinition {
            key: "missing".to_owned(),
        }
    );
}

#[test]
fn effect_definition_constructors_match_literals_and_validate() {
    assert_eq!(
        EffectDefinition::instant("instant", "payload/schema"),
        EffectDefinition {
            key: "instant".to_owned(),
            kind: EffectKind::Instant,
            duration: None,
            period: None,
            routing: EffectRouting::default(),
            payload_schema: "payload/schema",
        }
    );
    EffectDefinition::instant("instant", ()).validate().unwrap();

    assert_eq!(
        EffectDefinition::duration("duration", 100, "payload/schema"),
        EffectDefinition {
            key: "duration".to_owned(),
            kind: EffectKind::Duration,
            duration: Some(EffectClockPolicy { units: 100 }),
            period: None,
            routing: EffectRouting::default(),
            payload_schema: "payload/schema",
        }
    );
    EffectDefinition::duration("duration", 100, ())
        .validate()
        .unwrap();
    assert_eq!(
        EffectDefinition::duration("bad_duration", 0, ())
            .validate()
            .unwrap_err(),
        EffectDefinitionError::InvalidDuration {
            key: "bad_duration".to_owned(),
        }
    );

    assert_eq!(
        EffectDefinition::periodic("periodic", 300, 50, "payload/schema"),
        EffectDefinition {
            key: "periodic".to_owned(),
            kind: EffectKind::Periodic,
            duration: Some(EffectClockPolicy { units: 300 }),
            period: Some(EffectClockPolicy { units: 50 }),
            routing: EffectRouting::default(),
            payload_schema: "payload/schema",
        }
    );
    EffectDefinition::periodic("periodic", 300, 50, ())
        .validate()
        .unwrap();

    assert_eq!(
        EffectDefinition::indefinite("indefinite", "payload/schema"),
        EffectDefinition {
            key: "indefinite".to_owned(),
            kind: EffectKind::Indefinite,
            duration: None,
            period: None,
            routing: EffectRouting::default(),
            payload_schema: "payload/schema",
        }
    );
    EffectDefinition::indefinite("indefinite", ())
        .validate()
        .unwrap();
}

#[test]
fn effect_definition_routing_helpers_match_literals_and_validate() {
    let routed = EffectDefinition::instant("routed", ())
        .requiring_lifecycle_channel("effects/lifecycle")
        .with_signal_channels(["signals/effects"]);

    assert_eq!(
        routed,
        EffectDefinition {
            key: "routed".to_owned(),
            kind: EffectKind::Instant,
            duration: None,
            period: None,
            routing: EffectRouting {
                requires_lifecycle_channel: true,
                lifecycle_channel_keys: vec!["effects/lifecycle".to_owned()],
                signal_channel_keys: vec!["signals/effects".to_owned()],
            },
            payload_schema: (),
        }
    );
    routed.validate().unwrap();

    let routing = EffectRouting {
        requires_lifecycle_channel: true,
        lifecycle_channel_keys: vec!["effects/alternate".to_owned()],
        signal_channel_keys: vec!["signals/alternate".to_owned()],
    };
    assert_eq!(
        EffectDefinition::duration("manual_routing", 10, ())
            .with_lifecycle_channels(["ignored"])
            .with_routing(routing.clone()),
        EffectDefinition {
            key: "manual_routing".to_owned(),
            kind: EffectKind::Duration,
            duration: Some(EffectClockPolicy { units: 10 }),
            period: None,
            routing,
            payload_schema: (),
        }
    );
}

#[test]
fn effect_application_input_constructors_match_literals() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let source = ObjectId::new(10);
    let target = ObjectId::new(20);
    let tags = TagSet::new([Tag::new([TestAtom::Category, TestAtom::Variant])]);

    assert_eq!(
        EffectApplicationInput::accept(source, target, tags.clone(), Payload::Hit),
        EffectApplicationInput {
            source_id: Some(source),
            target_id: target,
            tags: tags.clone(),
            payload: Payload::Hit,
            decision: EffectApplicationDecision::Accept,
        }
    );

    assert_eq!(
        EffectApplicationInput::reject(None, target, tags.clone(), Payload::Hit, "blocked"),
        EffectApplicationInput {
            source_id: None,
            target_id: target,
            tags,
            payload: Payload::Hit,
            decision: EffectApplicationDecision::Reject {
                reason: "blocked".to_owned(),
            },
        }
    );
}

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
        pipeline.apply_checked_with_events(
            &objects,
            &effect_definition("hit", EffectKind::Instant, None, None),
            EffectApplicationInput::accept(
                source,
                missing_target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Hit,
            ),
            EffectSourcePolicy::RequireLiveSource,
            |event| events.push(event),
        ),
        Err(EffectApplicationError::InvalidTarget {
            target_id: missing_target,
        })
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
        pipeline.apply_checked_with_events(
            &objects,
            &effect_definition("hit", EffectKind::Instant, None, None),
            EffectApplicationInput::accept(
                missing_source,
                target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Hit,
            ),
            EffectSourcePolicy::RequireLiveSource,
            |event| events.push(event),
        ),
        Err(EffectApplicationError::InvalidSource {
            source_id: missing_source,
        })
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

    let outcome = pipeline
        .apply_checked_with_events(
            &objects,
            &effect_definition("hit", EffectKind::Instant, None, None),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Hit,
            ),
            EffectSourcePolicy::AllowSystemSource,
            |event| events.push(event),
        )
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
        pipeline.apply_checked(
            &objects,
            &effect_definition("requires_source", EffectKind::Instant, None, None),
            EffectApplicationInput::accept(
                None,
                target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload::Hit,
            ),
            EffectSourcePolicy::RequireLiveSource,
        ),
        Err(EffectApplicationError::MissingSource)
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
    pipeline
        .apply_checked_with_events(
            &objects,
            &effect_definition("hit", EffectKind::Instant, None, None),
            input,
            EffectSourcePolicy::RequireLiveSource,
            |event| events.push(event),
        )
        .unwrap();

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

    let outcome = pipeline
        .apply_with_events(
            &definition,
            application(Payload::Hit, EffectApplicationDecision::Accept),
            |event| events.push(event),
        )
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

    let outcome = pipeline
        .apply_with_events(
            &definition,
            application(
                Payload::Buff,
                EffectApplicationDecision::Reject {
                    reason: "blocked".to_owned(),
                },
            ),
            |event| events.push(event),
        )
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
        pipeline
            .apply(
                &instant,
                application(Payload::Hit, EffectApplicationDecision::Accept),
            )
            .unwrap(),
        EffectApplyOutcome::ExecutedInstant
    );
    assert_eq!(pipeline.count(), 0);

    assert_eq!(
        pipeline
            .apply(
                &duration,
                application(
                    Payload::Buff,
                    EffectApplicationDecision::Reject {
                        reason: "blocked".to_owned(),
                    },
                ),
            )
            .unwrap(),
        EffectApplyOutcome::Rejected
    );
    assert_eq!(pipeline.count(), 0);

    assert_eq!(
        pipeline
            .apply(
                &duration,
                application(Payload::Buff, EffectApplicationDecision::Accept),
            )
            .unwrap(),
        EffectApplyOutcome::ActiveCreated(ActiveEffectId::new(1))
    );
    assert_eq!(pipeline.count(), 1);
}

#[test]
fn duration_effects_advance_expire_and_remove_in_lifecycle_order() {
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
    let active_id = pipeline
        .apply_with_events(
            &definition,
            application(Payload::Buff, EffectApplicationDecision::Accept),
            |event| events.push(event),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    events.clear();

    pipeline.tick_with_events(40, |event| events.push(event));
    let [EffectLifecycleEvent::Advanced(advanced)] = events.as_slice() else {
        panic!("partial duration tick should emit one advance fact");
    };
    assert_eq!(advanced.elapsed_units, 40);
    assert_eq!(advanced.previous_remaining_units, Some(100));
    assert_eq!(advanced.effect.remaining_units, Some(60));
    events.clear();

    let removed = pipeline
        .remove_with_events(active_id, |event| events.push(event))
        .unwrap();
    assert_eq!(removed.id, active_id);
    let [EffectLifecycleEvent::Removed(removed_event)] = events.as_slice() else {
        panic!("manual removal should emit removed, not expired");
    };
    assert_eq!(removed_event.id, active_id);
    assert_eq!(pipeline.count(), 0);
    events.clear();

    pipeline.tick_with_events(100, |event| events.push(event));
    assert!(events.is_empty());

    let expiring_id = pipeline
        .apply(
            &definition,
            application(Payload::Buff, EffectApplicationDecision::Accept),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    pipeline.tick_with_events(100, |event| events.push(event));
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("natural timeout should advance then expire");
    };
    assert_eq!(expiring_advance.effect.remaining_units, Some(0));
    assert_eq!(expired.id, expiring_id);
    assert_eq!(pipeline.count(), 0);
}

#[test]
fn periodic_effects_execute_at_deterministic_intervals() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Pulse,
    }

    let definition = effect_definition(
        "pulse",
        EffectKind::Periodic,
        Some(EffectClockPolicy { units: 100 }),
        Some(EffectClockPolicy { units: 30 }),
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    pipeline
        .apply(
            &definition,
            application(Payload::Pulse, EffectApplicationDecision::Accept),
        )
        .unwrap();
    let mut events = Vec::new();

    pipeline.tick_with_events(70, |event| events.push(event));
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(first),
        EffectLifecycleEvent::PeriodicExecuted(second),
    ] = events.as_slice()
    else {
        panic!("70 units with a 30-unit period should execute twother");
    };
    assert_eq!(advanced.effect.remaining_units, Some(30));
    assert_eq!(first.elapsed_units, Some(30));
    assert_eq!(second.elapsed_units, Some(30));
    events.clear();

    pipeline.tick_with_events(30, |event| events.push(event));
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::PeriodicExecuted(final_pulse),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("final period should execute before natural expiration");
    };
    assert_eq!(expiring_advance.effect.remaining_units, Some(0));
    assert_eq!(final_pulse.elapsed_units, Some(30));
    assert_eq!(expired.remaining_units, Some(0));
}

#[test]
fn periodic_effect_action_must_complete_before_periodic_executed_fact() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Runtime {
        applied: Vec<i32>,
        fail_on_attempt: usize,
        attempts: usize,
    }

    let definition = effect_definition(
        "pulse",
        EffectKind::Periodic,
        Some(EffectClockPolicy { units: 100 }),
        Some(EffectClockPolicy { units: 30 }),
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let active_id = pipeline
        .apply(
            &definition,
            application(Payload { amount: 5 }, EffectApplicationDecision::Accept),
        )
        .unwrap()
        .active_effect_id()
        .expect("periodic effect should create active state");
    let mut runtime = Runtime {
        applied: Vec::new(),
        fail_on_attempt: 2,
        attempts: 0,
    };
    let mut action = |context: &mut Runtime,
                      execution: EffectExecutionView<'_, TagSet<TestAtom>, Payload>|
     -> Result<(), &'static str> {
        context.attempts += 1;
        assert_eq!(execution.active_effect_id, Some(active_id));
        assert_eq!(execution.elapsed_units, Some(30));
        if context.attempts == context.fail_on_attempt {
            return Err("periodic action failed");
        }
        context.applied.push(execution.payload.amount);
        Ok(())
    };
    let mut events = Vec::new();

    let error = {
        let mut executor =
            EffectActionExecutor::new(&mut action).with_owned_events(|event| events.push(event));
        EffectTick::new(70)
            .run_with_executor(&mut pipeline, &mut runtime, &mut executor)
            .unwrap_err()
    };

    assert_eq!(error, "periodic action failed");
    assert_eq!(runtime.attempts, 2);
    assert_eq!(runtime.applied, vec![5]);
    let [
        EffectLifecycleEvent::Advanced(_),
        EffectLifecycleEvent::PeriodicExecuted(executed),
    ] = events.as_slice()
    else {
        panic!("failed second action should not emit a second periodic execution fact");
    };
    assert_eq!(executed.active_effect_id, Some(active_id));
    assert_eq!(pipeline.count(), 1);
}

#[test]
fn periodic_effects_do_not_execute_past_their_lifetime() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Pulse,
    }

    let definition = effect_definition(
        "short_pulse",
        EffectKind::Periodic,
        Some(EffectClockPolicy { units: 50 }),
        Some(EffectClockPolicy { units: 20 }),
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    pipeline
        .apply(
            &definition,
            application(Payload::Pulse, EffectApplicationDecision::Accept),
        )
        .unwrap();
    let mut events = Vec::new();

    pipeline.tick_with_events(100, |event| events.push(event));

    let [
        EffectLifecycleEvent::Advanced(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::Expired(_),
    ] = events.as_slice()
    else {
        panic!("periodic execution should be capped to the active lifetime");
    };
}

#[test]
fn caller_publishes_effect_lifecycle_events_to_named_channels() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Hit,
    }

    let definition = EffectDefinition {
        routing: EffectRouting {
            requires_lifecycle_channel: true,
            lifecycle_channel_keys: vec!["effects/lifecycle".to_owned()],
            signal_channel_keys: vec!["signals/effects".to_owned()],
        },
        ..effect_definition("hit", EffectKind::Instant, None, None)
    };
    definition.validate().unwrap();

    let channel_definition = EventChannelDefinition::new(
        "effects/lifecycle",
        [
            LifecycleEventKind::EffectApplicationAccepted,
            LifecycleEventKind::EffectExecuted,
        ],
    )
    .unwrap();
    let mut channel = EventChannel::with_retention(channel_definition, EventRetention::Retain);
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    pipeline
        .apply_with_events(
            &definition,
            application(Payload::Hit, EffectApplicationDecision::Accept),
            |event| channel.publish(event).unwrap(),
        )
        .unwrap();

    let retained = channel.drain_retained();
    assert_eq!(retained.len(), 2);
    assert!(matches!(
        retained[0],
        EffectLifecycleEvent::ApplicationAccepted(_)
    ));
    assert!(matches!(retained[1], EffectLifecycleEvent::Executed(_)));
}

#[test]
fn borrowed_effect_lifecycle_accepts_non_clone_payloads() {
    #[derive(Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    let definition = effect_definition(
        "borrowed",
        EffectKind::Duration,
        Some(EffectClockPolicy { units: 10 }),
        None,
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut application_kinds = Vec::new();

    let active_id = pipeline
        .apply_with_borrowed_events(
            &definition,
            application(Payload { amount: 7 }, EffectApplicationDecision::Accept),
            |event| {
                application_kinds.push(match event {
                    EffectLifecycleEventView::ApplicationAccepted(application) => {
                        assert_eq!(application.payload.amount, 7);
                        LifecycleEventKind::EffectApplicationAccepted
                    }
                    EffectLifecycleEventView::ActiveCreated(effect) => {
                        assert_eq!(effect.payload.amount, 7);
                        LifecycleEventKind::EffectActiveCreated
                    }
                    _ => panic!("unexpected borrowed application event"),
                });
            },
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");

    assert_eq!(
        application_kinds,
        vec![
            LifecycleEventKind::EffectApplicationAccepted,
            LifecycleEventKind::EffectActiveCreated,
        ]
    );

    let mut tick_kinds = Vec::new();
    pipeline.tick_with_borrowed_events(5, |event| {
        let EffectLifecycleEventView::Advanced(advanced) = event else {
            panic!("partial tick should only advance");
        };
        assert_eq!(advanced.effect.payload.amount, 7);
        tick_kinds.push(LifecycleEventKind::EffectAdvanced);
    });
    assert_eq!(tick_kinds, vec![LifecycleEventKind::EffectAdvanced]);

    let removed = pipeline
        .remove_with_borrowed_events(active_id, |event| {
            let EffectLifecycleEventView::Removed(effect) = event else {
                panic!("manual removal should emit removed");
            };
            assert_eq!(effect.payload.amount, 7);
        })
        .unwrap();
    assert_eq!(removed.payload.amount, 7);
}

#[test]
fn effect_no_event_paths_accept_non_clone_payloads() {
    #[derive(Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    let definition = effect_definition(
        "no_events",
        EffectKind::Duration,
        Some(EffectClockPolicy { units: 3 }),
        None,
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    let active_id = pipeline
        .apply(
            &definition,
            application(Payload { amount: 11 }, EffectApplicationDecision::Accept),
        )
        .unwrap()
        .active_effect_id()
        .expect("duration effect should create an active effect");
    pipeline.tick(1);
    let removed = pipeline.remove(active_id).unwrap();

    assert_eq!(removed.payload.amount, 11);
}

fn effect_definition(
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

fn application<Payload>(
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

trait EffectApplyOutcomeTestExt {
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
