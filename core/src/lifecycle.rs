//! Domain-neutral lifecycle events and caller-owned event channels.
//!
//! Lifecycle events are raw mechanics facts. An [`EventChannel`] validates,
//! retains, and notifies facts that caller code publishes into it. Channels do
//! not subscribe to stores or route facts by definition metadata on their own.

mod channel;
mod definition;
mod kind;

pub use channel::{
    EventChannel, EventChannelError, EventConnectionHandle, EventRetention, ScopedEventConnection,
};
pub use definition::{
    EventChannelDefinition, EventChannelDefinitionError, EventChannelDefinitions,
    EventChannelRouteDefinition,
};
pub use kind::{LifecycleEvent, LifecycleEventKind, LocalLifecycleEvent};
