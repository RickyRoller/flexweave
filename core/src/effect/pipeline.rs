use crate::clock::ClockUnits;
use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;
use std::collections::HashMap;
use std::fmt;

use super::application::{
    EffectApplicationDecision, EffectApplicationDraft, EffectApplicationInput,
    EffectApplicationRejectionView, EffectApplicationView, EffectExecutor, EffectInitializer,
    EffectSourcePolicy,
};
use super::definition::{EffectClockPolicy, EffectDefinition, EffectDefinitionError, EffectKind};
use super::events::{
    EffectAdvanceView, EffectExecutionView, EffectInstance, EffectInstanceView,
    EffectLifecycleEvent, EffectLifecycleEventView,
};
use super::ids::ActiveEffectId;

/// Effect application and execution pipeline with caller-owned payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectPipeline<Tags, Payload>
where
    Tags: TagCollection,
{
    next_id: ActiveEffectId,
    effects: Vec<EffectInstance<Tags, Payload>>,
    index_by_id: HashMap<ActiveEffectId, usize>,
    effect_ids_by_target: HashMap<ObjectId, Vec<ActiveEffectId>>,
}

/// Which object references cause active effects to be removed during cleanup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectObjectRemovalPolicy {
    Source,
    Target,
    SourceOrTarget,
}

/// Runtime effect application failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectApplicationError {
    Definition(EffectDefinitionError),
    MissingSource,
    InvalidSource { source_id: ObjectId },
    InvalidTarget { target_id: ObjectId },
}

/// Outcome of applying an effect definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectApplyOutcome {
    Rejected,
    ExecutedInstant,
    ActiveCreated(ActiveEffectId),
}

pub(super) struct PreparedEffectApplication<'definition, Tags, Payload>
where
    Tags: TagCollection,
{
    pub(super) definition_key: &'definition str,
    pub(super) kind: EffectKind,
    pub(super) duration: Option<EffectClockPolicy>,
    pub(super) period: Option<EffectClockPolicy>,
    pub(super) source_id: Option<ObjectId>,
    pub(super) target_id: ObjectId,
    pub(super) tags: Tags,
    pub(super) payload: Payload,
    pub(super) decision: EffectApplicationDecision,
}

impl<'definition, Tags, Payload> PreparedEffectApplication<'definition, Tags, Payload>
where
    Tags: TagCollection,
{
    pub(super) fn new<Schema>(
        definition: &'definition EffectDefinition<Schema>,
        input: EffectApplicationInput<Tags, Payload>,
    ) -> Self {
        let EffectApplicationInput {
            source_id,
            target_id,
            tags,
            payload,
            decision,
        } = input;

        Self {
            definition_key: definition.key.as_str(),
            kind: definition.kind,
            duration: definition.duration,
            period: definition.period,
            source_id,
            target_id,
            tags,
            payload,
            decision,
        }
    }

    pub(super) fn initialize<Context, Initializer>(
        &mut self,
        context: &mut Context,
        initializer: &mut Initializer,
    ) -> Result<(), Initializer::Error>
    where
        Initializer: EffectInitializer<Context, Tags, Payload>,
    {
        initializer.initialize(
            context,
            EffectApplicationDraft {
                definition_key: self.definition_key,
                source_id: self.source_id,
                target_id: self.target_id,
                tags: &self.tags,
                payload: &mut self.payload,
                duration: &mut self.duration,
                period: &mut self.period,
            },
        )
    }
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
            index_by_id: HashMap::new(),
            effect_ids_by_target: HashMap::new(),
        }
    }

    pub(super) fn apply_prepared_with_executor<'definition, Context, Executor>(
        &mut self,
        prepared: PreparedEffectApplication<'definition, Tags, Payload>,
        context: &mut Context,
        executor: &mut Executor,
    ) -> Result<EffectApplyOutcome, Executor::Error>
    where
        Executor: EffectExecutor<Context, Tags, Payload>,
    {
        let PreparedEffectApplication {
            definition_key,
            kind,
            duration,
            period,
            source_id,
            target_id,
            tags,
            payload,
            decision,
        } = prepared;

        let application = EffectApplicationView {
            definition_key: Some(definition_key),
            source_id,
            target_id,
            tags: &tags,
            payload: &payload,
        };

        match decision {
            EffectApplicationDecision::Reject { reason } => {
                executor.emit_effect_lifecycle(EffectLifecycleEventView::ApplicationRejected(
                    EffectApplicationRejectionView {
                        application,
                        reason: &reason,
                    },
                ));
                Ok(EffectApplyOutcome::Rejected)
            }
            EffectApplicationDecision::Accept => {
                executor.emit_effect_lifecycle(EffectLifecycleEventView::ApplicationAccepted(
                    application,
                ));

                if kind == EffectKind::Instant {
                    let execution = EffectExecutionView {
                        active_effect_id: None,
                        definition_key: Some(definition_key),
                        source_id,
                        target_id,
                        tags: &tags,
                        payload: &payload,
                        elapsed_units: None,
                    };
                    executor.execute_effect(context, execution)?;
                    executor.emit_effect_lifecycle(EffectLifecycleEventView::Executed(
                        EffectExecutionView {
                            active_effect_id: None,
                            definition_key: Some(definition_key),
                            source_id,
                            target_id,
                            tags: &tags,
                            payload: &payload,
                            elapsed_units: None,
                        },
                    ));
                    return Ok(EffectApplyOutcome::ExecutedInstant);
                }

                let id = self.next_id;
                self.next_id = ActiveEffectId::new(self.next_id.get() + 1);
                let effect = EffectInstance {
                    id,
                    definition_key: Some(definition_key.to_owned()),
                    source_id,
                    target_id,
                    remaining_units: duration.map(|duration| duration.units),
                    period,
                    period_elapsed_units: 0,
                    tags,
                    payload,
                };
                self.push_effect(effect);
                let effect = self.effects.last().expect("effect was just pushed");
                executor
                    .emit_effect_lifecycle(EffectLifecycleEventView::ActiveCreated(effect.into()));
                Ok(EffectApplyOutcome::ActiveCreated(id))
            }
        }
    }

    pub(super) fn advance_with_executor<Context, Executor>(
        &mut self,
        elapsed_units: ClockUnits,
        context: &mut Context,
        executor: &mut Executor,
    ) -> Result<(), Executor::Error>
    where
        Executor: EffectExecutor<Context, Tags, Payload>,
    {
        if elapsed_units == 0 {
            return Ok(());
        }

        let mut index = 0;
        while index < self.effects.len() {
            let previous_remaining_units = self.effects[index].remaining_units;
            let elapsed_for_effect = previous_remaining_units
                .map(|previous| elapsed_units.min(previous))
                .unwrap_or(elapsed_units);
            if let Some(previous) = previous_remaining_units {
                self.effects[index].remaining_units = Some(previous.saturating_sub(elapsed_units));
                executor.emit_effect_lifecycle(EffectLifecycleEventView::Advanced(
                    EffectAdvanceView {
                        effect: (&self.effects[index]).into(),
                        elapsed_units: elapsed_for_effect,
                        previous_remaining_units,
                    },
                ));
            }

            if let Some(period) = self.effects[index].period {
                self.effects[index].period_elapsed_units += elapsed_for_effect;
                while self.effects[index].period_elapsed_units >= period.units {
                    let effect = &self.effects[index];
                    let execution = EffectExecutionView {
                        active_effect_id: Some(effect.id),
                        definition_key: effect.definition_key.as_deref(),
                        source_id: effect.source_id,
                        target_id: effect.target_id,
                        tags: &effect.tags,
                        payload: &effect.payload,
                        elapsed_units: Some(period.units),
                    };
                    executor.execute_effect(context, execution)?;
                    let effect = &self.effects[index];
                    executor.emit_effect_lifecycle(EffectLifecycleEventView::PeriodicExecuted(
                        EffectExecutionView {
                            active_effect_id: Some(effect.id),
                            definition_key: effect.definition_key.as_deref(),
                            source_id: effect.source_id,
                            target_id: effect.target_id,
                            tags: &effect.tags,
                            payload: &effect.payload,
                            elapsed_units: Some(period.units),
                        },
                    ));
                    self.effects[index].period_elapsed_units -= period.units;
                }
            }

            if previous_remaining_units.is_some_and(|previous| elapsed_units >= previous) {
                let expired = self.remove_effect_at_index(index);
                executor
                    .emit_effect_lifecycle(EffectLifecycleEventView::Expired((&expired).into()));
            } else {
                index += 1;
            }
        }

        Ok(())
    }

    /// Removes an active effect instance by id without constructing lifecycle events.
    pub fn remove(&mut self, effect_id: ActiveEffectId) -> Option<EffectInstance<Tags, Payload>> {
        let index = self.find_index(effect_id)?;
        Some(self.remove_effect_at_index(index))
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
        self.remove_with_borrowed_events(effect_id, |event| {
            on_event(event.to_owned_event());
        })
    }

    /// Removes an active effect instance by id and streams a borrowed removal fact.
    pub fn remove_with_borrowed_events<F>(
        &mut self,
        effect_id: ActiveEffectId,
        mut on_event: F,
    ) -> Option<EffectInstance<Tags, Payload>>
    where
        F: for<'event> FnMut(EffectLifecycleEventView<'event, Tags, Payload>),
    {
        let removed = self.remove(effect_id)?;
        on_event(EffectLifecycleEventView::Removed(EffectInstanceView::from(
            &removed,
        )));
        Some(removed)
    }

    /// Removes active effects that reference `object_id` according to `policy`.
    #[must_use]
    pub fn remove_for_object(
        &mut self,
        object_id: ObjectId,
        policy: EffectObjectRemovalPolicy,
    ) -> Vec<EffectInstance<Tags, Payload>> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.effects.len() {
            if policy.matches(&self.effects[index], object_id) {
                removed.push(self.remove_effect_at_index(index));
            } else {
                index += 1;
            }
        }
        removed
    }

    /// Removes active effects that reference `object_id` and emits removal facts.
    pub fn remove_for_object_with_events<F>(
        &mut self,
        object_id: ObjectId,
        policy: EffectObjectRemovalPolicy,
        mut on_event: F,
    ) -> Vec<EffectInstance<Tags, Payload>>
    where
        Payload: Clone,
        F: FnMut(EffectLifecycleEvent<Tags, Payload>),
    {
        self.remove_for_object_with_borrowed_events(object_id, policy, |event| {
            on_event(event.to_owned_event());
        })
    }

    /// Removes active effects that reference `object_id` and streams borrowed removal facts.
    pub fn remove_for_object_with_borrowed_events<F>(
        &mut self,
        object_id: ObjectId,
        policy: EffectObjectRemovalPolicy,
        mut on_event: F,
    ) -> Vec<EffectInstance<Tags, Payload>>
    where
        F: for<'event> FnMut(EffectLifecycleEventView<'event, Tags, Payload>),
    {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.effects.len() {
            if policy.matches(&self.effects[index], object_id) {
                let effect = self.remove_effect_at_index(index);
                on_event(EffectLifecycleEventView::Removed((&effect).into()));
                removed.push(effect);
            } else {
                index += 1;
            }
        }
        removed
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.effects.len()
    }

    #[must_use]
    pub fn get(&self, effect_id: ActiveEffectId) -> Option<&EffectInstance<Tags, Payload>> {
        self.find_index(effect_id).map(|index| &self.effects[index])
    }

    #[must_use]
    pub fn has_tag(&self, target_id: ObjectId, tag: &Tags::Tag) -> bool {
        self.effect_ids_by_target
            .get(&target_id)
            .is_some_and(|effect_ids| {
                effect_ids.iter().any(|effect_id| {
                    self.get(*effect_id)
                        .is_some_and(|effect| effect.has_tag(tag))
                })
            })
    }

    /// Visits active effect instances for `target_id` in application order.
    pub fn visit_target<F>(&self, target_id: ObjectId, mut on_effect: F)
    where
        F: FnMut(&EffectInstance<Tags, Payload>),
    {
        if let Some(effect_ids) = self.effect_ids_by_target.get(&target_id) {
            for effect_id in effect_ids {
                if let Some(effect) = self.get(*effect_id) {
                    debug_assert_eq!(effect.target_id, target_id);
                    on_effect(effect);
                }
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

    fn find_index(&self, effect_id: ActiveEffectId) -> Option<usize> {
        self.index_by_id.get(&effect_id).copied()
    }

    fn push_effect(&mut self, effect: EffectInstance<Tags, Payload>) {
        self.index_by_id.insert(effect.id, self.effects.len());
        self.effect_ids_by_target
            .entry(effect.target_id)
            .or_default()
            .push(effect.id);
        self.effects.push(effect);
    }

    fn remove_effect_at_index(&mut self, index: usize) -> EffectInstance<Tags, Payload> {
        let removed = self.effects.remove(index);
        self.index_by_id.remove(&removed.id);
        self.remove_effect_from_target_index(removed.target_id, removed.id);
        self.reindex_effects_from(index);
        removed
    }

    fn remove_effect_from_target_index(&mut self, target_id: ObjectId, effect_id: ActiveEffectId) {
        let should_remove_target =
            if let Some(effect_ids) = self.effect_ids_by_target.get_mut(&target_id) {
                if let Some(index) = effect_ids.iter().position(|id| *id == effect_id) {
                    effect_ids.remove(index);
                }
                effect_ids.is_empty()
            } else {
                false
            };

        if should_remove_target {
            self.effect_ids_by_target.remove(&target_id);
        }
    }

    fn reindex_effects_from(&mut self, start: usize) {
        for index in start..self.effects.len() {
            self.index_by_id.insert(self.effects[index].id, index);
        }
    }
}

impl EffectObjectRemovalPolicy {
    fn matches<Tags, Payload>(
        self,
        effect: &EffectInstance<Tags, Payload>,
        object_id: ObjectId,
    ) -> bool
    where
        Tags: TagCollection,
    {
        match self {
            Self::Source => effect.source_id == Some(object_id),
            Self::Target => effect.target_id == object_id,
            Self::SourceOrTarget => {
                effect.source_id == Some(object_id) || effect.target_id == object_id
            }
        }
    }
}

pub(super) fn validate_application_references<Tags, Payload>(
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
