use crate::common::TestAtom;
use flexweave::{ObjectId, Tag, TagSet, TagSetQuery};
use std::collections::{BTreeMap, BTreeSet};

pub fn expected_object_ids_after_create(count: usize) -> Vec<ObjectId> {
    (1..=count as u64).map(ObjectId::new).collect()
}

pub fn expected_registered_order(inputs: &[ObjectId]) -> Vec<ObjectId> {
    let mut seen = BTreeSet::new();
    let mut accepted = Vec::new();

    for id in inputs {
        if !id.is_invalid() && seen.insert(*id) {
            accepted.push(*id);
        }
    }

    accepted
}

pub fn last_write_wins_attachment_map(
    ids: &[ObjectId],
    attachment_plan: &[(usize, u8)],
) -> BTreeMap<ObjectId, u8> {
    let mut expected = BTreeMap::new();
    if ids.is_empty() {
        return expected;
    }

    for (index, value) in attachment_plan {
        expected.insert(ids[*index % ids.len()], *value);
    }

    expected
}

pub fn collect_where_reference(ids: &[ObjectId], predicate_results: &[bool]) -> Vec<ObjectId> {
    ids.iter()
        .enumerate()
        .filter_map(|(index, id)| predicate_results[index].then_some(*id))
        .collect()
}

pub fn tag_set_has_exact_reference(set: &TagSet<TestAtom>, tag: &Tag<TestAtom>) -> bool {
    set.items().iter().any(|candidate| candidate == tag)
}

pub fn tag_set_has_prefix_reference(set: &TagSet<TestAtom>, prefix: &Tag<TestAtom>) -> bool {
    set.items()
        .iter()
        .any(|candidate| candidate.atoms().starts_with(prefix.atoms()))
}

pub fn tag_set_has_atom_reference(set: &TagSet<TestAtom>, atom: TestAtom) -> bool {
    set.items()
        .iter()
        .any(|candidate| candidate.atoms().contains(&atom))
}

pub fn tag_set_has_tag_with_all_atoms_reference(
    set: &TagSet<TestAtom>,
    atoms: &[TestAtom],
) -> bool {
    set.items()
        .iter()
        .any(|candidate| atoms.iter().all(|atom| candidate.atoms().contains(atom)))
}

pub fn tag_set_matches_reference(set: &TagSet<TestAtom>, query: &TagSetQuery<TestAtom>) -> bool {
    if query
        .none
        .iter()
        .any(|tag| tag_set_has_exact_reference(set, tag))
    {
        return false;
    }
    if query
        .all
        .iter()
        .any(|tag| !tag_set_has_exact_reference(set, tag))
    {
        return false;
    }
    query.any.is_empty()
        || query
            .any
            .iter()
            .any(|tag| tag_set_has_exact_reference(set, tag))
}
