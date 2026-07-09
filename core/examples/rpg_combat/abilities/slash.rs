use super::{AbilityBehavior, AbilityPayload, ability_definition};
use crate::{
    CombatError, CombatState, CombatTag, CombatTags, EffectPayload, effect_damage_tag, tag_set,
};
use flexweave::{
    AbilityDefinition, AbilityDefinitions, AbilityGrant, AbilityId, AttributeValue,
    EffectApplicationInput, EffectDefinition, Grant, ObjectId, ObjectStore, Tag,
};

const KEY: &str = "ability/slash";
const PAYLOAD_SCHEMA: &str = "SlashPayload";

pub(crate) fn definition() -> AbilityDefinition<&'static str> {
    ability_definition(KEY, PAYLOAD_SCHEMA)
}

pub(crate) fn grant(
    definitions: &AbilityDefinitions<&'static str>,
    objects: &ObjectStore,
    abilities: &mut flexweave::AbilityStore<CombatTags, AbilityPayload>,
    owner: ObjectId,
    target: ObjectId,
) -> AbilityId {
    AbilityGrant::registered(
        definitions,
        KEY,
        Grant::new(
            owner,
            tag_set([ability_tag()]),
            AbilityPayload::new(Slash {
                target,
                damage: 12.0,
                mana_cost: 3.0,
            }),
        ),
    )
    .checked(objects)
    .run(abilities)
    .expect("slash grant should be valid")
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Slash {
    target: ObjectId,
    damage: AttributeValue,
    mana_cost: AttributeValue,
}

impl AbilityBehavior for Slash {
    fn key(&self) -> &'static str {
        KEY
    }

    fn mana_cost(&self) -> AttributeValue {
        self.mana_cost
    }

    fn target(&self) -> Option<ObjectId> {
        Some(self.target)
    }

    fn commit(
        &self,
        context: &mut CombatState,
        source_id: ObjectId,
        _owner_id: ObjectId,
    ) -> Result<(), CombatError> {
        context.apply_effect(
            &EffectDefinition::instant("effect/slash-damage", "DamagePayload"),
            EffectApplicationInput::accept(
                Some(source_id),
                self.target,
                tag_set([effect_damage_tag()]),
                EffectPayload::Damage {
                    amount: self.damage,
                },
            ),
        );
        Ok(())
    }
}

fn ability_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Ability, CombatTag::Slash])
}
