mod common;
mod support;

use flexweave::{CoreError, DataStore, INVALID_OBJECT_ID, ObjectId, ObjectStore, query};
use proptest::prelude::*;
use std::collections::BTreeSet;
use support::{models, strategies};

proptest! {
    #[test]
    fn prop_object_store_create_allocates_contiguous_nonzero_ids(
        count in strategies::arb_object_count(),
    ) {
        let mut store = ObjectStore::new();
        let ids = (0..count).map(|_| store.create()).collect::<Vec<_>>();
        let expected = models::expected_object_ids_after_create(count);

        prop_assert_eq!(&ids, &expected);
        prop_assert_eq!(store.iter().collect::<Vec<_>>(), expected);
        prop_assert_eq!(store.count(), count);
        prop_assert!(!store.exists(INVALID_OBJECT_ID));
        for id in ids {
            prop_assert!(!id.is_invalid());
            prop_assert!(store.exists(id));
        }
    }

    #[test]
    fn prop_object_store_create_with_id_preserves_registration_order(
        inputs in prop::collection::vec(strategies::arb_external_object_id(), 0..64),
    ) {
        let mut store = ObjectStore::new();
        let mut seen = BTreeSet::new();

        for id in &inputs {
            let expected = if id.is_invalid() {
                Err(CoreError::InvalidObjectId)
            } else if seen.contains(id) {
                Err(CoreError::ObjectIdAlreadyExists)
            } else {
                Ok(*id)
            };

            prop_assert_eq!(store.create_with_id(*id), expected);
            if expected.is_ok() {
                seen.insert(*id);
            }
        }

        prop_assert_eq!(
            store.iter().collect::<Vec<_>>(),
            models::expected_registered_order(&inputs)
        );
    }

    #[test]
    fn prop_object_store_generated_id_advances_after_external_registration(
        inputs in prop::collection::vec(strategies::arb_external_object_id(), 0..64),
    ) {
        let mut store = ObjectStore::new();
        let mut accepted = Vec::new();

        for id in inputs {
            if store.create_with_id(id).is_ok() {
                accepted.push(id);
            }
        }

        let generated = store.create();
        let expected = accepted
            .iter()
            .map(|id| id.get())
            .max()
            .map_or(1, |max_registered| max_registered + 1);

        prop_assert_eq!(generated, ObjectId::new(expected));
        prop_assert!(store.exists(generated));
    }

    #[test]
    fn prop_data_store_last_attach_wins(
        count in strategies::arb_object_count(),
        attachment_plan in strategies::arb_attachment_plan(),
    ) {
        let mut objects = ObjectStore::new();
        let ids = (0..count).map(|_| objects.create()).collect::<Vec<_>>();
        let mut data = DataStore::new();

        if !ids.is_empty() {
            for (index, value) in &attachment_plan {
                data.attach(ids[*index % ids.len()], *value);
            }
        }

        let expected = models::last_write_wins_attachment_map(&ids, &attachment_plan);
        prop_assert_eq!(data.count(), expected.len());
        for id in ids {
            prop_assert_eq!(data.has(id), expected.contains_key(&id));
            prop_assert_eq!(data.get(id).copied(), expected.get(&id).copied());
        }
    }

    #[test]
    fn prop_collect_where_matches_reference_in_object_order(
        predicate_results in prop::collection::vec(any::<bool>(), 0..64),
    ) {
        let mut objects = ObjectStore::new();
        let ids = (0..predicate_results.len())
            .map(|_| objects.create())
            .collect::<Vec<_>>();

        let actual = query::collect_where(&objects, |candidate_id| {
            let index = (candidate_id.get() - 1) as usize;
            predicate_results[index]
        });

        prop_assert_eq!(
            actual,
            models::collect_where_reference(&ids, &predicate_results)
        );
    }

    #[test]
    fn prop_require_object_classifies_live_and_invalid_ids(
        count in strategies::arb_object_count(),
        probe in 0..96u64,
    ) {
        let mut objects = ObjectStore::new();
        for _ in 0..count {
            objects.create();
        }

        let id = ObjectId::new(probe);
        if objects.exists(id) {
            prop_assert_eq!(query::require_object(&objects, id), Ok(()));
        } else {
            prop_assert_eq!(query::require_object(&objects, id), Err(CoreError::InvalidObjectId));
        }
    }

    #[test]
    fn prop_require_attached_classifies_present_and_missing_data(
        count in strategies::arb_object_count(),
        attachment_plan in strategies::arb_attachment_plan(),
    ) {
        let mut objects = ObjectStore::new();
        let ids = (0..count).map(|_| objects.create()).collect::<Vec<_>>();
        let mut data = DataStore::new();

        if !ids.is_empty() {
            for (index, value) in &attachment_plan {
                data.attach(ids[*index % ids.len()], *value);
            }
        }

        let expected = models::last_write_wins_attachment_map(&ids, &attachment_plan);
        for id in ids {
            if let Some(expected_value) = expected.get(&id) {
                prop_assert_eq!(query::require_attached(&data, id), Ok(expected_value));
            } else {
                prop_assert_eq!(
                    query::require_attached::<u8>(&data, id),
                    Err(CoreError::MissingRequiredData)
                );
            }
        }
    }
}
