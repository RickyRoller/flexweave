use crate::tag::TagCollection;

use super::events::{AbilityActivationAttemptView, ActiveAbilityView};

/// Caller-owned decision returned by an ability blocking query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityActivationDecision<BlockReason> {
    Allow,
    Block(BlockReason),
}

/// Async hook interface for caller-owned ability orchestration.
///
/// Flexweave owns the domain-neutral lifecycle state. Callers own activation
/// blocking, animation work, costs, cooldown effects, attribute mutations, and
/// other domain behavior at these hook points.
#[allow(async_fn_in_trait)]
pub trait AbilityHooks<Context, Tags, Payload>
where
    Tags: TagCollection,
{
    type Error;
    type BlockReason;

    async fn can_activate(
        &mut self,
        _context: &mut Context,
        _attempt: AbilityActivationAttemptView<'_, Tags, Payload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
        Ok(AbilityActivationDecision::Allow)
    }

    async fn on_start(
        &mut self,
        _context: &mut Context,
        _active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_commit(
        &mut self,
        _context: &mut Context,
        _active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_end(
        &mut self,
        _context: &mut Context,
        _active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_cancel(
        &mut self,
        _context: &mut Context,
        _active: ActiveAbilityView<'_, Tags, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
