mod common;

use common::{TestAtom, block_on};
use flexweave::{
    AbilityCommitTiming, AbilityGrantError, AbilityHooks, AbilityLifecycleEvent, AbilityStore,
    Attribute, CoreError, DataStore, DerivedAttribute, EffectApplicationError,
    EffectApplicationInput, EffectClockPolicy, EffectDefinition, EffectKind, EffectLifecycleEvent,
    EffectObjectRemovalPolicy, EffectPipeline, EffectSourcePolicy, Grant, ObjectDestructionDriver,
    ObjectId, ObjectStore, Tag, TagSet, query,
};

#[test]
fn object_store_destroy_removes_live_ids_without_reuse() {
    let mut objects = ObjectStore::new();
    let first = objects.create();
    let destroyed = objects.create();
    let last = objects.create();

    assert_eq!(
        objects.destroy(ObjectId::INVALID),
        Err(CoreError::InvalidObjectId)
    );
    assert_eq!(
        objects.destroy(ObjectId::new(999)),
        Err(CoreError::InvalidObjectId)
    );
    assert_eq!(objects.destroy(destroyed), Ok(destroyed));

    assert!(!objects.exists(destroyed));
    assert_eq!(objects.iter().collect::<Vec<_>>(), vec![first, last]);
    assert_eq!(query::collect_where(&objects, |_| true), vec![first, last]);
    assert_eq!(objects.create(), ObjectId::new(4));
}

#[test]
fn object_cleanup_driver_removes_registered_object_keyed_state() {
    let mut objects = ObjectStore::new();
    let removed = objects.create();
    let retained = objects.create();
    let mut labels = DataStore::new();
    let mut attribute = Attribute::new();
    let mut derived = DerivedAttribute::new(|_| Some(1.0));
    labels.attach(removed, "removed");
    labels.attach(retained, "retained");
    attribute.attach(removed, 10.0);
    attribute.attach(retained, 20.0);
    derived.sync(removed);
    derived.sync(retained);

    let events = ObjectDestructionDriver::<()>::new(&mut objects)
        .with_store(&mut labels)
        .with_store(&mut attribute)
        .with_store(&mut derived)
        .destroy(removed)
        .unwrap();

    assert!(events.is_empty());
    assert!(!objects.exists(removed));
    assert_eq!(labels.get(removed), None);
    assert_eq!(labels.get(retained), Some(&"retained"));
    assert_eq!(attribute.get(removed), None);
    assert_eq!(attribute.get(retained), Some(20.0));
    assert_eq!(derived.count(), 1);
}

#[test]
fn ability_owner_cleanup_revokes_grants_and_active_abilities() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    struct Hooks;

    impl AbilityHooks<(), TagSet<TestAtom>, Payload> for Hooks {
        type Error = ();
        type BlockReason = ();
    }

    let mut objects = ObjectStore::new();
    let owner = objects.create();
    let other_owner = objects.create();
    let mut abilities = AbilityStore::<TagSet<TestAtom>, Payload>::new();
    let owned = abilities
        .grant_checked(
            &objects,
            Grant::new(owner, TagSet::new([Tag::new([TestAtom::Ability])]), Payload),
        )
        .unwrap();
    let retained = abilities
        .grant_checked(
            &objects,
            Grant::new(
                other_owner,
                TagSet::new([Tag::new([TestAtom::Ability, TestAtom::Burst])]),
                Payload,
            ),
        )
        .unwrap();
    let mut context = ();
    let mut hooks = Hooks;
    let owned_activation = block_on(abilities.begin_activation_for_owner_with(
        owner,
        owned,
        AbilityCommitTiming::OnStart,
        &mut context,
        &mut hooks,
    ))
    .unwrap();
    let retained_activation = block_on(abilities.begin_activation_for_owner_with(
        other_owner,
        retained,
        AbilityCommitTiming::OnStart,
        &mut context,
        &mut hooks,
    ))
    .unwrap();
    let mut events: Vec<AbilityLifecycleEvent<TagSet<TestAtom>, Payload>> = Vec::new();

    let revoked = abilities.revoke_owner_with_events(owner, |event| events.push(event));

    assert_eq!(revoked.grants.len(), 1);
    assert_eq!(revoked.grants[0].id, owned);
    assert_eq!(revoked.active_abilities.len(), 1);
    assert_eq!(revoked.active_abilities[0].activation_id, owned_activation);
    assert_eq!(abilities.count(), 1);
    assert_eq!(abilities.active_activation_count(), 1);
    assert_eq!(abilities.get(retained).unwrap().owner_id, other_owner);
    assert!(
        abilities
            .get_active_activation(retained_activation)
            .is_some()
    );
    let [AbilityLifecycleEvent::Revoked(revoked)] = events.as_slice() else {
        panic!("owner cleanup should emit one revocation fact");
    };
    assert_eq!(revoked.activation_id, owned_activation);
}

#[test]
fn effect_object_cleanup_removes_source_and_target_matches_with_events() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let mut objects = ObjectStore::new();
    let source = objects.create();
    let removed_target = objects.create();
    let retained_target = objects.create();
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    let definition = EffectDefinition {
        key: "active".to_owned(),
        kind: EffectKind::Duration,
        duration: Some(EffectClockPolicy { units: 100 }),
        period: None,
        routing: Default::default(),
        payload_schema: (),
    };
    effects
        .apply_checked(
            &objects,
            &definition,
            EffectApplicationInput::accept(
                source,
                removed_target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload,
            ),
            EffectSourcePolicy::RequireLiveSource,
        )
        .unwrap();
    effects
        .apply_checked(
            &objects,
            &definition,
            EffectApplicationInput::accept(
                None,
                retained_target,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload,
            ),
            EffectSourcePolicy::AllowSystemSource,
        )
        .unwrap();
    let mut events = Vec::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new();

    let removed = effects.remove_for_object_with_events(
        removed_target,
        EffectObjectRemovalPolicy::SourceOrTarget,
        |event| events.push(event),
    );

    assert_eq!(removed.len(), 1);
    assert_eq!(events.len(), 1);
    assert_eq!(effects.count(), 1);
    assert!(effects.has_tag(retained_target, &Tag::new([TestAtom::Category])));

    events.clear();
    effects
        .apply_checked(
            &objects,
            &definition,
            EffectApplicationInput::accept(
                source,
                retained_target,
                TagSet::new([Tag::new([TestAtom::Category, TestAtom::Variant])]),
                Payload,
            ),
            EffectSourcePolicy::RequireLiveSource,
        )
        .unwrap();

    let removed =
        effects.remove_for_object_with_events(source, EffectObjectRemovalPolicy::Source, |event| {
            events.push(event)
        });

    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].source_id, Some(source));
    assert_eq!(events.len(), 1);
    assert_eq!(effects.count(), 1);
    assert!(effects.has_tag(retained_target, &Tag::new([TestAtom::Category])));
}

#[test]
fn destroyed_objects_are_rejected_by_checked_runtime_paths() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Payload;

    let mut objects = ObjectStore::new();
    let destroyed = objects.create();
    let live = objects.create();
    assert_eq!(objects.destroy(destroyed), Ok(destroyed));

    let mut abilities = AbilityStore::<TagSet<TestAtom>, Payload>::new();
    assert_eq!(
        abilities.grant_checked(
            &objects,
            Grant::new(
                destroyed,
                TagSet::new([Tag::new([TestAtom::Ability])]),
                Payload,
            ),
        ),
        Err(AbilityGrantError::InvalidOwner {
            owner_id: destroyed,
        })
    );

    let definition = EffectDefinition::instant("instant", ());
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    assert_eq!(
        effects.apply_checked(
            &objects,
            &definition,
            EffectApplicationInput::accept(
                live,
                destroyed,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload,
            ),
            EffectSourcePolicy::RequireLiveSource,
        ),
        Err(EffectApplicationError::InvalidTarget {
            target_id: destroyed,
        })
    );
    assert_eq!(
        effects.apply_checked(
            &objects,
            &definition,
            EffectApplicationInput::accept(
                destroyed,
                live,
                TagSet::new([Tag::new([TestAtom::Category])]),
                Payload,
            ),
            EffectSourcePolicy::RequireLiveSource,
        ),
        Err(EffectApplicationError::InvalidSource {
            source_id: destroyed,
        })
    );
}
