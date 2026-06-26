use crate::clock::ClockUnits;
use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;
use std::fmt;

use super::application::{
    EffectApplication, EffectApplicationDecision, EffectApplicationInput,
    EffectApplicationRejection, EffectSourcePolicy,
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

/// Runtime effect application failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectApplicationError {
    Definition(EffectDefinitionError),
    MissingSource,
    InvalidSource { source_id: ObjectId },
    InvalidTarget { target_id: ObjectId },
}

impl fmt::Display for EffectApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Definition(error) => write!(formatter, "{error}"),
            Self::MissingSource => formatter.write_str("effect application requires a source"),
            Self::InvalidSource { .. } => formatter.write_str("invalid effect source"),
            Self::InvalidTarget { .. } => formatter.write_str("invalid effect target"),
        }
    }
}

impl std::error::Error for EffectApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Definition(error) => Some(error),
            Self::MissingSource | Self::InvalidSource { .. } | Self::InvalidTarget { .. } => None,
        }
    }
}

impl From<EffectDefinitionError> for EffectApplicationError {
    fn from(value: EffectDefinitionError) -> Self {
        Self::Definition(value)
    }
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
    ///
    /// This is the low-level unchecked path for object references:
    /// `source_id` and `target_id` are copied as-is into emitted/stored facts.
    /// Prefer [`Self::apply_checked_with_events`] when an `ObjectStore` is
    /// available.
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

    /// Applies or executes an effect after validating source and target object references.
    pub fn apply_checked_with_events<Schema, F>(
        &mut self,
        objects: &ObjectStore,
        definition: &EffectDefinition<Schema>,
        input: EffectApplicationInput<Tags, Payload>,
        source_policy: EffectSourcePolicy,
        on_event: F,
    ) -> Result<Option<ActiveEffectId>, EffectApplicationError>
    where
        Tags: Clone,
        Payload: Clone,
        F: FnMut(EffectLifecycleEvent<Tags, Payload>),
    {
        validate_application_references(objects, &input, source_policy)?;
        self.apply_with_events(definition, input, on_event)
            .map_err(EffectApplicationError::Definition)
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

fn validate_application_references<Tags, Payload>(
    objects: &ObjectStore,
    input: &EffectApplicationInput<Tags, Payload>,
    source_policy: EffectSourcePolicy,
) -> Result<(), EffectApplicationError>
where
    Tags: TagCollection,
{
    match input.source_id {
        Some(source_id) => {
            if !objects.exists(source_id) {
                return Err(EffectApplicationError::InvalidSource { source_id });
            }
        }
        None if source_policy == EffectSourcePolicy::RequireLiveSource => {
            return Err(EffectApplicationError::MissingSource);
        }
        None => {}
    }

    if !objects.exists(input.target_id) {
        return Err(EffectApplicationError::InvalidTarget {
            target_id: input.target_id,
        });
    }

    Ok(())
}
