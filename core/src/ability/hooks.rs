use std::convert::Infallible;

use crate::tag::TagCollection;

use super::events::{
    AbilityActivationAttemptView, AbilityLifecycleEvent, AbilityLifecycleEventView,
    ActiveAbilityView,
};

/// Caller-owned decision returned by an ability blocking query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityActivationDecision<BlockReason> {
    Allow,
    Block(BlockReason),
}

/// Synchronous caller-owned activation gate.
pub trait AbilityActivationGate<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;
    type BlockReason;

    fn can_activate(
        &mut self,
        context: &Context,
        attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error>;
}

/// Synchronous caller-owned commit action.
pub trait AbilityCommitAction<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;

    fn apply_commit(
        &mut self,
        context: &mut Context,
        active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error>;
}

/// Activation gate that always allows activation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AllowActivation;

impl<Context, Tags, Payload> AbilityActivationGate<Context, Tags, Payload> for AllowActivation
where
    Tags: TagCollection,
{
    type Error = Infallible;
    type BlockReason = Infallible;

    fn can_activate(
        &mut self,
        _context: &Context,
        _attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
        Ok(AbilityActivationDecision::Allow)
    }
}

impl<Context, Tags, Payload, Error, BlockReason, F> AbilityActivationGate<Context, Tags, Payload>
    for F
where
    Tags: TagCollection,
    F: for<'event> FnMut(
        &Context,
        AbilityActivationAttemptView<'event, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<BlockReason>, Error>,
{
    type Error = Error;
    type BlockReason = BlockReason;

    fn can_activate(
        &mut self,
        context: &Context,
        attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
        self(context, attempt)
    }
}

/// Commit action that performs no caller-owned work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoCommitAction;

impl<Context, Tags, Payload> AbilityCommitAction<Context, Tags, Payload> for NoCommitAction
where
    Tags: TagCollection,
{
    type Error = Infallible;

    fn apply_commit(
        &mut self,
        _context: &mut Context,
        _active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<Context, Tags, Payload, Error, F> AbilityCommitAction<Context, Tags, Payload> for F
where
    Tags: TagCollection,
    F: for<'event> FnMut(
        &mut Context,
        ActiveAbilityView<'event, Tags, Payload>,
    ) -> Result<(), Error>,
{
    type Error = Error;

    fn apply_commit(
        &mut self,
        context: &mut Context,
        active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        self(context, active)
    }
}

/// Sink for ability lifecycle facts produced while executing ability commands.
pub trait AbilityLifecycleSink<Tags, Payload>
where
    Tags: TagCollection,
{
    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>);
}

/// Ability lifecycle sink that drops emitted facts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DiscardAbilityLifecycleEvents;

impl<Tags, Payload> AbilityLifecycleSink<Tags, Payload> for DiscardAbilityLifecycleEvents
where
    Tags: TagCollection,
{
    fn emit_ability_lifecycle(&mut self, _event: AbilityLifecycleEventView<'_, Tags, Payload>) {}
}

/// Ability lifecycle sink that converts borrowed facts into owned facts.
pub struct OwnedAbilityLifecycleEvents<F> {
    emit: F,
}

impl<F> OwnedAbilityLifecycleEvents<F> {
    #[must_use]
    pub fn new(emit: F) -> Self {
        Self { emit }
    }
}

impl<Tags, Payload, F> AbilityLifecycleSink<Tags, Payload> for OwnedAbilityLifecycleEvents<F>
where
    Tags: Clone + TagCollection,
    Payload: Clone,
    F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
{
    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>) {
        (self.emit)(event.to_owned_event());
    }
}

impl<Tags, Payload, F> AbilityLifecycleSink<Tags, Payload> for F
where
    Tags: TagCollection,
    F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
{
    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>) {
        self(event);
    }
}

/// Execution participant for ability activation commands.
pub trait AbilityActivationExecutor<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;
    type BlockReason;

    fn can_activate(
        &mut self,
        context: &Context,
        attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error>;

    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>);
}

/// Activation executor that always allows activation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoAbilityActivationExecutor<Sink = DiscardAbilityLifecycleEvents> {
    sink: Sink,
}

impl NoAbilityActivationExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sink: DiscardAbilityLifecycleEvents,
        }
    }
}

impl<Sink> NoAbilityActivationExecutor<Sink> {
    #[must_use]
    pub fn with_borrowed_events<F>(self, emit: F) -> NoAbilityActivationExecutor<F> {
        NoAbilityActivationExecutor { sink: emit }
    }

    #[must_use]
    pub fn with_owned_events<F>(
        self,
        emit: F,
    ) -> NoAbilityActivationExecutor<OwnedAbilityLifecycleEvents<F>> {
        NoAbilityActivationExecutor {
            sink: OwnedAbilityLifecycleEvents::new(emit),
        }
    }
}

impl<Context, Tags, Payload, Sink> AbilityActivationExecutor<Context, Tags, Payload>
    for NoAbilityActivationExecutor<Sink>
where
    Tags: TagCollection,
    Sink: AbilityLifecycleSink<Tags, Payload>,
{
    type Error = Infallible;
    type BlockReason = Infallible;

    fn can_activate(
        &mut self,
        _context: &Context,
        _attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
        Ok(AbilityActivationDecision::Allow)
    }

    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>) {
        self.sink.emit_ability_lifecycle(event);
    }
}

/// Activation executor that adapts a caller-owned gate and optional lifecycle sink.
pub struct AbilityGateExecutor<'gate, Gate, Sink = DiscardAbilityLifecycleEvents> {
    gate: &'gate mut Gate,
    sink: Sink,
}

impl<'gate, Gate> AbilityGateExecutor<'gate, Gate> {
    #[must_use]
    pub fn new(gate: &'gate mut Gate) -> Self {
        Self {
            gate,
            sink: DiscardAbilityLifecycleEvents,
        }
    }
}

impl<'gate, Gate, Sink> AbilityGateExecutor<'gate, Gate, Sink> {
    #[must_use]
    pub fn with_borrowed_events<F>(self, emit: F) -> AbilityGateExecutor<'gate, Gate, F> {
        AbilityGateExecutor {
            gate: self.gate,
            sink: emit,
        }
    }

    #[must_use]
    pub fn with_owned_events<F>(
        self,
        emit: F,
    ) -> AbilityGateExecutor<'gate, Gate, OwnedAbilityLifecycleEvents<F>> {
        AbilityGateExecutor {
            gate: self.gate,
            sink: OwnedAbilityLifecycleEvents::new(emit),
        }
    }
}

impl<Context, Gate, Tags, Payload, Sink> AbilityActivationExecutor<Context, Tags, Payload>
    for AbilityGateExecutor<'_, Gate, Sink>
where
    Tags: TagCollection,
    Gate: AbilityActivationGate<Context, Tags, Payload>,
    Sink: AbilityLifecycleSink<Tags, Payload>,
{
    type Error = Gate::Error;
    type BlockReason = Gate::BlockReason;

    fn can_activate(
        &mut self,
        context: &Context,
        attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
        self.gate.can_activate(context, attempt)
    }

    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>) {
        self.sink.emit_ability_lifecycle(event);
    }
}

/// Execution participant for ability commit commands.
pub trait AbilityCommitExecutor<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;

    fn apply_commit(
        &mut self,
        context: &mut Context,
        active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error>;

    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>);
}

/// Commit executor that performs no caller-owned work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoAbilityCommitExecutor<Sink = DiscardAbilityLifecycleEvents> {
    sink: Sink,
}

impl NoAbilityCommitExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sink: DiscardAbilityLifecycleEvents,
        }
    }
}

impl<Sink> NoAbilityCommitExecutor<Sink> {
    #[must_use]
    pub fn with_borrowed_events<F>(self, emit: F) -> NoAbilityCommitExecutor<F> {
        NoAbilityCommitExecutor { sink: emit }
    }

    #[must_use]
    pub fn with_owned_events<F>(
        self,
        emit: F,
    ) -> NoAbilityCommitExecutor<OwnedAbilityLifecycleEvents<F>> {
        NoAbilityCommitExecutor {
            sink: OwnedAbilityLifecycleEvents::new(emit),
        }
    }
}

impl<Context, Tags, Payload, Sink> AbilityCommitExecutor<Context, Tags, Payload>
    for NoAbilityCommitExecutor<Sink>
where
    Tags: TagCollection,
    Sink: AbilityLifecycleSink<Tags, Payload>,
{
    type Error = Infallible;

    fn apply_commit(
        &mut self,
        _context: &mut Context,
        _active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>) {
        self.sink.emit_ability_lifecycle(event);
    }
}

/// Commit executor that adapts a caller-owned action and optional lifecycle sink.
pub struct AbilityCommitActionExecutor<'action, Action, Sink = DiscardAbilityLifecycleEvents> {
    action: &'action mut Action,
    sink: Sink,
}

impl<'action, Action> AbilityCommitActionExecutor<'action, Action> {
    #[must_use]
    pub fn new(action: &'action mut Action) -> Self {
        Self {
            action,
            sink: DiscardAbilityLifecycleEvents,
        }
    }
}

impl<'action, Action, Sink> AbilityCommitActionExecutor<'action, Action, Sink> {
    #[must_use]
    pub fn with_borrowed_events<F>(
        self,
        emit: F,
    ) -> AbilityCommitActionExecutor<'action, Action, F> {
        AbilityCommitActionExecutor {
            action: self.action,
            sink: emit,
        }
    }

    #[must_use]
    pub fn with_owned_events<F>(
        self,
        emit: F,
    ) -> AbilityCommitActionExecutor<'action, Action, OwnedAbilityLifecycleEvents<F>> {
        AbilityCommitActionExecutor {
            action: self.action,
            sink: OwnedAbilityLifecycleEvents::new(emit),
        }
    }
}

impl<Context, Action, Tags, Payload, Sink> AbilityCommitExecutor<Context, Tags, Payload>
    for AbilityCommitActionExecutor<'_, Action, Sink>
where
    Tags: TagCollection,
    Action: AbilityCommitAction<Context, Tags, Payload>,
    Sink: AbilityLifecycleSink<Tags, Payload>,
{
    type Error = Action::Error;

    fn apply_commit(
        &mut self,
        context: &mut Context,
        active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        self.action.apply_commit(context, active)
    }

    fn emit_ability_lifecycle(&mut self, event: AbilityLifecycleEventView<'_, Tags, Payload>) {
        self.sink.emit_ability_lifecycle(event);
    }
}
