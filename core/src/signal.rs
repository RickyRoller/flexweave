//! Domain-neutral Signal projection from mechanics lifecycle facts.

mod definition;
mod facts;
mod projection;

pub use definition::{
    SignalDefinition, SignalDefinitionError, SignalDefinitions, SignalExportPolicy, SignalKind,
    SignalRetentionPolicy, SignalTagMatch,
};
pub use facts::{SignalFact, SignalRemovalReason};
pub use projection::SignalProjection;
