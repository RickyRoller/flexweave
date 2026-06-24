use crate::common::TestAtom;
use flexweave::{ObjectId, Tag, TagSet, TagSetQuery};
use proptest::prelude::*;

pub fn arb_object_count() -> impl Strategy<Value = usize> {
    0..64usize
}

pub fn arb_external_object_id() -> impl Strategy<Value = ObjectId> {
    (0..96u64).prop_map(ObjectId::new)
}

pub fn arb_marker_value() -> impl Strategy<Value = u8> {
    any::<u8>()
}

pub fn arb_attachment_plan() -> impl Strategy<Value = Vec<(usize, u8)>> {
    prop::collection::vec((0..64usize, arb_marker_value()), 0..128)
}

pub fn arb_test_atom() -> impl Strategy<Value = TestAtom> {
    prop_oneof![
        Just(TestAtom::Ability),
        Just(TestAtom::Burst),
        Just(TestAtom::Category),
        Just(TestAtom::Group),
        Just(TestAtom::Family),
        Just(TestAtom::Variant),
    ]
}

pub fn arb_tag() -> impl Strategy<Value = Tag<TestAtom>> {
    prop::collection::vec(arb_test_atom(), 0..4).prop_map(Tag::new)
}

pub fn arb_tag_set() -> impl Strategy<Value = TagSet<TestAtom>> {
    prop::collection::vec(arb_tag(), 0..12).prop_map(TagSet::new)
}

pub fn arb_tag_query() -> impl Strategy<Value = TagSetQuery<TestAtom>> {
    (
        prop::collection::vec(arb_tag(), 0..12),
        prop::collection::vec(arb_tag(), 0..12),
        prop::collection::vec(arb_tag(), 0..12),
    )
        .prop_map(|(all, any, none)| TagSetQuery { all, any, none })
}
