mod common;

use common::{TestAtom, block_on};
use flexweave::{
    AbilityActivationDecision, AbilityActivationError, AbilityActivationRejectionReason,
    AbilityCancelOutcome, AbilityCommitOutcome, AbilityCommitTiming, AbilityDefinition,
    AbilityDefinitionRegistryError, AbilityDefinitions, AbilityEndOutcome, AbilityGrantError,
    AbilityHookPhase, AbilityHooks, AbilityId, AbilityLifecycleEvent, AbilityLifecycleEventView,
    AbilityStore, ActiveAbilityView, EffectApplicationDecision, EffectApplicationInput,
    EffectDefinition, EffectLifecycleEvent, EffectPipeline, EventChannel, EventChannelDefinition,
    EventRetention, Grant, INVALID_OBJECT_ID, LifecycleEvent, LifecycleEventKind, ObjectId,
    ObjectStore, Tag, TagSet,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn ability_commit_can_trigger_cost_and_cooldown_effects_then_block_on_tags() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AbilityPayload {
        Burst,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum EffectPayload {
        ManaCost(u8),
        Cooldown,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum BlockReason {
        Cooldown,
        Mana,
    }

    #[derive(Debug, Eq, PartialEq)]
    enum HookError {
        Effect,
    }

    struct Runtime {
        mana: u8,
        effects: EffectPipeline<TagSet<TestAtom>, EffectPayload>,
        events: Vec<&'static str>,
    }

    struct Hooks {
        cost: u8,
        cooldown_units: u64,
    }

    impl AbilityHooks<Runtime, TagSet<TestAtom>, AbilityPayload> for Hooks {
        type Error = HookError;
        type BlockReason = BlockReason;

        async fn can_activate(
            &mut self,
            context: &mut Runtime,
            attempt: flexweave::AbilityActivationAttemptView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
            context.events.push("can_activate");
            if context.effects.has_tag(attempt.owner_id, &cooldown_tag()) {
                return Ok(AbilityActivationDecision::Block(BlockReason::Cooldown));
            }
            if context.mana < self.cost {
                return Ok(AbilityActivationDecision::Block(BlockReason::Mana));
            }
            Ok(AbilityActivationDecision::Allow)
        }

        async fn on_start(
            &mut self,
            context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<(), Self::Error> {
            context.events.push("start");
            Ok(())
        }

        async fn on_commit(
            &mut self,
            context: &mut Runtime,
            active: ActiveAbilityView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<(), Self::Error> {
            context.events.push("commit");
            context
                .effects
                .apply_with_events(
                    &EffectDefinition::instant("mana_cost", ()),
                    EffectApplicationInput {
                        source_id: Some(active.source_id()),
                        target_id: active.owner_id,
                        tags: TagSet::new([Tag::new([TestAtom::Category])]),
                        payload: EffectPayload::ManaCost(self.cost),
                        decision: EffectApplicationDecision::Accept,
                    },
                    |event| {
                        if let EffectLifecycleEvent::Executed(executed) = event
                            && let EffectPayload::ManaCost(amount) = executed.payload
                        {
                            context.mana -= amount;
                        }
                    },
                )
                .map_err(|_| HookError::Effect)?;
            context
                .effects
                .apply(
                    &EffectDefinition::duration("cooldown", self.cooldown_units, ()),
                    EffectApplicationInput {
                        source_id: Some(active.source_id()),
                        target_id: active.owner_id,
                        tags: TagSet::new([cooldown_tag()]),
                        payload: EffectPayload::Cooldown,
                        decision: EffectApplicationDecision::Accept,
                    },
                )
                .map_err(|_| HookError::Effect)?;
            Ok(())
        }

        async fn on_end(
            &mut self,
            context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<(), Self::Error> {
            context.events.push("end");
            Ok(())
        }
    }

    let owner = ObjectId::new(42);
    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(Grant::new(
        owner,
        TagSet::new([Tag::new([TestAtom::Ability, TestAtom::Burst])]),
        AbilityPayload::Burst,
    ));
    let mut runtime = Runtime {
        mana: 10,
        effects: EffectPipeline::new(),
        events: Vec::new(),
    };
    let mut hooks = Hooks {
        cost: 3,
        cooldown_units: 1000,
    };
    let mut events = Vec::new();

    let activation_id = block_on(abilities.begin_activation_with_events(
        ability_id,
        AbilityCommitTiming::OnStart,
        &mut runtime,
        &mut hooks,
        |event| events.push(event),
    ))
    .unwrap();
    let ended =
        block_on(abilities.end_activation_with(activation_id, &mut runtime, &mut hooks)).unwrap();

    assert!(matches!(ended, AbilityEndOutcome::Ended(_)));
    assert_eq!(runtime.mana, 7);
    assert!(runtime.effects.has_tag(owner, &cooldown_tag()));
    assert_eq!(
        runtime.events,
        vec!["can_activate", "start", "commit", "end"]
    );
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
        ]
    );

    let mut blocked_events = Vec::new();
    assert_eq!(
        block_on(abilities.begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut runtime,
            &mut hooks,
            |event| blocked_events.push(event),
        )),
        Err(AbilityActivationError::Blocked(BlockReason::Cooldown))
    );
    let [_, AbilityLifecycleEvent::Rejected(rejected)] = blocked_events.as_slice() else {
        panic!("blocked activation should emit attempted and rejected");
    };
    assert_eq!(rejected.reason, AbilityActivationRejectionReason::Blocked);

    runtime.effects.tick(1000);
    assert!(!runtime.effects.has_tag(owner, &cooldown_tag()));

    let activation_id = block_on(abilities.begin_activation_with(
        ability_id,
        AbilityCommitTiming::OnStart,
        &mut runtime,
        &mut hooks,
    ))
    .unwrap();
    block_on(abilities.end_activation_with(activation_id, &mut runtime, &mut hooks)).unwrap();
    assert_eq!(runtime.mana, 4);
}

#[test]
fn manual_commit_and_cancel_are_separate_lifecycle_commands() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    struct Runtime {
        events: Vec<&'static str>,
    }

    struct Hooks;

    impl AbilityHooks<Runtime, TagSet<TestAtom>, Payload> for Hooks {
        type Error = ();
        type BlockReason = &'static str;

        async fn on_commit(
            &mut self,
            context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push("commit");
            Ok(())
        }

        async fn on_cancel(
            &mut self,
            context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push("cancel");
            Ok(())
        }
    }

    let mut abilities = AbilityStore::new();
    let first = grant_payload(&mut abilities, Payload);
    let second = grant_payload(&mut abilities, Payload);
    let mut runtime = Runtime { events: Vec::new() };
    let mut hooks = Hooks;
    let mut events = Vec::new();

    let activation_id = block_on(abilities.begin_activation_with_events(
        first,
        AbilityCommitTiming::Manual,
        &mut runtime,
        &mut hooks,
        |event| events.push(event),
    ))
    .unwrap();
    assert!(
        !abilities
            .get_active_activation(activation_id)
            .unwrap()
            .committed
    );
    assert_eq!(
        block_on(abilities.commit_activation_with_events(
            activation_id,
            &mut runtime,
            &mut hooks,
            |event| events.push(event),
        ))
        .unwrap(),
        AbilityCommitOutcome::Committed
    );
    assert_eq!(
        block_on(abilities.commit_activation_with(activation_id, &mut runtime, &mut hooks,))
            .unwrap(),
        AbilityCommitOutcome::AlreadyCommitted
    );
    let ended =
        block_on(abilities.end_activation_with(activation_id, &mut runtime, &mut hooks)).unwrap();
    assert!(matches!(ended, AbilityEndOutcome::Ended(_)));

    let cancel_activation_id = block_on(abilities.begin_activation_with(
        second,
        AbilityCommitTiming::Manual,
        &mut runtime,
        &mut hooks,
    ))
    .unwrap();
    let canceled = block_on(abilities.cancel_activation_with_events(
        cancel_activation_id,
        &mut runtime,
        &mut hooks,
        |event| events.push(event),
    ))
    .unwrap();
    let AbilityCancelOutcome::Canceled(canceled) = canceled else {
        panic!("active activation should cancel");
    };

    assert_eq!(canceled.activation_id, cancel_activation_id);
    assert_eq!(runtime.events, vec!["commit", "cancel"]);
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationCanceled,
        ]
    );
}

#[test]
fn checked_grant_and_owner_activation_reject_invalid_object_references_before_hooks() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    struct Hooks;

    impl AbilityHooks<(), TagSet<TestAtom>, Payload> for Hooks {
        type Error = ();
        type BlockReason = ();
    }

    let mut objects = ObjectStore::new();
    let live_owner = objects.create();
    let other_owner = objects.create();
    let tag = Tag::new([TestAtom::Ability]);
    let mut abilities = AbilityStore::<TagSet<TestAtom>, Payload>::new();

    assert_eq!(
        abilities.grant_checked(
            &objects,
            Grant::new(INVALID_OBJECT_ID, TagSet::new([tag.clone()]), Payload),
        ),
        Err(AbilityGrantError::InvalidOwner {
            owner_id: INVALID_OBJECT_ID,
        })
    );

    let ability_id = abilities
        .grant_checked(
            &objects,
            Grant::new(live_owner, TagSet::new([tag]), Payload),
        )
        .unwrap();
    let mut hooks = Hooks;
    let mut context = ();
    let mut events = Vec::new();

    assert_eq!(
        block_on(abilities.begin_activation_for_owner_with_events(
            other_owner,
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        )),
        Err(AbilityActivationError::Ability(
            flexweave::AbilityError::OwnerMismatch {
                expected_owner_id: other_owner,
                actual_owner_id: live_owner,
            }
        ))
    );
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationRejected,
        ]
    );
}

#[test]
fn registered_definitions_provide_activation_timing_without_cost_or_cooldown_state() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    struct Hooks;

    impl AbilityHooks<(), TagSet<TestAtom>, Payload> for Hooks {
        type Error = ();
        type BlockReason = ();
    }

    let definitions =
        AbilityDefinitions::new([ability_definition("channel", AbilityCommitTiming::OnEnd)])
            .unwrap();
    let mut abilities = AbilityStore::new();
    let ability_id = abilities
        .grant_registered(
            &definitions,
            "channel",
            Grant::new(
                ObjectId::new(9),
                TagSet::new([Tag::new([TestAtom::Ability])]),
                Payload,
            ),
        )
        .unwrap();
    let mut hooks = Hooks;
    let mut context = ();
    let mut events = Vec::new();

    let activation_id = block_on(abilities.begin_registered_activation_with_events(
        &definitions,
        ability_id,
        &mut context,
        &mut hooks,
        |event| events.push(event),
    ))
    .unwrap();

    let active = abilities.get_active_activation(activation_id).unwrap();
    assert_eq!(active.definition_key.as_deref(), Some("channel"));
    assert_eq!(active.commit_timing, AbilityCommitTiming::OnEnd);
    assert!(!active.committed);

    block_on(abilities.end_activation_with_events(
        activation_id,
        &mut context,
        &mut hooks,
        |event| events.push(event),
    ))
    .unwrap();

    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationEnded,
        ]
    );
}

#[test]
fn ability_definition_metadata_supports_async_lifecycle_timing() {
    assert_eq!(
        AbilityDefinition::new("ability", "payload/schema")
            .with_commit_timing(AbilityCommitTiming::OnEnd),
        AbilityDefinition {
            key: "ability".to_owned(),
            commit_timing: AbilityCommitTiming::OnEnd,
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema: "payload/schema",
        }
    );

    let definition = ability_definition("channel", AbilityCommitTiming::Manual);
    definition.validate().unwrap();
    definition
        .validate_channels(&["abilities/lifecycle"])
        .unwrap();

    let definitions = AbilityDefinitions::new([definition.clone()]).unwrap();
    assert_eq!(definitions.definitions()[0].key, "channel");
    assert_eq!(
        definitions.require("missing").unwrap_err(),
        AbilityDefinitionRegistryError::MissingDefinition {
            key: "missing".to_owned(),
        }
    );
    assert_eq!(
        AbilityDefinitions::new([definition.clone(), definition]).unwrap_err(),
        AbilityDefinitionRegistryError::DuplicateKey {
            key: "channel".to_owned(),
        }
    );
}

#[test]
fn caller_publishes_ability_lifecycle_events_to_named_channels() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    struct Hooks;

    impl AbilityHooks<(), TagSet<TestAtom>, Payload> for Hooks {
        type Error = ();
        type BlockReason = ();
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut hooks = Hooks;
    let mut context = ();
    let channel_definition = EventChannelDefinition::new(
        "abilities/lifecycle",
        [
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
        ],
    )
    .unwrap();
    let mut channel = EventChannel::with_retention(channel_definition, EventRetention::Retain);

    block_on(abilities.begin_activation_with_events(
        ability_id,
        AbilityCommitTiming::OnStart,
        &mut context,
        &mut hooks,
        |event| channel.publish(event).unwrap(),
    ))
    .unwrap();

    let retained = channel.drain_retained();
    assert_eq!(
        lifecycle_kinds(&retained),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
        ]
    );
}

#[test]
fn start_hook_failure_rolls_back_active_activation_without_cancel_hook() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HookError {
        Start,
        Cancel,
    }

    struct Runtime {
        cancel_calls: usize,
    }

    struct Hooks;

    impl AbilityHooks<Runtime, TagSet<TestAtom>, Payload> for Hooks {
        type Error = HookError;
        type BlockReason = ();

        async fn on_start(
            &mut self,
            _context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            Err(HookError::Start)
        }

        async fn on_cancel(
            &mut self,
            context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            context.cancel_calls += 1;
            Err(HookError::Cancel)
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut runtime = Runtime { cancel_calls: 0 };
    let mut hooks = Hooks;
    let mut events = Vec::new();

    assert_eq!(
        block_on(abilities.begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::Manual,
            &mut runtime,
            &mut hooks,
            |event| events.push(event),
        )),
        Err(AbilityActivationError::Hook {
            phase: AbilityHookPhase::Start,
            error: HookError::Start,
        })
    );

    assert_eq!(runtime.cancel_calls, 0);
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationRolledBack,
        ]
    );
    let [_, _, AbilityLifecycleEvent::RolledBack(rolled_back)] = events.as_slice() else {
        panic!("start failure should emit a rollback fact");
    };
    assert_eq!(rolled_back.ability_id, ability_id);
}

#[test]
fn borrowed_ability_attempt_event_does_not_clone_payload_for_publication() {
    #[derive(Debug)]
    struct Payload {
        clone_count: Rc<Cell<usize>>,
    }

    impl Clone for Payload {
        fn clone(&self) -> Self {
            self.clone_count.set(self.clone_count.get() + 1);
            Self {
                clone_count: Rc::clone(&self.clone_count),
            }
        }
    }

    struct Hooks;

    impl AbilityHooks<(), TagSet<TestAtom>, Payload> for Hooks {
        type Error = ();
        type BlockReason = ();
    }

    let clone_count = Rc::new(Cell::new(0));
    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(Grant::new(
        ObjectId::new(9),
        TagSet::new([Tag::new([TestAtom::Ability])]),
        Payload {
            clone_count: Rc::clone(&clone_count),
        },
    ));
    let mut hooks = Hooks;
    let mut context = ();
    let mut kinds = Vec::new();

    let activation_id = block_on(abilities.begin_activation_with_borrowed_events(
        ability_id,
        AbilityCommitTiming::OnStart,
        &mut context,
        &mut hooks,
        |event| {
            match &event {
                AbilityLifecycleEventView::Attempted(attempt) => {
                    assert_eq!(attempt.payload.clone_count.get(), 0);
                }
                AbilityLifecycleEventView::Started(active) => {
                    assert_eq!(active.payload.clone_count.get(), 1);
                }
                AbilityLifecycleEventView::Committed(commit) => {
                    assert_eq!(commit.attempt.payload.clone_count.get(), 3);
                }
                _ => panic!("unexpected borrowed ability event"),
            }
            kinds.push(event.lifecycle_event_kind());
        },
    ))
    .unwrap();

    assert_eq!(clone_count.get(), 3);
    assert_eq!(
        kinds,
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
        ]
    );

    block_on(abilities.end_activation_with_borrowed_events(
        activation_id,
        &mut context,
        &mut hooks,
        |event| {
            let AbilityLifecycleEventView::Ended(active) = event else {
                panic!("end should emit an ended event");
            };
            assert_eq!(active.payload.clone_count.get(), 4);
        },
    ))
    .unwrap();

    assert_eq!(clone_count.get(), 4);
}

#[test]
fn instant_activation_rollback_preserves_execute_error_when_cancel_hook_fails() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HookError {
        Execute,
        Cancel,
    }

    struct Runtime {
        cancel_calls: usize,
    }

    struct Hooks;

    impl AbilityHooks<Runtime, TagSet<TestAtom>, Payload> for Hooks {
        type Error = HookError;
        type BlockReason = ();

        async fn on_cancel(
            &mut self,
            context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            context.cancel_calls += 1;
            Err(HookError::Cancel)
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut runtime = Runtime { cancel_calls: 0 };
    let mut hooks = Hooks;
    let mut events = Vec::new();

    assert_eq!(
        block_on(abilities.activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::Manual,
            &mut runtime,
            &mut hooks,
            |_context, _active| Err(HookError::Execute),
            |event| events.push(event),
        )),
        Err(AbilityActivationError::Hook {
            phase: AbilityHookPhase::ExecuteInstant,
            error: HookError::Execute,
        })
    );

    assert_eq!(runtime.cancel_calls, 0);
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationRolledBack,
        ]
    );
    let [_, _, AbilityLifecycleEvent::RolledBack(rolled_back)] = events.as_slice() else {
        panic!("instant execute failure should emit a rollback fact");
    };
    assert_eq!(rolled_back.ability_id, ability_id);
}

#[test]
fn on_start_auto_commit_failure_rolls_back_begin_activation() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HookError {
        Commit,
    }

    struct Hooks;

    impl AbilityHooks<(), TagSet<TestAtom>, Payload> for Hooks {
        type Error = HookError;
        type BlockReason = ();

        async fn on_commit(
            &mut self,
            _context: &mut (),
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            Err(HookError::Commit)
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut hooks = Hooks;
    let mut context = ();
    let mut events = Vec::new();

    assert_eq!(
        block_on(abilities.begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        )),
        Err(AbilityActivationError::Hook {
            phase: AbilityHookPhase::Commit,
            error: HookError::Commit,
        })
    );

    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationRolledBack,
        ]
    );
    let [_, _, AbilityLifecycleEvent::RolledBack(rolled_back)] = events.as_slice() else {
        panic!("auto-commit failure should emit a rollback fact");
    };
    assert_eq!(rolled_back.ability_id, ability_id);
}

#[test]
fn on_start_auto_commit_failure_rolls_back_instant_activation_before_execute() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HookError {
        Commit,
        Execute,
    }

    struct Runtime {
        execute_calls: usize,
    }

    struct Hooks;

    impl AbilityHooks<Runtime, TagSet<TestAtom>, Payload> for Hooks {
        type Error = HookError;
        type BlockReason = ();

        async fn on_commit(
            &mut self,
            _context: &mut Runtime,
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            Err(HookError::Commit)
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut runtime = Runtime { execute_calls: 0 };
    let mut hooks = Hooks;
    let mut events = Vec::new();

    assert_eq!(
        block_on(abilities.activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut runtime,
            &mut hooks,
            |context, _active| {
                context.execute_calls += 1;
                Err(HookError::Execute)
            },
            |event| events.push(event),
        )),
        Err(AbilityActivationError::Hook {
            phase: AbilityHookPhase::Commit,
            error: HookError::Commit,
        })
    );

    assert_eq!(runtime.execute_calls, 0);
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationRolledBack,
        ]
    );
    let [_, _, AbilityLifecycleEvent::RolledBack(rolled_back)] = events.as_slice() else {
        panic!("instant auto-commit failure should emit a rollback fact");
    };
    assert_eq!(rolled_back.ability_id, ability_id);
}

fn grant_payload<Payload>(
    abilities: &mut AbilityStore<TagSet<TestAtom>, Payload>,
    payload: Payload,
) -> AbilityId {
    abilities.grant(Grant::new(
        ObjectId::new(9),
        TagSet::new([Tag::new([TestAtom::Ability, TestAtom::Burst])]),
        payload,
    ))
}

fn cooldown_tag() -> Tag<TestAtom> {
    Tag::new([TestAtom::Ability, TestAtom::Variant])
}

fn ability_definition(
    key: &str,
    commit_timing: AbilityCommitTiming,
) -> AbilityDefinition<&'static str> {
    AbilityDefinition::new(key, "test/payload")
        .with_commit_timing(commit_timing)
        .with_lifecycle_channels(["abilities/lifecycle"])
}

fn lifecycle_kinds<Payload>(
    events: &[AbilityLifecycleEvent<TagSet<TestAtom>, Payload>],
) -> Vec<LifecycleEventKind> {
    events
        .iter()
        .map(LifecycleEvent::lifecycle_event_kind)
        .collect()
}
