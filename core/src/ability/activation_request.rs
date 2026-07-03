use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::definition::{AbilityDefinitionRegistryError, AbilityDefinitions};
use super::events::{
    AbilityActivationAttemptView, AbilityActivationRejectionReason, ActiveAbility,
};
use super::ids::{AbilityActivationId, AbilityId};
use super::results::AbilityError;
use super::store::GrantedAbility;

pub(super) struct AbilityActivationRequest<'ability, Tags, Payload>
where
    Tags: TagCollection,
{
    ability: &'ability GrantedAbility<Tags, Payload>,
}

impl<'ability, Tags, Payload> AbilityActivationRequest<'ability, Tags, Payload>
where
    Tags: TagCollection,
{
    pub(super) fn attempt_view(&self) -> AbilityActivationAttemptView<'ability, Tags, Payload> {
        attempt_view_from_ability(self.ability)
    }

    pub(super) fn to_seed(&self) -> AbilityActivationSeed<Tags, Payload>
    where
        Payload: Clone,
    {
        AbilityActivationSeed {
            ability_id: self.ability.id,
            definition_key: self.ability.definition_key.clone(),
            owner_id: self.ability.owner_id,
            tags: self.ability.tags.clone(),
            payload: self.ability.payload.clone(),
        }
    }
}

pub(super) struct AbilityActivationSeed<Tags, Payload>
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
    pub(super) fn into_active(
        self,
        activation_id: AbilityActivationId,
    ) -> ActiveAbility<Tags, Payload> {
        ActiveAbility {
            activation_id,
            ability_id: self.ability_id,
            definition_key: self.definition_key,
            owner_id: self.owner_id,
            tags: self.tags,
            payload: self.payload,
            committed: false,
        }
    }
}

pub(super) enum AbilityActivationRequestError<'ability, Tags, Payload>
where
    Tags: TagCollection,
{
    MissingAbility,
    InvalidOwner {
        owner_id: ObjectId,
    },
    OwnerMismatch {
        expected_owner_id: ObjectId,
        actual_owner_id: ObjectId,
        ability: &'ability GrantedAbility<Tags, Payload>,
    },
}

impl<'ability, Tags, Payload> AbilityActivationRequestError<'ability, Tags, Payload>
where
    Tags: TagCollection,
{
    pub(super) fn ability_error(&self) -> AbilityError {
        match self {
            Self::MissingAbility => AbilityError::MissingAbility,
            Self::InvalidOwner { owner_id } => AbilityError::InvalidOwner {
                owner_id: *owner_id,
            },
            Self::OwnerMismatch {
                expected_owner_id,
                actual_owner_id,
                ..
            } => AbilityError::OwnerMismatch {
                expected_owner_id: *expected_owner_id,
                actual_owner_id: *actual_owner_id,
            },
        }
    }

    pub(super) fn reason(&self) -> AbilityActivationRejectionReason {
        match self {
            Self::MissingAbility => AbilityActivationRejectionReason::MissingAbility,
            Self::InvalidOwner { .. } => AbilityActivationRejectionReason::InvalidOwner,
            Self::OwnerMismatch { .. } => AbilityActivationRejectionReason::OwnerMismatch,
        }
    }

    pub(super) fn attempt_view(
        &self,
    ) -> Option<AbilityActivationAttemptView<'ability, Tags, Payload>> {
        match self {
            Self::MissingAbility | Self::InvalidOwner { .. } => None,
            Self::OwnerMismatch { ability, .. } => Some(attempt_view_from_ability(ability)),
        }
    }
}

pub(super) enum RegisteredActivationRequestError<'ability, Tags, Payload>
where
    Tags: TagCollection,
{
    Activation(AbilityActivationRequestError<'ability, Tags, Payload>),
    MissingGrantedDefinitionKey { ability_id: AbilityId },
    Definition(AbilityDefinitionRegistryError),
}

pub(super) fn resolve_activation_request<'ability, Tags, Payload>(
    ability: Option<&'ability GrantedAbility<Tags, Payload>>,
) -> Result<
    AbilityActivationRequest<'ability, Tags, Payload>,
    AbilityActivationRequestError<'ability, Tags, Payload>,
>
where
    Tags: TagCollection,
{
    let Some(ability) = ability else {
        return Err(AbilityActivationRequestError::MissingAbility);
    };

    Ok(AbilityActivationRequest { ability })
}

pub(super) fn resolve_owner_activation_request<'ability, Tags, Payload>(
    owner_id: ObjectId,
    ability: Option<&'ability GrantedAbility<Tags, Payload>>,
) -> Result<
    AbilityActivationRequest<'ability, Tags, Payload>,
    AbilityActivationRequestError<'ability, Tags, Payload>,
>
where
    Tags: TagCollection,
{
    if owner_id.is_invalid() {
        return Err(AbilityActivationRequestError::InvalidOwner { owner_id });
    }

    let request = resolve_activation_request(ability)?;
    let actual_owner_id = request.ability.owner_id;
    if actual_owner_id != owner_id {
        return Err(AbilityActivationRequestError::OwnerMismatch {
            expected_owner_id: owner_id,
            actual_owner_id,
            ability: request.ability,
        });
    }

    Ok(request)
}

pub(super) fn resolve_registered_activation_request<'ability, PayloadSchema, Tags, Payload>(
    definitions: &AbilityDefinitions<PayloadSchema>,
    ability_id: AbilityId,
    ability: Option<&'ability GrantedAbility<Tags, Payload>>,
) -> Result<
    AbilityActivationRequest<'ability, Tags, Payload>,
    RegisteredActivationRequestError<'ability, Tags, Payload>,
>
where
    Tags: TagCollection,
{
    let request = resolve_activation_request(ability)
        .map_err(RegisteredActivationRequestError::Activation)?;
    let definition_key = request
        .ability
        .definition_key
        .as_deref()
        .ok_or(RegisteredActivationRequestError::MissingGrantedDefinitionKey { ability_id })?;

    definitions
        .require(definition_key)
        .map_err(RegisteredActivationRequestError::Definition)?;

    Ok(request)
}

fn attempt_view_from_ability<Tags, Payload>(
    ability: &GrantedAbility<Tags, Payload>,
) -> AbilityActivationAttemptView<'_, Tags, Payload>
where
    Tags: TagCollection,
{
    AbilityActivationAttemptView {
        ability_id: ability.id,
        definition_key: ability.definition_key.as_deref(),
        owner_id: ability.owner_id,
        tags: &ability.tags,
        payload: &ability.payload,
    }
}
