use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::events::ActiveAbility;
use super::ids::{AbilityActivationId, AbilityId};
use super::indexed_store::{ActiveAbilityIndex, GrantedAbilityIndex};

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

    pub(in crate::ability) fn insert_grant(
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
