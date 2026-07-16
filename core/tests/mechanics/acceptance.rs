use super::support::*;

#[test]
fn mechanics_acceptance_registers_activates_ticks_and_expires_without_game_nouns() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct AbilityDefinition {
        key: &'static str,
        effect_key: &'static str,
        cooldown_units: ClockUnits,
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
        cooldowns: EffectPipeline<TagSet<TestAtom>, EffectPayload>,
        application_events: Vec<EffectLifecycleEvent<TagSet<TestAtom>, EffectPayload>>,
    }

    struct Commit {
        abilities: Registry<'static, AbilityDefinition>,
    }

    impl AbilityCommitAction<Runtime, TagSet<TestAtom>, AbilityPayload> for Commit {
        type Error = HookError;

        fn apply_commit(
            &mut self,
            context: &mut Runtime,
            active: ActiveAbilityView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<(), Self::Error> {
            let definition = self
                .abilities
                .definition(active.payload.definition_key)
                .ok_or(HookError::MissingAbilityDefinition)?;
            apply_effect(
                &mut context.cooldowns,
                &duration_effect_definition("spark_cooldown", definition.cooldown_units),
                EffectApplicationInput {
                    source_id: Some(active.source_id()),
                    target_id: active.owner_id,
                    tags: TagSet::new([cooldown_tag()]),
                    payload: EffectPayload { amount: 0 },
                    decision: EffectApplicationDecision::Accept,
                },
            )
            .map_err(|_| HookError::MissingEffectDefinition)?;
            Ok(())
        }
    }

    let mut objects = ObjectStore::new();
    let source = objects.create();
    let target = objects.create();
    let mut abilities = AbilityStore::new();
    let ability_id = AbilityGrant::new(Grant::new(
        source,
        TagSet::new([Tag::new([TestAtom::Ability, TestAtom::Burst])]),
        AbilityPayload {
            definition_key: "spark",
        },
    ))
    .run(&mut abilities)
    .unwrap();
    let mut runtime = Runtime {
        target_id: target,
        effects: EffectPipeline::new(),
        cooldowns: EffectPipeline::new(),
        application_events: Vec::new(),
    };
    let mut commit = Commit {
        abilities: Registry::new(ABILITY_DEFINITIONS),
    };
    let effects = Registry::new(EFFECT_DEFINITIONS);

    let activation_id = AbilityActivation::new(ability_id)
        .run(&mut abilities)
        .unwrap();
    let mut executor = AbilityCommitActionExecutor::new(&mut commit);
    AbilityCommit::new(activation_id)
        .run_with_executor(&mut abilities, &mut runtime, &mut executor)
        .unwrap();
    let active = abilities
        .get_active_activation(activation_id)
        .unwrap()
        .clone();
    let ability_definition = commit
        .abilities
        .definition(active.payload.definition_key)
        .ok_or(HookError::MissingAbilityDefinition)
        .unwrap();
    let effect_definition = effects
        .definition(ability_definition.effect_key)
        .ok_or(HookError::MissingEffectDefinition)
        .unwrap();
    apply_effect_with_events(
        &mut runtime.effects,
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
    AbilityEnd::new(activation_id).run(&mut abilities).unwrap();

    assert_eq!(runtime.effects.count(), 1);
    assert_eq!(runtime.cooldowns.count(), 1);
    assert!(runtime.cooldowns.has_tag(source, &cooldown_tag()));
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

    let ticked_events = MechanicsTick::new(400).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, EffectPayload>>::new()
            .with_store(&mut runtime.cooldowns)
            .with_store(&mut runtime.effects),
    );

    let [
        EffectLifecycleEvent::Advanced(cooldown_advanced),
        EffectLifecycleEvent::Advanced(advanced),
    ] = ticked_events.as_slice()
    else {
        panic!("partial advancement should emit cooldown and effect advanced events");
    };
    assert_eq!(cooldown_advanced.effect.remaining_units, Some(600));
    assert_eq!(advanced.elapsed_units, 400);
    assert_eq!(advanced.previous_remaining_units, Some(1000));
    assert_eq!(advanced.effect.remaining_units, Some(600));

    let expired_events = MechanicsTick::new(600).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, EffectPayload>>::new()
            .with_store(&mut runtime.cooldowns)
            .with_store(&mut runtime.effects),
    );

    assert_eq!(runtime.cooldowns.count(), 0);
    assert_eq!(runtime.effects.count(), 0);
    let [
        EffectLifecycleEvent::Advanced(cooldown_expiring_advance),
        EffectLifecycleEvent::Expired(cooldown_expired),
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::Expired(expired),
    ] = expired_events.as_slice()
    else {
        panic!("final advancement should emit cooldown and effect expiration events");
    };
    assert_eq!(cooldown_expiring_advance.elapsed_units, 600);
    assert_eq!(cooldown_expired.target_id, source);
    assert_eq!(expiring_advance.elapsed_units, 600);
    assert_eq!(expired.source_id, Some(source));
    assert_eq!(expired.target_id, target);
    assert_eq!(expired.remaining_units, Some(0));
    assert_eq!(expired.payload, EffectPayload { amount: 7 });
    assert!(expired.has_tag(&Tag::new([TestAtom::Category, TestAtom::Variant])));
}
