use super::{AbilityBehavior, AbilityPayload, ability_definition};
use crate::{
    CombatError, CombatState, CombatTag, CombatTags, EffectPayload, TEN_SECONDS, effect_buff_tag,
    effect_max_health_tag, tag_set,
};
use flexweave::{
    AbilityDefinition, AbilityDefinitions, AbilityGrant, AbilityId, AttributeValue,
    EffectApplicationInput, EffectDefinition, Grant, ObjectId, ObjectStore, Tag,
};

const KEY: &str = "ability/fortify";
const PAYLOAD_SCHEMA: &str = "FortifyPayload";

pub(crate) fn definition() -> AbilityDefinition<&'static str> {
    ability_definition(KEY, PAYLOAD_SCHEMA)
}

pub(crate) fn grant(
    definitions: &AbilityDefinitions<&'static str>,
    objects: &ObjectStore,
    abilities: &mut flexweave::AbilityStore<CombatTags, AbilityPayload>,
    owner: ObjectId,
) -> AbilityId {
    AbilityGrant::registered(
        definitions,
        KEY,
        Grant::new(
            owner,
            tag_set([ability_tag()]),
            AbilityPayload::new(Fortify {
                max_health_bonus: 25.0,
                duration_units: TEN_SECONDS,
                mana_cost: 4.0,
            }),
        ),
    )
    .checked(objects)
    .run(abilities)
    .expect("fortify grant should be valid")
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fortify {
    max_health_bonus: AttributeValue,
    duration_units: u64,
    mana_cost: AttributeValue,
}

impl AbilityBehavior for Fortify {
    fn key(&self) -> &'static str {
        KEY
    }

    fn mana_cost(&self) -> AttributeValue {
        self.mana_cost
    }

    fn cooldown_tag(&self) -> Option<Tag<CombatTag>> {
        Some(cooldown_tag())
    }

    fn commit(
        &self,
        context: &mut CombatState,
        source_id: ObjectId,
        owner_id: ObjectId,
    ) -> Result<(), CombatError> {
        context.apply_cooldown(source_id, owner_id, cooldown_tag());
        context.apply_effect(
            &EffectDefinition::duration(
                "effect/fortify",
                self.duration_units,
                "MaxHealthBuffPayload",
            ),
            EffectApplicationInput::accept(
                Some(source_id),
                owner_id,
                tag_set([effect_buff_tag(), effect_max_health_tag()]),
                EffectPayload::MaxHealthBonus {
                    amount: self.max_health_bonus,
                },
            ),
        );
        Ok(())
    }
}

fn ability_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Ability, CombatTag::Fortify])
}

fn cooldown_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Cooldown, CombatTag::Fortify])
}
