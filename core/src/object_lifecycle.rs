//! Domain-neutral object destruction and object-keyed cleanup driver.

use crate::ability::{
    AbilityLifecycleEvent, AbilityRevokeOwner, AbilityStore, OwnedAbilityLifecycleEvents,
};
use crate::attribute::Attribute;
use crate::data_store::DataStore;
use crate::derived_attribute::DerivedAttribute;
use crate::effect::{
    EffectLifecycleEvent, EffectObjectRemovalPolicy, EffectPipeline, EffectRemoveForObject,
    OwnedEffectLifecycleEvents,
};
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

/// Object destruction command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectDestroy {
    id: ObjectId,
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

    fn destroy_object<F>(mut self, id: ObjectId, mut emit: F) -> Result<ObjectId, CoreError>
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

impl ObjectDestroy {
    #[must_use]
    pub const fn new(id: ObjectId) -> Self {
        Self { id }
    }

    /// Destroys an object and returns cleanup lifecycle events in registration order.
    pub fn run<Event>(
        self,
        driver: ObjectDestructionDriver<'_, Event>,
    ) -> Result<Vec<Event>, CoreError> {
        let mut events = Vec::new();
        self.run_streaming(driver, |event| events.push(event))?;
        Ok(events)
    }

    /// Destroys an object and streams cleanup lifecycle events.
    pub fn run_streaming<Event, F>(
        self,
        driver: ObjectDestructionDriver<'_, Event>,
        emit: F,
    ) -> Result<ObjectId, CoreError>
    where
        F: FnMut(Event),
    {
        driver.destroy_object(self.id, emit)
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

impl<Tags, Payload> ObjectLifecycleStore<AbilityLifecycleEvent<Tags, Payload>>
    for AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
    Payload: Clone,
{
    fn remove_object(
        &mut self,
        id: ObjectId,
        emit: &mut dyn FnMut(AbilityLifecycleEvent<Tags, Payload>),
    ) {
        let mut sink = OwnedAbilityLifecycleEvents::new(emit);
        AbilityRevokeOwner::new(id).run_with_sink(self, &mut sink);
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
        let mut sink = OwnedEffectLifecycleEvents::new(emit);
        EffectRemoveForObject::new(id, EffectObjectRemovalPolicy::SourceOrTarget)
            .run_with_sink(self, &mut sink);
    }
}
