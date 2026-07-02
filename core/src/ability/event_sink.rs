use crate::tag::TagCollection;

use super::events::{AbilityLifecycleEvent, AbilityLifecycleEventView};

pub(super) fn discard_lifecycle_event<Tags, Payload>(
    _event: AbilityLifecycleEventView<'_, Tags, Payload>,
) where
    Tags: TagCollection,
{
}

pub(super) fn owned_lifecycle_events<'emit, Tags, Payload, F>(
    emit: &'emit mut F,
) -> impl for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>) + 'emit
where
    Tags: TagCollection + 'emit,
    Payload: Clone + 'emit,
    F: FnMut(AbilityLifecycleEvent<Tags, Payload>) + 'emit,
{
    move |event| emit(event.to_owned_event())
}
