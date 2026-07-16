use super::support::*;

#[test]
fn event_channel_disconnects_handles_during_emission_without_reordering() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Tick,
    }

    let definition = EventChannelDefinition::new(
        "mechanics/effect-advances",
        [LifecycleEventKind::EffectAdvanced],
    )
    .unwrap();
    let mut channel =
        EventChannel::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new(definition);
    let trace = Arc::new(Mutex::new(Vec::new()));
    let later_handle = Arc::new(Mutex::new(None::<EventConnectionHandle>));

    let first_trace = Arc::clone(&trace);
    let first_later_handle = Arc::clone(&later_handle);
    channel.subscribe(move |_| {
        first_trace.lock().unwrap().push("first");
        if let Some(handle) = first_later_handle.lock().unwrap().as_ref() {
            handle.disconnect();
        }
    });

    let second_trace = Arc::clone(&trace);
    let second = channel.subscribe(move |_| {
        second_trace.lock().unwrap().push("second");
    });
    *later_handle.lock().unwrap() = Some(second.clone());

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut effects,
        &duration_effect_definition("tick", 10),
        EffectApplicationInput {
            source_id: None,
            target_id: ObjectId::new(1),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Tick,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();
    let mut events = MechanicsTick::new(1).run(
        MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    assert_eq!(events.len(), 1);
    channel.publish(events.remove(0)).unwrap();

    assert_eq!(*trace.lock().unwrap(), vec!["first"]);
    assert!(!second.is_connected());
    assert_eq!(channel.listener_count(), 1);
}

#[test]
fn scoped_event_connections_disconnect_on_drop_and_retained_batches_drain() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Tick,
    }

    let definition = EventChannelDefinition::new(
        "mechanics/effect-advances",
        [LifecycleEventKind::EffectAdvanced],
    )
    .unwrap();
    let mut channel = EventChannel::with_retention(definition, EventRetention::Retain);
    let trace = Arc::new(Mutex::new(Vec::new()));
    let listener_trace = Arc::clone(&trace);
    let scoped = channel.subscribe_scoped(
        move |event: &LocalLifecycleEvent<TagSet<TestAtom>, Payload>| {
            listener_trace
                .lock()
                .unwrap()
                .push(event.lifecycle_event_kind());
        },
    );

    let first_event = active_effect_advance_event(Payload::Tick, 10, 9);
    let second_event = active_effect_advance_event(Payload::Tick, 9, 8);
    channel.publish(first_event).unwrap();
    drop(scoped);
    channel.publish(second_event).unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![LifecycleEventKind::EffectAdvanced]
    );
    assert_eq!(channel.listener_count(), 0);
    assert_eq!(channel.retained().len(), 2);
    assert_eq!(channel.drain_retained().len(), 2);
    assert!(channel.retained().is_empty());
}

#[test]
fn event_channel_validation_rejects_invalid_definitions_and_routes() {
    assert_eq!(
        EventChannelDefinition::new("", [LifecycleEventKind::EffectActiveCreated]).unwrap_err(),
        EventChannelDefinitionError::EmptyChannelName
    );
    assert_eq!(
        EventChannelDefinition::new("bad name", [LifecycleEventKind::EffectActiveCreated])
            .unwrap_err(),
        EventChannelDefinitionError::InvalidChannelName {
            channel_name: "bad name".to_owned()
        }
    );
    assert_eq!(
        EventChannelDefinition::new("empty", []).unwrap_err(),
        EventChannelDefinitionError::EmptyPayloadContract {
            channel_name: "empty".to_owned()
        }
    );
    assert_eq!(
        EventChannelDefinition::new(
            "duplicates",
            [
                LifecycleEventKind::EffectActiveCreated,
                LifecycleEventKind::EffectActiveCreated,
            ],
        )
        .unwrap_err(),
        EventChannelDefinitionError::DuplicatePayloadKind {
            channel_name: "duplicates".to_owned(),
            kind: LifecycleEventKind::EffectActiveCreated,
        }
    );

    let effect_definition =
        EventChannelDefinition::new("effects", [LifecycleEventKind::EffectActiveCreated]).unwrap();
    assert_eq!(
        EventChannelDefinitions::new([effect_definition.clone(), effect_definition.clone()])
            .unwrap_err(),
        EventChannelDefinitionError::DuplicateChannelDefinition {
            channel_name: "effects".to_owned()
        }
    );

    let definitions = EventChannelDefinitions::new([effect_definition]).unwrap();
    let invalid_route =
        EventChannelRouteDefinition::new("effects", LifecycleEventKind::AttributeChanged).unwrap();
    assert_eq!(
        definitions.validate_route(&invalid_route).unwrap_err(),
        EventChannelDefinitionError::PayloadMismatch {
            channel_name: "effects".to_owned(),
            kind: LifecycleEventKind::AttributeChanged,
        }
    );
    let missing_route =
        EventChannelRouteDefinition::new("missing", LifecycleEventKind::EffectActiveCreated)
            .unwrap();
    assert_eq!(
        definitions.validate_route(&missing_route).unwrap_err(),
        EventChannelDefinitionError::MissingChannelDefinition {
            channel_name: "missing".to_owned(),
        }
    );

    let mut channel = EventChannel::<LocalLifecycleEvent<TagSet<TestAtom>, ()>>::new(
        EventChannelDefinition::new("effects", [LifecycleEventKind::EffectActiveCreated]).unwrap(),
    );
    assert_eq!(
        channel
            .publish(LocalLifecycleEvent::AttributeChanged(
                flexweave::AttributeChange {
                    id: ObjectId::new(1),
                    previous: None,
                    requested: 1.0,
                    current: 1.0,
                }
            ))
            .unwrap_err(),
        EventChannelError::PayloadMismatch {
            channel_name: "effects".to_owned(),
            kind: LifecycleEventKind::AttributeChanged,
        }
    );
}
