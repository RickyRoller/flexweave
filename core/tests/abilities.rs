mod common;

use common::TestAtom;
use flexweave::{
    AbilityActivation, AbilityActivationDecision, AbilityActivationError, AbilityActivationGate,
    AbilityActivationRejectionReason, AbilityBeginError, AbilityCancelOutcome, AbilityCommit,
    AbilityCommitAction, AbilityCommitActionExecutor, AbilityCommitError, AbilityCommitOutcome,
    AbilityDefinition, AbilityDefinitionRegistryError, AbilityDefinitions, AbilityEndError,
    AbilityEndOutcome, AbilityGateExecutor, AbilityGrantError, AbilityId, AbilityLifecycleEvent,
    AbilityLifecycleEventView, AbilityRollbackError, AbilityRollbackOutcome, AbilityStore,
    ActiveAbilityView, EffectActionExecutor, EffectApplicationInput, EffectApply, EffectDefinition,
    EffectExecutionView, EffectPipeline, EffectTick, EventChannel, EventChannelDefinition,
    EventRetention, Grant, INVALID_OBJECT_ID, LifecycleEvent, LifecycleEventKind,
    NoAbilityActivationExecutor, NoAbilityCommitExecutor, ObjectId, ObjectStore, Tag, TagSet,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn begin_without_gate_emits_attempted_and_started() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut events = Vec::new();

    let mut executor =
        NoAbilityActivationExecutor::new().with_owned_events(|event| events.push(event));
    let activation_id = AbilityActivation::new(ability_id)
        .run_with_executor(&mut abilities, &(), &mut executor)
        .unwrap();

    assert_eq!(activation_id.get(), 1);
    assert_eq!(abilities.active_activation_count(), 1);
    assert!(
        !abilities
            .get_active_activation(activation_id)
            .unwrap()
            .committed
    );
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
        ]
    );
}

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
    enum CommitError {
        Effect,
    }

    struct Runtime {
        mana: u8,
        effects: EffectPipeline<TagSet<TestAtom>, EffectPayload>,
        events: Vec<&'static str>,
    }

    struct Gate {
        cost: u8,
    }

    impl AbilityActivationGate<Runtime, TagSet<TestAtom>, AbilityPayload> for Gate {
        type Error = ();
        type BlockReason = BlockReason;

        fn can_activate(
            &mut self,
            context: &Runtime,
            attempt: flexweave::AbilityActivationAttemptView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
            if context.effects.has_tag(attempt.owner_id, &cooldown_tag()) {
                return Ok(AbilityActivationDecision::Block(BlockReason::Cooldown));
            }
            if context.mana < self.cost {
                return Ok(AbilityActivationDecision::Block(BlockReason::Mana));
            }
            Ok(AbilityActivationDecision::Allow)
        }
    }

    struct Commit {
        cost: u8,
        cooldown_units: u64,
    }

    impl AbilityCommitAction<Runtime, TagSet<TestAtom>, AbilityPayload> for Commit {
        type Error = CommitError;

        fn apply_commit(
            &mut self,
            context: &mut Runtime,
            active: ActiveAbilityView<'_, TagSet<TestAtom>, AbilityPayload>,
        ) -> Result<(), Self::Error> {
            context.events.push("commit");
            assert!(!active.committed);

            let mut charge_mana =
                |mana: &mut u8,
                 execution: EffectExecutionView<'_, TagSet<TestAtom>, EffectPayload>|
                 -> Result<(), CommitError> {
                    let EffectPayload::ManaCost(amount) = execution.payload else {
                        panic!("mana action should only execute mana cost effects");
                    };
                    *mana -= amount;
                    Ok(())
                };
            let mut executor = EffectActionExecutor::new(&mut charge_mana);
            EffectApply::definition(
                &EffectDefinition::instant("mana_cost", ()),
                EffectApplicationInput::accept(
                    Some(active.source_id()),
                    active.owner_id,
                    TagSet::new([Tag::new([TestAtom::Category])]),
                    EffectPayload::ManaCost(self.cost),
                ),
            )
            .run_with_executor(&mut context.effects, &mut context.mana, &mut executor)
            .map_err(|_| CommitError::Effect)?;

            EffectApply::definition(
                &EffectDefinition::duration("cooldown", self.cooldown_units, ()),
                EffectApplicationInput::accept(
                    Some(active.source_id()),
                    active.owner_id,
                    TagSet::new([cooldown_tag()]),
                    EffectPayload::Cooldown,
                ),
            )
            .run(&mut context.effects)
            .map_err(|_| CommitError::Effect)?;
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
    let mut gate = Gate { cost: 3 };
    let mut commit = Commit {
        cost: 3,
        cooldown_units: 1000,
    };
    let mut events = Vec::new();

    let activation_id = {
        let mut executor =
            AbilityGateExecutor::new(&mut gate).with_owned_events(|event| events.push(event));
        AbilityActivation::new(ability_id)
            .run_with_executor(&mut abilities, &runtime, &mut executor)
            .unwrap()
    };
    assert_eq!(
        {
            let mut executor = AbilityCommitActionExecutor::new(&mut commit)
                .with_owned_events(|event| events.push(event));
            AbilityCommit::new(activation_id)
                .run_with_executor(&mut abilities, &mut runtime, &mut executor)
                .unwrap()
        },
        AbilityCommitOutcome::Committed
    );
    let ended = abilities.end_activation(activation_id).unwrap();

    assert!(matches!(ended, AbilityEndOutcome::Ended(_)));
    assert_eq!(runtime.mana, 7);
    assert!(runtime.effects.has_tag(owner, &cooldown_tag()));
    assert_eq!(runtime.events, vec!["commit"]);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
        ]
    );
    let [_, _, AbilityLifecycleEvent::Committed(committed)] = events.as_slice() else {
        panic!("commit should emit committed active state");
    };
    assert_eq!(committed.activation_id, activation_id);
    assert!(committed.committed);

    let mut blocked_events = Vec::new();
    assert_eq!(
        {
            let mut executor = AbilityGateExecutor::new(&mut gate)
                .with_owned_events(|event| blocked_events.push(event));
            AbilityActivation::new(ability_id).run_with_executor(
                &mut abilities,
                &runtime,
                &mut executor,
            )
        },
        Err(AbilityActivationError::Activation(
            AbilityBeginError::Blocked(BlockReason::Cooldown)
        ))
    );
    let [_, AbilityLifecycleEvent::Rejected(rejected)] = blocked_events.as_slice() else {
        panic!("blocked activation should emit attempted and rejected");
    };
    assert_eq!(rejected.reason, AbilityActivationRejectionReason::Blocked);

    EffectTick::new(1000).run(&mut runtime.effects);
    assert!(!runtime.effects.has_tag(owner, &cooldown_tag()));

    let activation_id = {
        let mut executor = AbilityGateExecutor::new(&mut gate);
        AbilityActivation::new(ability_id)
            .run_with_executor(&mut abilities, &runtime, &mut executor)
            .unwrap()
    };
    {
        let mut executor = AbilityCommitActionExecutor::new(&mut commit);
        AbilityCommit::new(activation_id)
            .run_with_executor(&mut abilities, &mut runtime, &mut executor)
            .unwrap();
    }
    abilities.end_activation(activation_id).unwrap();
    assert_eq!(runtime.mana, 4);
}

#[test]
fn begin_gate_error_emits_rejection_without_active_state() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum GateError {
        Unavailable,
    }

    struct Gate;

    impl AbilityActivationGate<(), TagSet<TestAtom>, Payload> for Gate {
        type Error = GateError;
        type BlockReason = ();

        fn can_activate(
            &mut self,
            _context: &(),
            _attempt: flexweave::AbilityActivationAttemptView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
            Err(GateError::Unavailable)
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let mut gate = Gate;
    let mut events = Vec::new();

    assert_eq!(
        abilities.begin_activation_with_gate_events(ability_id, &(), &mut gate, |event| {
            events.push(event)
        }),
        Err(AbilityBeginError::Gate(GateError::Unavailable))
    );

    assert_eq!(abilities.active_activation_count(), 0);
    let [_, AbilityLifecycleEvent::Rejected(rejected)] = events.as_slice() else {
        panic!("gate error should emit attempted and rejected");
    };
    assert_eq!(rejected.reason, AbilityActivationRejectionReason::Gate);
}

#[test]
fn commit_action_failure_rolls_back_active_state() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum CommitError {
        Failed,
    }

    struct Commit;

    impl AbilityCommitAction<(), TagSet<TestAtom>, Payload> for Commit {
        type Error = CommitError;

        fn apply_commit(
            &mut self,
            _context: &mut (),
            _active: ActiveAbilityView<'_, TagSet<TestAtom>, Payload>,
        ) -> Result<(), Self::Error> {
            Err(CommitError::Failed)
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let activation_id = abilities.begin_activation(ability_id).unwrap();
    let mut commit = Commit;
    let mut events = Vec::new();

    assert_eq!(
        abilities.commit_activation_with_action_events(
            activation_id,
            &mut (),
            &mut commit,
            |event| events.push(event),
        ),
        Err(AbilityCommitError::Action(CommitError::Failed))
    );

    assert_eq!(abilities.active_activation_count(), 0);
    let [AbilityLifecycleEvent::RolledBack(rolled_back)] = events.as_slice() else {
        panic!("failed commit action should emit a rollback fact");
    };
    assert_eq!(rolled_back.activation_id, activation_id);
    assert!(!rolled_back.committed);
}

#[test]
fn explicit_commit_end_cancel_and_rollback_have_separate_lifecycle_contracts() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let mut abilities = AbilityStore::new();
    let first = grant_payload(&mut abilities, Payload);
    let second = grant_payload(&mut abilities, Payload);
    let third = grant_payload(&mut abilities, Payload);
    let fourth = grant_payload(&mut abilities, Payload);
    let mut events = Vec::new();

    let activation_id = abilities.begin_activation(first).unwrap();
    assert_eq!(
        abilities.end_activation(activation_id),
        Err(AbilityEndError::UncommittedActivation)
    );
    assert!(abilities.get_active_activation(activation_id).is_some());
    assert_eq!(
        abilities
            .commit_activation_with_events(activation_id, |event| events.push(event))
            .unwrap(),
        AbilityCommitOutcome::Committed
    );
    assert_eq!(
        abilities.commit_activation(activation_id).unwrap(),
        AbilityCommitOutcome::AlreadyCommitted
    );
    let ended = abilities
        .end_activation_with_events(activation_id, |event| events.push(event))
        .unwrap();
    assert!(matches!(ended, AbilityEndOutcome::Ended(_)));

    let uncommitted_cancel_id = abilities.begin_activation(second).unwrap();
    let committed_cancel_id = abilities.begin_activation(third).unwrap();
    abilities.commit_activation(committed_cancel_id).unwrap();
    let canceled_uncommitted =
        abilities.cancel_activation_with_events(uncommitted_cancel_id, |event| events.push(event));
    let canceled_committed =
        abilities.cancel_activation_with_events(committed_cancel_id, |event| events.push(event));
    assert!(matches!(
        canceled_uncommitted,
        AbilityCancelOutcome::Canceled(_)
    ));
    assert!(matches!(
        canceled_committed,
        AbilityCancelOutcome::Canceled(_)
    ));

    let rollback_id = abilities.begin_activation(fourth).unwrap();
    let rolled_back = abilities
        .rollback_activation_with_events(rollback_id, |event| events.push(event))
        .unwrap();
    let AbilityRollbackOutcome::RolledBack(rolled_back) = rolled_back;
    assert_eq!(rolled_back.activation_id, rollback_id);

    assert_eq!(
        abilities.end_activation(rollback_id),
        Err(AbilityEndError::MissingActivation)
    );
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationEnded,
            LifecycleEventKind::AbilityActivationCanceled,
            LifecycleEventKind::AbilityActivationCanceled,
            LifecycleEventKind::AbilityActivationRolledBack,
        ]
    );
}

#[test]
fn rollback_rejects_committed_activation_and_leaves_state() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
    let activation_id = abilities.begin_activation(ability_id).unwrap();

    abilities.commit_activation(activation_id).unwrap();

    assert_eq!(
        abilities.rollback_activation(activation_id),
        Err(AbilityRollbackError::AlreadyCommitted)
    );
    assert!(
        abilities
            .get_active_activation(activation_id)
            .unwrap()
            .committed
    );
}

#[test]
fn checked_grant_and_owner_activation_reject_invalid_object_references_before_gate() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

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
    let mut events = Vec::new();

    assert_eq!(
        {
            let mut executor =
                NoAbilityActivationExecutor::new().with_owned_events(|event| events.push(event));
            AbilityActivation::new(ability_id)
                .for_owner(other_owner)
                .run_with_executor(&mut abilities, &(), &mut executor)
        },
        Err(AbilityActivationError::Activation(
            AbilityBeginError::Ability(flexweave::AbilityError::OwnerMismatch {
                expected_owner_id: other_owner,
                actual_owner_id: live_owner,
            })
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
fn registered_definitions_validate_key_without_orchestration_metadata() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let definitions = AbilityDefinitions::new([ability_definition("channel")]).unwrap();
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
    let mut events = Vec::new();

    let activation_id = {
        let mut executor =
            NoAbilityActivationExecutor::new().with_owned_events(|event| events.push(event));
        AbilityActivation::registered(&definitions, ability_id)
            .run_with_executor(&mut abilities, &(), &mut executor)
            .unwrap()
    };

    let active = abilities.get_active_activation(activation_id).unwrap();
    assert_eq!(active.definition_key.as_deref(), Some("channel"));
    assert!(!active.committed);
    assert_eq!(
        abilities.end_activation(activation_id),
        Err(AbilityEndError::UncommittedActivation)
    );
    {
        let mut context = ();
        let mut executor =
            NoAbilityCommitExecutor::new().with_owned_events(|event| events.push(event));
        AbilityCommit::new(activation_id)
            .run_with_executor(&mut abilities, &mut context, &mut executor)
            .unwrap();
    }
    abilities
        .end_activation_with_events(activation_id, |event| events.push(event))
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
fn ability_definition_metadata_supports_lifecycle_channels() {
    assert_eq!(
        AbilityDefinition::new("ability", "payload/schema"),
        AbilityDefinition {
            key: "ability".to_owned(),
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema: "payload/schema",
        }
    );

    let definition = ability_definition("channel");
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

    let mut abilities = AbilityStore::new();
    let ability_id = grant_payload(&mut abilities, Payload);
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

    let activation_id = abilities
        .begin_activation_with_events(ability_id, |event| channel.publish(event).unwrap())
        .unwrap();
    abilities
        .commit_activation_with_events(activation_id, |event| channel.publish(event).unwrap())
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
fn borrowed_ability_events_do_not_clone_payload_for_publication() {
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

    let clone_count = Rc::new(Cell::new(0));
    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(Grant::new(
        ObjectId::new(9),
        TagSet::new([Tag::new([TestAtom::Ability])]),
        Payload {
            clone_count: Rc::clone(&clone_count),
        },
    ));
    let mut kinds = Vec::new();

    let activation_id = abilities
        .begin_activation_with_borrowed_events(ability_id, |event| {
            match &event {
                AbilityLifecycleEventView::Attempted(attempt) => {
                    assert_eq!(attempt.payload.clone_count.get(), 0);
                }
                AbilityLifecycleEventView::Started(active) => {
                    assert_eq!(active.payload.clone_count.get(), 1);
                }
                _ => panic!("unexpected borrowed ability event"),
            }
            kinds.push(event.lifecycle_event_kind());
        })
        .unwrap();
    abilities
        .commit_activation_with_borrowed_events(activation_id, |event| {
            let AbilityLifecycleEventView::Committed(active) = event else {
                panic!("commit should emit committed active state");
            };
            assert_eq!(active.payload.clone_count.get(), 1);
        })
        .unwrap();
    abilities
        .end_activation_with_borrowed_events(activation_id, |event| {
            let AbilityLifecycleEventView::Ended(active) = event else {
                panic!("end should emit an ended event");
            };
            assert_eq!(active.payload.clone_count.get(), 1);
        })
        .unwrap();

    assert_eq!(clone_count.get(), 1);
    assert_eq!(
        kinds,
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
        ]
    );
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

fn ability_definition(key: &str) -> AbilityDefinition<&'static str> {
    AbilityDefinition::new(key, "test/payload").with_lifecycle_channels(["abilities/lifecycle"])
}

fn lifecycle_kinds<Payload>(
    events: &[AbilityLifecycleEvent<TagSet<TestAtom>, Payload>],
) -> Vec<LifecycleEventKind> {
    events
        .iter()
        .map(LifecycleEvent::lifecycle_event_kind)
        .collect()
}
