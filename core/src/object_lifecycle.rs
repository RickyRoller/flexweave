//! Domain-neutral object destruction and object-keyed cleanup driver.

use crate::ability::{AbilityLifecycleEvent, AbilityStore};
use crate::attribute::Attribute;
use crate::data_store::DataStore;
use crate::derived_attribute::DerivedAttribute;
use crate::effect::{EffectLifecycleEvent, EffectObjectRemovalPolicy, EffectPipeline};
use crate::errors::CoreError;
use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;

/// A store that can remove Flexweave-owned state keyed by an object id.
pub trait ObjectLifecycleStore<Event> {
    fn remove_object(&mut self, id: ObjectId, emit: &mut dyn FnMut(Event));
}

/// Coordinates object destruction with caller-registered cleanup stores.
pub struct ObjectDestructionDriver<'store, Event> {
    objects: &'store mut ObjectStore,
    stores: Vec<&'store mut dyn ObjectLifecycleStore<Event>>,
}

impl<'store, Event> ObjectDestructionDriver<'store, Event> {
    /// Creates a cleanup driver for one object store.
    #[must_use]
    pub fn new(objects: &'store mut ObjectStore) -> Self {
        Self {
            objects,
            stores: Vec::new(),
        }
    }

    /// Registers a store to clean up after object destruction succeeds.
    pub fn register<Store>(&mut self, store: &'store mut Store)
    where
        Store: ObjectLifecycleStore<Event> + 'store,
    {
        self.stores.push(store);
    }

    /// Registers a store and returns the driver for fluent setup.
    #[must_use]
    pub fn with_store<Store>(mut self, store: &'store mut Store) -> Self
    where
        Store: ObjectLifecycleStore<Event> + 'store,
    {
        self.register(store);
        self
    }

    /// Destroys an object and returns cleanup lifecycle events in registration order.
    pub fn destroy(self, id: ObjectId) -> Result<Vec<Event>, CoreError> {
        let mut events = Vec::new();
        self.destroy_with(id, |event| events.push(event))?;
        Ok(events)
    }

    /// Destroys an object and streams cleanup lifecycle events.
    pub fn destroy_with<F>(mut self, id: ObjectId, mut emit: F) -> Result<ObjectId, CoreError>
    where
        F: FnMut(Event),
    {
        let destroyed = self.objects.destroy(id)?;
        for store in &mut self.stores {
            store.remove_object(destroyed, &mut emit);
        }
        Ok(destroyed)
    }
}

impl<T, Event> ObjectLifecycleStore<Event> for DataStore<T> {
    fn remove_object(&mut self, id: ObjectId, _emit: &mut dyn FnMut(Event)) {
        self.detach(id);
    }
}

impl<Event> ObjectLifecycleStore<Event> for Attribute {
    fn remove_object(&mut self, id: ObjectId, _emit: &mut dyn FnMut(Event)) {
        self.detach(id);
    }
}

impl<Event> ObjectLifecycleStore<Event> for DerivedAttribute {
    fn remove_object(&mut self, id: ObjectId, _emit: &mut dyn FnMut(Event)) {
        self.untrack(id);
    }
}

impl<Tags, Payload, BlockReason>
    ObjectLifecycleStore<AbilityLifecycleEvent<Tags, Payload, BlockReason>>
    for AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
    Payload: Clone,
    BlockReason: Clone,
{
    fn remove_object(
        &mut self,
        id: ObjectId,
        emit: &mut dyn FnMut(AbilityLifecycleEvent<Tags, Payload, BlockReason>),
    ) {
        self.revoke_owner_with_events(id, emit);
    }
}

impl<Tags, Payload> ObjectLifecycleStore<EffectLifecycleEvent<Tags, Payload>>
    for EffectPipeline<Tags, Payload>
where
    Tags: TagCollection,
    Payload: Clone,
{
    fn remove_object(
        &mut self,
        id: ObjectId,
        emit: &mut dyn FnMut(EffectLifecycleEvent<Tags, Payload>),
    ) {
        self.remove_for_object_with_events(id, EffectObjectRemovalPolicy::SourceOrTarget, emit);
    }
}
