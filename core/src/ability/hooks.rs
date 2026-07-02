use std::convert::Infallible;

use crate::tag::TagCollection;

use super::events::{AbilityActivationAttemptView, ActiveAbilityView};

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
