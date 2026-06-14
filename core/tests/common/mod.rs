#![allow(dead_code)]

use flexweave::{DataStore, ObjectId, ObjectStore};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestAtom {
    Ability,
    Burst,
    Damage,
    Resistance,
    Elemental,
    Fire,
}

pub fn marker_setup() -> (ObjectStore, DataStore<u8>, ObjectId, ObjectId, ObjectId) {
    let mut objects = ObjectStore::new();
    let mut markers = DataStore::new();

    let excluded = objects.create();
    markers.attach(excluded, 1);

    let first_match = objects.create();
    markers.attach(first_match, 1);

    let non_match = objects.create();
    markers.attach(non_match, 2);

    let second_match = objects.create();
    markers.attach(second_match, 1);

    let no_data = objects.create();
    let _ = no_data;

    let third_match = objects.create();
    markers.attach(third_match, 1);

    (objects, markers, excluded, first_match, second_match)
}
