use crate::common::TestAtom;
use flexweave::{
    ActiveEffectId, EffectActionExecutor, EffectApplicationDraft, EffectApplicationInput,
    EffectApply, EffectApplyError, EffectApplyOutcome, EffectClockPolicy, EffectDefinition,
    EffectDefinitionError, EffectExecutionView, EffectInitializer, EffectLifecycleEvent,
    EffectPipeline, NoEffectExecutor, ObjectId, Tag, TagSet,
};

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

    let outcome = {
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &EffectDefinition::duration("buff", 100, ()),
            EffectApplicationInput::accept(
                Some(ObjectId::new(1)),
                ObjectId::new(2),
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload { amount: 10 },
            ),
        )
        .initialized(&mut initializer)
        .run_with_executor(&mut pipeline, &mut runtime, &mut executor)
    }
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
    let mut context = ();
    let mut executor = NoEffectExecutor::new();

    let error = EffectApply::definition(
        &EffectDefinition::duration("buff", 100, ()),
        EffectApplicationInput::accept(
            Some(ObjectId::new(1)),
            ObjectId::new(2),
            TagSet::new([Tag::new([TestAtom::Category])]),
            (),
        ),
    )
    .initialized(&mut initializer)
    .run_with_executor(&mut pipeline, &mut context, &mut executor)
    .unwrap_err();

    assert_eq!(
        error,
        EffectApplyError::Definition(EffectDefinitionError::PeriodNotAllowed {
            key: "buff".to_owned(),
        })
    );
}
