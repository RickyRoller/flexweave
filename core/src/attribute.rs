//! Reactive signed floating-point attributes.

use crate::identity::ObjectId;
use crate::object_map::ObjectMap;

/// Signed numeric attribute value.
pub type AttributeValue = f64;

/// Attribute change visible to listeners after commit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttributeChange {
    pub id: ObjectId,
    pub previous: Option<AttributeValue>,
    pub requested: AttributeValue,
    pub current: AttributeValue,
}

impl AttributeChange {
    /// Difference between current and previous, treating missing previous as 0.
    #[must_use]
    pub fn delta(self) -> AttributeValue {
        self.current - self.previous.unwrap_or(0.0)
    }
}

/// One requested Attribute mutation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttributeMutationRequest {
    pub id: ObjectId,
    pub requested: AttributeValue,
}

/// Pre-mutation hook view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttributeMutation<'a, Context> {
    pub id: ObjectId,
    pub previous: Option<AttributeValue>,
    pub requested: AttributeValue,
    pub current: AttributeValue,
    pub context: &'a Context,
}

/// Pre-mutation hook decision.
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeMutationDecision<Error> {
    Allow,
    Transform(AttributeValue),
    Reject(Error),
}

/// Rejected Attribute mutation details.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeMutationRejection<Error> {
    pub id: ObjectId,
    pub previous: Option<AttributeValue>,
    pub requested: AttributeValue,
    pub current: AttributeValue,
    pub reason: Error,
}

/// Attribute mutation result.
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeMutationResult<Error> {
    Unchanged(AttributeValue),
    Committed(AttributeChange),
    Rejected(AttributeMutationRejection<Error>),
}

type Listener = Box<dyn FnMut(&AttributeChange) + Send>;
type PreHook<Context, Error> =
    Box<dyn FnMut(AttributeMutation<'_, Context>) -> AttributeMutationDecision<Error> + Send>;
type PostHook<Context> = Box<dyn FnMut(&Context, &AttributeChange) + Send>;

/// Ordered pre/post mutation hooks for an Attribute channel.
pub struct AttributeMutationHooks<Context, Error> {
    pre_hooks: Vec<PreHook<Context, Error>>,
    post_hooks: Vec<PostHook<Context>>,
}

impl<Context, Error> AttributeMutationHooks<Context, Error> {
    /// Creates an empty hook collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }

    /// Adds a pre-mutation hook in deterministic registration order.
    pub fn add_pre_hook<F>(&mut self, hook: F)
    where
        F: FnMut(AttributeMutation<'_, Context>) -> AttributeMutationDecision<Error>
            + Send
            + 'static,
    {
        self.pre_hooks.push(Box::new(hook));
    }

    /// Adds a post-commit hook in deterministic registration order.
    pub fn add_post_hook<F>(&mut self, hook: F)
    where
        F: FnMut(&Context, &AttributeChange) + Send + 'static,
    {
        self.post_hooks.push(Box::new(hook));
    }
}

impl<Context, Error> Default for AttributeMutationHooks<Context, Error> {
    fn default() -> Self {
        Self::new()
    }
}

/// Object-keyed attribute channel.
#[derive(Default)]
pub struct Attribute {
    values: ObjectMap<AttributeValue>,
    listeners: Vec<Listener>,
}

impl Attribute {
    /// Creates an empty attribute channel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: ObjectMap::new(),
            listeners: Vec::new(),
        }
    }

    /// Seeds or overwrites a value without notifying listeners.
    pub fn attach(&mut self, id: ObjectId, value: AttributeValue) {
        self.values.put(id, value);
    }

    /// Detaches the stored value for `id` without emitting an attribute-change fact.
    pub fn detach(&mut self, id: ObjectId) -> bool {
        self.values.remove(id)
    }

    /// Registers a listener in deterministic registration order.
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: FnMut(&AttributeChange) + Send + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Alias for `add_listener`.
    pub fn subscribe<F>(&mut self, listener: F)
    where
        F: FnMut(&AttributeChange) + Send + 'static,
    {
        self.add_listener(listener);
    }

    /// Returns true when `id` has a value.
    #[must_use]
    pub fn has(&self, id: ObjectId) -> bool {
        self.values.contains(id)
    }

    /// Returns the current value for `id`.
    #[must_use]
    pub fn get(&self, id: ObjectId) -> Option<AttributeValue> {
        self.values.get(id).copied()
    }

    /// Number of attached values.
    #[must_use]
    pub fn count(&self) -> usize {
        self.values.count()
    }

    /// Commits `requested` and notifies listeners only when the value changes.
    pub fn set(&mut self, id: ObjectId, requested: AttributeValue) -> AttributeValue {
        self.commit_change(id, requested, requested);
        requested
    }

    /// Commits `requested`, notifies existing listeners, and emits a local event
    /// only when the value changes.
    pub fn set_with_events<F>(
        &mut self,
        id: ObjectId,
        requested: AttributeValue,
        mut on_event: F,
    ) -> AttributeValue
    where
        F: FnMut(AttributeChange),
    {
        if let Some(change) = self.commit_change(id, requested, requested) {
            on_event(change);
        }
        requested
    }

    /// Runs pre-mutation hooks, commits the final value if it changed, then
    /// notifies listeners and post-commit hooks.
    pub fn set_with_hooks<Context, Error>(
        &mut self,
        request: AttributeMutationRequest,
        context: &Context,
        hooks: &mut AttributeMutationHooks<Context, Error>,
    ) -> AttributeMutationResult<Error> {
        self.set_with_hooks_and_events(request, context, hooks, |_| {})
    }

    /// Runs hook-bearing mutation and emits a committed change fact when storage changes.
    pub fn set_with_hooks_and_events<Context, Error, F>(
        &mut self,
        request: AttributeMutationRequest,
        context: &Context,
        hooks: &mut AttributeMutationHooks<Context, Error>,
        mut on_event: F,
    ) -> AttributeMutationResult<Error>
    where
        F: FnMut(AttributeChange),
    {
        let previous = self.get(request.id);
        let mut current = request.requested;

        for hook in &mut hooks.pre_hooks {
            match hook(AttributeMutation {
                id: request.id,
                previous,
                requested: request.requested,
                current,
                context,
            }) {
                AttributeMutationDecision::Allow => {}
                AttributeMutationDecision::Transform(transformed) => {
                    current = transformed;
                }
                AttributeMutationDecision::Reject(reason) => {
                    return AttributeMutationResult::Rejected(AttributeMutationRejection {
                        id: request.id,
                        previous,
                        requested: request.requested,
                        current,
                        reason,
                    });
                }
            }
        }

        let Some(change) = self.commit_change(request.id, request.requested, current) else {
            return AttributeMutationResult::Unchanged(current);
        };

        for hook in &mut hooks.post_hooks {
            hook(context, &change);
        }
        on_event(change);
        AttributeMutationResult::Committed(change)
    }

    fn commit_change(
        &mut self,
        id: ObjectId,
        requested: AttributeValue,
        current: AttributeValue,
    ) -> Option<AttributeChange> {
        let previous = self.get(id);
        if previous == Some(current) {
            return None;
        }

        self.attach(id, current);
        let change = AttributeChange {
            id,
            previous,
            requested,
            current,
        };
        self.notify(&change);
        Some(change)
    }

    fn notify(&mut self, change: &AttributeChange) {
        for listener in &mut self.listeners {
            listener(change);
        }
    }
}
