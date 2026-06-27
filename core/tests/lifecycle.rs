use flexweave::{
    EventChannel, EventChannelDefinition, EventChannelError, EventRetention, LifecycleEvent,
    LifecycleEventKind,
};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct NonClonePayload {
    amount: i32,
}

#[derive(Debug)]
struct NonCloneEvent {
    payload: NonClonePayload,
}

impl LifecycleEvent for NonCloneEvent {
    fn lifecycle_event_kind(&self) -> LifecycleEventKind {
        LifecycleEventKind::EffectExecuted
    }
}

#[test]
fn drop_event_channel_publishes_non_clone_events() {
    let mut channel = EventChannel::new(
        EventChannelDefinition::new("effects/drop", [LifecycleEventKind::EffectExecuted]).unwrap(),
    );
    let seen = Arc::new(Mutex::new(Vec::new()));
    let listener_seen = Arc::clone(&seen);

    channel.subscribe(move |event: &NonCloneEvent| {
        listener_seen.lock().unwrap().push(event.payload.amount);
    });

    channel
        .publish(NonCloneEvent {
            payload: NonClonePayload { amount: 5 },
        })
        .unwrap();

    assert_eq!(*seen.lock().unwrap(), vec![5]);
    assert!(channel.retained().is_empty());
}

#[test]
fn drop_event_channel_publishes_borrowed_non_clone_events() {
    let mut channel = EventChannel::new(
        EventChannelDefinition::new("effects/drop", [LifecycleEventKind::EffectExecuted]).unwrap(),
    );
    let seen = Arc::new(Mutex::new(Vec::new()));
    let listener_seen = Arc::clone(&seen);

    channel.subscribe(move |event: &NonCloneEvent| {
        listener_seen.lock().unwrap().push(event.payload.amount);
    });

    let event = NonCloneEvent {
        payload: NonClonePayload { amount: 8 },
    };
    channel.publish_borrowed(&event).unwrap();

    assert_eq!(*seen.lock().unwrap(), vec![8]);
    assert!(channel.retained().is_empty());
}

#[test]
fn retained_event_channel_rejects_borrowed_publication() {
    let mut channel = EventChannel::with_retention(
        EventChannelDefinition::new("effects/retained", [LifecycleEventKind::EffectExecuted])
            .unwrap(),
        EventRetention::Retain,
    );
    let event = NonCloneEvent {
        payload: NonClonePayload { amount: 13 },
    };

    assert_eq!(
        channel.publish_borrowed(&event),
        Err(EventChannelError::BorrowedRetention {
            channel_name: "effects/retained".to_owned(),
            kind: LifecycleEventKind::EffectExecuted,
        })
    );
    assert!(channel.retained().is_empty());
}
