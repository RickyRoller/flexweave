use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;

use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;

use super::activation_request::{
    AbilityActivationRequest, AbilityActivationRequestError, AbilityActivationSeed,
    RegisteredActivationRequestError, resolve_activation_request, resolve_owner_activation_request,
    resolve_registered_activation_request,
};
use super::definition::{
    AbilityDefinition, AbilityDefinitionError, AbilityDefinitionRegistryError, AbilityDefinitions,
};
use super::events::{
    AbilityActivationRejectionReason, AbilityActivationRejectionView, AbilityLifecycleEvent,
    AbilityLifecycleEventView, ActiveAbility, ActiveAbilityView,
};
use super::hooks::{
    AbilityActivationDecision, AbilityActivationGate, AbilityCommitAction, AllowActivation,
    NoCommitAction,
};
use super::ids::{AbilityActivationId, AbilityId};

/// Ability begin errors, including caller-owned blocking and gate failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityBeginError<GateError, BlockReason = ()> {
    Ability(AbilityError),
    Blocked(BlockReason),
    Gate(GateError),
}

impl<GateError, BlockReason> fmt::Display for AbilityBeginError<GateError, BlockReason>
where
    GateError: fmt::Display,
    BlockReason: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ability(error) => write!(formatter, "ability activation failed: {error}"),
            Self::Blocked(reason) => write!(formatter, "ability activation blocked: {reason:?}"),
            Self::Gate(error) => write!(formatter, "ability activation gate failed: {error}"),
        }
    }
}

impl<GateError, BlockReason> std::error::Error for AbilityBeginError<GateError, BlockReason>
where
    GateError: std::error::Error + 'static,
    BlockReason: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ability(error) => Some(error),
            Self::Blocked(_) => None,
            Self::Gate(error) => Some(error),
        }
    }
}

/// Ability commit errors, including caller-owned action failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityCommitError<CommitError> {
    Ability(AbilityError),
    Action(CommitError),
}

impl<CommitError> fmt::Display for AbilityCommitError<CommitError>
where
    CommitError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ability(error) => write!(formatter, "ability commit failed: {error}"),
            Self::Action(error) => write!(formatter, "ability commit action failed: {error}"),
        }
    }
}

impl<CommitError> std::error::Error for AbilityCommitError<CommitError>
where
    CommitError: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ability(error) => Some(error),
            Self::Action(error) => Some(error),
        }
    }
}

/// Ability end command errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityEndError {
    MissingActivation,
    UncommittedActivation,
}

impl fmt::Display for AbilityEndError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingActivation => "missing ability activation",
            Self::UncommittedActivation => "uncommitted ability activation",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityEndError {}

/// Ability rollback command errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityRollbackError {
    MissingActivation,
    AlreadyCommitted,
}

impl fmt::Display for AbilityRollbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingActivation => "missing ability activation",
            Self::AlreadyCommitted => "ability activation is already committed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityRollbackError {}

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

/// Outcome of a rollback command for an active ability activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityRollbackOutcome<Tags, Payload>
where
    Tags: TagCollection,
{
    RolledBack(ActiveAbility<Tags, Payload>),
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

/// Registered activation errors for key-aware ability workflows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisteredAbilityActivationError<GateError, BlockReason = ()> {
    MissingGrantedDefinitionKey { ability_id: AbilityId },
    Definition(AbilityDefinitionRegistryError),
    Activation(AbilityBeginError<GateError, BlockReason>),
}

impl<GateError, BlockReason> fmt::Display
    for RegisteredAbilityActivationError<GateError, BlockReason>
where
    GateError: fmt::Display,
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

impl<GateError, BlockReason> std::error::Error
    for RegisteredAbilityActivationError<GateError, BlockReason>
where
    GateError: std::error::Error + 'static,
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
        self.revoke_owner_with_borrowed_events(owner_id, |event| {
            emit(event.to_owned_event());
        })
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
        let mut active_abilities = Vec::new();
        let mut active_index = 0;
        while active_index < self.active_abilities.len() {
            if self.active_abilities[active_index].owner_id == owner_id {
                let active = self.remove_active_at_index(active_index);
                emit(AbilityLifecycleEventView::Revoked((&active).into()));
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

    /// Begins an activation without a caller-owned activation gate.
    pub fn begin_activation(
        &mut self,
        ability_id: AbilityId,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
    {
        self.begin_activation_with_borrowed_events(ability_id, |_| {})
    }

    /// Begins an activation and emits owned attempt, rejection, and start facts.
    pub fn begin_activation_with_events<F>(
        &mut self,
        ability_id: AbilityId,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_activation_with_borrowed_events(ability_id, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Begins an activation and streams borrowed lifecycle facts.
    pub fn begin_activation_with_borrowed_events<F>(
        &mut self,
        ability_id: AbilityId,
        emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let context = ();
        let mut gate = AllowActivation;
        self.begin_activation_with_gate_borrowed_events(ability_id, &context, &mut gate, emit)
    }

    /// Begins an activation after consulting a synchronous caller-owned gate.
    pub fn begin_activation_with_gate<Context, Gate>(
        &mut self,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
    ) -> Result<AbilityActivationId, AbilityBeginError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.begin_activation_with_gate_borrowed_events(ability_id, context, gate, |_| {})
    }

    /// Begins a gate-backed activation and emits owned attempt, rejection, and start facts.
    pub fn begin_activation_with_gate_events<Context, Gate, F>(
        &mut self,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_activation_with_gate_borrowed_events(ability_id, context, gate, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Begins a gate-backed activation and streams borrowed lifecycle facts.
    pub fn begin_activation_with_gate_borrowed_events<Context, Gate, F>(
        &mut self,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let request = match resolve_activation_request(self.find(ability_id)) {
            Ok(request) => request,
            Err(error) => {
                Self::emit_activation_request_rejection(&error, &mut emit);
                return Err(AbilityBeginError::Ability(error.ability_error()));
            }
        };
        let seed = Self::prepare_activation_seed(request, context, gate, &mut emit)?;

        Ok(self.start_activation_from_seed(seed, &mut emit))
    }

    /// Begins an activation for an expected owner without a caller-owned gate.
    pub fn begin_activation_for_owner(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
    {
        self.begin_activation_for_owner_with_borrowed_events(owner_id, ability_id, |_| {})
    }

    /// Begins an activation for an expected owner and emits owned lifecycle facts.
    pub fn begin_activation_for_owner_with_events<F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_activation_for_owner_with_borrowed_events(owner_id, ability_id, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Begins an activation for an expected owner and streams borrowed lifecycle facts.
    pub fn begin_activation_for_owner_with_borrowed_events<F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let context = ();
        let mut gate = AllowActivation;
        self.begin_activation_for_owner_with_gate_borrowed_events(
            owner_id, ability_id, &context, &mut gate, emit,
        )
    }

    /// Begins a gate-backed activation for an expected owner.
    pub fn begin_activation_for_owner_with_gate<Context, Gate>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
    ) -> Result<AbilityActivationId, AbilityBeginError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.begin_activation_for_owner_with_gate_borrowed_events(
            owner_id,
            ability_id,
            context,
            gate,
            |_| {},
        )
    }

    /// Begins a gate-backed activation for an expected owner and emits owned facts.
    pub fn begin_activation_for_owner_with_gate_events<Context, Gate, F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_activation_for_owner_with_gate_borrowed_events(
            owner_id,
            ability_id,
            context,
            gate,
            |event| emit(event.to_owned_event()),
        )
    }

    /// Begins a gate-backed activation for an expected owner and streams borrowed facts.
    pub fn begin_activation_for_owner_with_gate_borrowed_events<Context, Gate, F>(
        &mut self,
        owner_id: ObjectId,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
        mut emit: F,
    ) -> Result<AbilityActivationId, AbilityBeginError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let request = match resolve_owner_activation_request(owner_id, self.find(ability_id)) {
            Ok(request) => request,
            Err(error) => {
                Self::emit_activation_request_rejection(&error, &mut emit);
                return Err(AbilityBeginError::Ability(error.ability_error()));
            }
        };
        let seed = Self::prepare_activation_seed(request, context, gate, &mut emit)?;

        Ok(self.start_activation_from_seed(seed, &mut emit))
    }

    /// Begins an activation using the granted ability's registered definition key.
    pub fn begin_registered_activation<PayloadSchema>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
    ) -> Result<AbilityActivationId, RegisteredAbilityActivationError<Infallible, Infallible>>
    where
        Payload: Clone,
    {
        self.begin_registered_activation_with_borrowed_events(definitions, ability_id, |_| {})
    }

    /// Begins a registered activation and emits owned lifecycle facts.
    pub fn begin_registered_activation_with_events<PayloadSchema, F>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        mut emit: F,
    ) -> Result<AbilityActivationId, RegisteredAbilityActivationError<Infallible, Infallible>>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_registered_activation_with_borrowed_events(definitions, ability_id, |event| {
            emit(event.to_owned_event())
        })
    }

    /// Begins a registered activation and streams borrowed lifecycle facts.
    pub fn begin_registered_activation_with_borrowed_events<PayloadSchema, F>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        emit: F,
    ) -> Result<AbilityActivationId, RegisteredAbilityActivationError<Infallible, Infallible>>
    where
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let context = ();
        let mut gate = AllowActivation;
        self.begin_registered_activation_with_gate_borrowed_events(
            definitions,
            ability_id,
            &context,
            &mut gate,
            emit,
        )
    }

    /// Begins a registered activation after consulting a synchronous gate.
    pub fn begin_registered_activation_with_gate<PayloadSchema, Context, Gate>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
    ) -> Result<AbilityActivationId, RegisteredAbilityActivationError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
    {
        self.begin_registered_activation_with_gate_borrowed_events(
            definitions,
            ability_id,
            context,
            gate,
            |_| {},
        )
    }

    /// Begins a gate-backed registered activation and emits owned lifecycle facts.
    pub fn begin_registered_activation_with_gate_events<PayloadSchema, Context, Gate, F>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
        mut emit: F,
    ) -> Result<AbilityActivationId, RegisteredAbilityActivationError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.begin_registered_activation_with_gate_borrowed_events(
            definitions,
            ability_id,
            context,
            gate,
            |event| emit(event.to_owned_event()),
        )
    }

    /// Begins a gate-backed registered activation and streams borrowed facts.
    pub fn begin_registered_activation_with_gate_borrowed_events<PayloadSchema, Context, Gate, F>(
        &mut self,
        definitions: &AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
        context: &Context,
        gate: &mut Gate,
        emit: F,
    ) -> Result<AbilityActivationId, RegisteredAbilityActivationError<Gate::Error, Gate::BlockReason>>
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let mut emit = emit;
        let request = match resolve_registered_activation_request(
            definitions,
            ability_id,
            self.find(ability_id),
        ) {
            Ok(request) => request,
            Err(RegisteredActivationRequestError::Activation(error)) => {
                Self::emit_activation_request_rejection(&error, &mut emit);
                return Err(RegisteredAbilityActivationError::Activation(
                    AbilityBeginError::Ability(error.ability_error()),
                ));
            }
            Err(RegisteredActivationRequestError::MissingGrantedDefinitionKey { ability_id }) => {
                return Err(
                    RegisteredAbilityActivationError::MissingGrantedDefinitionKey { ability_id },
                );
            }
            Err(RegisteredActivationRequestError::Definition(error)) => {
                return Err(RegisteredAbilityActivationError::Definition(error));
            }
        };
        let seed = Self::prepare_activation_seed(request, context, gate, &mut emit)
            .map_err(RegisteredAbilityActivationError::Activation)?;

        Ok(self.start_activation_from_seed(seed, &mut emit))
    }

    /// Commits an active activation with no caller-owned action.
    pub fn commit_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Infallible>> {
        self.commit_activation_with_borrowed_events(activation_id, |_| {})
    }

    /// Commits an active activation and emits an owned commit fact when this call commits.
    pub fn commit_activation_with_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        mut emit: F,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Infallible>>
    where
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.commit_activation_with_borrowed_events(activation_id, |event| {
            emit(event.to_owned_event());
        })
    }

    /// Commits an active activation and streams a borrowed commit fact when this call commits.
    pub fn commit_activation_with_borrowed_events<F>(
        &mut self,
        activation_id: AbilityActivationId,
        emit: F,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Infallible>>
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let mut context = ();
        let mut action = NoCommitAction;
        self.commit_activation_with_action_borrowed_events(
            activation_id,
            &mut context,
            &mut action,
            emit,
        )
    }

    /// Commits an active activation after applying a synchronous caller-owned action.
    pub fn commit_activation_with_action<Context, Action>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        action: &mut Action,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Action::Error>>
    where
        Action: AbilityCommitAction<Context, Tags, Payload>,
    {
        self.commit_activation_with_action_borrowed_events(activation_id, context, action, |_| {})
    }

    /// Commits with a caller-owned action and emits owned lifecycle facts.
    pub fn commit_activation_with_action_events<Context, Action, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        action: &mut Action,
        mut emit: F,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Action::Error>>
    where
        Action: AbilityCommitAction<Context, Tags, Payload>,
        Payload: Clone,
        F: FnMut(AbilityLifecycleEvent<Tags, Payload>),
    {
        self.commit_activation_with_action_borrowed_events(
            activation_id,
            context,
            action,
            |event| emit(event.to_owned_event()),
        )
    }

    /// Commits with a caller-owned action and streams borrowed lifecycle facts.
    pub fn commit_activation_with_action_borrowed_events<Context, Action, F>(
        &mut self,
        activation_id: AbilityActivationId,
        context: &mut Context,
        action: &mut Action,
        mut emit: F,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Action::Error>>
    where
        Action: AbilityCommitAction<Context, Tags, Payload>,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let active_index = self
            .find_active_index(activation_id)
            .ok_or(AbilityCommitError::Ability(AbilityError::MissingActivation))?;
        if self.active_abilities[active_index].committed {
            return Ok(AbilityCommitOutcome::AlreadyCommitted);
        }

        let active_view = ActiveAbilityView::from(&self.active_abilities[active_index]);
        if let Err(error) = action.apply_commit(context, active_view) {
            let active = self.remove_active_at_index(active_index);
            emit(AbilityLifecycleEventView::RolledBack((&active).into()));
            return Err(AbilityCommitError::Action(error));
        }

        self.active_abilities[active_index].committed = true;
        emit(AbilityLifecycleEventView::Committed(
            (&self.active_abilities[active_index]).into(),
        ));
        Ok(AbilityCommitOutcome::Committed)
    }

    /// Ends a committed active activation without emitting lifecycle facts.
    pub fn end_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError> {
        self.end_activation_with_borrowed_events(activation_id, |_| {})
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
        self.end_activation_with_borrowed_events(activation_id, |event| {
            emit(event.to_owned_event());
        })
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
        let Some(active_index) = self.find_active_index(activation_id) else {
            return Err(AbilityEndError::MissingActivation);
        };
        if !self.active_abilities[active_index].committed {
            return Err(AbilityEndError::UncommittedActivation);
        }

        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::Ended((&active).into()));
        Ok(AbilityEndOutcome::Ended(active))
    }

    /// Cancels an active activation without lifecycle facts.
    pub fn cancel_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> AbilityCancelOutcome<Tags, Payload> {
        self.cancel_activation_with_borrowed_events(activation_id, |_| {})
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
        self.cancel_activation_with_borrowed_events(activation_id, |event| {
            emit(event.to_owned_event());
        })
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
        let Some(active_index) = self.find_active_index(activation_id) else {
            return AbilityCancelOutcome::MissingActivation;
        };
        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::Canceled((&active).into()));
        AbilityCancelOutcome::Canceled(active)
    }

    /// Rolls back an uncommitted active activation without lifecycle facts.
    pub fn rollback_activation(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError> {
        self.rollback_activation_with_borrowed_events(activation_id, |_| {})
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
        self.rollback_activation_with_borrowed_events(activation_id, |event| {
            emit(event.to_owned_event());
        })
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
        let Some(active_index) = self.find_active_index(activation_id) else {
            return Err(AbilityRollbackError::MissingActivation);
        };
        if self.active_abilities[active_index].committed {
            return Err(AbilityRollbackError::AlreadyCommitted);
        }

        let active = self.remove_active_at_index(active_index);
        emit(AbilityLifecycleEventView::RolledBack((&active).into()));
        Ok(AbilityRollbackOutcome::RolledBack(active))
    }

    fn emit_activation_request_rejection<F>(
        error: &AbilityActivationRequestError<'_, Tags, Payload>,
        emit: &mut F,
    ) where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        if let Some(attempt) = error.attempt_view() {
            emit(AbilityLifecycleEventView::Attempted(attempt));
        }
        emit(AbilityLifecycleEventView::Rejected(
            AbilityActivationRejectionView {
                attempt: error.attempt_view(),
                reason: error.reason(),
            },
        ));
    }

    fn prepare_activation_seed<Context, Gate, F>(
        request: AbilityActivationRequest<'_, Tags, Payload>,
        context: &Context,
        gate: &mut Gate,
        emit: &mut F,
    ) -> Result<
        AbilityActivationSeed<Tags, Payload>,
        AbilityBeginError<Gate::Error, Gate::BlockReason>,
    >
    where
        Gate: AbilityActivationGate<Context, Tags, Payload>,
        Payload: Clone,
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let attempt = request.attempt_view();
        emit(AbilityLifecycleEventView::Attempted(attempt));

        match gate.can_activate(context, request.attempt_view()) {
            Ok(AbilityActivationDecision::Allow) => Ok(request.to_seed()),
            Ok(AbilityActivationDecision::Block(block_reason)) => {
                emit(AbilityLifecycleEventView::Rejected(
                    AbilityActivationRejectionView {
                        attempt: Some(request.attempt_view()),
                        reason: AbilityActivationRejectionReason::Blocked,
                    },
                ));
                Err(AbilityBeginError::Blocked(block_reason))
            }
            Err(error) => {
                emit(AbilityLifecycleEventView::Rejected(
                    AbilityActivationRejectionView {
                        attempt: Some(request.attempt_view()),
                        reason: AbilityActivationRejectionReason::Gate,
                    },
                ));
                Err(AbilityBeginError::Gate(error))
            }
        }
    }

    fn start_activation_from_seed<F>(
        &mut self,
        seed: AbilityActivationSeed<Tags, Payload>,
        emit: &mut F,
    ) -> AbilityActivationId
    where
        F: for<'event> FnMut(AbilityLifecycleEventView<'event, Tags, Payload>),
    {
        let activation_id = self.next_activation_id;
        self.next_activation_id = AbilityActivationId::new(self.next_activation_id.get() + 1);
        let active = seed.into_active(activation_id, false);
        self.push_active_ability(active);
        let active_index = self
            .find_active_index(activation_id)
            .expect("activation was just pushed");
        emit(AbilityLifecycleEventView::Started(
            (&self.active_abilities[active_index]).into(),
        ));
        activation_id
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
