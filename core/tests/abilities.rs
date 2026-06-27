mod common;

use common::TestAtom;
use flexweave::{
    AbilityActivationError, AbilityActivationId, AbilityActivationMode,
    AbilityActivationRejectionReason, AbilityCancelPolicy, AbilityCommitTiming, AbilityDefinition,
    AbilityDefinitionError, AbilityError, AbilityGrantError, AbilityHooks, AbilityId,
    AbilityLifecycleEvent, AbilityStore, EventChannel, EventChannelDefinition, EventRetention,
    Grant, GrantedAbility, INVALID_OBJECT_ID, LifecycleEvent, LifecycleEventKind, ObjectId,
    ObjectStore, Tag, TagSet, ability,
};

#[test]
fn val_core_012_ability_activation_runs_hooks_and_cooldown_gates_reactivation() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        CanActivate,
        Commit,
        Activate,
        End,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Cost {
        amount: u8,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        AddCounter(u8),
    }

    #[derive(Debug, Eq, PartialEq)]
    enum HookError {
        InsufficientResource,
    }

    #[derive(Debug)]
    struct TestContext {
        resource: u8,
        counter: u8,
        events: Vec<Event>,
    }

    struct Hooks;

    impl AbilityHooks<TestContext, TagSet<TestAtom>, Cost, Payload> for Hooks {
        type Error = HookError;

        fn can_activate(
            &mut self,
            context: &mut TestContext,
            ability: &GrantedAbility<TagSet<TestAtom>, Cost, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push(Event::CanActivate);
            let Some(cost) = ability.cost else {
                return Ok(());
            };
            if context.resource < cost.amount {
                return Err(HookError::InsufficientResource);
            }
            Ok(())
        }

        fn commit(
            &mut self,
            context: &mut TestContext,
            ability: &GrantedAbility<TagSet<TestAtom>, Cost, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push(Event::Commit);
            if let Some(cost) = ability.cost {
                context.resource -= cost.amount;
            }
            Ok(())
        }

        fn end(
            &mut self,
            context: &mut TestContext,
            _ability: &GrantedAbility<TagSet<TestAtom>, Cost, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push(Event::End);
            Ok(())
        }
    }

    let owner = ObjectId::new(42);
    let burst = Tag::new([TestAtom::Ability, TestAtom::Burst]);
    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(ability::Grant {
        owner_id: owner,
        tags: TagSet::new([burst.clone()]),
        cost: Some(Cost { amount: 3 }),
        cooldown_units: Some(1000),
        payload: Payload::AddCounter(5),
    });
    let mut context = TestContext {
        resource: 10,
        counter: 0,
        events: Vec::new(),
    };
    let mut hooks = Hooks;

    let activation_id = abilities
        .begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
        )
        .unwrap();
    let payload = abilities
        .get_active_activation(activation_id)
        .unwrap()
        .payload;
    context.events.push(Event::Activate);
    match payload {
        Payload::AddCounter(amount) => context.counter += amount,
    }
    abilities
        .end_activation_with(activation_id, &mut context, &mut hooks)
        .unwrap();

    assert_eq!(context.resource, 7);
    assert_eq!(context.counter, 5);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(1000));
    assert!(abilities.has_tag(owner, &burst));
    assert_eq!(
        context.events,
        vec![
            Event::CanActivate,
            Event::Commit,
            Event::Activate,
            Event::End
        ]
    );

    assert_eq!(
        abilities.begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
        ),
        Err(AbilityActivationError::Ability(
            AbilityError::AbilityOnCooldown
        ))
    );

    abilities.tick_cooldowns(999);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(1));

    abilities.tick_cooldowns(1);
    let activation_id = abilities
        .begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
        )
        .unwrap();
    let payload = abilities
        .get_active_activation(activation_id)
        .unwrap()
        .payload;
    context.events.push(Event::Activate);
    match payload {
        Payload::AddCounter(amount) => context.counter += amount,
    }
    abilities
        .end_activation_with(activation_id, &mut context, &mut hooks)
        .unwrap();

    assert_eq!(context.resource, 4);
    assert_eq!(context.counter, 10);
}

#[test]
fn val_core_012_failed_ability_hooks_do_not_apply_cooldown_or_later_hooks() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Cost {
        amount: u8,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    #[derive(Debug, Eq, PartialEq)]
    enum HookError {
        Rejected,
    }

    #[derive(Debug)]
    struct TestContext {
        events: Vec<&'static str>,
    }

    struct RejectCanActivate;

    impl AbilityHooks<TestContext, TagSet<TestAtom>, Cost, Payload> for RejectCanActivate {
        type Error = HookError;

        fn can_activate(
            &mut self,
            context: &mut TestContext,
            _ability: &GrantedAbility<TagSet<TestAtom>, Cost, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push("can_activate");
            Err(HookError::Rejected)
        }

        fn commit(
            &mut self,
            context: &mut TestContext,
            _ability: &GrantedAbility<TagSet<TestAtom>, Cost, Payload>,
        ) -> Result<(), Self::Error> {
            context.events.push("commit");
            Ok(())
        }
    }

    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(ability::Grant {
        owner_id: ObjectId::new(1),
        tags: TagSet::new([Tag::new([TestAtom::Ability])]),
        cost: Some(Cost { amount: 1 }),
        cooldown_units: Some(1000),
        payload: Payload,
    });
    let mut context = TestContext { events: Vec::new() };
    let mut hooks = RejectCanActivate;

    assert_eq!(
        abilities.begin_activation_with(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
        ),
        Err(AbilityActivationError::Hook(HookError::Rejected))
    );
    assert_eq!(context.events, vec!["can_activate"]);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
}

#[test]
fn checked_grant_rejects_invalid_or_missing_owner() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let mut objects = ObjectStore::new();
    let live_owner = objects.create();
    let tag = Tag::new([TestAtom::Ability]);
    let mut abilities = AbilityStore::<TagSet<TestAtom>, (), Payload>::new();

    assert_eq!(
        abilities.grant_checked(
            &objects,
            Grant::new(INVALID_OBJECT_ID, TagSet::new([tag.clone()]), Payload),
        ),
        Err(AbilityGrantError::InvalidOwner {
            owner_id: INVALID_OBJECT_ID,
        })
    );

    let missing_owner = ObjectId::new(9_999);
    assert_eq!(
        abilities.grant_checked(
            &objects,
            Grant::new(missing_owner, TagSet::new([tag.clone()]), Payload),
        ),
        Err(AbilityGrantError::InvalidOwner {
            owner_id: missing_owner,
        })
    );

    let ability_id = abilities
        .grant_checked(
            &objects,
            Grant::new(live_owner, TagSet::new([tag]), Payload),
        )
        .unwrap();
    assert_eq!(ability_id, AbilityId::new(1));
    assert_eq!(abilities.count(), 1);
}

#[test]
fn active_ability_begin_on_start_commits_and_remains_active() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(500));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();

    let activation_id = abilities
        .begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        )
        .unwrap();

    assert_eq!(activation_id, AbilityActivationId::new(1));
    assert_eq!(context.resource, 8);
    assert_eq!(context.events, vec!["can_activate", "cooldown", "commit"]);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(500));
    assert_eq!(abilities.active_activation_count(), 1);

    let active = abilities.get_active_activation(activation_id).unwrap();
    assert_eq!(active.ability_id, ability_id);
    assert_eq!(active.owner_id, ObjectId::new(9));
    assert_eq!(active.commit_timing, AbilityCommitTiming::OnStart);
    assert!(active.committed);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationStarted,
        ]
    );
}

#[test]
fn active_ability_end_on_end_commits_then_removes_state() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(700));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();

    let activation_id = abilities
        .begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::OnEnd,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        )
        .unwrap();

    assert_eq!(context.resource, 10);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    assert!(
        !abilities
            .get_active_activation(activation_id)
            .unwrap()
            .committed
    );

    let ended = abilities
        .end_activation_with_events(activation_id, &mut context, &mut hooks, |event| {
            events.push(event);
        })
        .unwrap()
        .unwrap();

    assert_eq!(context.resource, 8);
    assert_eq!(
        context.events,
        vec!["can_activate", "cooldown", "commit", "end"]
    );
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(700));
    assert_eq!(abilities.active_activation_count(), 0);
    assert!(ended.committed);
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
fn active_ability_cancel_removes_state_without_committing() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(700));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();

    let activation_id = abilities
        .begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::Manual,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        )
        .unwrap();
    let canceled = abilities
        .cancel_activation_with_events(activation_id, |event| events.push(event))
        .unwrap();

    assert_eq!(canceled.activation_id, activation_id);
    assert_eq!(context.resource, 10);
    assert_eq!(context.events, vec!["can_activate"]);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCanceled,
        ]
    );
}

#[test]
fn active_ability_rejection_leaves_no_active_state_or_cooldown() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(500));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: true };
    let mut events = Vec::new();

    assert_eq!(
        abilities.begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        ),
        Err(AbilityActivationError::Hook(ActiveHookError::Rejected))
    );

    assert_eq!(context.resource, 10);
    assert_eq!(context.events, vec!["can_activate"]);
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationRejected,
        ]
    );
    match &events[1] {
        AbilityLifecycleEvent::Rejected(rejection) => {
            assert_eq!(rejection.reason, AbilityActivationRejectionReason::Hook);
            assert!(rejection.attempt.is_some());
        }
        event => panic!("expected rejection event, got {event:?}"),
    }
}

#[test]
fn checked_activation_rejects_owner_mismatch_before_hooks() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(500));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();
    let expected_owner_id = ObjectId::new(10);

    assert_eq!(
        abilities.begin_activation_for_owner_with_events(
            expected_owner_id,
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |event| events.push(event),
        ),
        Err(AbilityActivationError::Ability(
            AbilityError::OwnerMismatch {
                expected_owner_id,
                actual_owner_id: ObjectId::new(9),
            }
        ))
    );

    assert_eq!(context.events, Vec::<&'static str>::new());
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationRejected,
        ]
    );
    match &events[1] {
        AbilityLifecycleEvent::Rejected(rejection) => {
            assert_eq!(
                rejection.reason,
                AbilityActivationRejectionReason::OwnerMismatch
            );
            assert_eq!(
                rejection.attempt.as_ref().map(|attempt| attempt.owner_id),
                Some(ObjectId::new(9)),
            );
        }
        event => panic!("expected owner mismatch rejection, got {event:?}"),
    }
}

#[test]
fn instant_activation_success_emits_lifecycle_and_clears_active_state() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(600));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();

    let ended = abilities
        .activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |context, active| {
                assert_eq!(active.ability_id, ability_id);
                assert_eq!(active.owner_id, ObjectId::new(9));
                assert!(active.committed);
                context.events.push("execute");
                Ok(())
            },
            |event| events.push(event),
        )
        .unwrap()
        .unwrap();

    assert_eq!(ended.activation_id, AbilityActivationId::new(1));
    assert!(ended.committed);
    assert_eq!(context.resource, 8);
    assert_eq!(
        context.events,
        vec!["can_activate", "cooldown", "commit", "execute", "end"]
    );
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(600));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationEnded,
        ]
    );
}

#[test]
fn instant_activation_executor_failure_cancels_and_clears_active_state() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(600));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();

    assert_eq!(
        abilities.activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |context, active| {
                assert_eq!(active.ability_id, ability_id);
                assert!(active.committed);
                context.events.push("execute");
                Err(ActiveHookError::Rejected)
            },
            |event| events.push(event),
        ),
        Err(AbilityActivationError::Hook(ActiveHookError::Rejected))
    );

    assert_eq!(context.resource, 8);
    assert_eq!(
        context.events,
        vec!["can_activate", "cooldown", "commit", "execute"]
    );
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(600));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCanceled,
        ]
    );
}

#[test]
fn instant_activation_end_hook_failure_cancels_and_clears_active_state() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(600));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = FailingActiveHooks {
        fail_commit: false,
        fail_end: true,
    };
    let mut events = Vec::new();

    assert_eq!(
        abilities.activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |context, active| {
                assert_eq!(active.ability_id, ability_id);
                assert!(active.committed);
                context.events.push("execute");
                Ok(())
            },
            |event| events.push(event),
        ),
        Err(AbilityActivationError::Hook(
            FailingActiveHookError::EndRejected
        ))
    );

    assert_eq!(context.resource, 8);
    assert_eq!(
        context.events,
        vec!["can_activate", "cooldown", "commit", "execute", "end"]
    );
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(600));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCanceled,
        ]
    );
}

#[test]
fn instant_activation_deferred_commit_failure_cancels_and_clears_active_state() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(300));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = FailingActiveHooks {
        fail_commit: true,
        fail_end: false,
    };
    let mut events = Vec::new();

    assert_eq!(
        abilities.activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnEnd,
            &mut context,
            &mut hooks,
            |context, active| {
                assert_eq!(active.ability_id, ability_id);
                assert!(!active.committed);
                context.events.push("execute");
                Ok(())
            },
            |event| events.push(event),
        ),
        Err(AbilityActivationError::Hook(
            FailingActiveHookError::CommitRejected
        ))
    );

    assert_eq!(context.resource, 10);
    assert_eq!(
        context.events,
        vec!["can_activate", "execute", "cooldown", "commit"]
    );
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(0));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCanceled,
        ]
    );
}

#[test]
fn instant_activation_cooldown_semantics_match_deferred_commit_end() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(300));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let mut events = Vec::new();

    abilities
        .activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnEnd,
            &mut context,
            &mut hooks,
            |context, active| {
                assert!(!active.committed);
                assert_eq!(active.commit_timing, AbilityCommitTiming::OnEnd);
                context.events.push("execute");
                Ok(())
            },
            |event| events.push(event),
        )
        .unwrap()
        .unwrap();

    assert_eq!(context.resource, 8);
    assert_eq!(
        context.events,
        vec!["can_activate", "execute", "cooldown", "commit", "end"]
    );
    assert_eq!(abilities.cooldown_remaining(ability_id), Ok(300));
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationStarted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationEnded,
        ]
    );

    let mut rejection_events = Vec::new();
    assert_eq!(
        abilities.activate_instant_with_events(
            ability_id,
            AbilityCommitTiming::OnEnd,
            &mut context,
            &mut hooks,
            |_, _| Ok(()),
            |event| rejection_events.push(event),
        ),
        Err(AbilityActivationError::Ability(
            AbilityError::AbilityOnCooldown
        ))
    );
    assert_eq!(abilities.active_activation_count(), 0);
    assert_eq!(
        lifecycle_kinds(&rejection_events),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationRejected,
        ]
    );
}

#[test]
fn grant_constructor_matches_literal_and_store_grants() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Cost {
        amount: u8,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        AddCounter(u8),
    }

    let owner = ObjectId::new(42);
    let burst = Tag::new([TestAtom::Ability, TestAtom::Burst]);
    let tags = TagSet::new([burst.clone()]);
    let grant = Grant::new(owner, tags.clone(), Payload::AddCounter(5))
        .with_cost(Cost { amount: 3 })
        .with_cooldown(1000);

    assert_eq!(
        grant,
        Grant {
            owner_id: owner,
            tags: tags.clone(),
            cost: Some(Cost { amount: 3 }),
            cooldown_units: Some(1000),
            payload: Payload::AddCounter(5),
        }
    );

    let mut abilities = AbilityStore::new();
    let ability_id = abilities.grant(grant);
    let ability = abilities.get(ability_id).unwrap();
    assert_eq!(ability.owner_id, owner);
    assert_eq!(ability.cost, Some(Cost { amount: 3 }));
    assert_eq!(ability.cooldown_units, Some(1000));
    assert_eq!(ability.payload, Payload::AddCounter(5));
    assert!(abilities.has_tag(owner, &burst));
}

#[test]
fn ability_ids_are_typed_value_objects_and_store_uses_them() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let ability_id = AbilityId::new(42);
    assert_eq!(ability_id.get(), 42);
    assert_eq!(AbilityId::from(42).get(), 42);
    assert_eq!(u64::from(ability_id), 42);
    assert_eq!(ability_id.to_string(), "42");

    let activation_id = AbilityActivationId::new(7);
    assert_eq!(activation_id.get(), 7);
    assert_eq!(AbilityActivationId::from(7).get(), 7);
    assert_eq!(u64::from(activation_id), 7);
    assert_eq!(activation_id.to_string(), "7");

    let owner = ObjectId::new(12);
    let tag = Tag::new([TestAtom::Ability, TestAtom::Burst]);
    let mut abilities = AbilityStore::<TagSet<TestAtom>, (), Payload>::new();

    let granted = abilities.grant(Grant::new(owner, TagSet::new([tag.clone()]), Payload));

    assert_eq!(granted, AbilityId::new(1));
    assert_eq!(granted.get(), 1);
    assert_eq!(abilities.get(granted).unwrap().id, granted);
    assert_eq!(abilities.ids_with_tag(owner, &tag), vec![granted]);
    assert_eq!(abilities.cooldown_remaining(granted), Ok(0));
    assert_eq!(abilities.is_ready(granted), Ok(true));
    abilities.set_cooldown_units(granted, Some(50)).unwrap();
    assert_eq!(abilities.get(granted).unwrap().cooldown_units, Some(50));
}

#[test]
fn ability_definition_constructors_match_literals_and_validate() {
    assert_eq!(
        AbilityDefinition::instant("instant", "payload/schema"),
        AbilityDefinition {
            key: "instant".to_owned(),
            activation_mode: AbilityActivationMode::Instant,
            commit_timing: AbilityCommitTiming::OnStart,
            cancel_policy: AbilityCancelPolicy::CannotCancel,
            tag_requirement_keys: Vec::new(),
            activation_tag_keys: Vec::new(),
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema: "payload/schema",
        }
    );
    AbilityDefinition::instant("instant", ())
        .validate()
        .unwrap();

    assert_eq!(
        AbilityDefinition::active("active", "payload/schema"),
        AbilityDefinition {
            key: "active".to_owned(),
            activation_mode: AbilityActivationMode::Active,
            commit_timing: AbilityCommitTiming::OnStart,
            cancel_policy: AbilityCancelPolicy::CanCancel,
            tag_requirement_keys: Vec::new(),
            activation_tag_keys: Vec::new(),
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema: "payload/schema",
        }
    );
    AbilityDefinition::active("active", ()).validate().unwrap();
}

#[test]
fn ability_definition_builders_populate_authoring_metadata() {
    let definition = AbilityDefinition::active("channel", "test/payload")
        .with_commit_timing(AbilityCommitTiming::OnEnd)
        .with_cancel_policy(AbilityCancelPolicy::CanCancel)
        .with_tag_requirement_keys(["ability"])
        .with_activation_tag_keys(["channeling"])
        .with_lifecycle_channels(["abilities/lifecycle"]);

    assert_eq!(
        definition,
        AbilityDefinition {
            key: "channel".to_owned(),
            activation_mode: AbilityActivationMode::Active,
            commit_timing: AbilityCommitTiming::OnEnd,
            cancel_policy: AbilityCancelPolicy::CanCancel,
            tag_requirement_keys: vec!["ability".to_owned()],
            activation_tag_keys: vec!["channeling".to_owned()],
            emits_lifecycle: true,
            emitted_channel_keys: vec!["abilities/lifecycle".to_owned()],
            payload_schema: "test/payload",
        }
    );
    definition.validate().unwrap();
    definition
        .validate_channels(&["abilities/lifecycle"])
        .unwrap();
}

#[test]
fn ability_definitions_validate_authoring_contracts_before_grant() {
    let valid = active_ability_definition(
        "channel",
        AbilityActivationMode::Active,
        AbilityCommitTiming::OnEnd,
        AbilityCancelPolicy::CanCancel,
    );
    valid.validate().unwrap();
    valid.validate_channels(&["abilities/lifecycle"]).unwrap();

    let missing_channel = AbilityDefinition {
        emitted_channel_keys: Vec::new(),
        ..valid.clone()
    };
    assert_eq!(
        missing_channel.validate().unwrap_err(),
        AbilityDefinitionError::MissingEmittedChannelKey {
            key: "channel".to_owned(),
        }
    );

    assert_eq!(
        active_ability_definition(
            "unknown_channel",
            AbilityActivationMode::Active,
            AbilityCommitTiming::OnStart,
            AbilityCancelPolicy::CanCancel,
        )
        .validate_channels(&["other/channel"])
        .unwrap_err(),
        AbilityDefinitionError::UnknownEmittedChannelKey {
            key: "unknown_channel".to_owned(),
            channel_key: "abilities/lifecycle".to_owned(),
        }
    );

    assert_eq!(
        active_ability_definition(
            "instant_cancel",
            AbilityActivationMode::Instant,
            AbilityCommitTiming::OnStart,
            AbilityCancelPolicy::CanCancel,
        )
        .validate()
        .unwrap_err(),
        AbilityDefinitionError::InstantCannotBeCanceled {
            key: "instant_cancel".to_owned(),
        }
    );

    assert_eq!(
        active_ability_definition(
            "instant_on_end",
            AbilityActivationMode::Instant,
            AbilityCommitTiming::OnEnd,
            AbilityCancelPolicy::CannotCancel,
        )
        .validate()
        .unwrap_err(),
        AbilityDefinitionError::InstantCannotCommitOnEnd {
            key: "instant_on_end".to_owned(),
        }
    );

    let malformed_tags = AbilityDefinition {
        tag_requirement_keys: vec![String::new()],
        ..valid.clone()
    };
    assert_eq!(
        malformed_tags.validate().unwrap_err(),
        AbilityDefinitionError::EmptyTagRequirementKey {
            key: "channel".to_owned(),
        }
    );

    let mut abilities = ActiveAbilityStore::new();
    let invalid = active_ability_definition(
        "invalid_grant",
        AbilityActivationMode::Instant,
        AbilityCommitTiming::OnStart,
        AbilityCancelPolicy::CanCancel,
    );
    assert_eq!(
        abilities
            .grant_with_definition(&invalid, active_grant(Some(100)))
            .unwrap_err(),
        AbilityDefinitionError::InstantCannotBeCanceled {
            key: "invalid_grant".to_owned(),
        }
    );
    assert_eq!(abilities.count(), 0);
}

#[test]
fn ability_lifecycle_events_route_through_named_channels() {
    let mut abilities = ActiveAbilityStore::new();
    let ability_id = grant_active_ability(&mut abilities, Some(250));
    let mut context = ActiveContext {
        resource: 10,
        events: Vec::new(),
    };
    let mut hooks = ActiveHooks { reject: false };
    let channel_definition = EventChannelDefinition::new(
        "abilities/lifecycle",
        [
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationStarted,
        ],
    )
    .unwrap();
    let mut channel = EventChannel::with_retention(channel_definition, EventRetention::Retain);

    abilities
        .begin_activation_with_events(
            ability_id,
            AbilityCommitTiming::OnStart,
            &mut context,
            &mut hooks,
            |event| channel.publish(event).unwrap(),
        )
        .unwrap();

    let retained = channel.drain_retained();
    assert_eq!(
        lifecycle_kinds(&retained),
        vec![
            LifecycleEventKind::AbilityActivationAttempted,
            LifecycleEventKind::AbilityActivationCommitted,
            LifecycleEventKind::AbilityActivationStarted,
        ]
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActiveCost {
    amount: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActivePayload {
    Channel,
}

#[derive(Debug)]
struct ActiveContext {
    resource: u8,
    events: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActiveHookError {
    Rejected,
}

struct ActiveHooks {
    reject: bool,
}

type ActiveAbilityStore = AbilityStore<TagSet<TestAtom>, ActiveCost, ActivePayload>;
type ActiveEvent = AbilityLifecycleEvent<TagSet<TestAtom>, ActiveCost, ActivePayload>;

impl AbilityHooks<ActiveContext, TagSet<TestAtom>, ActiveCost, ActivePayload> for ActiveHooks {
    type Error = ActiveHookError;

    fn can_activate(
        &mut self,
        context: &mut ActiveContext,
        ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<(), Self::Error> {
        context.events.push("can_activate");
        if self.reject {
            return Err(ActiveHookError::Rejected);
        }
        if let Some(cost) = ability.cost
            && context.resource < cost.amount
        {
            return Err(ActiveHookError::Rejected);
        }
        Ok(())
    }

    fn cooldown_units(
        &mut self,
        context: &mut ActiveContext,
        ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<Option<u64>, Self::Error> {
        context.events.push("cooldown");
        Ok(ability.cooldown_units)
    }

    fn commit(
        &mut self,
        context: &mut ActiveContext,
        ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<(), Self::Error> {
        context.events.push("commit");
        if let Some(cost) = ability.cost {
            context.resource -= cost.amount;
        }
        Ok(())
    }

    fn end(
        &mut self,
        context: &mut ActiveContext,
        _ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<(), Self::Error> {
        context.events.push("end");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailingActiveHookError {
    CommitRejected,
    EndRejected,
}

struct FailingActiveHooks {
    fail_commit: bool,
    fail_end: bool,
}

impl AbilityHooks<ActiveContext, TagSet<TestAtom>, ActiveCost, ActivePayload>
    for FailingActiveHooks
{
    type Error = FailingActiveHookError;

    fn can_activate(
        &mut self,
        context: &mut ActiveContext,
        ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<(), Self::Error> {
        context.events.push("can_activate");
        if let Some(cost) = ability.cost
            && context.resource < cost.amount
        {
            return Err(FailingActiveHookError::CommitRejected);
        }
        Ok(())
    }

    fn cooldown_units(
        &mut self,
        context: &mut ActiveContext,
        ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<Option<u64>, Self::Error> {
        context.events.push("cooldown");
        Ok(ability.cooldown_units)
    }

    fn commit(
        &mut self,
        context: &mut ActiveContext,
        ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<(), Self::Error> {
        context.events.push("commit");
        if self.fail_commit {
            return Err(FailingActiveHookError::CommitRejected);
        }
        if let Some(cost) = ability.cost {
            context.resource -= cost.amount;
        }
        Ok(())
    }

    fn end(
        &mut self,
        context: &mut ActiveContext,
        _ability: &GrantedAbility<TagSet<TestAtom>, ActiveCost, ActivePayload>,
    ) -> Result<(), Self::Error> {
        context.events.push("end");
        if self.fail_end {
            return Err(FailingActiveHookError::EndRejected);
        }
        Ok(())
    }
}

fn grant_active_ability(
    abilities: &mut ActiveAbilityStore,
    cooldown_units: Option<u64>,
) -> AbilityId {
    abilities.grant(active_grant(cooldown_units))
}

fn active_grant(
    cooldown_units: Option<u64>,
) -> ability::Grant<TagSet<TestAtom>, ActiveCost, ActivePayload> {
    ability::Grant {
        owner_id: ObjectId::new(9),
        tags: TagSet::new([active_tag()]),
        cost: Some(ActiveCost { amount: 2 }),
        cooldown_units,
        payload: ActivePayload::Channel,
    }
}

fn active_tag() -> Tag<TestAtom> {
    Tag::new([TestAtom::Ability, TestAtom::Burst])
}

fn active_ability_definition(
    key: &str,
    activation_mode: AbilityActivationMode,
    commit_timing: AbilityCommitTiming,
    cancel_policy: AbilityCancelPolicy,
) -> AbilityDefinition<&'static str> {
    match activation_mode {
        AbilityActivationMode::Instant => AbilityDefinition::instant(key, "test/payload"),
        AbilityActivationMode::Active => AbilityDefinition::active(key, "test/payload"),
    }
    .with_commit_timing(commit_timing)
    .with_cancel_policy(cancel_policy)
    .with_tag_requirement_keys(["ability"])
    .with_activation_tag_keys(["channeling"])
    .with_lifecycle_channels(["abilities/lifecycle"])
}

fn lifecycle_kinds(events: &[ActiveEvent]) -> Vec<LifecycleEventKind> {
    events
        .iter()
        .map(|event| event.lifecycle_event_kind())
        .collect()
}
