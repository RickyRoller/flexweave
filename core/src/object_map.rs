//! Internal object-keyed ordered storage.

use crate::identity::ObjectId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ObjectMap<T> {
    entries: Vec<(ObjectId, T)>,
}

impl<T> ObjectMap<T> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub(crate) fn put(&mut self, id: ObjectId, value: T) {
        if let Some((_, existing)) = self
            .entries
            .iter_mut()
            .find(|(existing_id, _)| *existing_id == id)
        {
            *existing = value;
            return;
        }
        self.entries.push((id, value));
    }

    pub(crate) fn contains(&self, id: ObjectId) -> bool {
        self.entries
            .iter()
            .any(|(existing_id, _)| *existing_id == id)
    }

    pub(crate) fn get(&self, id: ObjectId) -> Option<&T> {
        self.entries
            .iter()
            .find(|(existing_id, _)| *existing_id == id)
            .map(|(_, value)| value)
    }

    pub(crate) fn replace_existing(&mut self, id: ObjectId, value: T) -> bool {
        let Some((_, existing)) = self
            .entries
            .iter_mut()
            .find(|(existing_id, _)| *existing_id == id)
        else {
            return false;
        };
        *existing = value;
        true
    }

    pub(crate) fn remove(&mut self, id: ObjectId) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|(existing_id, _)| *existing_id == id)
        else {
            return false;
        };
        self.entries.remove(index);
        true
    }

    pub(crate) fn count(&self) -> usize {
        self.entries.len()
    }
}
