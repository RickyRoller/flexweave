mod common;

use common::TestAtom;
use flexweave::{
    AbilityCommitTiming, AbilityHooks, AbilityStore, ActiveEffectId, Clock, ClockUnits,
    CooldownUnits, DefinitionRegistryEntry, EffectApplicationDecision, EffectApplicationInput,
    EffectClockPolicy, EffectDefinition as FlexEffectDefinition, EffectKind, EffectLifecycleEvent,
    EffectPipeline, EffectRouting, EventChannel, EventChannelDefinition,
    EventChannelDefinitionError, EventChannelDefinitions, EventChannelError,
    EventChannelRouteDefinition, EventConnectionHandle, EventRetention, FixedStepClock, Grant,
    GrantedAbility, LifecycleEvent, LifecycleEventKind, LocalLifecycleEvent, MechanicsDriver,
    ObjectId, ObjectStore, RealtimeClock, RealtimeClockAccumulator, Registry, RegistryEntry, Tag,
    TagSet,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn mechanics_acceptance_registers_activates_ticks_and_expires_without_game_nouns() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct AbilityDefinition {
        key: &'static str,
        effect_key: &'static str,
        cooldown_units: CooldownUnits,
    }

    impl RegistryEntry for AbilityDefinition {
        fn key(&self) -> &str {
            self.key
        }
    }

    impl DefinitionRegistryEntry for AbilityDefinition {
        type Definition = Self;

        fn build_definition(&self) -> Self::Definition {
            *self
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct EffectDefinition {
        key: &'static str,
        duration_units: ClockUnits,
        payload: EffectPayload,
    }

    impl EffectDefinition {
        fn tags(self) -> TagSet<TestAtom> {
            TagSet::new([Tag::new([TestAtom::Category, TestAtom::Variant])])
        }
    }

    impl RegistryEntry for EffectDefinition {
        fn key(&self) -> &str {
            self.key
        }
    }

    impl DefinitionRegistryEntry for EffectDefinition {
        type Definition = Self;

        fn build_definition(&self) -> Self::Definition {
            *self
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct AbilityPayload {
        definition_key: &'static str,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct EffectPayload {
        amount: i32,
    }

    static ABILITY_DEFINITIONS: &[AbilityDefinition] = &[AbilityDefinition {
        key: "spark",
        effect_key: "charged",
        cooldown_units: 1000,
    }];
    static EFFECT_DEFINITIONS: &[EffectDefinition] = &[EffectDefinition {
        key: "charged",
        duration_units: 1000,
        payload: EffectPayload { amount: 7 },
    }];

    #[derive(Debug, Eq, PartialEq)]
    enum HookError {
        MissingAbilityDefinition,
        MissingEffectDefinition,
    }

    struct Runtime {
        target_id: ObjectId,
        effects: EffectPipeline<TagSet<TestAtom>, EffectPayload>,
        application_events: Vec<EffectLifecycleEvent<TagSet<TestAtom>, EffectPayload>>,
    }

    struct Hooks {
        abilities: Registry<'static, AbilityDefinition>,
        effects: Registry<'static, EffectDefinition>,
    }

    impl AbilityHooks<Runtime, TagSet<TestAtom>, (), AbilityPayload> for Hooks {
        type Error = HookError;

        fn cooldown_units(
            &mut self,
            _context: &mut Runtime,
            ability: &GrantedAbility<TagSet<TestAtom>, (), AbilityPayload>,
        ) -> Result<Option<CooldownUnits>, Self::Error> {
            let definition = self
                .abilities
                .definition(ability.payload.definition_key)
                .ok_or(HookError::MissingAbilityDefinition)?;
            Ok(Some(definition.cooldown_units))
        }
    }

    let mut objects = ObjectStore::new();
    let source = objects.create();
    let target = objects.create();
    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(Grant {
        owner_id: source,
        tags: TagSet::new([Tag::new([TestAtom::Ability, TestAtom::Burst])]),
        cost: None,
        cooldown_units: None,
        payload: AbilityPayload {
            definition_key: "spark",
        },
    });
    let mut runtime = Runtime {
        target_id: target,
        effects: EffectPipeline::new(),
        application_events: Vec::new(),
    };
    let mut hooks = Hooks {
        abilities: Registry::new(ABILITY_DEFINITIONS),
        effects: Registry::new(EFFECT_DEFINITIONS),
    };

    let activation_id = abilities
        .begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut runtime,
            &mut hooks,
        )
        .unwrap();
    let active = abilities
        .get_active_activation(activation_id)
        .unwrap()
        .clone();
    let ability_definition = hooks
        .abilities
        .definition(active.payload.definition_key)
        .ok_or(HookError::MissingAbilityDefinition)
        .unwrap();
    let effect_definition = hooks
        .effects
        .definition(ability_definition.effect_key)
        .ok_or(HookError::MissingEffectDefinition)
        .unwrap();
    runtime
        .effects
        .apply_with_events(
            &duration_effect_definition(effect_definition.key, effect_definition.duration_units),
            EffectApplicationInput {
                source_id: Some(active.owner_id),
                target_id: runtime.target_id,
                tags: effect_definition.tags(),
                payload: effect_definition.payload,
                decision: EffectApplicationDecision::Accept,
            },
            |event| runtime.application_events.push(event),
        )
        .unwrap();
    abilities
        .end_activation_with(activation_id, &mut runtime, &mut hooks)
        .unwrap();

    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(1000));
    assert_eq!(runtime.effects.count(), 1);
    let [
        EffectLifecycleEvent::ApplicationAccepted(accepted),
        EffectLifecycleEvent::ActiveCreated(created),
    ] = runtime.application_events.as_slice()
    else {
        panic!("activation should emit accepted and active-created events");
    };
    assert_eq!(accepted.source_id, Some(source));
    assert_eq!(accepted.target_id, target);
    assert_eq!(accepted.payload, EffectPayload { amount: 7 });
    assert_eq!(created.source_id, Some(source));
    assert_eq!(created.target_id, target);
    assert_eq!(created.payload, EffectPayload { amount: 7 });
    assert!(created.has_tag(&Tag::new([TestAtom::Category, TestAtom::Variant])));

    let ticked_events =
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, EffectPayload>>::new()
            .with_store(&mut abilities)
            .with_store(&mut runtime.effects)
            .tick(400);

    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(600));
    let [EffectLifecycleEvent::Advanced(advanced)] = ticked_events.as_slice() else {
        panic!("partial advancement should emit one advanced event");
    };
    assert_eq!(advanced.elapsed_units, 400);
    assert_eq!(advanced.previous_remaining_units, Some(1000));
    assert_eq!(advanced.effect.remaining_units, Some(600));

    let expired_events =
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, EffectPayload>>::new()
            .with_store(&mut abilities)
            .with_store(&mut runtime.effects)
            .tick(600);

    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    assert_eq!(runtime.effects.count(), 0);
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::Expired(expired),
    ] = expired_events.as_slice()
    else {
        panic!("final advancement should emit advanced and expired events");
    };
    assert_eq!(expiring_advance.elapsed_units, 600);
    assert_eq!(expired.source_id, Some(source));
    assert_eq!(expired.target_id, target);
    assert_eq!(expired.remaining_units, Some(0));
    assert_eq!(expired.payload, EffectPayload { amount: 7 });
    assert!(expired.has_tag(&Tag::new([TestAtom::Category, TestAtom::Variant])));
}

#[test]
fn lifecycle_events_emit_in_registration_order_through_mechanics_driver() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        First,
        Second,
    }

    let mut first = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut second = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    first
        .apply_with_events(
            &duration_effect_definition("first", 100),
            EffectApplicationInput {
                source_id: Some(ObjectId::new(1)),
                target_id: ObjectId::new(2),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::First,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();
    second
        .apply_with_events(
            &duration_effect_definition("second", 100),
            EffectApplicationInput {
                source_id: Some(ObjectId::new(3)),
                target_id: ObjectId::new(4),
                tags: TagSet::new([Tag::new([TestAtom::Group])]),
                payload: Payload::Second,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();

    let events = MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut first)
        .with_store(&mut second)
        .tick(40);

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
    effects
        .apply_with_events(
            &duration_effect_definition("timed", 100),
            EffectApplicationInput {
                source_id: None,
                target_id: ObjectId::new(1),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Timed,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();

    let events = MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick(0);

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
    effects
        .apply_with_events(
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
    MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick_with(100, |event| channel.publish(event).unwrap());

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
    effects
        .apply_with_events(
            &duration_effect_definition("tick", 10),
            EffectApplicationInput {
                source_id: None,
                target_id: ObjectId::new(1),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Tick,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();
    let mut events = MechanicsDriver::<LocalLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick(1);
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

#[test]
fn turn_based_clock_advances_cooldowns_and_effect_lifetimes_in_turns() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Shield,
    }

    let turn_clock = FixedStepClock::new(1);
    let source = ObjectId::new(1);
    let target = ObjectId::new(2);
    let mut abilities = AbilityStore::<TagSet<TestAtom>, (), Payload>::new();
    let ability_id = abilities.grant(Grant {
        owner_id: source,
        tags: TagSet::new([Tag::new([TestAtom::Ability])]),
        cost: None,
        cooldown_units: Some(turn_clock.units_for(2)),
        payload: Payload::Shield,
    });
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut hooks = NoopHooks;
    let mut context = ();

    abilities
        .begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
        )
        .unwrap();
    effects
        .apply_with_events(
            &FlexEffectDefinition {
                key: "turn_shield".to_owned(),
                kind: EffectKind::Periodic,
                duration: Some(EffectClockPolicy::from_clock(&turn_clock, 3)),
                period: Some(EffectClockPolicy::from_clock(&turn_clock, 1)),
                routing: EffectRouting::default(),
                payload_schema: (),
            },
            EffectApplicationInput {
                source_id: Some(source),
                target_id: target,
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Shield,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut abilities)
        .with_store(&mut effects)
        .tick_clock(&turn_clock, 1);

    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(1));
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(pulse),
    ] = events.as_slice()
    else {
        panic!("one turn should advance the effect and execute one period");
    };
    assert_eq!(advanced.elapsed_units, 1);
    assert_eq!(advanced.effect.remaining_units, Some(2));
    assert_eq!(pulse.elapsed_units, Some(1));

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut abilities)
        .with_store(&mut effects)
        .tick_clock(&turn_clock, 2);

    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("two more turns should execute remaining periods and expire");
    };
    assert_eq!(expiring_advance.elapsed_units, 2);
    assert_eq!(expired.remaining_units, Some(0));
    assert_eq!(effects.count(), 0);
}

#[test]
fn realtime_clock_lets_callers_choose_duration_to_unit_scale() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Pulse,
    }

    let realtime = RealtimeClock::new(1000);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    effects
        .apply_with_events(
            &FlexEffectDefinition {
                key: "realtime_pulse".to_owned(),
                kind: EffectKind::Periodic,
                duration: Some(EffectClockPolicy::from_clock(
                    &realtime,
                    Duration::from_secs(2),
                )),
                period: Some(EffectClockPolicy::from_clock(
                    &realtime,
                    Duration::from_millis(500),
                )),
                routing: EffectRouting::default(),
                payload_schema: (),
            },
            EffectApplicationInput {
                source_id: None,
                target_id: ObjectId::new(20),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Pulse,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick_clock(&realtime, Duration::from_millis(250));

    let [EffectLifecycleEvent::Advanced(advanced)] = events.as_slice() else {
        panic!("quarter-second tick should only advance when period is 500 ms");
    };
    assert_eq!(advanced.elapsed_units, 250);
    assert_eq!(advanced.effect.remaining_units, Some(1750));

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick_clock(&realtime, Duration::from_millis(250));
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(pulse),
    ] = events.as_slice()
    else {
        panic!("second quarter-second tick should complete one period");
    };
    assert_eq!(advanced.elapsed_units, 250);
    assert_eq!(pulse.elapsed_units, Some(500));

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick_clock(&realtime, Duration::from_millis(1500));
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("remaining realtime duration should emit final periods before expiration");
    };
    assert_eq!(expiring_advance.elapsed_units, 1500);
    assert_eq!(expired.remaining_units, Some(0));
}

#[test]
fn realtime_accumulator_matches_aggregate_elapsed_time_for_fractional_frames() {
    let realtime = RealtimeClock::new(1000);
    let frame = Duration::from_nanos(16_666_667);
    let mut accumulator = RealtimeClockAccumulator::from_clock(realtime);

    let accumulated_units: ClockUnits = (0..60).map(|_| accumulator.advance(frame)).sum();

    assert_eq!(
        accumulated_units,
        realtime.units_for(Duration::from_nanos(1_000_000_020))
    );
    assert_eq!(
        (0..60)
            .map(|_| realtime.units_for(frame))
            .sum::<ClockUnits>(),
        960
    );
}

#[test]
fn realtime_accumulator_advances_cooldowns_from_repeated_sub_unit_deltas() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Spark,
    }

    let mut abilities = AbilityStore::<TagSet<TestAtom>, (), Payload>::new();
    let ability_id = abilities.grant(Grant {
        owner_id: ObjectId::new(1),
        tags: TagSet::new([Tag::new([TestAtom::Ability])]),
        cost: None,
        cooldown_units: Some(1),
        payload: Payload::Spark,
    });
    let mut hooks = NoopHooks;
    let mut context = ();
    abilities
        .begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
        )
        .unwrap();

    let mut accumulator = RealtimeClockAccumulator::new(60);
    let frame = Duration::from_millis(16);

    let events = MechanicsDriver::<()>::new()
        .with_store(&mut abilities)
        .tick(accumulator.advance(frame));
    assert!(events.is_empty());
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(1));

    let events = MechanicsDriver::<()>::new()
        .with_store(&mut abilities)
        .tick(accumulator.advance(frame));
    assert!(events.is_empty());
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
}

#[test]
fn realtime_accumulator_expires_effect_duration_from_repeated_sub_unit_deltas() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Brief,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    effects
        .apply_with_events(
            &duration_effect_definition("brief", 1),
            EffectApplicationInput {
                source_id: None,
                target_id: ObjectId::new(1),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Brief,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();

    let mut accumulator = RealtimeClockAccumulator::new(60);
    let frame = Duration::from_millis(16);

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick(accumulator.advance(frame));
    assert!(events.is_empty());
    assert_eq!(effects.count(), 1);

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick(accumulator.advance(frame));
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("second sub-unit frame should advance and expire the effect");
    };
    assert_eq!(advanced.elapsed_units, 1);
    assert_eq!(advanced.effect.remaining_units, Some(0));
    assert_eq!(expired.remaining_units, Some(0));
    assert_eq!(effects.count(), 0);
}

#[test]
fn realtime_accumulator_executes_periodic_effects_from_repeated_sub_unit_deltas() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Pulse,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    effects
        .apply_with_events(
            &FlexEffectDefinition {
                key: "pulse".to_owned(),
                kind: EffectKind::Periodic,
                duration: Some(EffectClockPolicy::new(3)),
                period: Some(EffectClockPolicy::new(1)),
                routing: EffectRouting::default(),
                payload_schema: (),
            },
            EffectApplicationInput {
                source_id: None,
                target_id: ObjectId::new(1),
                tags: TagSet::new([Tag::new([TestAtom::Category])]),
                payload: Payload::Pulse,
                decision: EffectApplicationDecision::Accept,
            },
            |_| {},
        )
        .unwrap();

    let mut accumulator = RealtimeClockAccumulator::new(60);
    let frame = Duration::from_millis(16);

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick(accumulator.advance(frame));
    assert!(events.is_empty());

    let events = MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
        .with_store(&mut effects)
        .tick(accumulator.advance(frame));
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(pulse),
    ] = events.as_slice()
    else {
        panic!("second sub-unit frame should complete one periodic interval");
    };
    assert_eq!(advanced.elapsed_units, 1);
    assert_eq!(advanced.effect.remaining_units, Some(2));
    assert_eq!(pulse.elapsed_units, Some(1));
    assert_eq!(effects.count(), 1);
}

struct NoopHooks;

impl<Tags, Cost, Payload> AbilityHooks<(), Tags, Cost, Payload> for NoopHooks
where
    Tags: flexweave::TagCollection,
{
    type Error = ();
}

fn duration_effect_definition(key: &str, duration_units: ClockUnits) -> FlexEffectDefinition {
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

fn active_effect_advance_event<Payload>(
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
