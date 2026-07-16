use super::support::*;

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

    let active_id = {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::registered(
            &definitions,
            "buff",
            application(Payload::Buff, EffectApplicationDecision::Accept),
        )
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
    }
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
        EffectApply::registered(
            &definitions,
            "missing",
            application(Payload::Buff, EffectApplicationDecision::Accept),
        )
        .run(&mut pipeline)
        .unwrap_err(),
        EffectApplyError::RegisteredDefinition(EffectDefinitionRegistryError::MissingDefinition {
            key: "missing".to_owned(),
        })
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
