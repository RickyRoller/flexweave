mod common;
mod support;

use proptest::prelude::*;
use support::{models, strategies};

proptest! {
    #[test]
    fn prop_tag_has_atom_matches_path_membership(
        tag in strategies::arb_tag(),
        atom in strategies::arb_test_atom(),
    ) {
        prop_assert_eq!(tag.has_atom(&atom), tag.atoms().contains(&atom));
    }

    #[test]
    fn prop_tag_starts_with_matches_slice_prefix(
        tag in strategies::arb_tag(),
        prefix in strategies::arb_tag(),
    ) {
        prop_assert_eq!(tag.starts_with(&prefix), tag.atoms().starts_with(prefix.atoms()));
    }

    #[test]
    fn prop_tag_set_has_exact_tag_matches_reference(
        set in strategies::arb_tag_set(),
        tag in strategies::arb_tag(),
    ) {
        prop_assert_eq!(set.has(&tag), models::tag_set_has_exact_reference(&set, &tag));
    }

    #[test]
    fn prop_tag_set_prefix_and_atom_queries_match_reference(
        set in strategies::arb_tag_set(),
        prefix in strategies::arb_tag(),
        atom in strategies::arb_test_atom(),
        atoms in prop::collection::vec(strategies::arb_test_atom(), 0..4),
    ) {
        prop_assert_eq!(
            set.has_prefix(&prefix),
            models::tag_set_has_prefix_reference(&set, &prefix)
        );
        prop_assert_eq!(
            set.has_atom(&atom),
            models::tag_set_has_atom_reference(&set, atom)
        );
        prop_assert_eq!(
            set.has_tag_with_all_atoms(atoms.iter()),
            models::tag_set_has_tag_with_all_atoms_reference(&set, &atoms)
        );
    }

    #[test]
    fn prop_tag_set_matches_exact_path_query_reference(
        set in strategies::arb_tag_set(),
        query in strategies::arb_tag_query(),
    ) {
        prop_assert_eq!(set.matches(&query), models::tag_set_matches_reference(&set, &query));
    }
}
