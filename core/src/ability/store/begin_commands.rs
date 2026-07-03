use std::convert::Infallible;

use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::AbilityStore;
use crate::ability::activation_request::{
    AbilityActivationRequest, AbilityActivationRequestError, AbilityActivationSeed,
    RegisteredActivationRequestError, resolve_activation_request, resolve_owner_activation_request,
    resolve_registered_activation_request,
};
use crate::ability::definition::AbilityDefinitions;
use crate::ability::event_sink::{discard_lifecycle_event, owned_lifecycle_events};
use crate::ability::events::{
    AbilityActivationRejectionReason, AbilityActivationRejectionView, AbilityLifecycleEvent,
    AbilityLifecycleEventView,
};
use crate::ability::hooks::{AbilityActivationDecision, AbilityActivationGate, AllowActivation};
use crate::ability::ids::{AbilityActivationId, AbilityId};
use crate::ability::lifecycle_transaction::{ActiveAbilityTransition, emit_active_transition};
use crate::ability::results::{AbilityBeginError, RegisteredAbilityActivationError};

impl<Tags, Payload> AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
{
    /// Begins an activation without a caller-owned activation gate.
    pub fn begin_activation(
        &mut self,
        ability_id: AbilityId,
    ) -> Result<AbilityActivationId, AbilityBeginError<Infallible, Infallible>>
    where
        Payload: Clone,
    {
        self.begin_activation_with_borrowed_events(ability_id, discard_lifecycle_event)
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
        self.begin_activation_with_borrowed_events(ability_id, owned_lifecycle_events(&mut emit))
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
        self.begin_activation_with_gate_borrowed_events(
            ability_id,
            context,
            gate,
            discard_lifecycle_event,
        )
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
        self.begin_activation_with_gate_borrowed_events(
            ability_id,
            context,
            gate,
            owned_lifecycle_events(&mut emit),
        )
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
                emit_activation_request_rejection(&error, &mut emit);
                return Err(AbilityBeginError::Ability(error.ability_error()));
            }
        };
        let seed = prepare_activation_seed(request, context, gate, &mut emit)?;

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
        self.begin_activation_for_owner_with_borrowed_events(
            owner_id,
            ability_id,
            discard_lifecycle_event,
        )
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
        self.begin_activation_for_owner_with_borrowed_events(
            owner_id,
            ability_id,
            owned_lifecycle_events(&mut emit),
        )
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
            discard_lifecycle_event,
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
            owned_lifecycle_events(&mut emit),
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
                emit_activation_request_rejection(&error, &mut emit);
                return Err(AbilityBeginError::Ability(error.ability_error()));
            }
        };
        let seed = prepare_activation_seed(request, context, gate, &mut emit)?;

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
        self.begin_registered_activation_with_borrowed_events(
            definitions,
            ability_id,
            discard_lifecycle_event,
        )
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
        self.begin_registered_activation_with_borrowed_events(
            definitions,
            ability_id,
            owned_lifecycle_events(&mut emit),
        )
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
            discard_lifecycle_event,
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
            owned_lifecycle_events(&mut emit),
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
                emit_activation_request_rejection(&error, &mut emit);
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
        let seed = prepare_activation_seed(request, context, gate, &mut emit)
            .map_err(RegisteredAbilityActivationError::Activation)?;

        Ok(self.start_activation_from_seed(seed, &mut emit))
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
        let active = seed.into_active(activation_id);
        let active = self.active_abilities.push(active);
        emit_active_transition(ActiveAbilityTransition::Started, active, emit);
        activation_id
    }
}

fn emit_activation_request_rejection<Tags, Payload, F>(
    error: &AbilityActivationRequestError<'_, Tags, Payload>,
    emit: &mut F,
) where
    Tags: TagCollection,
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

fn prepare_activation_seed<Context, Gate, Tags, Payload, F>(
    request: AbilityActivationRequest<'_, Tags, Payload>,
    context: &Context,
    gate: &mut Gate,
    emit: &mut F,
) -> Result<AbilityActivationSeed<Tags, Payload>, AbilityBeginError<Gate::Error, Gate::BlockReason>>
where
    Tags: TagCollection,
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
