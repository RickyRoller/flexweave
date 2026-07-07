use crate::ability::ActiveAbility;
use crate::clock::ClockUnits;
use crate::identity::ObjectId;
use crate::tag::TagCollection;
use std::convert::Infallible;

use super::definition::EffectClockPolicy;
use super::events::{EffectExecutionView, EffectLifecycleEvent, EffectLifecycleEventView};

/// One effect application attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplication<Tags, Payload>
where
    Tags: TagCollection,
{
    pub definition_key: Option<String>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
}

/// Borrowed view of one effect application attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectApplicationView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub definition_key: Option<&'event str>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: &'event Tags,
    pub payload: &'event Payload,
}

impl<'event, Tags, Payload> EffectApplicationView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_application(&self) -> EffectApplication<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectApplication {
            definition_key: self.definition_key.map(str::to_owned),
            source_id: self.source_id,
            target_id: self.target_id,
            tags: self.tags.clone(),
            payload: self.payload.clone(),
        }
    }
}

/// Runtime application policy selected by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectApplicationDecision {
    Accept,
    Reject { reason: String },
}

/// Policy for applications whose `source_id` is absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectSourcePolicy {
    /// Allow `source_id: None` for environmental or system effects.
    AllowSystemSource,
    /// Require `source_id: Some(_)` and validate that source against the object store.
    RequireLiveSource,
}

/// Application input for the effect pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplicationInput<Tags, Payload>
where
    Tags: TagCollection,
{
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
    pub decision: EffectApplicationDecision,
}

/// Mutable effect application draft exposed to caller-owned initialization logic.
pub struct EffectApplicationDraft<'draft, Tags, Payload>
where
    Tags: TagCollection,
{
    pub definition_key: &'draft str,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: &'draft Tags,
    pub payload: &'draft mut Payload,
    pub duration: &'draft mut Option<EffectClockPolicy>,
    pub period: &'draft mut Option<EffectClockPolicy>,
}

impl<Tags, Payload> EffectApplicationDraft<'_, Tags, Payload>
where
    Tags: TagCollection,
{
    pub fn set_duration_units(&mut self, units: impl Into<Option<ClockUnits>>) {
        *self.duration = units.into().map(EffectClockPolicy::new);
    }

    pub fn set_period_units(&mut self, units: impl Into<Option<ClockUnits>>) {
        *self.period = units.into().map(EffectClockPolicy::new);
    }
}

/// Caller-owned effect application initializer.
pub trait EffectInitializer<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;

    fn initialize(
        &mut self,
        _context: &mut Context,
        _draft: EffectApplicationDraft<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// No-op effect initializer for pipelines that do not need context.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoopEffectInitializer;

impl<Context, Tags, Payload> EffectInitializer<Context, Tags, Payload> for NoopEffectInitializer
where
    Tags: TagCollection,
{
    type Error = std::convert::Infallible;
}

impl<Context, Tags, Payload, Initializer> EffectInitializer<Context, Tags, Payload>
    for &mut Initializer
where
    Tags: TagCollection,
    Initializer: EffectInitializer<Context, Tags, Payload>,
{
    type Error = Initializer::Error;

    fn initialize(
        &mut self,
        context: &mut Context,
        draft: EffectApplicationDraft<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        (**self).initialize(context, draft)
    }
}

/// Synchronous caller-owned action run when an effect execution completes.
pub trait EffectExecutionAction<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;

    fn execute_effect(
        &mut self,
        context: &mut Context,
        execution: EffectExecutionView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error>;
}

impl<Context, Tags, Payload, Error, F> EffectExecutionAction<Context, Tags, Payload> for F
where
    Tags: TagCollection,
    F: for<'event> FnMut(
        &mut Context,
        EffectExecutionView<'event, Tags, Payload>,
    ) -> Result<(), Error>,
{
    type Error = Error;

    fn execute_effect(
        &mut self,
        context: &mut Context,
        execution: EffectExecutionView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        self(context, execution)
    }
}

/// Sink for effect lifecycle facts produced while executing effect pipeline commands.
pub trait EffectLifecycleSink<Tags, Payload>
where
    Tags: TagCollection,
{
    fn emit_effect_lifecycle(&mut self, event: EffectLifecycleEventView<'_, Tags, Payload>);
}

/// Effect lifecycle sink that drops emitted facts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DiscardEffectLifecycleEvents;

impl<Tags, Payload> EffectLifecycleSink<Tags, Payload> for DiscardEffectLifecycleEvents
where
    Tags: TagCollection,
{
    fn emit_effect_lifecycle(&mut self, _event: EffectLifecycleEventView<'_, Tags, Payload>) {}
}

/// Effect lifecycle sink that converts borrowed facts into owned facts.
pub struct OwnedEffectLifecycleEvents<F> {
    emit: F,
}

impl<F> OwnedEffectLifecycleEvents<F> {
    #[must_use]
    pub fn new(emit: F) -> Self {
        Self { emit }
    }
}

impl<Tags, Payload, F> EffectLifecycleSink<Tags, Payload> for OwnedEffectLifecycleEvents<F>
where
    Tags: Clone + TagCollection,
    Payload: Clone,
    F: FnMut(EffectLifecycleEvent<Tags, Payload>),
{
    fn emit_effect_lifecycle(&mut self, event: EffectLifecycleEventView<'_, Tags, Payload>) {
        (self.emit)(event.to_owned_event());
    }
}

impl<Tags, Payload, F> EffectLifecycleSink<Tags, Payload> for F
where
    Tags: TagCollection,
    F: for<'event> FnMut(EffectLifecycleEventView<'event, Tags, Payload>),
{
    fn emit_effect_lifecycle(&mut self, event: EffectLifecycleEventView<'_, Tags, Payload>) {
        self(event);
    }
}

/// Execution participant for effect commands.
pub trait EffectExecutor<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;

    fn execute_effect(
        &mut self,
        context: &mut Context,
        execution: EffectExecutionView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error>;

    fn emit_effect_lifecycle(&mut self, event: EffectLifecycleEventView<'_, Tags, Payload>);
}

/// Executor for effect commands that need lifecycle facts but no caller-owned action.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoEffectExecutor<Sink = DiscardEffectLifecycleEvents> {
    sink: Sink,
}

impl NoEffectExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sink: DiscardEffectLifecycleEvents,
        }
    }
}

impl<Sink> NoEffectExecutor<Sink> {
    #[must_use]
    pub fn with_borrowed_events<F>(self, emit: F) -> NoEffectExecutor<F> {
        NoEffectExecutor { sink: emit }
    }

    #[must_use]
    pub fn with_owned_events<F>(self, emit: F) -> NoEffectExecutor<OwnedEffectLifecycleEvents<F>> {
        NoEffectExecutor {
            sink: OwnedEffectLifecycleEvents::new(emit),
        }
    }
}

impl<Context, Tags, Payload, Sink> EffectExecutor<Context, Tags, Payload> for NoEffectExecutor<Sink>
where
    Tags: TagCollection,
    Sink: EffectLifecycleSink<Tags, Payload>,
{
    type Error = Infallible;

    fn execute_effect(
        &mut self,
        _context: &mut Context,
        _execution: EffectExecutionView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn emit_effect_lifecycle(&mut self, event: EffectLifecycleEventView<'_, Tags, Payload>) {
        self.sink.emit_effect_lifecycle(event);
    }
}

/// Executor that adapts a caller-owned action and optional lifecycle sink.
pub struct EffectActionExecutor<'action, Action, Sink = DiscardEffectLifecycleEvents> {
    action: &'action mut Action,
    sink: Sink,
}

impl<'action, Action> EffectActionExecutor<'action, Action> {
    #[must_use]
    pub fn new(action: &'action mut Action) -> Self {
        Self {
            action,
            sink: DiscardEffectLifecycleEvents,
        }
    }
}

impl<'action, Action, Sink> EffectActionExecutor<'action, Action, Sink> {
    #[must_use]
    pub fn with_borrowed_events<F>(self, emit: F) -> EffectActionExecutor<'action, Action, F> {
        EffectActionExecutor {
            action: self.action,
            sink: emit,
        }
    }

    #[must_use]
    pub fn with_owned_events<F>(
        self,
        emit: F,
    ) -> EffectActionExecutor<'action, Action, OwnedEffectLifecycleEvents<F>> {
        EffectActionExecutor {
            action: self.action,
            sink: OwnedEffectLifecycleEvents::new(emit),
        }
    }
}

impl<Context, Action, Tags, Payload, Sink> EffectExecutor<Context, Tags, Payload>
    for EffectActionExecutor<'_, Action, Sink>
where
    Tags: TagCollection,
    Action: EffectExecutionAction<Context, Tags, Payload>,
    Sink: EffectLifecycleSink<Tags, Payload>,
{
    type Error = Action::Error;

    fn execute_effect(
        &mut self,
        context: &mut Context,
        execution: EffectExecutionView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        self.action.execute_effect(context, execution)
    }

    fn emit_effect_lifecycle(&mut self, event: EffectLifecycleEventView<'_, Tags, Payload>) {
        self.sink.emit_effect_lifecycle(event);
    }
}

impl<Tags, Payload> EffectApplicationInput<Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn accept(
        source_id: impl Into<Option<ObjectId>>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id,
            tags,
            payload,
            decision: EffectApplicationDecision::Accept,
        }
    }

    #[must_use]
    pub fn reject(
        source_id: impl Into<Option<ObjectId>>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id,
            tags,
            payload,
            decision: EffectApplicationDecision::Reject {
                reason: reason.into(),
            },
        }
    }

    #[must_use]
    pub fn accept_from_active_ability<AbilityTags, AbilityPayload>(
        active: &ActiveAbility<AbilityTags, AbilityPayload>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
    ) -> Self
    where
        AbilityTags: TagCollection,
    {
        Self::accept(active.source_id(), target_id, tags, payload)
    }

    #[must_use]
    pub fn reject_from_active_ability<AbilityTags, AbilityPayload>(
        active: &ActiveAbility<AbilityTags, AbilityPayload>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
        reason: impl Into<String>,
    ) -> Self
    where
        AbilityTags: TagCollection,
    {
        Self::reject(active.source_id(), target_id, tags, payload, reason)
    }
}

/// Rejected effect application fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplicationRejection<Tags, Payload>
where
    Tags: TagCollection,
{
    pub application: EffectApplication<Tags, Payload>,
    pub reason: String,
}

/// Borrowed rejected effect application fact for streaming emission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectApplicationRejectionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub application: EffectApplicationView<'event, Tags, Payload>,
    pub reason: &'event str,
}

impl<'event, Tags, Payload> EffectApplicationRejectionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_rejection(&self) -> EffectApplicationRejection<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectApplicationRejection {
            application: self.application.to_owned_application(),
            reason: self.reason.to_owned(),
        }
    }
}
