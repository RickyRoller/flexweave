//! Read-only derived signed floating-point attributes.

use crate::attribute::AttributeValue;
use crate::identity::ObjectId;
use crate::object_map::ObjectMap;

/// Derived attribute change visible to listeners after cache update.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DerivedChange {
    pub id: ObjectId,
    pub previous: Option<AttributeValue>,
    pub current: Option<AttributeValue>,
}

impl DerivedChange {
    /// Difference between current and previous, treating missing values as 0.
    #[must_use]
    pub fn delta(self) -> AttributeValue {
        self.current.unwrap_or(0.0) - self.previous.unwrap_or(0.0)
    }
}

type Calculator = Box<dyn Fn(ObjectId) -> Option<AttributeValue>>;
type Listener = Box<dyn FnMut(&DerivedChange)>;

/// Derived attribute channel with explicit tracked cache.
pub struct DerivedAttribute {
    calculator: Calculator,
    tracked: ObjectMap<AttributeValue>,
    listeners: Vec<Listener>,
}

impl DerivedAttribute {
    /// Creates a derived attribute backed by a safe Rust calculator closure.
    #[must_use]
    pub fn new<F>(calculator: F) -> Self
    where
        F: Fn(ObjectId) -> Option<AttributeValue> + 'static,
    {
        Self {
            calculator: Box::new(calculator),
            tracked: ObjectMap::new(),
            listeners: Vec::new(),
        }
    }

    /// Registers a listener in deterministic registration order.
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: FnMut(&DerivedChange) + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Alias for `add_listener`.
    pub fn subscribe<F>(&mut self, listener: F)
    where
        F: FnMut(&DerivedChange) + 'static,
    {
        self.add_listener(listener);
    }

    /// Evaluates current derived value without mutating the tracked cache.
    #[must_use]
    pub fn get(&self, id: ObjectId) -> Option<AttributeValue> {
        (self.calculator)(id)
    }

    /// Returns true when the calculator currently produces a value.
    #[must_use]
    pub fn has(&self, id: ObjectId) -> bool {
        self.get(id).is_some()
    }

    /// Number of tracked cached values.
    #[must_use]
    pub fn count(&self) -> usize {
        self.tracked.count()
    }

    /// Removes the tracked cache entry for `id` without running the calculator.
    pub fn untrack(&mut self, id: ObjectId) -> bool {
        self.tracked.remove(id)
    }

    /// Seeds or overwrites the tracked cache without notifying listeners.
    pub fn sync(&mut self, id: ObjectId) -> Option<AttributeValue> {
        let current = self.get(id);
        self.commit(id, current, false);
        current
    }

    /// Re-evaluates `id`, updates the cache, and notifies on meaningful changes.
    pub fn refresh(&mut self, id: ObjectId) -> Option<AttributeValue> {
        let current = self.get(id);
        self.commit(id, current, true);
        current
    }

    /// Re-evaluates `id`, updates the cache, notifies existing listeners, and
    /// emits a local event on meaningful changes.
    pub fn refresh_with_events<F>(
        &mut self,
        id: ObjectId,
        mut on_event: F,
    ) -> Option<AttributeValue>
    where
        F: FnMut(DerivedChange),
    {
        let current = self.get(id);
        if let Some(change) = self.commit(id, current, true) {
            on_event(change);
        }
        current
    }

    /// Refreshes an already-tracked id without allocating new cache entries.
    pub fn refresh_tracked(&mut self, id: ObjectId) -> Option<AttributeValue> {
        self.refresh_tracked_change(id, true).0
    }

    /// Refreshes an already-tracked id and emits a local event on meaningful changes.
    pub fn refresh_tracked_with_events<F>(
        &mut self,
        id: ObjectId,
        mut on_event: F,
    ) -> Option<AttributeValue>
    where
        F: FnMut(DerivedChange),
    {
        let (current, change) = self.refresh_tracked_change(id, true);
        if let Some(change) = change {
            on_event(change);
        }
        current
    }

    fn refresh_tracked_change(
        &mut self,
        id: ObjectId,
        notify: bool,
    ) -> (Option<AttributeValue>, Option<DerivedChange>) {
        let current = self.get(id);
        let Some(previous) = self.tracked.get(id).copied() else {
            return (current, None);
        };

        match current {
            Some(value) if previous == value => return (Some(value), None),
            Some(value) => {
                debug_assert!(self.tracked.replace_existing(id, value));
            }
            None => {
                self.tracked.remove(id);
            }
        }

        let change = DerivedChange {
            id,
            previous: Some(previous),
            current,
        };
        if notify {
            self.notify(change);
        }
        (current, Some(change))
    }

    fn commit(
        &mut self,
        id: ObjectId,
        current: Option<AttributeValue>,
        notify: bool,
    ) -> Option<DerivedChange> {
        let previous = self.tracked.get(id).copied();

        match (previous, current) {
            (Some(previous), Some(current)) if previous == current => return None,
            (Some(_), Some(current)) => self.tracked.put(id, current),
            (Some(_), None) => {
                self.tracked.remove(id);
            }
            (None, Some(current)) => self.tracked.put(id, current),
            (None, None) => return None,
        }

        let change = DerivedChange {
            id,
            previous,
            current,
        };
        if notify {
            self.notify(change);
        }
        Some(change)
    }

    fn notify(&mut self, change: DerivedChange) {
        for listener in &mut self.listeners {
            listener(&change);
        }
    }
}
