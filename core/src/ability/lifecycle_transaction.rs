use crate::tag::TagCollection;

use super::events::{AbilityLifecycleEventView, ActiveAbility, ActiveAbilityView};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ActiveAbilityTransition {
    Started,
    Committed,
    Canceled,
    Revoked,
    RolledBack,
    Ended,
}

impl ActiveAbilityTransition {
    pub(super) fn event<'event, Tags, Payload>(
        self,
        active: &'event ActiveAbility<Tags, Payload>,
    ) -> AbilityLifecycleEventView<'event, Tags, Payload>
    where
        Tags: TagCollection,
    {
        let active = ActiveAbilityView::from(active);
        match self {
            Self::Started => AbilityLifecycleEventView::Started(active),
            Self::Committed => AbilityLifecycleEventView::Committed(active),
            Self::Canceled => AbilityLifecycleEventView::Canceled(active),
            Self::Revoked => AbilityLifecycleEventView::Revoked(active),
            Self::RolledBack => AbilityLifecycleEventView::RolledBack(active),
            Self::Ended => AbilityLifecycleEventView::Ended(active),
        }
    }
}

pub(super) fn emit_active_transition<Tags, Payload, F>(
    transition: ActiveAbilityTransition,
    active: &ActiveAbility<Tags, Payload>,
    emit: &mut F,
) where
    Tags: TagCollection,
    F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
{
    emit(transition.event(active));
}
