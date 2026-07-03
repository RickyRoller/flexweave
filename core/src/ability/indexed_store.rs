use std::collections::HashMap;

use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::events::ActiveAbility;
use super::ids::{AbilityActivationId, AbilityId};
use super::store::GrantedAbility;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GrantedAbilityIndex<Tags, Payload>
where
    Tags: TagCollection,
{
    records: Vec<GrantedAbility<Tags, Payload>>,
    index_by_id: HashMap<AbilityId, usize>,
}

impl<Tags, Payload> GrantedAbilityIndex<Tags, Payload>
where
    Tags: TagCollection,
{
    pub(super) fn new() -> Self {
        Self {
            records: Vec::new(),
            index_by_id: HashMap::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    pub(super) fn iter(&self) -> std::slice::Iter<'_, GrantedAbility<Tags, Payload>> {
        self.records.iter()
    }

    pub(super) fn get(&self, ability_id: AbilityId) -> Option<&GrantedAbility<Tags, Payload>> {
        let index = self.index_by_id.get(&ability_id).copied()?;
        self.records.get(index)
    }

    pub(super) fn push(&mut self, ability: GrantedAbility<Tags, Payload>) {
        let index = self.records.len();
        self.index_by_id.insert(ability.id, index);
        self.records.push(ability);
    }

    fn remove_at(&mut self, index: usize) -> GrantedAbility<Tags, Payload> {
        let removed = self.records.remove(index);
        self.index_by_id.remove(&removed.id);
        self.reindex_from(index);
        removed
    }

    pub(super) fn remove_owner(
        &mut self,
        owner_id: ObjectId,
    ) -> Vec<GrantedAbility<Tags, Payload>> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.records.len() {
            if self.records[index].owner_id == owner_id {
                removed.push(self.remove_at(index));
            } else {
                index += 1;
            }
        }
        removed
    }

    fn reindex_from(&mut self, start: usize) {
        for index in start..self.records.len() {
            self.index_by_id.insert(self.records[index].id, index);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ActiveAbilityIndex<Tags, Payload>
where
    Tags: TagCollection,
{
    records: Vec<ActiveAbility<Tags, Payload>>,
    index_by_activation_id: HashMap<AbilityActivationId, usize>,
}

impl<Tags, Payload> ActiveAbilityIndex<Tags, Payload>
where
    Tags: TagCollection,
{
    pub(super) fn new() -> Self {
        Self {
            records: Vec::new(),
            index_by_activation_id: HashMap::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    pub(super) fn as_slice(&self) -> &[ActiveAbility<Tags, Payload>] {
        &self.records
    }

    pub(super) fn get(
        &self,
        activation_id: AbilityActivationId,
    ) -> Option<&ActiveAbility<Tags, Payload>> {
        let index = self.index_by_activation_id.get(&activation_id).copied()?;
        self.records.get(index)
    }

    pub(super) fn get_mut(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Option<&mut ActiveAbility<Tags, Payload>> {
        let index = self.index_by_activation_id.get(&activation_id).copied()?;
        self.records.get_mut(index)
    }

    pub(super) fn push(
        &mut self,
        active: ActiveAbility<Tags, Payload>,
    ) -> &ActiveAbility<Tags, Payload> {
        let index = self.records.len();
        self.index_by_activation_id
            .insert(active.activation_id, index);
        self.records.push(active);
        &self.records[index]
    }

    pub(super) fn remove(
        &mut self,
        activation_id: AbilityActivationId,
    ) -> Option<ActiveAbility<Tags, Payload>> {
        let index = self.index_by_activation_id.get(&activation_id).copied()?;
        Some(self.remove_at(index))
    }

    fn remove_at(&mut self, index: usize) -> ActiveAbility<Tags, Payload> {
        let removed = self.records.remove(index);
        self.index_by_activation_id.remove(&removed.activation_id);
        self.reindex_from(index);
        removed
    }

    pub(super) fn remove_owner_with<F>(
        &mut self,
        owner_id: ObjectId,
        mut on_remove: F,
    ) -> Vec<ActiveAbility<Tags, Payload>>
    where
        F: FnMut(&ActiveAbility<Tags, Payload>),
    {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.records.len() {
            if self.records[index].owner_id == owner_id {
                let active = self.remove_at(index);
                on_remove(&active);
                removed.push(active);
            } else {
                index += 1;
            }
        }
        removed
    }

    fn reindex_from(&mut self, start: usize) {
        for index in start..self.records.len() {
            self.index_by_activation_id
                .insert(self.records[index].activation_id, index);
        }
    }
}
