use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::{AbilityStore, RevokedOwnerAbilities};
use crate::ability::events::{AbilityLifecycleEventView, ActiveAbility, ActiveAbilityView};
use crate::ability::hooks::{AbilityCommitExecutor, AbilityLifecycleSink};
use crate::ability::ids::AbilityActivationId;
use crate::ability::lifecycle_transaction::{ActiveAbilityTransition, emit_active_transition};
use crate::ability::results::{
    AbilityCancelOutcome, AbilityCommitError, AbilityCommitOutcome, AbilityEndError,
    AbilityEndOutcome, AbilityError, AbilityRollbackError, AbilityRollbackOutcome,
};

impl<Tags, Payload> AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
{
    pub(in crate::ability) fn remove_owner_with_sink<Sink>(
        &mut self,
        owner_id: ObjectId,
        sink: &mut Sink,
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        let active_abilities = self.active_abilities.remove_owner_with(owner_id, |active| {
            emit_active_transition(ActiveAbilityTransition::Revoked, active, &mut |event| {
                sink.emit_ability_lifecycle(event);
            });
        });
        let grants = self.abilities.remove_owner(owner_id);

        RevokedOwnerAbilities {
            grants,
            active_abilities,
        }
    }

    pub(in crate::ability) fn commit_with_executor<Context, Executor>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        executor: &mut Executor,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Executor::Error>>
    where
        Executor: AbilityCommitExecutor<Context, Tags, Payload>,
    {
        let active = self
            .find_active(activation_id)
            .ok_or(AbilityCommitError::Ability(AbilityError::MissingActivation))?;
        if active.committed {
            return Ok(AbilityCommitOutcome::AlreadyCommitted);
        }

        let active_view = ActiveAbilityView::from(active);
        if let Err(error) = executor.apply_commit(context, active_view) {
            self.remove_active_for_transition(
                activation_id,
                ActiveAbilityTransition::RolledBack,
                &mut |event| executor.emit_ability_lifecycle(event),
            )
            .expect("active ability exists after commit action");
            return Err(AbilityCommitError::Action(error));
        }

        let active = self
            .active_abilities
            .get_mut(activation_id)
            .expect("active ability exists after commit action");
        active.committed = true;
        emit_active_transition(ActiveAbilityTransition::Committed, active, &mut |event| {
            executor.emit_ability_lifecycle(event)
        });
        Ok(AbilityCommitOutcome::Committed)
    }

    pub(in crate::ability) fn end_with_sink<Sink>(
        &mut self,
        activation_id: AbilityActivationId,
        sink: &mut Sink,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError>
    where
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        let Some(active) = self.find_active(activation_id) else {
            return Err(AbilityEndError::MissingActivation);
        };
        if !active.committed {
            return Err(AbilityEndError::UncommittedActivation);
        }

        let active = self
            .remove_active_for_transition(
                activation_id,
                ActiveAbilityTransition::Ended,
                &mut |event| {
                    sink.emit_ability_lifecycle(event);
                },
            )
            .expect("active ability exists after commit check");
        Ok(AbilityEndOutcome::Ended(active))
    }

    pub(in crate::ability) fn cancel_with_sink<Sink>(
        &mut self,
        activation_id: AbilityActivationId,
        sink: &mut Sink,
    ) -> AbilityCancelOutcome<Tags, Payload>
    where
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        let Some(active) = self.remove_active_for_transition(
            activation_id,
            ActiveAbilityTransition::Canceled,
            &mut |event| sink.emit_ability_lifecycle(event),
        ) else {
            return AbilityCancelOutcome::MissingActivation;
        };
        AbilityCancelOutcome::Canceled(active)
    }

    pub(in crate::ability) fn rollback_with_sink<Sink>(
        &mut self,
        activation_id: AbilityActivationId,
        sink: &mut Sink,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError>
    where
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        let Some(active) = self.find_active(activation_id) else {
            return Err(AbilityRollbackError::MissingActivation);
        };
        if active.committed {
            return Err(AbilityRollbackError::AlreadyCommitted);
        }

        let active = self
            .remove_active_for_transition(
                activation_id,
                ActiveAbilityTransition::RolledBack,
                &mut |event| sink.emit_ability_lifecycle(event),
            )
            .expect("active ability exists after rollback check");
        Ok(AbilityRollbackOutcome::RolledBack(active))
    }

    fn remove_active_for_transition<F>(
        &mut self,
        activation_id: AbilityActivationId,
        transition: ActiveAbilityTransition,
        emit: &mut F,
    ) -> Option<ActiveAbility<Tags, Payload>>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let active = self.active_abilities.remove(activation_id)?;
        emit_active_transition(transition, &active, emit);
        Some(active)
    }
}
