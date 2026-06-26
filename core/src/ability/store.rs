use crate::clock::ClockUnits;
use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;
use std::fmt;

use super::definition::{AbilityCommitTiming, AbilityDefinition, AbilityDefinitionError};
use super::events::{
    AbilityActivationAttempt, AbilityActivationCommit, AbilityActivationRejection,
    AbilityActivationRejectionReason, AbilityLifecycleEvent, ActiveAbility,
};
use super::hooks::AbilityHooks;
use super::ids::{AbilityActivationId, AbilityId, CooldownUnits};

/// Result shape for active ability end operations.
pub type AbilityEndResult<Tags, Cost, Payload, Error> =
    Result<Option<ActiveAbility<Tags, Cost, Payload>>, AbilityActivationError<Error>>;

/// Store-level ability errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityError {
    MissingAbility,
    MissingActivation,
    InvalidOwner {
        owner_id: ObjectId,
    },
    OwnerMismatch {
        expected_owner_id: ObjectId,
        actual_owner_id: ObjectId,
    },
    AbilityOnCooldown,
}

impl fmt::Display for AbilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingAbility => "missing ability",
            Self::MissingActivation => "missing ability activation",
            Self::InvalidOwner { .. } => "invalid ability owner",
            Self::OwnerMismatch { .. } => "ability owner mismatch",
            Self::AbilityOnCooldown => "ability is on cooldown",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityError {}

/// Ability activation errors, including caller-owned hook failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityActivationError<E> {
    Ability(AbilityError),
    Hook(E),
}

impl<E> fmt::Display for AbilityActivationError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ability(error) => write!(formatter, "ability activation failed: {error}"),
            Self::Hook(error) => write!(formatter, "ability activation hook failed: {error}"),
        }
    }
}

impl<E> std::error::Error for AbilityActivationError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ability(error) => Some(error),
            Self::Hook(error) => Some(error),
        }
    }
}

/// Ability grant validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityGrantError {
    InvalidOwner { owner_id: ObjectId },
}

impl fmt::Display for AbilityGrantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOwner { .. } => formatter.write_str("invalid ability grant owner"),
        }
    }
}

impl std::error::Error for AbilityGrantError {}

/// Grant input for `AbilityStore`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant<Tags, Cost, Payload> {
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub cost: Option<Cost>,
    pub cooldown_units: Option<CooldownUnits>,
    pub payload: Payload,
}

impl<Tags, Cost, Payload> Grant<Tags, Cost, Payload> {
    #[must_use]
    pub fn new(owner_id: ObjectId, tags: Tags, payload: Payload) -> Self {
        Self {
            owner_id,
            tags,
            cost: None,
            cooldown_units: None,
            payload,
        }
    }

    #[must_use]
    pub fn with_cost(mut self, cost: Cost) -> Self {
        self.cost = Some(cost);
        self
    }

    #[must_use]
    pub fn with_cooldown(mut self, units: CooldownUnits) -> Self {
        self.cooldown_units = Some(units);
        self
    }
}

/// Granted ability storage with cooldown lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityStore<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    next_id: AbilityId,
    next_activation_id: AbilityActivationId,
    abilities: Vec<GrantedAbility<Tags, Cost, Payload>>,
    active_abilities: Vec<ActiveAbility<Tags, Cost, Payload>>,
}

/// Stored ability record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantedAbility<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub id: AbilityId,
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub cost: Option<Cost>,
    pub cooldown_units: Option<CooldownUnits>,
    pub cooldown_remaining_units: CooldownUnits,
    pub payload: Payload,
}

impl<Tags, Cost, Payload> GrantedAbility<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn has_tag(&self, tag: &Tags::Tag) -> bool {
        self.tags.has_tag(tag)
    }
}

impl<Tags, Cost, Payload> AbilityStore<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: AbilityId::new(1),
            next_activation_id: AbilityActivationId::new(1),
            abilities: Vec::new(),
            active_abilities: Vec::new(),
        }
    }

    /// Grants a new ability and returns its deterministic id.
    ///
    /// This is the low-level unchecked path: `input.owner_id` is copied as-is.
    /// Prefer [`Self::grant_checked`] when an `ObjectStore` is available.
    pub fn grant(&mut self, input: Grant<Tags, Cost, Payload>) -> AbilityId {
        let id = self.next_id;
        self.next_id = AbilityId::new(self.next_id.get() + 1);
        self.abilities.push(GrantedAbility {
            id,
            owner_id: input.owner_id,
            tags: input.tags,
            cost: input.cost,
            cooldown_units: input.cooldown_units,
            cooldown_remaining_units: 0,
            payload: input.payload,
        });
        id
    }

    /// Grants a new ability after validating that its owner is live.
    pub fn grant_checked(
        &mut self,
        objects: &ObjectStore,
        input: Grant<Tags, Cost, Payload>,
    ) -> Result<AbilityId, AbilityGrantError> {
        if !objects.exists(input.owner_id) {
            return Err(AbilityGrantError::InvalidOwner {
                owner_id: input.owner_id,
            });
        }

        Ok(self.grant(input))
    }

    /// Validates an authorable definition before granting a runtime ability.
    ///
    /// This is the low-level unchecked grant path for object references: it
    /// validates authoring metadata but copies `input.owner_id` as-is.
    /// Prefer [`Self::grant_checked`] plus definition validation for common
    /// runtime flows.
    pub fn grant_with_definition<PayloadSchema>(
        &mut self,
        definition: &AbilityDefinition<PayloadSchema>,
        input: Grant<Tags, Cost, Payload>,
    ) -> Result<AbilityId, AbilityDefinitionError> {
        definition.validate()?;
        Ok(self.grant(input))
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.abilities.len()
    }

    #[must_use]
    pub fn get(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Cost, Payload>> {
        self.find(ability_id)
    }

    pub fn cooldown_remaining(&self, ability_id: AbilityId) -> Result<CooldownUnits, AbilityError> {
        let ability = self.find(ability_id).ok_or(AbilityError::MissingAbility)?;
        Ok(ability.cooldown_remaining_units)
    }

    pub fn is_ready(&self, ability_id: AbilityId) -> Result<bool, AbilityError> {
        Ok(self.cooldown_remaining(ability_id)? == 0)
    }

    /// Replaces the configured cooldown for future activations.
    pub fn set_cooldown_units(
        &mut self,
        ability_id: AbilityId,
        cooldown_units: Option<CooldownUnits>,
    ) -> Result<(), AbilityError> {
        let ability = self
            .find_mut(ability_id)
            .ok_or(AbilityError::MissingAbility)?;
        ability.cooldown_units = cooldown_units;
        Ok(())
    }

    #[must_use]
    pub fn has_tag(&self, owner_id: ObjectId, tag: &Tags::Tag) -> bool {
        self.abilities
            .iter()
            .any(|ability| ability.owner_id == owner_id && ability.has_tag(tag))
    }

    /// Returns granted ability ids for `owner_id` with `tag` in deterministic grant order.
    #[must_use]
    pub fn ids_with_tag(&self, owner_id: ObjectId, tag: &Tags::Tag) -> Vec<AbilityId> {
        self.abilities
            .iter()
            .filter(|ability| ability.owner_id == owner_id && ability.has_tag(tag))
            .map(|ability| ability.id)
            .collect()
    }

    pub fn tick_cooldowns(&mut self, elapsed_units: ClockUnits) {
        for ability in &mut self.abilities {
            ability.cooldown_remaining_units = ability
                .cooldown_remaining_units
                .saturating_sub(elapsed_units);
        }
    }

    #[must_use]
    pub fn active_activation_count(&self) -> usize {
        self.active_abilities.len()
    }

    #[must_use]
    pub fn active_activations(&self) -> &[ActiveAbility<Tags, Cost, Payload>] {
        &self.active_abilities
    }

    #[must_use]
    pub fn get_active_activation(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Cost, Payload>> {
        self.find_active(activation_id)
    }

    /// Begins a non-instant activation and stores active execution state.
    pub fn begin_activation_with<Context, Hooks>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
    {
        self.begin_activation_with_events(ability_id, commit_timing, context, hooks, |_| {})
    }

    /// Begins a non-instant activation for an expected owner.
    ///
    /// This checked wrapper rejects invalid expected owners and owner/ability
    /// mismatches before caller-owned hooks run.
    pub fn begin_activation_for_with<Context, Hooks>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
    {
        self.begin_activation_for_with_events(
            owner_id,
            ability_id,
            commit_timing,
            context,
            hooks,
            |_| {},
        )
    }

    /// Begins a non-instant activation and emits attempt, rejection, commit, and start facts.
    pub fn begin_activation_with_events<Context, Hooks, F>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        let Some(ability_index) = self.find_index(ability_id) else {
            emit(AbilityLifecycleEvent::Rejected(
                AbilityActivationRejection {
                    attempt: None,
                    reason: AbilityActivationRejectionReason::MissingAbility,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::MissingAbility,
            ));
        };

        let attempt = Self::attempt_from_ability(&self.abilities[ability_index]);
        emit(AbilityLifecycleEvent::Attempted(attempt.clone()));

        if self.abilities[ability_index].cooldown_remaining_units > 0 {
            emit(AbilityLifecycleEvent::Rejected(
                AbilityActivationRejection {
                    attempt: Some(attempt),
                    reason: AbilityActivationRejectionReason::OnCooldown,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::AbilityOnCooldown,
            ));
        }

        if let Err(error) = hooks.can_activate(context, &self.abilities[ability_index]) {
            emit(AbilityLifecycleEvent::Rejected(
                AbilityActivationRejection {
                    attempt: Some(attempt),
                    reason: AbilityActivationRejectionReason::Hook,
                },
            ));
            return Err(AbilityActivationError::Hook(error));
        }

        let mut committed = false;
        if commit_timing == AbilityCommitTiming::OnStart {
            let cooldown_units = match self.commit_ability_at_index(ability_index, context, hooks) {
                Ok(cooldown_units) => cooldown_units,
                Err(error) => {
                    let reason = match &error {
                        AbilityActivationError::Ability(AbilityError::AbilityOnCooldown) => {
                            AbilityActivationRejectionReason::OnCooldown
                        }
                        AbilityActivationError::Ability(_) => {
                            AbilityActivationRejectionReason::MissingAbility
                        }
                        AbilityActivationError::Hook(_) => AbilityActivationRejectionReason::Hook,
                    };
                    emit(AbilityLifecycleEvent::Rejected(
                        AbilityActivationRejection {
                            attempt: Some(attempt),
                            reason,
                        },
                    ));
                    return Err(error);
                }
            };
            committed = true;
            emit(AbilityLifecycleEvent::Committed(AbilityActivationCommit {
                attempt: attempt.clone(),
                cooldown_units,
            }));
        }

        let activation_id = self.next_activation_id;
        self.next_activation_id = AbilityActivationId::new(self.next_activation_id.get() + 1);
        let active = ActiveAbility {
            activation_id,
            ability_id: attempt.ability_id,
            owner_id: attempt.owner_id,
            tags: attempt.tags,
            cost: attempt.cost,
            payload: attempt.payload,
            commit_timing,
            committed,
        };
        self.active_abilities.push(active.clone());
        emit(AbilityLifecycleEvent::Started(active));
        Ok(activation_id)
    }

    /// Begins a non-instant activation for an expected owner and emits lifecycle facts.
    ///
    /// This checked wrapper rejects invalid expected owners and owner/ability
    /// mismatches before caller-owned hooks run. It otherwise delegates to
    /// [`Self::begin_activation_with_events`].
    pub fn begin_activation_for_with_events<Context, Hooks, F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        if owner_id.is_invalid() {
            emit(AbilityLifecycleEvent::Rejected(
                AbilityActivationRejection {
                    attempt: None,
                    reason: AbilityActivationRejectionReason::InvalidOwner,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::InvalidOwner { owner_id },
            ));
        }

        let Some(ability_index) = self.find_index(ability_id) else {
            emit(AbilityLifecycleEvent::Rejected(
                AbilityActivationRejection {
                    attempt: None,
                    reason: AbilityActivationRejectionReason::MissingAbility,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::MissingAbility,
            ));
        };

        let actual_owner_id = self.abilities[ability_index].owner_id;
        if actual_owner_id != owner_id {
            let attempt = Self::attempt_from_ability(&self.abilities[ability_index]);
            emit(AbilityLifecycleEvent::Attempted(attempt.clone()));
            emit(AbilityLifecycleEvent::Rejected(
                AbilityActivationRejection {
                    attempt: Some(attempt),
                    reason: AbilityActivationRejectionReason::OwnerMismatch,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::OwnerMismatch {
                    expected_owner_id: owner_id,
                    actual_owner_id,
                },
            ));
        }

        self.begin_activation_with_events(ability_id, commit_timing, context, hooks, emit)
    }

    /// Runs an instant activation without emitting lifecycle facts.
    ///
    /// This performs the standard instant lifecycle: begin activation, run the
    /// caller executor with a cloned active activation view, cancel on executor
    /// failure, and end the activation on success.
    pub fn activate_instant_with<Context, Hooks, Execute>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        execute: Execute,
    ) -> AbilityEndResult<Tags, Cost, Payload, Hooks::Error>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
        Execute:
            FnOnce(&mut Context, &ActiveAbility<Tags, Cost, Payload>) -> Result<(), Hooks::Error>,
    {
        self.activate_instant_with_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            execute,
            |_| {},
        )
    }

    /// Runs an instant activation and emits lifecycle facts.
    ///
    /// This performs the standard instant lifecycle: begin activation, emit
    /// attempt/rejection/commit/start facts from begin, run the caller executor
    /// with a cloned active activation view, cancel and emit a cancel fact when
    /// execution or finishing fails, and end/emit end facts on success.
    pub fn activate_instant_with_events<Context, Hooks, Execute, F>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        execute: Execute,
        mut emit: F,
    ) -> AbilityEndResult<Tags, Cost, Payload, Hooks::Error>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
        Execute:
            FnOnce(&mut Context, &ActiveAbility<Tags, Cost, Payload>) -> Result<(), Hooks::Error>,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        let activation_id = self.begin_activation_with_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            &mut emit,
        )?;
        let active = self
            .get_active_activation(activation_id)
            .expect("activation was just started")
            .clone();

        if let Err(error) = execute(context, &active) {
            self.cancel_activation_with_events(activation_id, &mut emit);
            return Err(AbilityActivationError::Hook(error));
        }

        let result = self.end_activation_with_events(activation_id, context, hooks, &mut emit);
        if result.is_err() {
            self.cancel_activation_with_events(activation_id, &mut emit);
        }
        result
    }

    /// Commits an active activation's cost/cooldown policy if it has not already committed.
    pub fn commit_activation_with<Context, Hooks>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<bool, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
    {
        self.commit_activation_with_events(activation_id, context, hooks, |_| {})
    }

    /// Commits an active activation and emits a commit fact when this call performs the commit.
    pub fn commit_activation_with_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<bool, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        let active_index =
            self.find_active_index(activation_id)
                .ok_or(AbilityActivationError::Ability(
                    AbilityError::MissingActivation,
                ))?;
        if self.active_abilities[active_index].committed {
            return Ok(false);
        }

        let ability_id = self.active_abilities[active_index].ability_id;
        let ability_index = self
            .find_index(ability_id)
            .ok_or(AbilityActivationError::Ability(
                AbilityError::MissingAbility,
            ))?;
        let attempt = Self::attempt_from_active(&self.active_abilities[active_index]);
        let cooldown_units = self.commit_ability_at_index(ability_index, context, hooks)?;
        self.active_abilities[active_index].committed = true;
        emit(AbilityLifecycleEvent::Committed(AbilityActivationCommit {
            attempt,
            cooldown_units,
        }));
        Ok(true)
    }

    /// Ends an active activation without emitting lifecycle facts.
    pub fn end_activation_with<Context, Hooks>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> AbilityEndResult<Tags, Cost, Payload, Hooks::Error>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
    {
        self.end_activation_with_events(activation_id, context, hooks, |_| {})
    }

    /// Ends an active activation and emits commit/end facts in deterministic order.
    pub fn end_activation_with_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> AbilityEndResult<Tags, Cost, Payload, Hooks::Error>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        Cost: Clone,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(None);
        };
        let needs_commit = self.active_abilities[active_index].commit_timing
            == AbilityCommitTiming::OnEnd
            && !self.active_abilities[active_index].committed;
        if needs_commit {
            self.commit_activation_with_events(activation_id, context, hooks, &mut emit)?;
        }

        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(None);
        };
        let ability_id = self.active_abilities[active_index].ability_id;
        let ability_index = self
            .find_index(ability_id)
            .ok_or(AbilityActivationError::Ability(
                AbilityError::MissingAbility,
            ))?;
        hooks
            .end(context, &self.abilities[ability_index])
            .map_err(AbilityActivationError::Hook)?;

        let active = self.active_abilities.remove(active_index);
        emit(AbilityLifecycleEvent::Ended(active.clone()));
        Ok(Some(active))
    }

    /// Cancels an active activation without emitting lifecycle facts.
    pub fn cancel_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Option<ActiveAbility<Tags, Cost, Payload>> {
        let active_index = self.find_active_index(activation_id)?;
        Some(self.active_abilities.remove(active_index))
    }

    /// Cancels an active activation and emits a cancel fact.
    pub fn cancel_activation_with_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Option<ActiveAbility<Tags, Cost, Payload>>
    where
        Cost: Clone,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        let active = self.cancel_activation(activation_id)?;
        emit(AbilityLifecycleEvent::Canceled(active.clone()));
        Some(active)
    }

    fn commit_ability_at_index<Context, Hooks>(
        &mut self,
        ability_index: usize,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<Option<CooldownUnits>, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
    {
        let ability = &mut self.abilities[ability_index];
        if ability.cooldown_remaining_units > 0 {
            return Err(AbilityActivationError::Ability(
                AbilityError::AbilityOnCooldown,
            ));
        }

        let cooldown_units = hooks
            .cooldown_units(context, ability)
            .map_err(AbilityActivationError::Hook)?;
        hooks
            .commit(context, ability)
            .map_err(AbilityActivationError::Hook)?;

        if let Some(cooldown_units) = cooldown_units {
            ability.cooldown_remaining_units = cooldown_units;
        }
        Ok(cooldown_units)
    }

    fn attempt_from_ability(
        ability: &GrantedAbility<Tags, Cost, Payload>,
    ) -> AbilityActivationAttempt<Tags, Cost, Payload>
    where
        Cost: Clone,
        Payload: Clone,
    {
        AbilityActivationAttempt {
            ability_id: ability.id,
            owner_id: ability.owner_id,
            tags: ability.tags.clone(),
            cost: ability.cost.clone(),
            payload: ability.payload.clone(),
        }
    }

    fn attempt_from_active(
        active: &ActiveAbility<Tags, Cost, Payload>,
    ) -> AbilityActivationAttempt<Tags, Cost, Payload>
    where
        Cost: Clone,
        Payload: Clone,
    {
        AbilityActivationAttempt {
            ability_id: active.ability_id,
            owner_id: active.owner_id,
            tags: active.tags.clone(),
            cost: active.cost.clone(),
            payload: active.payload.clone(),
        }
    }

    fn find_index(&self, ability_id: AbilityId) -> Option<usize> {
        self.abilities
            .iter()
            .position(|ability| ability.id == ability_id)
    }

    fn find(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Cost, Payload>> {
        self.abilities
            .iter()
            .find(|ability| ability.id == ability_id)
    }

    fn find_mut(
        &mut self,
        ability_id: AbilityId,
    ) -> Option<&mut GrantedAbility<Tags, Cost, Payload>> {
        self.abilities
            .iter_mut()
            .find(|ability| ability.id == ability_id)
    }

    fn find_active(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Cost, Payload>> {
        self.active_abilities
            .iter()
            .find(|active| active.activation_id == activation_id)
    }

    fn find_active_index(&self, activation_id: AbilityActivationId) -> Option<usize> {
        self.active_abilities
            .iter()
            .position(|active| active.activation_id == activation_id)
    }
}

impl<Tags, Cost, Payload> Default for AbilityStore<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    fn default() -> Self {
        Self::new()
    }
}
