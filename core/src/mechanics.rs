//! Domain-neutral mechanics lifecycle driver.
//!
//! `MechanicsDriver` advances registered stores and returns or streams raw
//! lifecycle facts. It does not project Signals, look up channel keys, or
//! publish events. Caller-owned code can pass `tick_with` a closure that
//! projects, publishes, exports, or adapts emitted facts.

use crate::ability::AbilityStore;
use crate::clock::{Clock, ClockUnits};
use crate::effect::{EffectLifecycleEvent, EffectPipeline};
use crate::lifecycle::LocalLifecycleEvent;
use crate::tag::TagCollection;

/// A mechanics store that can advance itself for one elapsed tick.
pub trait MechanicsStore<Event> {
    fn tick_mechanics(&mut self, elapsed_units: ClockUnits, emit: &mut dyn FnMut(Event));
}

/// Tick driver for caller-registered mechanics stores.
pub struct MechanicsDriver<'store, Event> {
    stores: Vec<&'store mut dyn MechanicsStore<Event>>,
}

impl<'store, Event> MechanicsDriver<'store, Event> {
    /// Creates an empty mechanics driver.
    #[must_use]
    pub fn new() -> Self {
        Self { stores: Vec::new() }
    }

    /// Registers a store to tick when the driver advances.
    pub fn register<Store>(&mut self, store: &'store mut Store)
    where
        Store: MechanicsStore<Event> + 'store,
    {
        self.stores.push(store);
    }

    /// Registers a store and returns the driver for fluent setup.
    #[must_use]
    pub fn with_store<Store>(mut self, store: &'store mut Store) -> Self
    where
        Store: MechanicsStore<Event> + 'store,
    {
        self.register(store);
        self
    }

    /// Advances registered stores and returns emitted lifecycle events.
    ///
    /// Returned facts are not projected or published to channels.
    #[must_use]
    pub fn tick(self, elapsed_units: ClockUnits) -> Vec<Event> {
        let mut events = Vec::new();
        self.tick_with(elapsed_units, |event| events.push(event));
        events
    }

    /// Converts a caller-owned clock step, advances registered stores, and
    /// returns emitted lifecycle events.
    #[must_use]
    pub fn tick_clock<C>(self, clock: &C, elapsed: C::Step) -> Vec<Event>
    where
        C: Clock,
    {
        self.tick(clock.units_for(elapsed))
    }

    /// Advances registered stores and streams emitted lifecycle events.
    ///
    /// Use the callback to publish to `EventChannel`, project Signals, or adapt
    /// facts to an external runtime.
    pub fn tick_with<F>(mut self, elapsed_units: ClockUnits, mut emit: F)
    where
        F: FnMut(Event),
    {
        for store in &mut self.stores {
            store.tick_mechanics(elapsed_units, &mut emit);
        }
    }

    /// Converts a caller-owned clock step, advances registered stores, and
    /// streams emitted lifecycle events.
    pub fn tick_clock_with<C, F>(self, clock: &C, elapsed: C::Step, emit: F)
    where
        C: Clock,
        F: FnMut(Event),
    {
        self.tick_with(clock.units_for(elapsed), emit);
    }
}

impl<'store, Event> Default for MechanicsDriver<'store, Event> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Tags, Cost, Payload, Event> MechanicsStore<Event> for AbilityStore<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    fn tick_mechanics(&mut self, elapsed_units: ClockUnits, _emit: &mut dyn FnMut(Event)) {
        self.tick_cooldowns(elapsed_units);
    }
}

impl<Tags, Payload> MechanicsStore<EffectLifecycleEvent<Tags, Payload>>
    for EffectPipeline<Tags, Payload>
where
    Tags: Clone + TagCollection,
    Payload: Clone,
{
    fn tick_mechanics(
        &mut self,
        elapsed_units: ClockUnits,
        emit: &mut dyn FnMut(EffectLifecycleEvent<Tags, Payload>),
    ) {
        self.tick_with_events(elapsed_units, emit);
    }
}

impl<Tags, Payload> MechanicsStore<LocalLifecycleEvent<Tags, Payload>>
    for EffectPipeline<Tags, Payload>
where
    Tags: Clone + TagCollection,
    Payload: Clone,
{
    fn tick_mechanics(
        &mut self,
        elapsed_units: ClockUnits,
        emit: &mut dyn FnMut(LocalLifecycleEvent<Tags, Payload>),
    ) {
        self.tick_with_events(elapsed_units, |event| {
            emit(LocalLifecycleEvent::Effect(event));
        });
    }
}
