mod common;

use common::marker_setup;
use flexweave::{CoreError, DataStore, INVALID_OBJECT_ID, ObjectId, ObjectStore, query};

#[test]
fn object_store_hands_out_unique_lookupable_handles() {
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
fn object_store_registers_externally_assigned_handles() {
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
fn data_store_attach_and_get_round_trip() {
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
fn data_store_supports_non_copy_values() {
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
fn require_object_rejects_invalid_handles_explicitly() {
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
fn require_attached_returns_missing_required_data_explicitly() {
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
fn collect_where_returns_matches_in_creation_order() {
    let (objects, markers, excluded, first_match, second_match) = marker_setup();
    let result = query::collect_where(&objects, |candidate_id| {
        candidate_id != excluded && markers.get(candidate_id).is_some_and(|value| *value == 1)
    });

    assert_eq!(result, vec![first_match, second_match, ObjectId::new(6)]);
}
