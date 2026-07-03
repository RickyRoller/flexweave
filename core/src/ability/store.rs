use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;

use super::definition::{
    AbilityDefinition, AbilityDefinitionError, AbilityDefinitionRegistryError, AbilityDefinitions,
};
use super::events::ActiveAbility;
use super::ids::{AbilityActivationId, AbilityId};
use super::indexed_store::{ActiveAbilityIndex, GrantedAbilityIndex};
use super::results::AbilityGrantError;

mod active_commands;
mod begin_commands;

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
    abilities: GrantedAbilityIndex<Tags, Payload>,
    active_abilities: ActiveAbilityIndex<Tags, Payload>,
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
            abilities: GrantedAbilityIndex::new(),
            active_abilities: ActiveAbilityIndex::new(),
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
        self.active_abilities.as_slice()
    }

    #[must_use]
    pub fn get_active_activation(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Payload>> {
        self.find_active(activation_id)
    }

    fn find(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Payload>> {
        self.abilities.get(ability_id)
    }

    fn find_active(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Payload>> {
        self.active_abilities.get(activation_id)
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
