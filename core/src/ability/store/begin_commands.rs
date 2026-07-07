use crate::tag::TagCollection;

use super::AbilityStore;
use crate::ability::activation_request::AbilityActivationSeed;
use crate::ability::events::AbilityLifecycleEventView;
use crate::ability::ids::AbilityActivationId;
use crate::ability::lifecycle_transaction::{ActiveAbilityTransition, emit_active_transition};

impl<Tags, Payload> AbilityStore<Tags, Payload>
where
    Tags: TagCollection,
{
    pub(in crate::ability) fn start_activation_from_seed<F>(
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
