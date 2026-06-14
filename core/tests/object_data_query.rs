mod common;

use common::marker_setup;
use flexweave::{CoreError, DataStore, INVALID_OBJECT_ID, ObjectId, ObjectStore, query};

#[test]
fn val_core_001_object_store_hands_out_unique_looked_up_able_handles() {
    let mut store = ObjectStore::new();

    let a = store.create();
    let b = store.create();
    let c = store.create();

    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
    assert!(store.exists(a));
    assert!(store.exists(b));
    assert!(store.exists(c));
    assert!(!store.exists(ObjectId::new(9_999)));
    assert!(!store.exists(INVALID_OBJECT_ID));
    assert_eq!(store.count(), 3);
    assert_eq!(a.get(), 1);
}

#[test]
fn val_core_001_repeated_identical_setup_yields_identical_iteration_order() {
    let mut run_a = ObjectStore::new();
    let ids_a = [
        run_a.create(),
        run_a.create(),
        run_a.create(),
        run_a.create(),
    ];

    let mut run_b = ObjectStore::new();
    let ids_b = [
        run_b.create(),
        run_b.create(),
        run_b.create(),
        run_b.create(),
    ];

    assert_eq!(ids_a, ids_b);
    assert_eq!(
        run_a.iter().collect::<Vec<_>>(),
        run_b.iter().collect::<Vec<_>>()
    );
}

#[test]
fn val_core_001_object_store_registers_externally_assigned_handles() {
    let mut store = ObjectStore::new();

    let external = store.create_with_id(ObjectId::new(1001)).unwrap();
    let generated = store.create();

    assert_eq!(external, ObjectId::new(1001));
    assert_eq!(generated, ObjectId::new(1002));
    assert!(store.exists(external));
    assert!(store.exists(generated));
    assert_eq!(
        store.create_with_id(INVALID_OBJECT_ID),
        Err(CoreError::InvalidObjectId)
    );
    assert_eq!(
        store.create_with_id(external),
        Err(CoreError::ObjectIdAlreadyExists)
    );
    assert_eq!(store.iter().collect::<Vec<_>>(), vec![external, generated]);
}

#[test]
fn val_core_002_generic_marker_data_attach_has_get_round_trips_correctly() {
    let mut store = ObjectStore::new();
    let mut markers = DataStore::new();

    let o1 = store.create();
    let o2 = store.create();
    let o3 = store.create();

    markers.attach(o1, 10_u16);
    markers.attach(o2, 20_u16);

    assert!(markers.has(o1));
    assert!(markers.has(o2));
    assert!(!markers.has(o3));
    assert_eq!(markers.get(o1), Some(&10));
    assert_eq!(markers.get(o2), Some(&20));
    assert_eq!(markers.get(o3), None);

    markers.attach(o1, 30);
    assert_eq!(markers.get(o1), Some(&30));
    assert_eq!(markers.count(), 2);
}

#[test]
fn val_core_002_generic_data_supports_non_copy_values() {
    let mut store = ObjectStore::new();
    let mut values = DataStore::new();

    let o1 = store.create();
    let o2 = store.create();

    values.attach(o1, String::from("attached"));
    assert!(values.has(o1));
    assert!(!values.has(o2));
    assert_eq!(values.get(o1).map(String::as_str), Some("attached"));
    assert_eq!(values.get(o2), None);
}

#[test]
fn val_core_003_require_object_rejects_invalid_handles_explicitly() {
    let mut store = ObjectStore::new();

    let valid = store.create();
    assert_eq!(query::require_object(&store, valid), Ok(()));
    assert_eq!(
        query::require_object(&store, INVALID_OBJECT_ID),
        Err(CoreError::InvalidObjectId)
    );
    assert_eq!(
        query::require_object(&store, ObjectId::new(9_999)),
        Err(CoreError::InvalidObjectId)
    );
}

#[test]
fn val_core_003_require_attached_returns_missing_required_data_explicitly() {
    let mut store = ObjectStore::new();
    let mut labels = DataStore::new();

    let attached = store.create();
    let missing = store.create();
    labels.attach(attached, 7_u8);

    assert_eq!(query::require_attached(&labels, attached), Ok(&7));
    assert_eq!(
        query::require_attached(&labels, missing),
        Err(CoreError::MissingRequiredData)
    );
}

#[test]
fn val_core_004_collect_where_returns_matches_in_creation_order() {
    let (objects, markers, excluded, first_match, second_match) = marker_setup();
    let result = query::collect_where(&objects, |candidate_id| {
        candidate_id != excluded && markers.get(candidate_id).is_some_and(|value| *value == 1)
    });

    assert_eq!(result, vec![first_match, second_match, ObjectId::new(6)]);
}

#[test]
fn val_core_005_identical_setups_produce_identical_collect_where_results() {
    fn run() -> Vec<ObjectId> {
        let mut store = ObjectStore::new();
        let mut markers = DataStore::new();

        let excluded = store.create();
        markers.attach(excluded, 1_u8);

        for i in 0..5 {
            let id = store.create();
            markers.attach(id, if i % 2 == 0 { 9 } else { 3 });
        }

        query::collect_where(&store, |candidate_id| {
            candidate_id != excluded && markers.get(candidate_id).is_some_and(|value| *value == 9)
        })
    }

    let first = run();
    let second = run();

    assert_eq!(first.len(), 3);
    assert_eq!(first, second);
}
