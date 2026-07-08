//! Domain-neutral mechanics lifecycle driver.
//!
//! `MechanicsDriver` advances registered stores and returns or streams raw
//! lifecycle facts. It does not project Signals, look up channel keys, or
//! publish events. Caller-owned code can pass `MechanicsTick::run_streaming` a closure that
//! projects, publishes, exports, or adapts emitted facts.

use crate::clock::{Clock, ClockUnits};
use crate::effect::{EffectLifecycleEvent, EffectPipeline, EffectTick, NoEffectExecutor};
use crate::lifecycle::LocalLifecycleEvent;
use crate::tag::TagCollection;
use std::convert::Infallible;

/// A mechanics store that can advance itself for one elapsed tick.
pub trait MechanicsStore<Event> {
    fn tick_mechanics(&mut self, elapsed_units: ClockUnits, emit: &mut dyn FnMut(Event));
}

/// Tick driver for caller-registered mechanics stores.
pub struct MechanicsDriver<'store, Event> {
    stores: Vec<&'store mut dyn MechanicsStore<Event>>,
}

/// Mechanics tick command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MechanicsTick {
    elapsed_units: ClockUnits,
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

    fn tick_stores<F>(mut self, elapsed_units: ClockUnits, mut emit: F)
    where
        F: FnMut(Event),
    {
        for store in &mut self.stores {
            store.tick_mechanics(elapsed_units, &mut emit);
        }
    }
}

impl MechanicsTick {
    #[must_use]
    pub const fn new(elapsed_units: ClockUnits) -> Self {
        Self { elapsed_units }
    }

    #[must_use]
    pub fn from_clock<C>(clock: &C, elapsed: C::Step) -> Self
    where
        C: Clock,
    {
        Self::new(clock.units_for(elapsed))
    }

    /// Advances registered stores and returns emitted lifecycle events.
    ///
    /// Returned facts are not projected or published to channels.
    #[must_use]
    pub fn run<Event>(self, driver: MechanicsDriver<'_, Event>) -> Vec<Event> {
        let mut events = Vec::new();
        self.run_streaming(driver, |event| events.push(event));
        events
    }

    /// Advances registered stores and streams emitted lifecycle events.
    ///
    /// Use the callback to publish to `EventChannel`, project Signals, or adapt
    /// facts to an external runtime.
    pub fn run_streaming<Event, F>(self, driver: MechanicsDriver<'_, Event>, emit: F)
    where
        F: FnMut(Event),
    {
        driver.tick_stores(self.elapsed_units, emit);
    }
}

impl<'store, Event> Default for MechanicsDriver<'store, Event> {
    fn default() -> Self {
        Self::new()
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
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(emit);
        EffectTick::new(elapsed_units)
            .run_with_executor(self, &mut context, &mut executor)
            .unwrap_or_else(infallible_error);
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
        let mut context = ();
        let mut executor = NoEffectExecutor::new().with_owned_events(|event| {
            emit(LocalLifecycleEvent::Effect(event));
        });
        EffectTick::new(elapsed_units)
            .run_with_executor(self, &mut context, &mut executor)
            .unwrap_or_else(infallible_error);
    }
}

fn infallible_error<T>(error: Infallible) -> T {
    match error {}
}
