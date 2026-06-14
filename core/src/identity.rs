//! Stable object identity primitive.

use crate::errors::CoreError;
use std::fmt;

/// Stable domain-neutral object handle.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(u64);

impl ObjectId {
    /// Reserved sentinel used to represent "no object".
    pub const INVALID: Self = Self(0);

    /// Creates an object id from its wire/storage value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the wire/storage value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns true when this id is the reserved invalid sentinel.
    #[must_use]
    pub const fn is_invalid(self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for ObjectId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<ObjectId> for u64 {
    fn from(value: ObjectId) -> Self {
        value.get()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Reserved sentinel used to represent "no object".
pub const INVALID_OBJECT_ID: ObjectId = ObjectId::INVALID;

/// Hands out stable object ids in deterministic creation order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectStore {
    ids: Vec<ObjectId>,
    next_id: u64,
}

impl ObjectStore {
    /// Creates an empty object store. Generated ids start at 1.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            next_id: 1,
        }
    }

    /// Allocates a new deterministic object handle.
    pub fn create(&mut self) -> ObjectId {
        let id = ObjectId::new(self.next_id);
        self.ids.push(id);
        self.next_id += 1;
        id
    }

    /// Registers an externally assigned object handle in iteration order.
    pub fn create_with_id(&mut self, id: ObjectId) -> Result<ObjectId, CoreError> {
        if id.is_invalid() {
            return Err(CoreError::InvalidObjectId);
        }
        if self.exists(id) {
            return Err(CoreError::ObjectIdAlreadyExists);
        }

        self.ids.push(id);
        if id.get() >= self.next_id {
            self.next_id = id.get() + 1;
        }
        Ok(id)
    }

    /// Returns true when `id` is live in this store.
    #[must_use]
    pub fn exists(&self, id: ObjectId) -> bool {
        !id.is_invalid() && self.ids.contains(&id)
    }

    /// Iterates object ids in creation/registration order.
    pub fn iter(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.ids.iter().copied()
    }

    /// Number of live object handles.
    #[must_use]
    pub fn count(&self) -> usize {
        self.ids.len()
    }

    /// Returns true when no objects have been created.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}
