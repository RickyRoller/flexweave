//! Internal object-keyed ordered storage.

use crate::identity::ObjectId;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ObjectMap<T> {
    entries: Vec<(ObjectId, T)>,
    index_by_id: HashMap<ObjectId, usize>,
}

impl<T> ObjectMap<T> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            index_by_id: HashMap::new(),
        }
    }

    pub(crate) fn put(&mut self, id: ObjectId, value: T) {
        if let Some(&index) = self.index_by_id.get(&id) {
            let (_, existing) = &mut self.entries[index];
            *existing = value;
            return;
        }
        self.index_by_id.insert(id, self.entries.len());
        self.entries.push((id, value));
    }

    pub(crate) fn contains(&self, id: ObjectId) -> bool {
        self.index_by_id.contains_key(&id)
    }

    pub(crate) fn get(&self, id: ObjectId) -> Option<&T> {
        self.index_by_id
            .get(&id)
            .map(|&index| &self.entries[index].1)
    }

    pub(crate) fn replace_existing(&mut self, id: ObjectId, value: T) -> bool {
        let Some(&index) = self.index_by_id.get(&id) else {
            return false;
        };
        let (_, existing) = &mut self.entries[index];
        *existing = value;
        true
    }

    pub(crate) fn remove(&mut self, id: ObjectId) -> bool {
        let Some(index) = self.index_by_id.remove(&id) else {
            return false;
        };
        self.entries.remove(index);
        self.reindex_from(index);
        true
    }

    pub(crate) fn count(&self) -> usize {
        self.entries.len()
    }

    fn reindex_from(&mut self, start: usize) {
        for index in start..self.entries.len() {
            self.index_by_id.insert(self.entries[index].0, index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overwrite_and_remove_keep_index_consistent() {
        let first = ObjectId::new(1);
        let second = ObjectId::new(2);
        let third = ObjectId::new(3);
        let mut map = ObjectMap::new();

        map.put(first, 10);
        map.put(second, 20);
        map.put(third, 30);
        map.put(second, 200);

        assert_eq!(map.count(), 3);
        assert_eq!(map.get(first), Some(&10));
        assert_eq!(map.get(second), Some(&200));
        assert_eq!(map.get(third), Some(&30));

        assert!(map.remove(first));
        assert!(!map.contains(first));
        assert_eq!(map.get(second), Some(&200));
        assert_eq!(map.get(third), Some(&30));

        assert!(map.replace_existing(third, 300));
        assert_eq!(map.get(third), Some(&300));
        assert!(map.remove(second));
        assert_eq!(map.count(), 1);
        assert_eq!(map.get(third), Some(&300));
        assert!(!map.replace_existing(first, 100));
        assert!(!map.remove(first));
    }
}
