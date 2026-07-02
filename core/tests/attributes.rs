use flexweave::{
    Attribute, AttributeMutationDecision, AttributeMutationHooks, AttributeMutationRequest,
    AttributeMutationResult, AttributeValue, DataStore, DerivedAttribute, EventChannel,
    EventChannelDefinition, EventRetention, LifecycleEvent, LifecycleEventKind, ObjectId,
    ObjectStore,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[test]
fn val_core_006_standalone_attribute_stores_signed_floats_with_stable_overwrite() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();

    let player = store.create();
    let enemy = store.create();

    attribute.attach(player, -4.5);
    attribute.attach(enemy, 12.25);

    assert!(attribute.has(player));
    assert!(attribute.has(enemy));
    assert_eq!(attribute.get(player), Some(-4.5));
    assert_eq!(attribute.get(enemy), Some(12.25));
    assert_eq!(attribute.count(), 2);

    attribute.attach(player, -1.0);
    assert_eq!(attribute.get(player), Some(-1.0));
    assert_eq!(attribute.count(), 2);
}

#[test]
fn val_core_007_attribute_commits_before_notifying_in_registration_order() {
    #[derive(Default, Debug)]
    struct Trace {
        steps: Vec<u8>,
        previous: Option<AttributeValue>,
        requested: AttributeValue,
        current: AttributeValue,
    }

    let mut store = ObjectStore::new();
    let mut health = Attribute::new();
    let target = store.create();
    health.attach(target, 20.0);

    let trace = Arc::new(Mutex::new(Trace::default()));
    let audit_trace = Arc::clone(&trace);
    health.add_listener(move |change| {
        let mut trace = audit_trace.lock().unwrap();
        trace.steps.push(1);
        trace.previous = change.previous;
        trace.requested = change.requested;
        trace.current = change.current;
    });

    let ui_trace = Arc::clone(&trace);
    health.subscribe(move |_| ui_trace.lock().unwrap().steps.push(2));

    let applied = health.set(target, 10.0);

    assert_eq!(applied, 10.0);
    assert_eq!(health.get(target), Some(10.0));
    let trace = trace.lock().unwrap();
    assert_eq!(trace.steps, vec![1, 2]);
    assert_eq!(trace.previous, Some(20.0));
    assert_eq!(trace.requested, 10.0);
    assert_eq!(trace.current, 10.0);
}

#[test]
fn val_core_008_attribute_does_not_emit_events_when_value_is_unchanged() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 12.0);

    let count = Arc::new(Mutex::new(0_usize));
    let listener_count = Arc::clone(&count);
    attribute.subscribe(move |_| *listener_count.lock().unwrap() += 1);

    let unchanged = attribute.set(target, 12.0);
    assert_eq!(unchanged, 12.0);
    assert_eq!(*count.lock().unwrap(), 0);

    let changed = attribute.set(target, -3.0);
    assert_eq!(changed, -3.0);
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn attribute_set_with_events_preserves_existing_listener_order() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 5.0);

    let steps = Arc::new(Mutex::new(Vec::new()));
    let listener_steps = Arc::clone(&steps);
    attribute.subscribe(move |change| {
        assert_eq!(
            change.lifecycle_event_kind(),
            LifecycleEventKind::AttributeChanged
        );
        listener_steps.lock().unwrap().push("listener");
    });

    let event_steps = Arc::clone(&steps);
    let applied = attribute.set_with_events(target, 7.0, move |change| {
        assert_eq!(change.previous, Some(5.0));
        assert_eq!(change.current, 7.0);
        assert_eq!(
            change.lifecycle_event_kind(),
            LifecycleEventKind::AttributeChanged
        );
        event_steps.lock().unwrap().push("event");
    });

    assert_eq!(applied, 7.0);
    assert_eq!(attribute.get(target), Some(7.0));
    assert_eq!(*steps.lock().unwrap(), vec!["listener", "event"]);
}

#[test]
fn attribute_mutation_hooks_can_clamp_before_commit() {
    #[derive(Clone, Copy)]
    struct Bounds {
        minimum: AttributeValue,
        maximum: AttributeValue,
    }

    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 5.0);

    let mut hooks = AttributeMutationHooks::<Bounds, &'static str>::new();
    hooks.add_pre_hook(|mutation| {
        AttributeMutationDecision::Transform(
            mutation
                .current
                .clamp(mutation.context.minimum, mutation.context.maximum),
        )
    });
    let context = Bounds {
        minimum: 0.0,
        maximum: 10.0,
    };

    let result = attribute.set_with_hooks(
        AttributeMutationRequest {
            id: target,
            requested: 15.0,
        },
        &context,
        &mut hooks,
    );

    let AttributeMutationResult::Committed(change) = result else {
        panic!("clamped mutation should commit");
    };
    assert_eq!(change.previous, Some(5.0));
    assert_eq!(change.requested, 15.0);
    assert_eq!(change.current, 10.0);
    assert_eq!(attribute.get(target), Some(10.0));
}

#[test]
fn attribute_mutation_hooks_can_reject_without_storage_or_events() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 5.0);
    let mut hooks = AttributeMutationHooks::<(), &'static str>::new();
    hooks.add_pre_hook(|mutation| {
        if mutation.current < 0.0 {
            AttributeMutationDecision::Reject("below-zero")
        } else {
            AttributeMutationDecision::Allow
        }
    });
    let mut emitted = Vec::new();

    let result = attribute.set_with_hooks_and_events(
        AttributeMutationRequest {
            id: target,
            requested: -1.0,
        },
        &(),
        &mut hooks,
        |change| emitted.push(change),
    );

    let AttributeMutationResult::Rejected(rejected) = result else {
        panic!("negative mutation should be rejected");
    };
    assert_eq!(rejected.reason, "below-zero");
    assert_eq!(rejected.previous, Some(5.0));
    assert_eq!(attribute.get(target), Some(5.0));
    assert!(emitted.is_empty());
}

#[test]
fn attribute_mutation_hooks_run_in_deterministic_pre_listener_post_order() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 1.0);

    let steps = Arc::new(Mutex::new(Vec::new()));
    let pre_one_steps = Arc::clone(&steps);
    let pre_two_steps = Arc::clone(&steps);
    let listener_steps = Arc::clone(&steps);
    let post_steps = Arc::clone(&steps);

    let mut hooks = AttributeMutationHooks::<(), &'static str>::new();
    hooks.add_pre_hook(move |_| {
        pre_one_steps.lock().unwrap().push("pre-1");
        AttributeMutationDecision::Transform(7.0)
    });
    hooks.add_pre_hook(move |mutation| {
        pre_two_steps.lock().unwrap().push("pre-2");
        AttributeMutationDecision::Transform(mutation.current + 1.0)
    });
    attribute.subscribe(move |change| {
        assert_eq!(change.current, 8.0);
        listener_steps.lock().unwrap().push("listener");
    });
    hooks.add_post_hook(move |_, change| {
        assert_eq!(change.current, 8.0);
        post_steps.lock().unwrap().push("post");
    });

    let result = attribute.set_with_hooks(
        AttributeMutationRequest {
            id: target,
            requested: 12.0,
        },
        &(),
        &mut hooks,
    );

    assert!(matches!(result, AttributeMutationResult::Committed(_)));
    assert_eq!(
        *steps.lock().unwrap(),
        vec!["pre-1", "pre-2", "listener", "post"]
    );
}

#[test]
fn attribute_mutation_hooks_do_not_emit_when_final_value_is_unchanged() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 5.0);
    let mut hooks = AttributeMutationHooks::<(), &'static str>::new();
    hooks.add_pre_hook(|_| AttributeMutationDecision::Transform(5.0));
    let mut emitted = Vec::new();

    let result = attribute.set_with_hooks_and_events(
        AttributeMutationRequest {
            id: target,
            requested: 12.0,
        },
        &(),
        &mut hooks,
        |change| emitted.push(change),
    );

    assert_eq!(result, AttributeMutationResult::Unchanged(5.0));
    assert!(emitted.is_empty());
    assert_eq!(attribute.get(target), Some(5.0));
}

#[test]
fn caller_publishes_committed_attribute_mutation_to_named_event_channel() {
    let mut store = ObjectStore::new();
    let mut attribute = Attribute::new();
    let target = store.create();
    attribute.attach(target, 5.0);
    let mut hooks = AttributeMutationHooks::<(), &'static str>::new();
    let mut channel = EventChannel::with_retention(
        EventChannelDefinition::new("attributes/changes", [LifecycleEventKind::AttributeChanged])
            .unwrap(),
        EventRetention::Retain,
    );

    assert!(channel.retained().is_empty());
    let result = attribute.set_with_hooks_and_events(
        AttributeMutationRequest {
            id: target,
            requested: 7.0,
        },
        &(),
        &mut hooks,
        |change| channel.publish(change).unwrap(),
    );

    assert!(matches!(result, AttributeMutationResult::Committed(_)));
    let retained = channel.drain_retained();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].id, target);
    assert_eq!(retained[0].current, 7.0);
}

#[derive(Clone, Copy, Debug, Default)]
struct DerivedBonus {
    flat_bonus: AttributeValue,
    increased_ratio: AttributeValue,
    more_multiplier: AttributeValue,
}

fn calculate_derived(
    base: &Attribute,
    bonuses: &DataStore<DerivedBonus>,
    id: ObjectId,
) -> Option<AttributeValue> {
    let base_value = base.get(id)?;
    let bonus = bonuses.get(id).copied().unwrap_or(DerivedBonus {
        flat_bonus: 0.0,
        increased_ratio: 0.0,
        more_multiplier: 1.0,
    });
    Some((base_value + bonus.flat_bonus) * (1.0 + bonus.increased_ratio) * bonus.more_multiplier)
}

#[test]
fn val_core_009_derived_attribute_evaluates_without_overwriting_base() {
    let mut store = ObjectStore::new();
    let mut base = Attribute::new();
    let mut bonuses = DataStore::new();

    let target = store.create();
    base.attach(target, 100.0);
    bonuses.attach(
        target,
        DerivedBonus {
            flat_bonus: 30.0,
            increased_ratio: 0.05,
            more_multiplier: 1.05,
        },
    );

    let base = Rc::new(base);
    let bonuses = Rc::new(bonuses);
    let calculator_base = Rc::clone(&base);
    let calculator_bonuses = Rc::clone(&bonuses);
    let mut derived = DerivedAttribute::new(move |id| {
        calculate_derived(&calculator_base, &calculator_bonuses, id)
    });

    assert_eq!(base.get(target), Some(100.0));
    assert!((derived.get(target).unwrap() - 143.325).abs() < 0.000001);

    let synced = derived.sync(target);
    assert!((synced.unwrap() - 143.325).abs() < 0.000001);
    assert_eq!(derived.count(), 1);
}

#[test]
fn val_core_010_derived_attribute_refreshes_tracked_values_only_on_changes() {
    let mut store = ObjectStore::new();
    let mut base = Attribute::new();
    let bonuses = DataStore::new();

    let target = store.create();
    base.attach(target, 10.0);

    let base = Rc::new(RefCell::new(base));
    let bonuses = Rc::new(bonuses);
    let calculator_base = Rc::clone(&base);
    let calculator_bonuses = Rc::clone(&bonuses);
    let mut derived = DerivedAttribute::new(move |id| {
        let base = calculator_base.borrow();
        calculate_derived(&base, &calculator_bonuses, id)
    });

    assert_eq!(derived.sync(target), Some(10.0));

    #[derive(Default, Debug)]
    struct Trace {
        count: usize,
        previous: Option<AttributeValue>,
        current: Option<AttributeValue>,
    }

    let trace = Rc::new(RefCell::new(Trace::default()));
    let listener_trace = Rc::clone(&trace);
    derived.subscribe(move |change| {
        let mut trace = listener_trace.borrow_mut();
        trace.count += 1;
        trace.previous = change.previous;
        trace.current = change.current;
    });

    assert_eq!(derived.refresh(target), Some(10.0));
    assert_eq!(trace.borrow().count, 0);

    base.borrow_mut().attach(target, 12.0);
    assert_eq!(derived.refresh(target), Some(12.0));
    let trace = trace.borrow();
    assert_eq!(trace.count, 1);
    assert_eq!(trace.previous, Some(10.0));
    assert_eq!(trace.current, Some(12.0));
}

#[test]
fn derived_attribute_refresh_with_events_preserves_existing_listener_order() {
    let mut store = ObjectStore::new();
    let mut base = Attribute::new();
    let bonuses = DataStore::new();

    let target = store.create();
    base.attach(target, 10.0);

    let base = Rc::new(RefCell::new(base));
    let bonuses = Rc::new(bonuses);
    let calculator_base = Rc::clone(&base);
    let calculator_bonuses = Rc::clone(&bonuses);
    let mut derived = DerivedAttribute::new(move |id| {
        let base = calculator_base.borrow();
        calculate_derived(&base, &calculator_bonuses, id)
    });
    assert_eq!(derived.sync(target), Some(10.0));

    let steps = Rc::new(RefCell::new(Vec::new()));
    let listener_steps = Rc::clone(&steps);
    derived.subscribe(move |change| {
        assert_eq!(
            change.lifecycle_event_kind(),
            LifecycleEventKind::DerivedAttributeChanged
        );
        listener_steps.borrow_mut().push("listener");
    });

    base.borrow_mut().attach(target, 12.0);
    let event_steps = Rc::clone(&steps);
    assert_eq!(
        derived.refresh_with_events(target, move |change| {
            assert_eq!(change.previous, Some(10.0));
            assert_eq!(change.current, Some(12.0));
            assert_eq!(
                change.lifecycle_event_kind(),
                LifecycleEventKind::DerivedAttributeChanged
            );
            event_steps.borrow_mut().push("event");
        }),
        Some(12.0)
    );

    assert_eq!(*steps.borrow(), vec!["listener", "event"]);
}
