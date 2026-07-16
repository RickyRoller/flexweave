use super::support::{EffectApplyOutcomeTestExt, application, effect_definition};
use crate::common::TestAtom;
use flexweave::{
    ActiveEffectId, EffectApplicationDecision, EffectApplicationInput, EffectApply,
    EffectClockPolicy, EffectKind, EffectLifecycleEvent, EffectPipeline, EffectRemove, EffectTick,
    NoEffectExecutor, ObjectId, OwnedEffectLifecycleEvents, Tag, TagSet,
};

#[test]
fn effect_pipeline_creates_advances_expires_and_visits_by_target() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Increase,
        More,
    }

    let source = ObjectId::new(7);
    let target = ObjectId::new(42);
    let other_target = ObjectId::new(99);
    let enhancement = Tag::new([TestAtom::Category, TestAtom::Variant]);
    let more = Tag::new([TestAtom::Category, TestAtom::Family]);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    let first = EffectApply::definition(
        &effect_definition(
            "short",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 1000 }),
            None,
        ),
        EffectApplicationInput {
            source_id: Some(source),
            target_id: target,
            tags: TagSet::new([enhancement.clone()]),
            payload: Payload::Increase,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    let second = EffectApply::definition(
        &effect_definition(
            "long",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 2000 }),
            None,
        ),
        EffectApplicationInput {
            source_id: None,
            target_id: target,
            tags: TagSet::new([more.clone()]),
            payload: Payload::More,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    let third = EffectApply::definition(
        &effect_definition(
            "other",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 1000 }),
            None,
        ),
        EffectApplicationInput {
            source_id: None,
            target_id: other_target,
            tags: TagSet::new([enhancement.clone()]),
            payload: Payload::Increase,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");

    assert_eq!(
        (first, second, third),
        (
            ActiveEffectId::new(1),
            ActiveEffectId::new(2),
            ActiveEffectId::new(3),
        )
    );
    assert_eq!(effects.count(), 3);
    assert!(effects.has_tag(target, &enhancement));
    assert!(!effects.has_tag(target, &Tag::new([TestAtom::Group])));

    let mut visited = Vec::new();
    effects.visit_target(target, |effect| {
        visited.push((effect.id, effect.remaining_units))
    });
    assert_eq!(
        visited,
        vec![
            (ActiveEffectId::new(1), Some(1000)),
            (ActiveEffectId::new(2), Some(2000)),
        ]
    );

    let mut events = Vec::new();
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(999)
            .run_with_executor(&mut effects, &mut context, &mut executor)
            .unwrap();
    }
    assert_eq!(events.len(), 3);
    assert_eq!(
        effects.get(first).map(|effect| effect.remaining_units),
        Some(Some(1))
    );
    assert_eq!(
        effects.get(second).map(|effect| effect.remaining_units),
        Some(Some(1001))
    );

    events.clear();
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(1)
            .run_with_executor(&mut effects, &mut context, &mut executor)
            .unwrap();
    }
    assert!(matches!(events[0], EffectLifecycleEvent::Advanced(_)));
    assert!(matches!(events[1], EffectLifecycleEvent::Expired(_)));
    assert!(matches!(events[2], EffectLifecycleEvent::Advanced(_)));
    assert!(matches!(events[3], EffectLifecycleEvent::Advanced(_)));
    assert!(matches!(events[4], EffectLifecycleEvent::Expired(_)));
    assert_eq!(effects.count(), 1);
    assert_eq!(
        effects.get(second).map(|effect| effect.remaining_units),
        Some(Some(1000))
    );
    assert!(!effects.has_tag(target, &enhancement));
    assert!(effects.has_tag(target, &more));
}

#[test]
fn effect_pipeline_removes_effects_with_distinct_lifecycle_fact() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let effect_id = EffectApply::definition(
        &effect_definition(
            "buff",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 1000 }),
            None,
        ),
        EffectApplicationInput {
            source_id: None,
            target_id: ObjectId::new(1),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Buff,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    let mut events = Vec::new();

    let removed = {
        let mut sink = OwnedEffectLifecycleEvents::new(|event| events.push(event));
        EffectRemove::new(effect_id)
            .run_with_sink(&mut effects, &mut sink)
            .unwrap()
    };

    assert_eq!(removed.id, effect_id);
    assert_eq!(effects.count(), 0);
    let [EffectLifecycleEvent::Removed(removed_event)] = events.as_slice() else {
        panic!("manual removal should emit a removed event");
    };
    assert_eq!(removed_event.id, effect_id);
}

#[test]
fn effect_indexes_survive_removal_expiration_and_keep_application_order() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        First,
        Second,
        Third,
        Fourth,
    }

    let target = ObjectId::new(42);
    let other_target = ObjectId::new(77);
    let enhancement = Tag::new([TestAtom::Category, TestAtom::Variant]);
    let family = Tag::new([TestAtom::Category, TestAtom::Family]);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();

    let first = EffectApply::definition(
        &effect_definition(
            "first",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 100 }),
            None,
        ),
        EffectApplicationInput::accept(
            None,
            target,
            TagSet::new([enhancement.clone()]),
            Payload::First,
        ),
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    let second = EffectApply::definition(
        &effect_definition(
            "second",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 200 }),
            None,
        ),
        EffectApplicationInput::accept(
            None,
            target,
            TagSet::new([family.clone()]),
            Payload::Second,
        ),
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    let third = EffectApply::definition(
        &effect_definition(
            "third",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 300 }),
            None,
        ),
        EffectApplicationInput::accept(
            None,
            other_target,
            TagSet::new([enhancement.clone()]),
            Payload::Third,
        ),
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    let fourth = EffectApply::definition(
        &effect_definition(
            "fourth",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 400 }),
            None,
        ),
        EffectApplicationInput::accept(
            None,
            target,
            TagSet::new([enhancement.clone()]),
            Payload::Fourth,
        ),
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");

    let mut target_order = Vec::new();
    effects.visit_target(target, |effect| target_order.push(effect.id));
    assert_eq!(target_order, vec![first, second, fourth]);

    let removed = EffectRemove::new(second).run(&mut effects).unwrap();
    assert_eq!(removed.id, second);
    assert!(effects.get(second).is_none());
    assert_eq!(effects.get(third).unwrap().id, third);
    assert_eq!(effects.get(fourth).unwrap().id, fourth);

    target_order.clear();
    effects.visit_target(target, |effect| target_order.push(effect.id));
    assert_eq!(target_order, vec![first, fourth]);
    assert!(effects.has_tag(target, &enhancement));
    assert!(!effects.has_tag(target, &family));

    EffectTick::new(100).run(&mut effects);
    assert!(effects.get(first).is_none());
    assert_eq!(effects.get(third).unwrap().id, third);
    assert_eq!(effects.get(fourth).unwrap().id, fourth);

    target_order.clear();
    effects.visit_target(target, |effect| target_order.push(effect.id));
    assert_eq!(target_order, vec![fourth]);
}

#[test]
fn active_effect_ids_are_typed_value_objects_and_pipeline_uses_them() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let id = ActiveEffectId::new(24);
    assert_eq!(id.get(), 24);
    assert_eq!(ActiveEffectId::from(24).get(), 24);
    assert_eq!(u64::from(id), 24);
    assert_eq!(id.to_string(), "24");

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let effect_id = EffectApply::definition(
        &effect_definition(
            "typed",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 1000 }),
            None,
        ),
        EffectApplicationInput::accept(
            None,
            ObjectId::new(5),
            TagSet::new([Tag::new([TestAtom::Category])]),
            Payload::Buff,
        ),
    )
    .run(&mut effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");

    assert_eq!(effect_id, ActiveEffectId::new(1));
    assert_eq!(effect_id.get(), 1);
    assert_eq!(effects.get(effect_id).unwrap().id, effect_id);
    let removed = EffectRemove::new(effect_id).run(&mut effects).unwrap();
    assert_eq!(removed.id, effect_id);

    let mut default_effects = EffectPipeline::<TagSet<TestAtom>, Payload>::default();
    let default_effect_id = EffectApply::definition(
        &effect_definition(
            "default-typed",
            EffectKind::Duration,
            Some(EffectClockPolicy { units: 1000 }),
            None,
        ),
        EffectApplicationInput::accept(
            None,
            ObjectId::new(6),
            TagSet::new([Tag::new([TestAtom::Category])]),
            Payload::Buff,
        ),
    )
    .run(&mut default_effects)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");

    assert_eq!(default_effect_id, ActiveEffectId::new(1));
    assert_ne!(default_effect_id, ActiveEffectId::INVALID);
}

#[test]
fn duration_effects_advance_expire_and_remove_in_lifecycle_order() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Buff,
    }

    let definition = effect_definition(
        "buff",
        EffectKind::Duration,
        Some(EffectClockPolicy { units: 100 }),
        None,
    );
    let mut pipeline = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let mut events = Vec::new();
    let active_id = {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectApply::definition(
            &definition,
            application(Payload::Buff, EffectApplicationDecision::Accept),
        )
        .run_with_executor(&mut pipeline, &mut context, &mut executor)
    }
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    events.clear();

    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(40)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }
    let [EffectLifecycleEvent::Advanced(advanced)] = events.as_slice() else {
        panic!("partial duration tick should emit one advance fact");
    };
    assert_eq!(advanced.elapsed_units, 40);
    assert_eq!(advanced.previous_remaining_units, Some(100));
    assert_eq!(advanced.effect.remaining_units, Some(60));
    events.clear();

    let removed = {
        let mut sink = OwnedEffectLifecycleEvents::new(|event| events.push(event));
        EffectRemove::new(active_id)
            .run_with_sink(&mut pipeline, &mut sink)
            .unwrap()
    };
    assert_eq!(removed.id, active_id);
    let [EffectLifecycleEvent::Removed(removed_event)] = events.as_slice() else {
        panic!("manual removal should emit removed, not expired");
    };
    assert_eq!(removed_event.id, active_id);
    assert_eq!(pipeline.count(), 0);
    events.clear();

    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(100)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }
    assert!(events.is_empty());

    let expiring_id = EffectApply::definition(
        &definition,
        application(Payload::Buff, EffectApplicationDecision::Accept),
    )
    .run(&mut pipeline)
    .unwrap()
    .active_effect_id()
    .expect("duration effect should create an active effect");
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| events.push(event));
        EffectTick::new(100)
            .run_with_executor(&mut pipeline, &mut context, &mut executor)
            .unwrap();
    }
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("natural timeout should advance then expire");
    };
    assert_eq!(expiring_advance.effect.remaining_units, Some(0));
    assert_eq!(expired.id, expiring_id);
    assert_eq!(pipeline.count(), 0);
}
