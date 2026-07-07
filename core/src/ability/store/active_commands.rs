use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::{AbilityStore, RevokedOwnerAbilities};
use crate::ability::event_sink::{discard_lifecycle_event, owned_lifecycle_events};
use crate::ability::events::{
    AbilityLifecycleEvent, AbilityLifecycleEventView, ActiveAbility, ActiveAbilityView,
};
use crate::ability::hooks::AbilityCommitExecutor;
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
    /// Revokes granted and active abilities owned by `owner_id`.
    #[must_use]
    pub fn revoke_owner(&mut self, owner_id: ObjectId) -> RevokedOwnerAbilities<Tags, Payload> {
        let active_abilities = self.active_abilities.remove_owner_with(owner_id, |_| {});
        let grants = self.abilities.remove_owner(owner_id);

        RevokedOwnerAbilities {
            grants,
            active_abilities,
        }
    }

    /// Revokes granted abilities and emits owned revocation facts for active abilities.
    pub fn revoke_owner_with_events<F>(
        &mut self,
        owner_id: ObjectId,
        mut emit: F,
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.revoke_owner_with_borrowed_events(owner_id, owned_lifecycle_events(&mut emit))
    }

    /// Revokes granted abilities and streams borrowed revocation facts for active abilities.
    pub fn revoke_owner_with_borrowed_events<F>(
        &mut self,
        owner_id: ObjectId,
        mut emit: F,
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let active_abilities = self.active_abilities.remove_owner_with(owner_id, |active| {
            emit_active_transition(ActiveAbilityTransition::Revoked, active, &mut emit);
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

    /// Ends a committed active activation without emitting lifecycle facts.
    pub fn end_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError> {
        self.end_activation_with_borrowed_events(activation_id, discard_lifecycle_event)
    }

    /// Ends a committed active activation and emits an owned end fact.
    pub fn end_activation_with_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.end_activation_with_borrowed_events(activation_id, owned_lifecycle_events(&mut emit))
    }

    /// Ends a committed active activation and streams a borrowed end fact.
    pub fn end_activation_with_borrowed_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(active) = self.find_active(activation_id) else {
            return Err(AbilityEndError::MissingActivation);
        };
        if !active.committed {
            return Err(AbilityEndError::UncommittedActivation);
        }

        let active = self
            .remove_active_for_transition(activation_id, ActiveAbilityTransition::Ended, &mut emit)
            .expect("active ability exists after commit check");
        Ok(AbilityEndOutcome::Ended(active))
    }

    /// Cancels an active activation without lifecycle facts.
    pub fn cancel_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> AbilityCancelOutcome<Tags, Payload> {
        self.cancel_activation_with_borrowed_events(activation_id, discard_lifecycle_event)
    }

    /// Cancels an active activation and emits an owned cancel fact.
    pub fn cancel_activation_with_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> AbilityCancelOutcome<Tags, Payload>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.cancel_activation_with_borrowed_events(
            activation_id,
            owned_lifecycle_events(&mut emit),
        )
    }

    /// Cancels an active activation and streams a borrowed cancel fact.
    pub fn cancel_activation_with_borrowed_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> AbilityCancelOutcome<Tags, Payload>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(active) = self.remove_active_for_transition(
            activation_id,
            ActiveAbilityTransition::Canceled,
            &mut emit,
        ) else {
            return AbilityCancelOutcome::MissingActivation;
        };
        AbilityCancelOutcome::Canceled(active)
    }

    /// Rolls back an uncommitted active activation without lifecycle facts.
    pub fn rollback_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError> {
        self.rollback_activation_with_borrowed_events(activation_id, discard_lifecycle_event)
    }

    /// Rolls back an uncommitted active activation and emits an owned rollback fact.
    pub fn rollback_activation_with_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.rollback_activation_with_borrowed_events(
            activation_id,
            owned_lifecycle_events(&mut emit),
        )
    }

    /// Rolls back an uncommitted active activation and streams a borrowed rollback fact.
    pub fn rollback_activation_with_borrowed_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
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
                &mut emit,
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
