use crate::clock::ClockUnits;
use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;
use std::collections::HashMap;
use std::fmt;

use super::definition::{AbilityCommitTiming, AbilityDefinition, AbilityDefinitionError};
use super::events::{
    AbilityActivationAttemptView, AbilityActivationCommitView, AbilityActivationRejectionReason,
    AbilityActivationRejectionView, AbilityLifecycleEvent, AbilityLifecycleEventView,
    ActiveAbility, ActiveAbilityView,
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
    ability_index_by_id: HashMap<AbilityId, usize>,
    active_abilities: Vec<ActiveAbility<Tags, Cost, Payload>>,
    active_index_by_activation_id: HashMap<AbilityActivationId, usize>,
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

/// Grants and active executions removed while cleaning up one owner object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevokedOwnerAbilities<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    pub grants: Vec<GrantedAbility<Tags, Cost, Payload>>,
    pub active_abilities: Vec<ActiveAbility<Tags, Cost, Payload>>,
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

struct AbilityActivationSeed<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    ability_id: AbilityId,
    owner_id: ObjectId,
    tags: Tags,
    cost: Option<Cost>,
    payload: Payload,
}

impl<Tags, Cost, Payload> AbilityActivationSeed<Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    fn from_ability(ability: &GrantedAbility<Tags, Cost, Payload>) -> Self
    where
        Cost: Clone,
        Payload: Clone,
    {
        Self {
            ability_id: ability.id,
            owner_id: ability.owner_id,
            tags: ability.tags.clone(),
            cost: ability.cost.clone(),
            payload: ability.payload.clone(),
        }
    }

    fn attempt_view(&self) -> AbilityActivationAttemptView<'_, Tags, Cost, Payload> {
        AbilityActivationAttemptView {
            ability_id: self.ability_id,
            owner_id: self.owner_id,
            tags: &self.tags,
            cost: self.cost.as_ref(),
            payload: &self.payload,
        }
    }

    fn into_active(
        self,
        activation_id: AbilityActivationId,
        commit_timing: AbilityCommitTiming,
        committed: bool,
    ) -> ActiveAbility<Tags, Cost, Payload> {
        ActiveAbility {
            activation_id,
            ability_id: self.ability_id,
            owner_id: self.owner_id,
            tags: self.tags,
            cost: self.cost,
            payload: self.payload,
            commit_timing,
            committed,
        }
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
            ability_index_by_id: HashMap::new(),
            active_abilities: Vec::new(),
            active_index_by_activation_id: HashMap::new(),
        }
    }

    /// Grants a new ability and returns its deterministic id.
    ///
    /// This is the low-level unchecked path: `input.owner_id` is copied as-is.
    /// Prefer [`Self::grant_checked`] when an `ObjectStore` is available.
    pub fn grant(&mut self, input: Grant<Tags, Cost, Payload>) -> AbilityId {
        let id = self.next_id;
        self.next_id = AbilityId::new(self.next_id.get() + 1);
        self.ability_index_by_id.insert(id, self.abilities.len());
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

    /// Revokes granted and active abilities owned by `owner_id`.
    #[must_use]
    pub fn revoke_owner(
        &mut self,
        owner_id: ObjectId,
    ) -> RevokedOwnerAbilities<Tags, Cost, Payload> {
        let mut active_abilities = Vec::new();
        let mut active_index = 0;
        while active_index < self.active_abilities.len() {
            if self.active_abilities[active_index].owner_id == owner_id {
                active_abilities.push(self.remove_active_at_index(active_index));
            } else {
                active_index += 1;
            }
        }

        let mut grants = Vec::new();
        let mut ability_index = 0;
        while ability_index < self.abilities.len() {
            if self.abilities[ability_index].owner_id == owner_id {
                grants.push(self.remove_ability_at_index(ability_index));
            } else {
                ability_index += 1;
            }
        }

        RevokedOwnerAbilities {
            grants,
            active_abilities,
        }
    }

    /// Revokes granted abilities and emits owned cancellation facts for active abilities.
    pub fn revoke_owner_with_events<F>(
        &mut self,
        owner_id: ObjectId,
        mut emit: F,
    ) -> RevokedOwnerAbilities<Tags, Cost, Payload>
    where
        Cost: Clone,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Cost, Payload>),
    {
        self.revoke_owner_with_borrowed_events(owner_id, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Revokes granted abilities and streams borrowed cancellation facts for active abilities.
    pub fn revoke_owner_with_borrowed_events<F>(
        &mut self,
        owner_id: ObjectId,
        mut emit: F,
    ) -> RevokedOwnerAbilities<Tags, Cost, Payload>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
    {
        let mut active_abilities = Vec::new();
        let mut active_index = 0;
        while active_index < self.active_abilities.len() {
            if self.active_abilities[active_index].owner_id == owner_id {
                let active = self.remove_active_at_index(active_index);
                emit(AbilityLifecycleEventView::Canceled((&active).into()));
                active_abilities.push(active);
            } else {
                active_index += 1;
            }
        }

        let mut grants = Vec::new();
        let mut ability_index = 0;
        while ability_index < self.abilities.len() {
            if self.abilities[ability_index].owner_id == owner_id {
                grants.push(self.remove_ability_at_index(ability_index));
            } else {
                ability_index += 1;
            }
        }

        RevokedOwnerAbilities {
            grants,
            active_abilities,
        }
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
        self.begin_activation_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            |_| {},
        )
    }

    /// Begins a non-instant activation for an expected owner.
    ///
    /// This checked wrapper rejects invalid expected owners and owner/ability
    /// mismatches before caller-owned hooks run.
    pub fn begin_activation_for_owner_with<Context, Hooks>(
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
        self.begin_activation_for_owner_with_borrowed_events(
            owner_id,
            ability_id,
            commit_timing,
            context,
            hooks,
            |_| {},
        )
    }

    /// Begins a non-instant activation and emits owned attempt, rejection, commit, and start facts.
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
        self.begin_activation_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            |event| emit(event.to_owned_event()),
        )
    }

    /// Begins a non-instant activation and streams borrowed lifecycle facts.
    ///
    /// Borrowed facts are valid only for the duration of the callback. Use
    /// `AbilityLifecycleEventView::to_owned_event` when a caller needs retained
    /// facts for diagnostics, replay, or tests.
    pub fn begin_activation_with_borrowed_events<Context, Hooks, F>(
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
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
    {
        let Some(ability_index) = self.find_index(ability_id) else {
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: None,
                    reason: AbilityActivationRejectionReason::MissingAbility,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::MissingAbility,
            ));
        };

        emit(AbilityLifecycleEventView::Attempted(
            Self::attempt_view_from_ability(&self.abilities[ability_index]),
        ));

        if self.abilities[ability_index].cooldown_remaining_units > 0 {
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: Some(Self::attempt_view_from_ability(
                        &self.abilities[ability_index],
                    )),
                    reason: AbilityActivationRejectionReason::OnCooldown,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::AbilityOnCooldown,
            ));
        }

        if let Err(error) = hooks.can_activate(context, &self.abilities[ability_index]) {
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: Some(Self::attempt_view_from_ability(
                        &self.abilities[ability_index],
                    )),
                    reason: AbilityActivationRejectionReason::Hook,
                },
            ));
            return Err(AbilityActivationError::Hook(error));
        }

        let seed = AbilityActivationSeed::from_ability(&self.abilities[ability_index]);
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
                    emit(AbilityLifecycleEventView::Rejected(
                        AbilityActivationRejectionView {
                            attempt: Some(seed.attempt_view()),
                            reason,
                        },
                    ));
                    return Err(error);
                }
            };
            committed = true;
            emit(AbilityLifecycleEventView::Committed(
                AbilityActivationCommitView {
                    attempt: seed.attempt_view(),
                    cooldown_units,
                },
            ));
        }

        let activation_id = self.next_activation_id;
        self.next_activation_id = AbilityActivationId::new(self.next_activation_id.get() + 1);
        let active = seed.into_active(activation_id, commit_timing, committed);
        self.push_active_ability(active);
        let active = self
            .active_abilities
            .last()
            .expect("activation was just pushed");
        emit(AbilityLifecycleEventView::Started(active.into()));
        Ok(activation_id)
    }

    /// Begins a non-instant activation for an expected owner and emits lifecycle facts.
    ///
    /// This checked wrapper rejects invalid expected owners and owner/ability
    /// mismatches before caller-owned hooks run. It otherwise delegates to
    /// [`Self::begin_activation_with_borrowed_events`].
    pub fn begin_activation_for_owner_with_events<Context, Hooks, F>(
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
        self.begin_activation_for_owner_with_borrowed_events(
            owner_id,
            ability_id,
            commit_timing,
            context,
            hooks,
            |event| emit(event.to_owned_event()),
        )
    }

    /// Begins a non-instant activation for an expected owner and streams borrowed lifecycle facts.
    pub fn begin_activation_for_owner_with_borrowed_events<Context, Hooks, F>(
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
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
    {
        if owner_id.is_invalid() {
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: None,
                    reason: AbilityActivationRejectionReason::InvalidOwner,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::InvalidOwner { owner_id },
            ));
        }

        let Some(ability_index) = self.find_index(ability_id) else {
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
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
            let attempt = Self::attempt_view_from_ability(&self.abilities[ability_index]);
            emit(AbilityLifecycleEventView::Attempted(attempt));
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: Some(Self::attempt_view_from_ability(
                        &self.abilities[ability_index],
                    )),
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

        self.begin_activation_with_borrowed_events(ability_id, commit_timing, context, hooks, emit)
    }

    /// Runs an instant activation without emitting lifecycle facts.
    ///
    /// This performs the standard instant lifecycle: begin activation, run the
    /// caller executor with the active activation view, cancel on executor
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
        self.activate_instant_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            execute,
            |_| {},
        )
    }

    /// Runs an instant activation and emits owned lifecycle facts.
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
        self.activate_instant_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            execute,
            |event| emit(event.to_owned_event()),
        )
    }

    /// Runs an instant activation and streams borrowed lifecycle facts.
    pub fn activate_instant_with_borrowed_events<Context, Hooks, Execute, F>(
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
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
    {
        let activation_id = self.begin_activation_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            &mut emit,
        )?;
        let active_index = self
            .find_active_index(activation_id)
            .expect("activation was just started");

        if let Err(error) = execute(context, &self.active_abilities[active_index]) {
            self.cancel_activation_with_borrowed_events(activation_id, &mut emit);
            return Err(AbilityActivationError::Hook(error));
        }

        let result =
            self.end_activation_with_borrowed_events(activation_id, context, hooks, &mut emit);
        if result.is_err() {
            self.cancel_activation_with_borrowed_events(activation_id, &mut emit);
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
    {
        self.commit_activation_with_borrowed_events(activation_id, context, hooks, |_| {})
    }

    /// Commits an active activation and emits an owned commit fact when this call performs the commit.
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
        self.commit_activation_with_borrowed_events(activation_id, context, hooks, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Commits an active activation and streams a borrowed commit fact when this call performs the commit.
    pub fn commit_activation_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<bool, AbilityActivationError<Hooks::Error>>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
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
        let cooldown_units = self.commit_ability_at_index(ability_index, context, hooks)?;
        self.active_abilities[active_index].committed = true;
        let active = ActiveAbilityView::from(&self.active_abilities[active_index]);
        emit(AbilityLifecycleEventView::Committed(
            AbilityActivationCommitView {
                attempt: active.attempt_view(),
                cooldown_units,
            },
        ));
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
    {
        self.end_activation_with_borrowed_events(activation_id, context, hooks, |_| {})
    }

    /// Ends an active activation and emits owned commit/end facts in deterministic order.
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
        self.end_activation_with_borrowed_events(activation_id, context, hooks, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Ends an active activation and streams borrowed commit/end facts in deterministic order.
    pub fn end_activation_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> AbilityEndResult<Tags, Cost, Payload, Hooks::Error>
    where
        Hooks: AbilityHooks<Context, Tags, Cost, Payload>,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
    {
        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(None);
        };
        let needs_commit = self.active_abilities[active_index].commit_timing
            == AbilityCommitTiming::OnEnd
            && !self.active_abilities[active_index].committed;
        if needs_commit {
            self.commit_activation_with_borrowed_events(activation_id, context, hooks, &mut emit)?;
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

        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::Ended((&active).into()));
        Ok(Some(active))
    }

    /// Cancels an active activation without emitting lifecycle facts.
    pub fn cancel_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Option<ActiveAbility<Tags, Cost, Payload>> {
        let active_index = self.find_active_index(activation_id)?;
        Some(self.remove_active_at_index(active_index))
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
        self.cancel_activation_with_borrowed_events(activation_id, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Cancels an active activation and streams a borrowed cancel fact.
    pub fn cancel_activation_with_borrowed_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Option<ActiveAbility<Tags, Cost, Payload>>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Cost, Payload>),
    {
        let active = self.cancel_activation(activation_id)?;
        emit(AbilityLifecycleEventView::Canceled((&active).into()));
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

    fn attempt_view_from_ability(
        ability: &GrantedAbility<Tags, Cost, Payload>,
    ) -> AbilityActivationAttemptView<'_, Tags, Cost, Payload> {
        AbilityActivationAttemptView {
            ability_id: ability.id,
            owner_id: ability.owner_id,
            tags: &ability.tags,
            cost: ability.cost.as_ref(),
            payload: &ability.payload,
        }
    }

    fn find_index(&self, ability_id: AbilityId) -> Option<usize> {
        self.ability_index_by_id.get(&ability_id).copied()
    }

    fn find(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Cost, Payload>> {
        self.find_index(ability_id)
            .map(|index| &self.abilities[index])
    }

    fn find_mut(
        &mut self,
        ability_id: AbilityId,
    ) -> Option<&mut GrantedAbility<Tags, Cost, Payload>> {
        let index = self.find_index(ability_id)?;
        Some(&mut self.abilities[index])
    }

    fn find_active(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Cost, Payload>> {
        self.find_active_index(activation_id)
            .map(|index| &self.active_abilities[index])
    }

    fn find_active_index(&self, activation_id: AbilityActivationId) -> Option<usize> {
        self.active_index_by_activation_id
            .get(&activation_id)
            .copied()
    }

    fn remove_ability_at_index(
        &mut self,
        ability_index: usize,
    ) -> GrantedAbility<Tags, Cost, Payload> {
        let removed = self.abilities.remove(ability_index);
        self.ability_index_by_id.remove(&removed.id);
        self.reindex_abilities_from(ability_index);
        removed
    }

    fn reindex_abilities_from(&mut self, start: usize) {
        for index in start..self.abilities.len() {
            self.ability_index_by_id
                .insert(self.abilities[index].id, index);
        }
    }

    fn push_active_ability(&mut self, active: ActiveAbility<Tags, Cost, Payload>) {
        self.active_index_by_activation_id
            .insert(active.activation_id, self.active_abilities.len());
        self.active_abilities.push(active);
    }

    fn remove_active_at_index(
        &mut self,
        active_index: usize,
    ) -> ActiveAbility<Tags, Cost, Payload> {
        let removed = self.active_abilities.remove(active_index);
        self.active_index_by_activation_id
            .remove(&removed.activation_id);
        self.reindex_active_from(active_index);
        removed
    }

    fn reindex_active_from(&mut self, start: usize) {
        for index in start..self.active_abilities.len() {
            self.active_index_by_activation_id
                .insert(self.active_abilities[index].activation_id, index);
        }
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
