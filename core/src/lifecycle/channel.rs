use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::definition::EventChannelDefinition;
use super::kind::{LifecycleEvent, LifecycleEventKind};

/// Runtime retention behavior for an event channel.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EventRetention {
    #[default]
    Drop,
    Retain,
}

/// Runtime channel emission failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventChannelError {
    PayloadMismatch {
        channel_name: String,
        kind: LifecycleEventKind,
    },
}

impl fmt::Display for EventChannelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadMismatch { channel_name, kind } => write!(
                formatter,
                "event channel `{channel_name}` does not accept payload kind {kind:?}"
            ),
        }
    }
}

impl std::error::Error for EventChannelError {}

/// Connection handle returned by event channel subscriptions.
#[derive(Clone, Debug)]
pub struct EventConnectionHandle {
    id: u64,
    connected: Arc<AtomicBool>,
}

impl EventConnectionHandle {
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Disconnects this subscription. During emission, this takes effect before
    /// any later listener whose handle has not yet been visited.
    pub fn disconnect(&self) {
        self.connected.store(false, Ordering::SeqCst);
    }
}

/// RAII subscription guard that disconnects when dropped.
#[derive(Debug)]
pub struct ScopedEventConnection {
    handle: EventConnectionHandle,
}

impl ScopedEventConnection {
    #[must_use]
    pub fn handle(&self) -> &EventConnectionHandle {
        &self.handle
    }
}

impl Drop for ScopedEventConnection {
    fn drop(&mut self) {
        self.handle.disconnect();
    }
}

type Listener<Event> = Box<dyn FnMut(&Event) + Send>;

struct EventListener<Event> {
    handle: EventConnectionHandle,
    listener: Listener<Event>,
}

/// Caller-owned event channel for one typed lifecycle event payload.
pub struct EventChannel<Event> {
    definition: EventChannelDefinition,
    retention: EventRetention,
    retained: Vec<Event>,
    listeners: Vec<EventListener<Event>>,
    next_connection_id: u64,
}

impl<Event> EventChannel<Event> {
    /// Creates a channel with no retained event batch.
    #[must_use]
    pub fn new(definition: EventChannelDefinition) -> Self {
        Self::with_retention(definition, EventRetention::Drop)
    }

    #[must_use]
    pub fn with_retention(definition: EventChannelDefinition, retention: EventRetention) -> Self {
        Self {
            definition,
            retention,
            retained: Vec::new(),
            listeners: Vec::new(),
            next_connection_id: 1,
        }
    }

    #[must_use]
    pub fn definition(&self) -> &EventChannelDefinition {
        &self.definition
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.definition.name()
    }

    #[must_use]
    pub fn retention(&self) -> EventRetention {
        self.retention
    }

    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.listeners
            .iter()
            .filter(|listener| listener.handle.is_connected())
            .count()
    }

    /// Registers a listener in deterministic registration order.
    pub fn subscribe<F>(&mut self, listener: F) -> EventConnectionHandle
    where
        F: FnMut(&Event) + Send + 'static,
    {
        let handle = EventConnectionHandle {
            id: self.next_connection_id,
            connected: Arc::new(AtomicBool::new(true)),
        };
        self.next_connection_id += 1;
        self.listeners.push(EventListener {
            handle: handle.clone(),
            listener: Box::new(listener),
        });
        handle
    }

    /// Registers a listener that disconnects when the returned guard is dropped.
    pub fn subscribe_scoped<F>(&mut self, listener: F) -> ScopedEventConnection
    where
        F: FnMut(&Event) + Send + 'static,
    {
        ScopedEventConnection {
            handle: self.subscribe(listener),
        }
    }

    #[must_use]
    pub fn retained(&self) -> &[Event] {
        &self.retained
    }

    /// Drains retained events in emission order.
    pub fn drain_retained(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.retained)
    }

    /// Removes disconnected listeners outside an emission.
    pub fn compact_disconnected(&mut self) {
        self.listeners
            .retain(|listener| listener.handle.is_connected());
    }
}

impl<Event> EventChannel<Event>
where
    Event: Clone + LifecycleEvent,
{
    /// Publishes one event to retained batches and connected listeners.
    pub fn publish(&mut self, event: Event) -> Result<(), EventChannelError> {
        let kind = event.lifecycle_event_kind();
        if !self.definition.accepts(kind) {
            return Err(EventChannelError::PayloadMismatch {
                channel_name: self.name().to_owned(),
                kind,
            });
        }

        if self.retention == EventRetention::Retain {
            self.retained.push(event.clone());
        }

        for listener in &mut self.listeners {
            if listener.handle.is_connected() {
                (listener.listener)(&event);
            }
        }
        self.compact_disconnected();
        Ok(())
    }
}
