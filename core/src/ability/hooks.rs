use crate::tag::TagCollection;

use super::ids::CooldownUnits;
use super::store::GrantedAbility;

/// Hook interface for caller-owned activation behavior.
pub trait AbilityHooks<Context, Tags, Cost, Payload>
where
    Tags: TagCollection,
{
    type Error;

    fn can_activate(
        &mut self,
        _context: &mut Context,
        _ability: &GrantedAbility<Tags, Cost, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn commit(
        &mut self,
        _context: &mut Context,
        _ability: &GrantedAbility<Tags, Cost, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn cooldown_units(
        &mut self,
        _context: &mut Context,
        ability: &GrantedAbility<Tags, Cost, Payload>,
    ) -> Result<Option<CooldownUnits>, Self::Error> {
        Ok(ability.cooldown_units)
    }

    fn end(
        &mut self,
        _context: &mut Context,
        _ability: &GrantedAbility<Tags, Cost, Payload>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
