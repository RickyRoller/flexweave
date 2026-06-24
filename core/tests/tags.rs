mod common;

use common::TestAtom;
use flexweave::{Tag, TagSet};

struct ConstructorOnlyAtom(&'static str);

#[derive(Debug, Eq, PartialEq)]
struct NonCloneAtom(&'static str);

#[test]
fn tag_constructors_and_accessors_do_not_require_clone_or_eq() {
    let tag = Tag::new([
        ConstructorOnlyAtom("category"),
        ConstructorOnlyAtom("variant"),
    ]);

    assert_eq!(tag.atoms().len(), 2);
    assert_eq!(tag.atoms()[0].0, "category");

    let set = TagSet::new([tag]);

    assert_eq!(set.items().len(), 1);
    assert_eq!(set.items()[0].atoms()[1].0, "variant");
}

#[test]
fn tag_apis_accept_non_clone_atoms_and_borrowed_atom_queries() {
    let tag = Tag::new([NonCloneAtom("category"), NonCloneAtom("variant")]);

    assert_eq!(tag.atoms().len(), 2);
    assert!(tag.has_atom(&NonCloneAtom("variant")));
    assert!(!tag.has_atom(&NonCloneAtom("other")));
    assert!(tag.has_all_atoms([&NonCloneAtom("category"), &NonCloneAtom("variant")]));
    assert!(!tag.has_all_atoms([&NonCloneAtom("category"), &NonCloneAtom("other")]));
}

#[test]
fn tag_set_apis_accept_non_clone_atoms_and_borrowed_atom_queries() {
    let tag = Tag::new([NonCloneAtom("category"), NonCloneAtom("variant")]);
    let exact = Tag::new([NonCloneAtom("category"), NonCloneAtom("variant")]);
    let prefix = Tag::new([NonCloneAtom("category")]);
    let set = TagSet::new([tag]);

    assert_eq!(set.items().len(), 1);
    assert!(set.has(&exact));
    assert!(set.has_prefix(&prefix));
    assert!(set.has_atom(&NonCloneAtom("variant")));
    assert!(!set.has_atom(&NonCloneAtom("other")));
    assert!(set.has_tag_with_all_atoms([&NonCloneAtom("category"), &NonCloneAtom("variant")]));
    assert!(!set.has_tag_with_all_atoms([&NonCloneAtom("category"), &NonCloneAtom("other")]));
}

#[test]
fn val_core_011_tag_sets_distinguish_shared_atoms_by_grouped_path() {
    let category_family = Tag::new([TestAtom::Category, TestAtom::Family]);
    let category_variant = Tag::new([TestAtom::Category, TestAtom::Family, TestAtom::Variant]);
    let group_family = Tag::new([TestAtom::Group, TestAtom::Family]);
    let split_category = Tag::new([TestAtom::Category]);

    let grouped = TagSet::new([category_variant.clone()]);
    assert!(grouped.has_atom(&TestAtom::Family));
    assert!(grouped.has_prefix(&category_family));
    assert!(grouped.has_tag_with_all_atoms([TestAtom::Category, TestAtom::Family]));
    assert!(!grouped.has(&group_family));

    let split = TagSet::new([split_category, group_family]);
    assert!(split.has_atom(&TestAtom::Category));
    assert!(split.has_atom(&TestAtom::Family));
    assert!(!split.has_tag_with_all_atoms([TestAtom::Category, TestAtom::Family]));
}
