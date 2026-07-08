use std::convert::Infallible;
use std::fmt;

use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;

use super::AbilityLifecycleEventView;
use super::activation_request::{
    AbilityActivationRequestError, resolve_activation_request, resolve_owner_activation_request,
};
use super::definition::{AbilityDefinition, AbilityDefinitionRegistryError, AbilityDefinitions};
use super::events::{AbilityActivationRejectionReason, AbilityActivationRejectionView};
use super::hooks::{
    AbilityActivationDecision, AbilityActivationExecutor, AbilityCommitExecutor,
    AbilityLifecycleSink, DiscardAbilityLifecycleEvents, NoAbilityActivationExecutor,
    NoAbilityCommitExecutor,
};
use super::ids::{AbilityActivationId, AbilityId};
use super::results::{
    AbilityBeginError, AbilityCancelOutcome, AbilityCommitError, AbilityCommitOutcome,
    AbilityEndError, AbilityEndOutcome, AbilityGrantError, AbilityRollbackError,
    AbilityRollbackOutcome,
};
use super::store::{AbilityStore, Grant, RevokedOwnerAbilities};

/// Ability grant command builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityGrant<'definition, PayloadSchema, Tags, Payload> {
    source: Option<AbilityGrantSource<'definition, PayloadSchema>>,
    objects: Option<&'definition ObjectStore>,
    input: Grant<Tags, Payload>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AbilityGrantSource<'definition, PayloadSchema> {
    Definition(&'definition AbilityDefinition<PayloadSchema>),
    Registered {
        definitions: &'definition AbilityDefinitions<PayloadSchema>,
        key: &'definition str,
    },
}

/// Ability activation command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActivation<'definition, PayloadSchema = ()> {
    ability_id: AbilityId,
    owner_id: Option<ObjectId>,
    definitions: Option<&'definition AbilityDefinitions<PayloadSchema>>,
}

/// Ability activation command failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityActivationError<GateError, BlockReason = Infallible> {
    Activation(AbilityBeginError<GateError, BlockReason>),
    MissingGrantedDefinitionKey { ability_id: AbilityId },
    Definition(AbilityDefinitionRegistryError),
}

/// Ability commit command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityCommit {
    activation_id: AbilityActivationId,
}

/// Ability owner revocation command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityRevokeOwner {
    owner_id: ObjectId,
}

/// Ability activation end command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityEnd {
    activation_id: AbilityActivationId,
}

/// Ability activation cancel command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityCancel {
    activation_id: AbilityActivationId,
}

/// Ability activation rollback command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityRollback {
    activation_id: AbilityActivationId,
}

impl<'definition, Tags, Payload> AbilityGrant<'definition, (), Tags, Payload> {
    #[must_use]
    pub fn new(input: Grant<Tags, Payload>) -> Self {
        Self {
            source: None,
            objects: None,
            input,
        }
    }
}

impl<'definition, PayloadSchema, Tags, Payload>
    AbilityGrant<'definition, PayloadSchema, Tags, Payload>
{
    #[must_use]
    pub fn definition(
        definition: &'definition AbilityDefinition<PayloadSchema>,
        input: Grant<Tags, Payload>,
    ) -> Self {
        Self {
            source: Some(AbilityGrantSource::Definition(definition)),
            objects: None,
            input,
        }
    }

    #[must_use]
    pub fn registered(
        definitions: &'definition AbilityDefinitions<PayloadSchema>,
        key: &'definition str,
        input: Grant<Tags, Payload>,
    ) -> Self {
        Self {
            source: Some(AbilityGrantSource::Registered { definitions, key }),
            objects: None,
            input,
        }
    }

    #[must_use]
    pub fn checked(mut self, objects: &'definition ObjectStore) -> Self {
        self.objects = Some(objects);
        self
    }

    pub fn run(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> Result<AbilityId, AbilityGrantError>
    where
        Tags: TagCollection,
    {
        if let Some(objects) = self.objects
            && !objects.exists(self.input.owner_id)
        {
            return Err(AbilityGrantError::InvalidOwner {
                owner_id: self.input.owner_id,
            });
        }

        let definition_key = match self.source {
            None => None,
            Some(AbilityGrantSource::Definition(definition)) => {
                definition
                    .validate()
                    .map_err(AbilityGrantError::Definition)?;
                Some(definition.key.clone())
            }
            Some(AbilityGrantSource::Registered { definitions, key }) => {
                let definition = definitions
                    .require(key)
                    .map_err(AbilityGrantError::RegisteredDefinition)?;
                Some(definition.key.clone())
            }
        };

        Ok(store.insert_grant(definition_key, self.input))
    }
}

impl AbilityActivation<'_, ()> {
    #[must_use]
    pub const fn new(ability_id: AbilityId) -> Self {
        Self {
            ability_id,
            owner_id: None,
            definitions: None,
        }
    }
}

impl<'definition, PayloadSchema> AbilityActivation<'definition, PayloadSchema> {
    #[must_use]
    pub fn registered(
        definitions: &'definition AbilityDefinitions<PayloadSchema>,
        ability_id: AbilityId,
    ) -> Self {
        Self {
            ability_id,
            owner_id: None,
            definitions: Some(definitions),
        }
    }

    #[must_use]
    pub const fn for_owner(mut self, owner_id: ObjectId) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    pub fn run<Tags, Payload>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> Result<AbilityActivationId, AbilityActivationError<Infallible, Infallible>>
    where
        Tags: TagCollection,
        Payload: Clone,
    {
        let context = ();
        let mut executor = NoAbilityActivationExecutor::new();
        self.run_with_executor(store, &context, &mut executor)
    }

    pub fn run_with_executor<Context, Tags, Payload, Executor>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
        context: &Context,
        executor: &mut Executor,
    ) -> Result<AbilityActivationId, AbilityActivationError<Executor::Error, Executor::BlockReason>>
    where
        Tags: TagCollection,
        Payload: Clone,
        Executor: AbilityActivationExecutor<Context, Tags, Payload>,
    {
        let request = match self.resolve_request(store) {
            Ok(request) => request,
            Err(ActivationResolveError::Activation(error)) => {
                emit_activation_request_rejection(&error, executor);
                return Err(AbilityActivationError::Activation(
                    AbilityBeginError::Ability(error.ability_error()),
                ));
            }
            Err(ActivationResolveError::MissingGrantedDefinitionKey { ability_id }) => {
                return Err(AbilityActivationError::MissingGrantedDefinitionKey { ability_id });
            }
            Err(ActivationResolveError::Definition(error)) => {
                return Err(AbilityActivationError::Definition(error));
            }
        };

        let attempt = request.attempt_view();
        executor.emit_ability_lifecycle(AbilityLifecycleEventView::Attempted(attempt));
        let seed = match executor.can_activate(context, request.attempt_view()) {
            Ok(AbilityActivationDecision::Allow) => request.to_seed(),
            Ok(AbilityActivationDecision::Block(block_reason)) => {
                executor.emit_ability_lifecycle(AbilityLifecycleEventView::Rejected(
                    AbilityActivationRejectionView {
                        attempt: Some(request.attempt_view()),
                        reason: AbilityActivationRejectionReason::Blocked,
                    },
                ));
                return Err(AbilityActivationError::Activation(
                    AbilityBeginError::Blocked(block_reason),
                ));
            }
            Err(error) => {
                executor.emit_ability_lifecycle(AbilityLifecycleEventView::Rejected(
                    AbilityActivationRejectionView {
                        attempt: Some(request.attempt_view()),
                        reason: AbilityActivationRejectionReason::Gate,
                    },
                ));
                return Err(AbilityActivationError::Activation(AbilityBeginError::Gate(
                    error,
                )));
            }
        };

        Ok(store.start_activation_from_seed(seed, &mut |event| {
            executor.emit_ability_lifecycle(event);
        }))
    }

    fn resolve_request<'store, Tags, Payload>(
        &self,
        store: &'store AbilityStore<Tags, Payload>,
    ) -> Result<
        super::activation_request::AbilityActivationRequest<'store, Tags, Payload>,
        ActivationResolveError<'store, Tags, Payload>,
    >
    where
        Tags: TagCollection,
    {
        let request = if let Some(owner_id) = self.owner_id {
            resolve_owner_activation_request(owner_id, store.get(self.ability_id))
                .map_err(ActivationResolveError::Activation)?
        } else {
            resolve_activation_request(store.get(self.ability_id))
                .map_err(ActivationResolveError::Activation)?
        };

        if let Some(definitions) = self.definitions {
            let definition_key = request.attempt_view().definition_key.ok_or(
                ActivationResolveError::MissingGrantedDefinitionKey {
                    ability_id: self.ability_id,
                },
            )?;
            definitions
                .require(definition_key)
                .map_err(ActivationResolveError::Definition)?;
        }

        Ok(request)
    }
}

enum ActivationResolveError<'ability, Tags, Payload>
where
    Tags: TagCollection,
{
    Activation(AbilityActivationRequestError<'ability, Tags, Payload>),
    MissingGrantedDefinitionKey { ability_id: AbilityId },
    Definition(AbilityDefinitionRegistryError),
}

impl<GateError, BlockReason> fmt::Display for AbilityActivationError<GateError, BlockReason>
where
    GateError: fmt::Display,
    BlockReason: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Activation(error) => write!(formatter, "{error}"),
            Self::MissingGrantedDefinitionKey { ability_id } => write!(
                formatter,
                "ability `{ability_id}` was not granted from a registered definition"
            ),
            Self::Definition(error) => {
                write!(formatter, "registered ability activation failed: {error}")
            }
        }
    }
}

impl<GateError, BlockReason> std::error::Error for AbilityActivationError<GateError, BlockReason>
where
    GateError: std::error::Error + 'static,
    BlockReason: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Activation(error) => Some(error),
            Self::MissingGrantedDefinitionKey { .. } => None,
            Self::Definition(error) => Some(error),
        }
    }
}

impl AbilityCommit {
    #[must_use]
    pub const fn new(activation_id: AbilityActivationId) -> Self {
        Self { activation_id }
    }

    pub fn run<Tags, Payload>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Infallible>>
    where
        Tags: TagCollection,
    {
        let mut context = ();
        let mut executor = NoAbilityCommitExecutor::new();
        self.run_with_executor(store, &mut context, &mut executor)
    }

    pub fn run_with_executor<Context, Tags, Payload, Executor>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
        context: &mut Context,
        executor: &mut Executor,
    ) -> Result<AbilityCommitOutcome, AbilityCommitError<Executor::Error>>
    where
        Tags: TagCollection,
        Executor: AbilityCommitExecutor<Context, Tags, Payload>,
    {
        store.commit_with_executor(self.activation_id, context, executor)
    }
}

impl AbilityRevokeOwner {
    #[must_use]
    pub const fn new(owner_id: ObjectId) -> Self {
        Self { owner_id }
    }

    #[must_use]
    pub fn run<Tags, Payload>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        Tags: TagCollection,
    {
        let mut sink = DiscardAbilityLifecycleEvents;
        self.run_with_sink(store, &mut sink)
    }

    pub fn run_with_sink<Tags, Payload, Sink>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
        sink: &mut Sink,
    ) -> RevokedOwnerAbilities<Tags, Payload>
    where
        Tags: TagCollection,
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        store.remove_owner_with_sink(self.owner_id, sink)
    }
}

impl AbilityEnd {
    #[must_use]
    pub const fn new(activation_id: AbilityActivationId) -> Self {
        Self { activation_id }
    }

    pub fn run<Tags, Payload>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError>
    where
        Tags: TagCollection,
    {
        let mut sink = DiscardAbilityLifecycleEvents;
        self.run_with_sink(store, &mut sink)
    }

    pub fn run_with_sink<Tags, Payload, Sink>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
        sink: &mut Sink,
    ) -> Result<AbilityEndOutcome<Tags, Payload>, AbilityEndError>
    where
        Tags: TagCollection,
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        store.end_with_sink(self.activation_id, sink)
    }
}

impl AbilityCancel {
    #[must_use]
    pub const fn new(activation_id: AbilityActivationId) -> Self {
        Self { activation_id }
    }

    pub fn run<Tags, Payload>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> AbilityCancelOutcome<Tags, Payload>
    where
        Tags: TagCollection,
    {
        let mut sink = DiscardAbilityLifecycleEvents;
        self.run_with_sink(store, &mut sink)
    }

    pub fn run_with_sink<Tags, Payload, Sink>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
        sink: &mut Sink,
    ) -> AbilityCancelOutcome<Tags, Payload>
    where
        Tags: TagCollection,
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        store.cancel_with_sink(self.activation_id, sink)
    }
}

impl AbilityRollback {
    #[must_use]
    pub const fn new(activation_id: AbilityActivationId) -> Self {
        Self { activation_id }
    }

    pub fn run<Tags, Payload>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError>
    where
        Tags: TagCollection,
    {
        let mut sink = DiscardAbilityLifecycleEvents;
        self.run_with_sink(store, &mut sink)
    }

    pub fn run_with_sink<Tags, Payload, Sink>(
        self,
        store: &mut AbilityStore<Tags, Payload>,
        sink: &mut Sink,
    ) -> Result<AbilityRollbackOutcome<Tags, Payload>, AbilityRollbackError>
    where
        Tags: TagCollection,
        Sink: AbilityLifecycleSink<Tags, Payload>,
    {
        store.rollback_with_sink(self.activation_id, sink)
    }
}

fn emit_activation_request_rejection<Context, Tags, Payload, Executor>(
    error: &AbilityActivationRequestError<'_, Tags, Payload>,
    executor: &mut Executor,
) where
    Tags: TagCollection,
    Executor: AbilityActivationExecutor<Context, Tags, Payload>,
{
    if let Some(attempt) = error.attempt_view() {
        executor.emit_ability_lifecycle(AbilityLifecycleEventView::Attempted(attempt));
    }
    executor.emit_ability_lifecycle(AbilityLifecycleEventView::Rejected(
        AbilityActivationRejectionView {
            attempt: error.attempt_view(),
            reason: error.reason(),
        },
    ));
}
