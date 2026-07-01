use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;
use std::collections::HashMap;
use std::fmt;

use super::definition::{
    AbilityCommitTiming, AbilityDefinition, AbilityDefinitionError, AbilityDefinitionRegistryError,
    AbilityDefinitions,
};
use super::events::{
    AbilityActivationAttemptView, AbilityActivationCommitView, AbilityActivationRejectionReason,
    AbilityActivationRejectionView, AbilityLifecycleEvent, AbilityLifecycleEventView,
    ActiveAbility, ActiveAbilityView,
};
use super::hooks::{AbilityActivationDecision, AbilityHooks};
use super::ids::{AbilityActivationId, AbilityId};

/// Result shape for active ability end operations with explicit command outcomes.
pub type AbilityEndOutcomeResult<Tags, Payload, Error, BlockReason = ()> =
    Result<AbilityEndOutcome<Tags, Payload>, AbilityActivationError<Error, BlockReason>>;

/// Ability hook phase that produced a caller-owned hook error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityHookPhase {
    CanActivate,
    Start,
    Commit,
    ExecuteInstant,
    End,
    Cancel,
}

impl fmt::Display for AbilityHookPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let phase = match self {
            Self::CanActivate => "can-activate",
            Self::Start => "start",
            Self::Commit => "commit",
            Self::ExecuteInstant => "execute-instant",
            Self::End => "end",
            Self::Cancel => "cancel",
        };
        formatter.write_str(phase)
    }
}

/// Outcome of a commit command for an active ability activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityCommitOutcome {
    Committed,
    AlreadyCommitted,
}

/// Outcome of an end command for an active ability activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityEndOutcome<Tags, Payload>
where
    Tags: TagCollection,
{
    Ended(ActiveAbility<Tags, Payload>),
    MissingActivation,
}

/// Outcome of a cancel command for an active ability activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityCancelOutcome<Tags, Payload>
where
    Tags: TagCollection,
{
    Canceled(ActiveAbility<Tags, Payload>),
    MissingActivation,
}

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
}

impl fmt::Display for AbilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingAbility => "missing ability",
            Self::MissingActivation => "missing ability activation",
            Self::InvalidOwner { .. } => "invalid ability owner",
            Self::OwnerMismatch { .. } => "ability owner mismatch",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityError {}

/// Ability activation errors, including caller-owned blocking and hook failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityActivationError<E, BlockReason = ()> {
    Ability(AbilityError),
    Blocked(BlockReason),
    Hook { phase: AbilityHookPhase, error: E },
}

impl<E, BlockReason> AbilityActivationError<E, BlockReason> {
    #[must_use]
    pub fn hook(phase: AbilityHookPhase, error: E) -> Self {
        Self::Hook { phase, error }
    }
}

impl<E, BlockReason> fmt::Display for AbilityActivationError<E, BlockReason>
where
    E: fmt::Display,
    BlockReason: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ability(error) => write!(formatter, "ability activation failed: {error}"),
            Self::Blocked(reason) => write!(formatter, "ability activation blocked: {reason:?}"),
            Self::Hook { phase, error } => {
                write!(
                    formatter,
                    "ability activation hook failed during {phase}: {error}"
                )
            }
        }
    }
}

impl<E, BlockReason> std::error::Error for AbilityActivationError<E, BlockReason>
where
    E: std::error::Error + 'static,
    BlockReason: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ability(error) => Some(error),
            Self::Blocked(_) => None,
            Self::Hook { error, .. } => Some(error),
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

/// Registered activation errors for key-aware ability workflows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisteredAbilityActivationError<E, BlockReason = ()> {
    MissingGrantedDefinitionKey { ability_id: AbilityId },
    Definition(AbilityDefinitionRegistryError),
    Activation(AbilityActivationError<E, BlockReason>),
}

impl<E, BlockReason> fmt::Display for RegisteredAbilityActivationError<E, BlockReason>
where
    E: fmt::Display,
    BlockReason: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGrantedDefinitionKey { ability_id } => write!(
                formatter,
                "ability `{ability_id}` was not granted from a registered definition"
            ),
            Self::Definition(error) => {
                write!(formatter, "registered ability activation failed: {error}")
            }
            Self::Activation(error) => {
                write!(formatter, "registered ability activation failed: {error}")
            }
        }
    }
}

impl<E, BlockReason> std::error::Error for RegisteredAbilityActivationError<E, BlockReason>
where
    E: std::error::Error + 'static,
    BlockReason: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingGrantedDefinitionKey { .. } => None,
            Self::Definition(error) => Some(error),
            Self::Activation(error) => Some(error),
        }
    }
}

/// Grant input for `AbilityStore`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant<Tags, Payload> {
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
}

impl<Tags, Payload> Grant<Tags, Payload> {
    #[must_use]
    pub fn new(owner_id: ObjectId, tags: Tags, payload: Payload) -> Self {
        Self {
            owner_id,
            tags,
            payload,
        }
    }
}

/// Granted ability storage with lifecycle orchestration only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
{
    next_id: AbilityId,
    next_activation_id: AbilityActivationId,
    abilities: Vec<GrantedAbility<Tags, Payload>>,
    ability_index_by_id: HashMap<AbilityId, usize>,
    active_abilities: Vec<ActiveAbility<Tags, Payload>>,
    active_index_by_activation_id: HashMap<AbilityActivationId, usize>,
}

/// Stored ability record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantedAbility<Tags, Payload>
where
    Tags: TagCollection,
{
    pub id: AbilityId,
    pub definition_key: Option<String>,
    pub owner_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
}

/// Grants and active executions removed while cleaning up one owner object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevokedOwnerAbilities<Tags, Payload>
where
    Tags: TagCollection,
{
    pub grants: Vec<GrantedAbility<Tags, Payload>>,
    pub active_abilities: Vec<ActiveAbility<Tags, Payload>>,
}

impl<Tags, Payload> GrantedAbility<Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn has_tag(&self, tag: &Tags::Tag) -> bool {
        self.tags.has_tag(tag)
    }
}

struct AbilityActivationSeed<Tags, Payload>
where
    Tags: TagCollection,
{
    ability_id: AbilityId,
    definition_key: Option<String>,
    owner_id: ObjectId,
    tags: Tags,
    payload: Payload,
}

impl<Tags, Payload> AbilityActivationSeed<Tags, Payload>
where
    Tags: TagCollection,
{
    fn from_ability(ability: &GrantedAbility<Tags, Payload>) -> Self
    where
        Payload: Clone,
    {
        Self {
            ability_id: ability.id,
            definition_key: ability.definition_key.clone(),
            owner_id: ability.owner_id,
            tags: ability.tags.clone(),
            payload: ability.payload.clone(),
        }
    }

    fn attempt_view(&self) -> AbilityActivationAttemptView<'_, Tags, Payload> {
        AbilityActivationAttemptView {
            ability_id: self.ability_id,
            definition_key: self.definition_key.as_deref(),
            owner_id: self.owner_id,
            tags: &self.tags,
            payload: &self.payload,
        }
    }

    fn into_active(
        self,
        activation_id: AbilityActivationId,
        commit_timing: AbilityCommitTiming,
        committed: bool,
    ) -> ActiveAbility<Tags, Payload> {
        ActiveAbility {
            activation_id,
            ability_id: self.ability_id,
            definition_key: self.definition_key,
            owner_id: self.owner_id,
            tags: self.tags,
            payload: self.payload,
            commit_timing,
            committed,
        }
    }
}

impl<Tags, Payload> AbilityStore<Tags, Payload>
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
    pub fn grant(&mut self, input: Grant<Tags, Payload>) -> AbilityId {
        self.grant_with_definition_key(None, input)
    }

    fn grant_with_definition_key(
        &mut self,
        definition_key: Option<String>,
        input: Grant<Tags, Payload>,
    ) -> AbilityId {
        let id = self.next_id;
        self.next_id = AbilityId::new(self.next_id.get() + 1);
        self.ability_index_by_id.insert(id, self.abilities.len());
        self.abilities.push(GrantedAbility {
            id,
            definition_key,
            owner_id: input.owner_id,
            tags: input.tags,
            payload: input.payload,
        });
        id
    }

    /// Grants a new ability after validating that its owner is live.
    pub fn grant_checked(
        &mut self,
        objects: &ObjectStore,
        input: Grant<Tags, Payload>,
    ) -> Result<AbilityId, AbilityGrantError> {
        if !objects.exists(input.owner_id) {
            return Err(AbilityGrantError::InvalidOwner {
                owner_id: input.owner_id,
            });
        }

        Ok(self.grant(input))
    }

    /// Validates an authorable definition before granting a runtime ability.
    pub fn grant_with_definition<PayloadSchema>(
        &mut self,
        definition: &AbilityDefinition<PayloadSchema>,
        input: Grant<Tags, Payload>,
    ) -> Result<AbilityId, AbilityDefinitionError> {
        definition.validate()?;
        Ok(self.grant_with_definition_key(Some(definition.key.clone()), input))
    }

    /// Grants an ability by looking up a previously validated definition key.
    pub fn grant_registered<PayloadSchema>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        key: &str,
        input: Grant<Tags, Payload>,
    ) -> Result<AbilityId, AbilityDefinitionRegistryError> {
        let definition = definitions.require(key)?;
        Ok(self.grant_with_definition_key(Some(definition.key.clone()), input))
    }

    /// Revokes granted and active abilities owned by `owner_id`.
    #[must_use]
    pub fn revoke_owner(&mut self, owner_id: ObjectId) -> RevokedOwnerAbilities<Tags, Payload> {
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
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
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
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
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
    pub fn get(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Payload>> {
        self.find(ability_id)
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

    #[must_use]
    pub fn active_activation_count(&self) -> usize {
        self.active_abilities.len()
    }

    #[must_use]
    pub fn active_activations(&self) -> &[ActiveAbility<Tags, Payload>] {
        &self.active_abilities
    }

    #[must_use]
    pub fn get_active_activation(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Payload>> {
        self.find_active(activation_id)
    }

    /// Begins an activation and stores active execution state.
    pub async fn begin_activation_with<Context, Hooks>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.begin_activation_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            |_| {},
        )
        .await
    }

    /// Begins an activation for an expected owner.
    ///
    /// This checked wrapper rejects invalid expected owners and owner/ability
    /// mismatches before caller-owned hooks run.
    pub async fn begin_activation_for_owner_with<Context, Hooks>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
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
        .await
    }

    /// Begins an activation and emits owned attempt, rejection, start, and commit facts.
    pub async fn begin_activation_with_events<Context, Hooks, F>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_activation_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            |event| emit(event.to_owned_event()),
        )
        .await
    }

    /// Begins an activation and streams borrowed lifecycle facts.
    pub async fn begin_activation_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(ability_index) = self.find_index(ability_id) else {
            let reason = AbilityActivationRejectionReason::MissingAbility;
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: None,
                    reason,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::MissingAbility,
            ));
        };

        emit(AbilityLifecycleEventView::Attempted(
            Self::attempt_view_from_ability(&self.abilities[ability_index]),
        ));
        let seed = AbilityActivationSeed::from_ability(&self.abilities[ability_index]);

        match hooks.can_activate(context, seed.attempt_view()).await {
            Ok(AbilityActivationDecision::Allow) => {}
            Ok(AbilityActivationDecision::Block(block_reason)) => {
                let reason = AbilityActivationRejectionReason::Blocked;
                emit(AbilityLifecycleEventView::Rejected(
                    AbilityActivationRejectionView {
                        attempt: Some(seed.attempt_view()),
                        reason,
                    },
                ));
                return Err(AbilityActivationError::Blocked(block_reason));
            }
            Err(error) => {
                let reason = AbilityActivationRejectionReason::Hook;
                emit(AbilityLifecycleEventView::Rejected(
                    AbilityActivationRejectionView {
                        attempt: Some(seed.attempt_view()),
                        reason,
                    },
                ));
                return Err(AbilityActivationError::hook(
                    AbilityHookPhase::CanActivate,
                    error,
                ));
            }
        }

        let activation_id = self.next_activation_id;
        self.next_activation_id = AbilityActivationId::new(self.next_activation_id.get() + 1);
        let active = seed.into_active(activation_id, commit_timing, false);
        self.push_active_ability(active);
        let active_index = self
            .find_active_index(activation_id)
            .expect("activation was just pushed");
        emit(AbilityLifecycleEventView::Started(
            (&self.active_abilities[active_index]).into(),
        ));

        let active_snapshot = self.active_abilities[active_index].clone();
        if let Err(error) = hooks.on_start(context, (&active_snapshot).into()).await {
            let canceled = self.remove_active_at_index(active_index);
            emit(AbilityLifecycleEventView::Canceled((&canceled).into()));
            return Err(AbilityActivationError::hook(AbilityHookPhase::Start, error));
        }

        if commit_timing == AbilityCommitTiming::OnStart {
            self.commit_activation_with_borrowed_events(activation_id, context, hooks, &mut emit)
                .await?;
        }

        Ok(activation_id)
    }

    /// Begins an activation for an expected owner and emits lifecycle facts.
    pub async fn begin_activation_for_owner_with_events<Context, Hooks, F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_activation_for_owner_with_borrowed_events(
            owner_id,
            ability_id,
            commit_timing,
            context,
            hooks,
            |event| emit(event.to_owned_event()),
        )
        .await
    }

    /// Begins an activation for an expected owner and streams borrowed lifecycle facts.
    pub async fn begin_activation_for_owner_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        if owner_id.is_invalid() {
            let reason = AbilityActivationRejectionReason::InvalidOwner;
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: None,
                    reason,
                },
            ));
            return Err(AbilityActivationError::Ability(
                AbilityError::InvalidOwner { owner_id },
            ));
        }

        let Some(ability_index) = self.find_index(ability_id) else {
            let reason = AbilityActivationRejectionReason::MissingAbility;
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: None,
                    reason,
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
            let reason = AbilityActivationRejectionReason::OwnerMismatch;
            emit(AbilityLifecycleEventView::Rejected(
                AbilityActivationRejectionView {
                    attempt: Some(Self::attempt_view_from_ability(
                        &self.abilities[ability_index],
                    )),
                    reason,
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
            .await
    }

    /// Begins an activation using commit timing from the granted ability's registered definition.
    pub async fn begin_registered_activation_with<PayloadSchema, Context, Hooks>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<
        AbilityActivationId,
        RegisteredAbilityActivationError<Hooks::Error, Hooks::BlockReason>,
    >
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.begin_registered_activation_with_borrowed_events(
            definitions,
            ability_id,
            context,
            hooks,
            |_| {},
        )
        .await
    }

    /// Begins an activation from a registered definition and emits owned lifecycle facts.
    pub async fn begin_registered_activation_with_events<PayloadSchema, Context, Hooks, F>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<
        AbilityActivationId,
        RegisteredAbilityActivationError<Hooks::Error, Hooks::BlockReason>,
    >
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_registered_activation_with_borrowed_events(
            definitions,
            ability_id,
            context,
            hooks,
            |event| emit(event.to_owned_event()),
        )
        .await
    }

    /// Begins an activation from a registered definition and streams borrowed facts.
    pub async fn begin_registered_activation_with_borrowed_events<
        PayloadSchema,
        Context,
        Hooks,
        F,
    >(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        context: &mut Context,
        hooks: &mut Hooks,
        emit: F,
    ) -> Result<
        AbilityActivationId,
        RegisteredAbilityActivationError<Hooks::Error, Hooks::BlockReason>,
    >
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(ability) = self.find(ability_id) else {
            return self
                .begin_activation_with_borrowed_events(
                    ability_id,
                    AbilityCommitTiming::OnStart,
                    context,
                    hooks,
                    emit,
                )
                .await
                .map_err(RegisteredAbilityActivationError::Activation);
        };
        let definition_key = ability
            .definition_key
            .as_deref()
            .ok_or(RegisteredAbilityActivationError::MissingGrantedDefinitionKey { ability_id })?;
        let definition = definitions
            .require(definition_key)
            .map_err(RegisteredAbilityActivationError::Definition)?;
        self.begin_activation_with_borrowed_events(
            ability_id,
            definition.commit_timing,
            context,
            hooks,
            emit,
        )
        .await
        .map_err(RegisteredAbilityActivationError::Activation)
    }

    /// Runs an instant activation without emitting lifecycle facts.
    pub async fn activate_instant_with<Context, Hooks, Execute>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        execute: Execute,
    ) -> AbilityEndOutcomeResult<Tags, Payload, Hooks::Error, Hooks::BlockReason>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        Execute: FnOnce(&mut Context, &ActiveAbility<Tags, Payload>) -> Result<(), Hooks::Error>,
    {
        self.activate_instant_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            execute,
            |_| {},
        )
        .await
    }

    /// Runs an instant activation and emits owned lifecycle facts.
    pub async fn activate_instant_with_events<Context, Hooks, Execute, F>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        execute: Execute,
        mut emit: F,
    ) -> AbilityEndOutcomeResult<Tags, Payload, Hooks::Error, Hooks::BlockReason>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        Execute: FnOnce(&mut Context, &ActiveAbility<Tags, Payload>) -> Result<(), Hooks::Error>,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.activate_instant_with_borrowed_events(
            ability_id,
            commit_timing,
            context,
            hooks,
            execute,
            |event| emit(event.to_owned_event()),
        )
        .await
    }

    /// Runs an instant activation and streams borrowed lifecycle facts.
    pub async fn activate_instant_with_borrowed_events<Context, Hooks, Execute, F>(
        &mut self,
        ability_id: AbilityId,
        commit_timing: AbilityCommitTiming,
        context: &mut Context,
        hooks: &mut Hooks,
        execute: Execute,
        mut emit: F,
    ) -> AbilityEndOutcomeResult<Tags, Payload, Hooks::Error, Hooks::BlockReason>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        Execute: FnOnce(&mut Context, &ActiveAbility<Tags, Payload>) -> Result<(), Hooks::Error>,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let activation_id = self
            .begin_activation_with_borrowed_events(
                ability_id,
                commit_timing,
                context,
                hooks,
                &mut emit,
            )
            .await?;
        let active_index = self
            .find_active_index(activation_id)
            .expect("activation was just started");

        if let Err(error) = execute(context, &self.active_abilities[active_index]) {
            self.discard_active_with_borrowed_event(activation_id, &mut emit);
            return Err(AbilityActivationError::hook(
                AbilityHookPhase::ExecuteInstant,
                error,
            ));
        }

        let result = self
            .end_activation_with_borrowed_events(activation_id, context, hooks, &mut emit)
            .await;
        if result.is_err() {
            self.discard_active_with_borrowed_event(activation_id, &mut emit);
        }
        result
    }

    /// Commits an active activation if it has not already committed.
    pub async fn commit_activation_with<Context, Hooks>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> Result<AbilityCommitOutcome, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.commit_activation_with_borrowed_events(activation_id, context, hooks, |_| {})
            .await
    }

    /// Commits an active activation and emits an owned commit fact when this call performs the commit.
    pub async fn commit_activation_with_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityCommitOutcome, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.commit_activation_with_borrowed_events(activation_id, context, hooks, |event| {
            emit(event.to_owned_event());
        })
        .await
    }

    /// Commits an active activation and streams a borrowed commit fact when this call performs the commit.
    pub async fn commit_activation_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<AbilityCommitOutcome, AbilityActivationError<Hooks::Error, Hooks::BlockReason>>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let active_index =
            self.find_active_index(activation_id)
                .ok_or(AbilityActivationError::Ability(
                    AbilityError::MissingActivation,
                ))?;
        if self.active_abilities[active_index].committed {
            return Ok(AbilityCommitOutcome::AlreadyCommitted);
        }

        let active_snapshot = self.active_abilities[active_index].clone();
        hooks
            .on_commit(context, (&active_snapshot).into())
            .await
            .map_err(|error| AbilityActivationError::hook(AbilityHookPhase::Commit, error))?;

        let Some(active_index) = self.find_active_index(activation_id) else {
            return Err(AbilityActivationError::Ability(
                AbilityError::MissingActivation,
            ));
        };
        self.active_abilities[active_index].committed = true;
        let active = ActiveAbilityView::from(&self.active_abilities[active_index]);
        emit(AbilityLifecycleEventView::Committed(
            AbilityActivationCommitView {
                attempt: active.attempt_view(),
            },
        ));
        Ok(AbilityCommitOutcome::Committed)
    }

    /// Ends an active activation without emitting lifecycle facts.
    pub async fn end_activation_with<Context, Hooks>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
    ) -> AbilityEndOutcomeResult<Tags, Payload, Hooks::Error, Hooks::BlockReason>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.end_activation_with_borrowed_events(activation_id, context, hooks, |_| {})
            .await
    }

    /// Ends an active activation and emits owned commit/end facts in deterministic order.
    pub async fn end_activation_with_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> AbilityEndOutcomeResult<Tags, Payload, Hooks::Error, Hooks::BlockReason>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.end_activation_with_borrowed_events(activation_id, context, hooks, |event| {
            emit(event.to_owned_event());
        })
        .await
    }

    /// Ends an active activation and streams borrowed commit/end facts in deterministic order.
    pub async fn end_activation_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> AbilityEndOutcomeResult<Tags, Payload, Hooks::Error, Hooks::BlockReason>
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(AbilityEndOutcome::MissingActivation);
        };
        let needs_commit = self.active_abilities[active_index].commit_timing
            == AbilityCommitTiming::OnEnd
            && !self.active_abilities[active_index].committed;
        if needs_commit {
            self.commit_activation_with_borrowed_events(activation_id, context, hooks, &mut emit)
                .await?;
        }

        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(AbilityEndOutcome::MissingActivation);
        };
        let active_snapshot = self.active_abilities[active_index].clone();
        hooks
            .on_end(context, (&active_snapshot).into())
            .await
            .map_err(|error| AbilityActivationError::hook(AbilityHookPhase::End, error))?;

        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(AbilityEndOutcome::MissingActivation);
        };
        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::Ended((&active).into()));
        Ok(AbilityEndOutcome::Ended(active))
    }

    /// Cancels an active activation without hook execution or lifecycle facts.
    pub fn cancel_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> AbilityCancelOutcome<Tags, Payload> {
        let Some(active_index) = self.find_active_index(activation_id) else {
            return AbilityCancelOutcome::MissingActivation;
        };
        AbilityCancelOutcome::Canceled(self.remove_active_at_index(active_index))
    }

    /// Cancels an active activation and emits a cancel fact.
    pub async fn cancel_activation_with_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<
        AbilityCancelOutcome<Tags, Payload>,
        AbilityActivationError<Hooks::Error, Hooks::BlockReason>,
    >
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.cancel_activation_with_borrowed_events(activation_id, context, hooks, |event| {
            emit(event.to_owned_event())
        })
        .await
    }

    /// Cancels an active activation and streams a borrowed cancel fact.
    pub async fn cancel_activation_with_borrowed_events<Context, Hooks, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        hooks: &mut Hooks,
        mut emit: F,
    ) -> Result<
        AbilityCancelOutcome<Tags, Payload>,
        AbilityActivationError<Hooks::Error, Hooks::BlockReason>,
    >
    where
        Hooks: AbilityHooks<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(AbilityCancelOutcome::MissingActivation);
        };
        let active_snapshot = self.active_abilities[active_index].clone();
        hooks
            .on_cancel(context, (&active_snapshot).into())
            .await
            .map_err(|error| AbilityActivationError::hook(AbilityHookPhase::Cancel, error))?;

        let Some(active_index) = self.find_active_index(activation_id) else {
            return Ok(AbilityCancelOutcome::MissingActivation);
        };
        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::Canceled((&active).into()));
        Ok(AbilityCancelOutcome::Canceled(active))
    }

    fn discard_active_with_borrowed_event<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> AbilityCancelOutcome<Tags, Payload>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let Some(active_index) = self.find_active_index(activation_id) else {
            return AbilityCancelOutcome::MissingActivation;
        };

        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::Canceled((&active).into()));
        AbilityCancelOutcome::Canceled(active)
    }

    fn attempt_view_from_ability(
        ability: &GrantedAbility<Tags, Payload>,
    ) -> AbilityActivationAttemptView<'_, Tags, Payload> {
        AbilityActivationAttemptView {
            ability_id: ability.id,
            definition_key: ability.definition_key.as_deref(),
            owner_id: ability.owner_id,
            tags: &ability.tags,
            payload: &ability.payload,
        }
    }

    fn find_index(&self, ability_id: AbilityId) -> Option<usize> {
        self.ability_index_by_id.get(&ability_id).copied()
    }

    fn find(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Payload>> {
        self.find_index(ability_id)
            .map(|index| &self.abilities[index])
    }

    fn find_active(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Payload>> {
        self.find_active_index(activation_id)
            .map(|index| &self.active_abilities[index])
    }

    fn find_active_index(&self, activation_id: AbilityActivationId) -> Option<usize> {
        self.active_index_by_activation_id
            .get(&activation_id)
            .copied()
    }

    fn remove_ability_at_index(&mut self, ability_index: usize) -> GrantedAbility<Tags, Payload> {
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

    fn push_active_ability(&mut self, active: ActiveAbility<Tags, Payload>) {
        self.active_index_by_activation_id
            .insert(active.activation_id, self.active_abilities.len());
        self.active_abilities.push(active);
    }

    fn remove_active_at_index(&mut self, active_index: usize) -> ActiveAbility<Tags, Payload> {
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

impl<Tags, Payload> Default for AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
{
    fn default() -> Self {
        Self::new()
    }
}
