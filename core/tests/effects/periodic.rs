use super::support::{EffectApplyOutcomeTestExt, application, effect_definition};
use crate::common::TestAtom;
use flexweave::{
    EffectActionExecutor, EffectApplicationDecision, EffectApply, EffectClockPolicy,
    EffectExecutionView, EffectKind, EffectLifecycleEvent, EffectPipeline, EffectTick,
    NoEffectExecutor, TagSet,
};

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
    EffectApply::definition(
        &definition,
        application(Payload::Pulse, EffectApplicationDecision::Accept),
    )
    .run(&mut pipeline)
    .unwrap();
    let mut events = Vec::new();

    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(70)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }
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

    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(30)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }
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
    let active_id = EffectApply::definition(
        &definition,
        application(Payload { amount: 5 }, EffectApplicationDecision::Accept),
    )
    .run(&mut pipeline)
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
    EffectApply::definition(
        &definition,
        application(Payload::Pulse, EffectApplicationDecision::Accept),
    )
    .run(&mut pipeline)
    .unwrap();
    let mut events = Vec::new();

    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(100)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }

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
