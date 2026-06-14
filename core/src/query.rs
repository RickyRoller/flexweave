//! Deterministic query helpers.

use crate::data_store::DataStore;
use crate::errors::CoreError;
use crate::identity::{ObjectId, ObjectStore};

/// Fails when `id` is not live in `store`.
pub fn require_object(store: &ObjectStore, id: ObjectId) -> Result<(), CoreError> {
    if store.exists(id) {
        Ok(())
    } else {
        Err(CoreError::InvalidObjectId)
    }
}

/// Returns data attached to `id` or fails with `MissingRequiredData`.
pub fn require_attached<T>(store: &DataStore<T>, id: ObjectId) -> Result<&T, CoreError> {
    store.get(id).ok_or(CoreError::MissingRequiredData)
}

/// Collects object ids matching `predicate` in object creation order.
pub fn collect_where<F>(store: &ObjectStore, mut predicate: F) -> Vec<ObjectId>
where
    F: FnMut(ObjectId) -> bool,
{
    store
        .iter()
        .filter(|candidate_id| predicate(*candidate_id))
        .collect()
}
