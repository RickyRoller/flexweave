//! Domain-neutral Signal projection from mechanics lifecycle facts.
//!
//! Signals are derived facts for export, runtime reactions, or author-defined
//! semantics. They do not replace source lifecycle facts. The current projection
//! API works from effect lifecycle facts and active effect instances, and caller
//! code owns any publication into [`crate::lifecycle::EventChannel`] or an
//! external runtime bus.

mod definition;
mod facts;
mod projection;

pub use definition::{
    SignalDefinition, SignalDefinitionError, SignalDefinitions, SignalExportPolicy, SignalKind,
    SignalRetentionPolicy, SignalTagMatch,
};
pub use facts::{SignalFact, SignalRemovalReason};
pub use projection::SignalProjection;
