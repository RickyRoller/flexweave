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

/// Derived attribute refresh command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedAttributeRefresh {
    id: ObjectId,
    mode: DerivedAttributeRefreshMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DerivedAttributeRefreshMode {
    Refresh,
    TrackedOnly,
}

impl DerivedAttributeRefresh {
    #[must_use]
    pub const fn new(id: ObjectId) -> Self {
        Self {
            id,
            mode: DerivedAttributeRefreshMode::Refresh,
        }
    }

    #[must_use]
    pub const fn tracked(id: ObjectId) -> Self {
        Self {
            id,
            mode: DerivedAttributeRefreshMode::TrackedOnly,
        }
    }

    #[must_use]
    pub const fn tracked_only(mut self) -> Self {
        self.mode = DerivedAttributeRefreshMode::TrackedOnly;
        self
    }

    pub fn run(self, attribute: &mut DerivedAttribute) -> Option<AttributeValue> {
        self.run_streaming(attribute, |_| {})
    }

    pub fn run_streaming<F>(
        self,
        attribute: &mut DerivedAttribute,
        mut emit: F,
    ) -> Option<AttributeValue>
    where
        F: FnMut(DerivedChange),
    {
        let (current, change) = match self.mode {
            DerivedAttributeRefreshMode::Refresh => attribute.refresh_change(self.id, true),
            DerivedAttributeRefreshMode::TrackedOnly => {
                attribute.refresh_tracked_change(self.id, true)
            }
        };
        if let Some(change) = change {
            emit(change);
        }
        current
    }
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

    fn refresh_change(
        &mut self,
        id: ObjectId,
        notify: bool,
    ) -> (Option<AttributeValue>, Option<DerivedChange>) {
        let current = self.get(id);
        let change = self.commit(id, current, notify);
        (current, change)
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
