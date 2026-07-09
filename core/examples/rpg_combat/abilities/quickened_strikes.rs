use super::{AbilityBehavior, AbilityPayload, ability_definition};
use crate::{
    CombatError, CombatState, CombatTag, CombatTags, EffectPayload, TEN_SECONDS,
    effect_attack_speed_tag, effect_buff_tag, tag_set,
};
use flexweave::{
    AbilityDefinition, AbilityDefinitions, AbilityGrant, AbilityId, AttributeValue,
    EffectApplicationInput, EffectDefinition, Grant, ObjectId, ObjectStore, Tag,
};

const KEY: &str = "ability/quickened-strikes";
const PAYLOAD_SCHEMA: &str = "QuickenedStrikesPayload";

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
            AbilityPayload::new(QuickenedStrikes {
                attack_speed_bonus: 0.5,
                duration_units: TEN_SECONDS,
                mana_cost: 5.0,
            }),
        ),
    )
    .checked(objects)
    .run(abilities)
    .expect("quickened strikes grant should be valid")
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct QuickenedStrikes {
    attack_speed_bonus: AttributeValue,
    duration_units: u64,
    mana_cost: AttributeValue,
}

impl AbilityBehavior for QuickenedStrikes {
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
                "effect/quickened-strikes",
                self.duration_units,
                "AttackSpeedBuffPayload",
            ),
            EffectApplicationInput::accept(
                Some(source_id),
                owner_id,
                tag_set([effect_buff_tag(), effect_attack_speed_tag()]),
                EffectPayload::AttackSpeedBonus {
                    amount: self.attack_speed_bonus,
                },
            ),
        );
        Ok(())
    }
}

fn ability_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Ability, CombatTag::QuickenedStrikes])
}

fn cooldown_tag() -> Tag<CombatTag> {
    Tag::new([
        CombatTag::Effect,
        CombatTag::Cooldown,
        CombatTag::QuickenedStrikes,
    ])
}
