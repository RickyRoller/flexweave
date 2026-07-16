use super::support::*;

#[test]
fn lifecycle_events_emit_in_registration_order_through_mechanics_driver() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        First,
        Second,
    }

    let mut first = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut second = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut first,
        &duration_effect_definition("first", 100),
        EffectApplicationInput {
            source_id: Some(ObjectId::new(1)),
            target_id: ObjectId::new(2),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::First,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();
    apply_effect(
        &mut second,
        &duration_effect_definition("second", 100),
        EffectApplicationInput {
            source_id: Some(ObjectId::new(3)),
            target_id: ObjectId::new(4),
            tags: TagSet::new([Tag::new([TestAtom::Group])]),
            payload: Payload::Second,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();

    let events = MechanicsTick::new(40).run(
        MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut first)
            .with_store(&mut second),
    );

    let [
        LocalLifecycleEvent::Effect(EffectLifecycleEvent::Advanced(first_advance)),
        LocalLifecycleEvent::Effect(EffectLifecycleEvent::Advanced(second_advance)),
    ] = events.as_slice()
    else {
        panic!("both effect pipelines should emit one advanced lifecycle event");
    };
    assert_eq!(first_advance.effect.payload, Payload::First);
    assert_eq!(first_advance.effect.target_id, ObjectId::new(2));
    assert_eq!(first_advance.elapsed_units, 40);
    assert_eq!(second_advance.effect.payload, Payload::Second);
    assert_eq!(second_advance.effect.target_id, ObjectId::new(4));
    assert_eq!(second_advance.elapsed_units, 40);
}

#[test]
fn zero_elapsed_lifecycle_tick_emits_no_events() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Timed,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut effects,
        &duration_effect_definition("timed", 100),
        EffectApplicationInput {
            source_id: None,
            target_id: ObjectId::new(1),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Timed,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();

    let events = MechanicsTick::new(0).run(
        MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );

    assert!(events.is_empty());
    assert_eq!(
        effects
            .get(ActiveEffectId::new(1))
            .map(|effect| effect.remaining_units),
        Some(Some(100))
    );
}

#[test]
fn caller_publishes_effect_lifecycle_events_to_named_retained_channel() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload {
        amount: i32,
    }

    let channel_definition = EventChannelDefinition::new(
        "mechanics/effects",
        [
            LifecycleEventKind::EffectApplicationAccepted,
            LifecycleEventKind::EffectActiveCreated,
            LifecycleEventKind::EffectAdvanced,
            LifecycleEventKind::EffectExpired,
        ],
    )
    .unwrap();
    assert_eq!(channel_definition.name(), "mechanics/effects");
    assert_eq!(
        channel_definition.accepted_kinds(),
        &[
            LifecycleEventKind::EffectApplicationAccepted,
            LifecycleEventKind::EffectActiveCreated,
            LifecycleEventKind::EffectAdvanced,
            LifecycleEventKind::EffectExpired,
        ]
    );

    let mut channel = EventChannel::with_retention(channel_definition, EventRetention::Retain);
    let trace = Arc::new(Mutex::new(Vec::new()));
    let listener_trace = Arc::clone(&trace);
    channel.subscribe(
        move |event: &LocalLifecycleEvent<TagSet<TestAtom>, Payload>| {
            listener_trace
                .lock()
                .unwrap()
                .push(event.lifecycle_event_kind());
        },
    );

    let source = ObjectId::new(11);
    let target = ObjectId::new(12);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect_with_events(
        &mut effects,
        &duration_effect_definition("buff", 100),
        EffectApplicationInput {
            source_id: Some(source),
            target_id: target,
            tags: TagSet::new([Tag::new([TestAtom::Category, TestAtom::Variant])]),
            payload: Payload { amount: 9 },
            decision: EffectApplicationDecision::Accept,
        },
        |event| channel.publish(event.into()).unwrap(),
    )
    .unwrap();
    MechanicsTick::new(100).run_streaming(
        MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
        |event| channel.publish(event).unwrap(),
    );

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            LifecycleEventKind::EffectApplicationAccepted,
            LifecycleEventKind::EffectActiveCreated,
            LifecycleEventKind::EffectAdvanced,
            LifecycleEventKind::EffectExpired,
        ]
    );
    let retained = channel.drain_retained();
    let [
        LocalLifecycleEvent::Effect(EffectLifecycleEvent::ApplicationAccepted(accepted)),
        LocalLifecycleEvent::Effect(EffectLifecycleEvent::ActiveCreated(created)),
        LocalLifecycleEvent::Effect(EffectLifecycleEvent::Advanced(advanced)),
        LocalLifecycleEvent::Effect(EffectLifecycleEvent::Expired(expired)),
    ] = retained.as_slice()
    else {
        panic!("retained batch should preserve effect lifecycle ordering");
    };
    assert_eq!(accepted.source_id, Some(source));
    assert_eq!(accepted.target_id, target);
    assert_eq!(accepted.payload, Payload { amount: 9 });
    assert_eq!(created.source_id, Some(source));
    assert_eq!(created.target_id, target);
    assert_eq!(created.payload, Payload { amount: 9 });
    assert_eq!(advanced.elapsed_units, 100);
    assert_eq!(advanced.previous_remaining_units, Some(100));
    assert_eq!(advanced.effect.remaining_units, Some(0));
    assert_eq!(expired.source_id, Some(source));
    assert_eq!(expired.target_id, target);
    assert_eq!(expired.remaining_units, Some(0));
    assert!(channel.retained().is_empty());
}
