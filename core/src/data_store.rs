//! Generic attached-data storage.

use crate::identity::ObjectId;
use crate::object_map::ObjectMap;

/// Object-keyed attached data store.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataStore<T> {
    values: ObjectMap<T>,
}

impl<T> DataStore<T> {
    /// Creates an empty data store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: ObjectMap::new(),
        }
    }

    /// Attaches or overwrites data for `id` without validating object existence.
    pub fn attach(&mut self, id: ObjectId, value: T) {
        self.values.put(id, value);
    }

    /// Detaches data for `id` without validating object existence.
    pub fn detach(&mut self, id: ObjectId) -> bool {
        self.values.remove(id)
    }

    /// Returns true when data is attached for `id`.
    #[must_use]
    pub fn has(&self, id: ObjectId) -> bool {
        self.values.contains(id)
    }

    /// Returns attached data for `id`.
    #[must_use]
    pub fn get(&self, id: ObjectId) -> Option<&T> {
        self.values.get(id)
    }

    /// Number of attached entries.
    #[must_use]
    pub fn count(&self) -> usize {
        self.values.count()
    }

    /// Returns true when no data is attached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}
