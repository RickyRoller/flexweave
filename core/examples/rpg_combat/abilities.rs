#[path = "abilities/fortify.rs"]
pub(crate) mod fortify;
#[path = "abilities/quickened_strikes.rs"]
pub(crate) mod quickened_strikes;
#[path = "abilities/slash.rs"]
pub(crate) mod slash;

use crate::{CombatError, CombatState, CombatTag};
use flexweave::{AbilityDefinition, AbilityDefinitions, AbilityId, AttributeValue, ObjectId, Tag};
use std::fmt;
use std::rc::Rc;

pub(crate) fn definitions() -> AbilityDefinitions<&'static str> {
    AbilityDefinitions::new([
        slash::definition(),
        quickened_strikes::definition(),
        fortify::definition(),
    ])
    .expect("demo ability definitions are valid")
}

pub(crate) trait AbilityBehavior: fmt::Debug {
    fn key(&self) -> &'static str;
    fn mana_cost(&self) -> AttributeValue;

    fn target(&self) -> Option<ObjectId> {
        None
    }

    fn cooldown_tag(&self) -> Option<Tag<CombatTag>> {
        None
    }

    fn commit(
        &self,
        context: &mut CombatState,
        source_id: ObjectId,
        owner_id: ObjectId,
    ) -> Result<(), CombatError>;
}

#[derive(Clone)]
pub(crate) struct AbilityPayload {
    behavior: Rc<dyn AbilityBehavior>,
}

impl AbilityPayload {
    pub(crate) fn new(behavior: impl AbilityBehavior + 'static) -> Self {
        Self {
            behavior: Rc::new(behavior),
        }
    }

    pub(crate) fn mana_cost(&self) -> AttributeValue {
        self.behavior.mana_cost()
    }

    pub(crate) fn target(&self) -> Option<ObjectId> {
        self.behavior.target()
    }

    pub(crate) fn cooldown_tag(&self) -> Option<Tag<CombatTag>> {
        self.behavior.cooldown_tag()
    }

    pub(crate) fn commit(
        &self,
        context: &mut CombatState,
        source_id: ObjectId,
        owner_id: ObjectId,
    ) -> Result<(), CombatError> {
        context.spend_mana(owner_id, self.mana_cost());
        self.behavior.commit(context, source_id, owner_id)
    }
}

impl fmt::Debug for AbilityPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AbilityPayload")
            .field("key", &self.behavior.key())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAbilities {
    pub(crate) slash: AbilityId,
    pub(crate) quickened_strikes: AbilityId,
    pub(crate) fortify: AbilityId,
}

fn ability_definition(
    key: &'static str,
    payload_schema: &'static str,
) -> AbilityDefinition<&'static str> {
    AbilityDefinition::new(key, payload_schema)
}
