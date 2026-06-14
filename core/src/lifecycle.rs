//! Domain-neutral lifecycle events and caller-owned event channels.

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
