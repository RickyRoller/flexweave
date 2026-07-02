use crate::effect::{ActiveEffectId, EffectInstance, EffectLifecycleEvent};
use crate::identity::ObjectId;
use crate::lifecycle::{LifecycleEvent, LifecycleEventKind};
use crate::tag::TagSet;

use super::definition::{SignalDefinitions, SignalKind};
use super::facts::{SignalFact, SignalRemovalReason};

/// Signal projection engine over validated definitions.
///
/// Projection converts source lifecycle facts into derived [`SignalFact`]s.
/// It does not publish those facts to `EventChannel`s or any external bus.
/// Caller code owns publication after projection.
pub struct SignalProjection<Atom, SignalPayload> {
    definitions: SignalDefinitions<Atom, SignalPayload>,
}

impl<Atom, SignalPayload> SignalProjection<Atom, SignalPayload> {
    #[must_use]
    pub fn new(definitions: SignalDefinitions<Atom, SignalPayload>) -> Self {
        Self { definitions }
    }
}

impl<Atom, SignalPayload> SignalProjection<Atom, SignalPayload>
where
    Atom: Clone + Eq,
    SignalPayload: Clone,
{
    /// Projects an effect lifecycle event into matching Signal facts.
    ///
    /// The returned facts are inert until caller code publishes or exports them.
    #[must_use]
    pub fn project_effect_event<SourcePayload>(
        &self,
        event: &EffectLifecycleEvent<TagSet<Atom>, SourcePayload>,
    ) -> Vec<SignalFact<Atom, SignalPayload, SourcePayload>>
    where
        SourcePayload: Clone,
    {
        self.project_context(signal_context_from_effect_event(event))
    }

    /// Reinvokes while-active Signal facts for currently active pipeline effects.
    ///
    /// Only definitions that declare [`LifecycleEventKind::SignalReinvoked`]
    /// are eligible for reinvocation projection.
    ///
    /// The returned facts are not automatically routed.
    #[must_use]
    pub fn reinvoke_effect_instances<'a, SourcePayload, I>(
        &self,
        effects: I,
    ) -> Vec<SignalFact<Atom, SignalPayload, SourcePayload>>
    where
        Atom: 'a,
        SourcePayload: Clone + 'a,
        I: IntoIterator<Item = &'a EffectInstance<TagSet<Atom>, SourcePayload>>,
    {
        let mut facts = Vec::new();
        for effect in effects {
            let context = SignalSourceContext {
                source_lifecycle_event_kind: LifecycleEventKind::SignalReinvoked,
                source_definition_key: effect.definition_key.clone(),
                source_id: effect.source_id,
                target_id: effect.target_id,
                owner_id: None,
                active_effect_id: Some(effect.id),
                clock_units: effect.remaining_units,
                removal_reason: None,
                tags: effect.tags.clone(),
                source_payload: Some(effect.payload.clone()),
            };
            facts.extend(self.project_context_for_kind(context, SignalKind::WhileActive));
        }
        facts
    }

    fn project_context<SourcePayload>(
        &self,
        context: SignalSourceContext<Atom, SourcePayload>,
    ) -> Vec<SignalFact<Atom, SignalPayload, SourcePayload>>
    where
        SourcePayload: Clone,
    {
        let signal_kind =
            default_signal_kind(context.source_lifecycle_event_kind, context.removal_reason);
        self.project_context_for_kind(context, signal_kind)
    }

    fn project_context_for_kind<SourcePayload>(
        &self,
        context: SignalSourceContext<Atom, SourcePayload>,
        signal_kind: SignalKind,
    ) -> Vec<SignalFact<Atom, SignalPayload, SourcePayload>>
    where
        SourcePayload: Clone,
    {
        let mut facts = Vec::new();
        for definition in self.definitions.definitions() {
            if definition.signal_kind != signal_kind {
                continue;
            }
            if !definition
                .lifecycle_event_kinds
                .contains(&context.source_lifecycle_event_kind)
            {
                continue;
            }
            if !definition.tag_match.matches(&context.tags) {
                continue;
            }
            facts.push(SignalFact {
                key: definition.key.clone(),
                signal_kind: definition.signal_kind,
                channel_key: definition.channel_key.clone(),
                category: definition.category.clone(),
                retention: definition.retention,
                export: definition.export,
                source_lifecycle_event_kind: context.source_lifecycle_event_kind,
                source_definition_key: context.source_definition_key.clone(),
                source_id: context.source_id,
                target_id: context.target_id,
                owner_id: context.owner_id,
                active_effect_id: context.active_effect_id,
                clock_units: context.clock_units,
                removal_reason: context.removal_reason,
                tags: context.tags.clone(),
                signal_payload: definition.signal_payload.clone(),
                source_payload: context.source_payload.clone(),
            });
        }
        facts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SignalSourceContext<Atom, SourcePayload> {
    source_lifecycle_event_kind: LifecycleEventKind,
    source_definition_key: Option<String>,
    source_id: Option<ObjectId>,
    target_id: ObjectId,
    owner_id: Option<ObjectId>,
    active_effect_id: Option<ActiveEffectId>,
    clock_units: Option<u64>,
    removal_reason: Option<SignalRemovalReason>,
    tags: TagSet<Atom>,
    source_payload: Option<SourcePayload>,
}

fn signal_context_from_effect_event<Atom, SourcePayload>(
    event: &EffectLifecycleEvent<TagSet<Atom>, SourcePayload>,
) -> SignalSourceContext<Atom, SourcePayload>
where
    Atom: Clone + Eq,
    SourcePayload: Clone,
{
    match event {
        EffectLifecycleEvent::ApplicationAccepted(application) => SignalSourceContext {
            source_lifecycle_event_kind: event.lifecycle_event_kind(),
            source_definition_key: application.definition_key.clone(),
            source_id: application.source_id,
            target_id: application.target_id,
            owner_id: None,
            active_effect_id: None,
            clock_units: None,
            removal_reason: None,
            tags: application.tags.clone(),
            source_payload: Some(application.payload.clone()),
        },
        EffectLifecycleEvent::ApplicationRejected(rejection) => SignalSourceContext {
            source_lifecycle_event_kind: event.lifecycle_event_kind(),
            source_definition_key: rejection.application.definition_key.clone(),
            source_id: rejection.application.source_id,
            target_id: rejection.application.target_id,
            owner_id: None,
            active_effect_id: None,
            clock_units: None,
            removal_reason: None,
            tags: rejection.application.tags.clone(),
            source_payload: Some(rejection.application.payload.clone()),
        },
        EffectLifecycleEvent::ActiveCreated(effect)
        | EffectLifecycleEvent::Removed(effect)
        | EffectLifecycleEvent::Expired(effect) => SignalSourceContext {
            source_lifecycle_event_kind: event.lifecycle_event_kind(),
            source_definition_key: effect.definition_key.clone(),
            source_id: effect.source_id,
            target_id: effect.target_id,
            owner_id: None,
            active_effect_id: Some(effect.id),
            clock_units: effect.remaining_units,
            removal_reason: match event {
                EffectLifecycleEvent::Removed(_) => Some(SignalRemovalReason::Removed),
                EffectLifecycleEvent::Expired(_) => Some(SignalRemovalReason::Expired),
                _ => None,
            },
            tags: effect.tags.clone(),
            source_payload: Some(effect.payload.clone()),
        },
        EffectLifecycleEvent::Executed(execution)
        | EffectLifecycleEvent::PeriodicExecuted(execution) => SignalSourceContext {
            source_lifecycle_event_kind: event.lifecycle_event_kind(),
            source_definition_key: execution.definition_key.clone(),
            source_id: execution.source_id,
            target_id: execution.target_id,
            owner_id: None,
            active_effect_id: execution.active_effect_id,
            clock_units: execution.elapsed_units,
            removal_reason: None,
            tags: execution.tags.clone(),
            source_payload: Some(execution.payload.clone()),
        },
        EffectLifecycleEvent::Advanced(advance) => SignalSourceContext {
            source_lifecycle_event_kind: event.lifecycle_event_kind(),
            source_definition_key: advance.effect.definition_key.clone(),
            source_id: advance.effect.source_id,
            target_id: advance.effect.target_id,
            owner_id: None,
            active_effect_id: Some(advance.effect.id),
            clock_units: Some(advance.elapsed_units),
            removal_reason: None,
            tags: advance.effect.tags.clone(),
            source_payload: Some(advance.effect.payload.clone()),
        },
    }
}

fn default_signal_kind(
    kind: LifecycleEventKind,
    removal_reason: Option<SignalRemovalReason>,
) -> SignalKind {
    match kind {
        LifecycleEventKind::EffectApplicationAccepted | LifecycleEventKind::EffectActiveCreated => {
            SignalKind::ActiveStart
        }
        LifecycleEventKind::EffectAdvanced => SignalKind::WhileActive,
        LifecycleEventKind::EffectExecuted => SignalKind::Executed,
        LifecycleEventKind::EffectPeriodicExecuted => SignalKind::Recurring,
        LifecycleEventKind::EffectRemoved | LifecycleEventKind::EffectExpired => {
            let _ = removal_reason;
            SignalKind::Removed
        }
        _ => SignalKind::Executed,
    }
}
