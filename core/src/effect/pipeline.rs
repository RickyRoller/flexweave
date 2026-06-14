use crate::clock::ClockUnits;
use crate::identity::ObjectId;
use crate::tag::TagCollection;

use super::application::{
    EffectApplication, EffectApplicationDecision, EffectApplicationInput,
    EffectApplicationRejection,
};
use super::definition::{EffectDefinition, EffectDefinitionError, EffectKind};
use super::events::{EffectAdvance, EffectExecution, EffectInstance, EffectLifecycleEvent};
use super::ids::ActiveEffectId;

/// Effect application and execution pipeline with caller-owned payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectPipeline<Tags, Payload>
where
    Tags: TagCollection,
{
    next_id: ActiveEffectId,
    effects: Vec<EffectInstance<Tags, Payload>>,
}

impl<Tags, Payload> Default for EffectPipeline<Tags, Payload>
where
    Tags: TagCollection,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Tags, Payload> EffectPipeline<Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: ActiveEffectId::new(1),
            effects: Vec::new(),
        }
    }

    /// Applies or executes an effect definition and emits lifecycle facts.
    pub fn apply_with_events<Schema, F>(
        &mut self,
        definition: &EffectDefinition<Schema>,
        input: EffectApplicationInput<Tags, Payload>,
        mut on_event: F,
    ) -> Result<Option<ActiveEffectId>, EffectDefinitionError>
    where
        Tags: Clone,
        Payload: Clone,
        F: FnMut(EffectLifecycleEvent<Tags, Payload>),
    {
        definition.validate()?;

        let application = EffectApplication {
            source_id: input.source_id,
            target_id: input.target_id,
            tags: input.tags.clone(),
            payload: input.payload.clone(),
        };

        match input.decision {
            EffectApplicationDecision::Reject { reason } => {
                on_event(EffectLifecycleEvent::ApplicationRejected(
                    EffectApplicationRejection {
                        application,
                        reason,
                    },
                ));
                Ok(None)
            }
            EffectApplicationDecision::Accept => {
                on_event(EffectLifecycleEvent::ApplicationAccepted(
                    application.clone(),
                ));

                if definition.kind == EffectKind::Instant {
                    on_event(EffectLifecycleEvent::Executed(EffectExecution {
                        active_effect_id: None,
                        source_id: application.source_id,
                        target_id: application.target_id,
                        tags: application.tags,
                        payload: application.payload,
                        elapsed_units: None,
                    }));
                    return Ok(None);
                }

                let id = self.next_id;
                self.next_id = ActiveEffectId::new(self.next_id.get() + 1);
                let effect = EffectInstance {
                    id,
                    source_id: application.source_id,
                    target_id: application.target_id,
                    remaining_units: definition.duration.map(|duration| duration.units),
                    period: definition.period,
                    period_elapsed_units: 0,
                    tags: application.tags,
                    payload: application.payload,
                };
                self.effects.push(effect.clone());
                on_event(EffectLifecycleEvent::ActiveCreated(effect));
                Ok(Some(id))
            }
        }
    }

    /// Advances active effect instances and emits advance, periodic execution,
    /// and expiration lifecycle facts in deterministic instance order.
    pub fn tick_with_events<F>(&mut self, elapsed_units: ClockUnits, mut on_event: F)
    where
        Tags: Clone,
        Payload: Clone,
        F: FnMut(EffectLifecycleEvent<Tags, Payload>),
    {
        if elapsed_units == 0 {
            return;
        }

        let mut index = 0;
        while index < self.effects.len() {
            let previous_remaining_units = self.effects[index].remaining_units;
            let elapsed_for_effect = previous_remaining_units
                .map(|previous| elapsed_units.min(previous))
                .unwrap_or(elapsed_units);
            if let Some(previous) = previous_remaining_units {
                self.effects[index].remaining_units = Some(previous.saturating_sub(elapsed_units));
                on_event(EffectLifecycleEvent::Advanced(EffectAdvance {
                    effect: self.effects[index].clone(),
                    elapsed_units: elapsed_for_effect,
                    previous_remaining_units,
                }));
            }

            if let Some(period) = self.effects[index].period {
                self.effects[index].period_elapsed_units += elapsed_for_effect;
                while self.effects[index].period_elapsed_units >= period.units {
                    self.effects[index].period_elapsed_units -= period.units;
                    let effect = self.effects[index].clone();
                    on_event(EffectLifecycleEvent::PeriodicExecuted(EffectExecution {
                        active_effect_id: Some(effect.id),
                        source_id: effect.source_id,
                        target_id: effect.target_id,
                        tags: effect.tags,
                        payload: effect.payload,
                        elapsed_units: Some(period.units),
                    }));
                }
            }

            if previous_remaining_units.is_some_and(|previous| elapsed_units >= previous) {
                on_event(EffectLifecycleEvent::Expired(self.effects.remove(index)));
            } else {
                index += 1;
            }
        }
    }

    /// Removes an active effect instance by id and emits a removal lifecycle fact.
    pub fn remove_with_events<F>(
        &mut self,
        effect_id: ActiveEffectId,
        mut on_event: F,
    ) -> Option<EffectInstance<Tags, Payload>>
    where
        Payload: Clone,
        F: FnMut(EffectLifecycleEvent<Tags, Payload>),
    {
        let index = self
            .effects
            .iter()
            .position(|effect| effect.id == effect_id)?;
        let removed = self.effects.remove(index);
        on_event(EffectLifecycleEvent::Removed(removed.clone()));
        Some(removed)
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.effects.len()
    }

    #[must_use]
    pub fn get(&self, effect_id: ActiveEffectId) -> Option<&EffectInstance<Tags, Payload>> {
        self.effects.iter().find(|effect| effect.id == effect_id)
    }

    #[must_use]
    pub fn has_tag(&self, target_id: ObjectId, tag: &Tags::Tag) -> bool {
        self.effects
            .iter()
            .any(|effect| effect.target_id == target_id && effect.has_tag(tag))
    }

    /// Visits active effect instances for `target_id` in application order.
    pub fn visit_target<F>(&self, target_id: ObjectId, mut on_effect: F)
    where
        F: FnMut(&EffectInstance<Tags, Payload>),
    {
        for effect in &self.effects {
            if effect.target_id == target_id {
                on_effect(effect);
            }
        }
    }

    /// Visits all active effect instances in application order.
    pub fn visit_instances<F>(&self, mut on_effect: F)
    where
        F: FnMut(&EffectInstance<Tags, Payload>),
    {
        for effect in &self.effects {
            on_effect(effect);
        }
    }
}
