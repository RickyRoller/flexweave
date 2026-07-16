use super::support::*;

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

    {
        let mut context = ();
        let mut executor =
            NoEffectExecutor::new().with_owned_events(|event| channel.publish(event).unwrap());
        EffectApply::definition(
            &definition,
            application(Payload::Hit, EffectApplicationDecision::Accept),
        )
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
        .unwrap();
    }

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

    let active_id = {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_borrowed_events(
            |event: EffectLifecycleEventView<'_, TagSet<TestAtom>, Payload>| {
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
        );
        EffectApply::definition(
            &definition,
            application(Payload { amount: 7 }, EffectApplicationDecision::Accept),
        )
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
    }
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
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_borrowed_events(
            |event: EffectLifecycleEventView<'_, TagSet<TestAtom>, Payload>| {
                let EffectLifecycleEventView::Advanced(advanced) = event else {
                    panic!("partial tick should only advance");
                };
                assert_eq!(advanced.effect.payload.amount, 7);
                tick_kinds.push(LifecycleEventKind::EffectAdvanced);
            },
        );
        EffectTick::new(5)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }
    assert_eq!(tick_kinds, vec![LifecycleEventKind::EffectAdvanced]);

    let removed = {
        let mut sink = |event: EffectLifecycleEventView<'_, TagSet<TestAtom>, Payload>| {
            let EffectLifecycleEventView::Removed(effect) = event else {
                panic!("manual removal should emit removed");
            };
            assert_eq!(effect.payload.amount, 7);
        };
        EffectRemove::new(active_id)
            .run_with_sink(&mut pipeline, &mut sink)
            .unwrap()
    };
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

    let active_id = EffectApply::definition(
        &definition,
        application(Payload { amount: 11 }, EffectApplicationDecision::Accept),
    )
    .run(&mut pipeline)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    EffectTick::new(1).run(&mut pipeline);
    let removed = EffectRemove::new(active_id).run(&mut pipeline).unwrap();

    assert_eq!(removed.payload.amount, 11);
}
