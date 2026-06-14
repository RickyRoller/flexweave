mod common;

use common::TestAtom;
use flexweave::{Tag, TagSet};

struct ConstructorOnlyAtom(&'static str);

#[derive(Debug, Eq, PartialEq)]
struct NonCloneAtom(&'static str);

#[test]
fn tag_constructors_and_accessors_do_not_require_clone_or_eq() {
    let tag = Tag::new([ConstructorOnlyAtom("damage"), ConstructorOnlyAtom("fire")]);

    assert_eq!(tag.atoms().len(), 2);
    assert_eq!(tag.atoms()[0].0, "damage");

    let set = TagSet::new([tag]);

    assert_eq!(set.items().len(), 1);
    assert_eq!(set.items()[0].atoms()[1].0, "fire");
}

#[test]
fn tag_apis_accept_non_clone_atoms_and_borrowed_atom_queries() {
    let tag = Tag::new([NonCloneAtom("damage"), NonCloneAtom("fire")]);

    assert_eq!(tag.atoms().len(), 2);
    assert!(tag.has_atom(&NonCloneAtom("fire")));
    assert!(!tag.has_atom(&NonCloneAtom("ice")));
    assert!(tag.has_all_atoms([&NonCloneAtom("damage"), &NonCloneAtom("fire")]));
    assert!(!tag.has_all_atoms([&NonCloneAtom("damage"), &NonCloneAtom("ice")]));
}

#[test]
fn tag_set_apis_accept_non_clone_atoms_and_borrowed_atom_queries() {
    let tag = Tag::new([NonCloneAtom("damage"), NonCloneAtom("fire")]);
    let exact = Tag::new([NonCloneAtom("damage"), NonCloneAtom("fire")]);
    let prefix = Tag::new([NonCloneAtom("damage")]);
    let set = TagSet::new([tag]);

    assert_eq!(set.items().len(), 1);
    assert!(set.has(&exact));
    assert!(set.has_prefix(&prefix));
    assert!(set.has_atom(&NonCloneAtom("fire")));
    assert!(!set.has_atom(&NonCloneAtom("ice")));
    assert!(set.has_tag_with_all_atoms([&NonCloneAtom("damage"), &NonCloneAtom("fire")]));
    assert!(!set.has_tag_with_all_atoms([&NonCloneAtom("damage"), &NonCloneAtom("ice")]));
}

#[test]
fn val_core_011_tag_sets_distinguish_shared_atoms_by_grouped_path() {
    let damage_elemental = Tag::new([TestAtom::Damage, TestAtom::Elemental]);
    let damage_fire = Tag::new([TestAtom::Damage, TestAtom::Elemental, TestAtom::Fire]);
    let resistance_elemental = Tag::new([TestAtom::Resistance, TestAtom::Elemental]);
    let split_damage = Tag::new([TestAtom::Damage]);

    let grouped = TagSet::new([damage_fire.clone()]);
    assert!(grouped.has_atom(&TestAtom::Elemental));
    assert!(grouped.has_prefix(&damage_elemental));
    assert!(grouped.has_tag_with_all_atoms([TestAtom::Damage, TestAtom::Elemental]));
    assert!(!grouped.has(&resistance_elemental));

    let split = TagSet::new([split_damage, resistance_elemental]);
    assert!(split.has_atom(&TestAtom::Damage));
    assert!(split.has_atom(&TestAtom::Elemental));
    assert!(!split.has_tag_with_all_atoms([TestAtom::Damage, TestAtom::Elemental]));
}
