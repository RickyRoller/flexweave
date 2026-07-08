use std::convert::Infallible;
use std::fmt;

use crate::clock::ClockUnits;
use crate::identity::{ObjectId, ObjectStore};
use crate::tag::TagCollection;

use super::application::{
    DiscardEffectLifecycleEvents, EffectApplicationDecision, EffectApplicationInput,
    EffectExecutor, EffectInitializer, EffectLifecycleSink, EffectSourcePolicy, NoEffectExecutor,
    NoopEffectInitializer,
};
use super::definition::{
    EffectDefinition, EffectDefinitionError, EffectDefinitionRegistryError, EffectDefinitions,
};
use super::events::EffectInstance;
use super::ids::ActiveEffectId;
use super::pipeline::{
    EffectApplicationError, EffectApplyOutcome, EffectObjectRemovalPolicy, EffectPipeline,
    PreparedEffectApplication, validate_application_references,
};

/// Effect application command builder.
pub struct EffectApply<'input, Schema, Tags, Payload, Initializer = NoopEffectInitializer>
where
    Tags: TagCollection,
{
    source: EffectApplySource<'input, Schema>,
    input: EffectApplicationInput<Tags, Payload>,
    reference_check: Option<EffectApplicationReferenceCheck<'input>>,
    initializer: Initializer,
}

enum EffectApplySource<'input, Schema> {
    Definition(&'input EffectDefinition<Schema>),
    Registered {
        definitions: &'input EffectDefinitions<Schema>,
        key: &'input str,
    },
}

struct EffectApplicationReferenceCheck<'input> {
    objects: &'input ObjectStore,
    source_policy: EffectSourcePolicy,
}

/// Effect application command failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectApplyError<InitializeError = Infallible, ExecutionError = Infallible> {
    Definition(EffectDefinitionError),
    RegisteredDefinition(EffectDefinitionRegistryError),
    Application(EffectApplicationError),
    Initialize(InitializeError),
    Execution(ExecutionError),
}

/// Effect ticking command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectTick {
    elapsed_units: ClockUnits,
}

/// Active effect removal command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectRemove {
    effect_id: ActiveEffectId,
}

/// Object-reference effect cleanup command builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectRemoveForObject {
    object_id: ObjectId,
    policy: EffectObjectRemovalPolicy,
}

impl<'input, Schema, Tags, Payload>
    EffectApply<'input, Schema, Tags, Payload, NoopEffectInitializer>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn definition(
        definition: &'input EffectDefinition<Schema>,
        input: EffectApplicationInput<Tags, Payload>,
    ) -> Self {
        Self {
            source: EffectApplySource::Definition(definition),
            input,
            reference_check: None,
            initializer: NoopEffectInitializer,
        }
    }

    #[must_use]
    pub fn registered(
        definitions: &'input EffectDefinitions<Schema>,
        key: &'input str,
        input: EffectApplicationInput<Tags, Payload>,
    ) -> Self {
        Self {
            source: EffectApplySource::Registered { definitions, key },
            input,
            reference_check: None,
            initializer: NoopEffectInitializer,
        }
    }
}

impl<'input, Schema, Tags, Payload, Initializer>
    EffectApply<'input, Schema, Tags, Payload, Initializer>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn checked(
        mut self,
        objects: &'input ObjectStore,
        source_policy: EffectSourcePolicy,
    ) -> Self {
        self.reference_check = Some(EffectApplicationReferenceCheck {
            objects,
            source_policy,
        });
        self
    }

    #[must_use]
    pub fn initialized<NextInitializer>(
        self,
        initializer: NextInitializer,
    ) -> EffectApply<'input, Schema, Tags, Payload, NextInitializer> {
        EffectApply {
            source: self.source,
            input: self.input,
            reference_check: self.reference_check,
            initializer,
        }
    }

    pub fn run(
        self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
    ) -> Result<EffectApplyOutcome, EffectApplyError<Initializer::Error, Infallible>>
    where
        Initializer: EffectInitializer<(), Tags, Payload>,
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new();
        self.run_with_executor(pipeline, &mut context, &mut executor)
    }

    pub fn run_with_executor<Context, Executor>(
        mut self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
        context: &mut Context,
        executor: &mut Executor,
    ) -> Result<EffectApplyOutcome, EffectApplyError<Initializer::Error, Executor::Error>>
    where
        Initializer: EffectInitializer<Context, Tags, Payload>,
        Executor: EffectExecutor<Context, Tags, Payload>,
    {
        if let Some(reference_check) = &self.reference_check {
            validate_application_references(
                reference_check.objects,
                &self.input,
                reference_check.source_policy,
            )
            .map_err(EffectApplyError::Application)?;
        }

        let definition = match self.source {
            EffectApplySource::Definition(definition) => {
                definition
                    .validate()
                    .map_err(EffectApplyError::Definition)?;
                definition
            }
            EffectApplySource::Registered { definitions, key } => definitions
                .require(key)
                .map_err(EffectApplyError::RegisteredDefinition)?,
        };

        let mut prepared = PreparedEffectApplication::new(definition, self.input);
        if matches!(&prepared.decision, EffectApplicationDecision::Accept) {
            prepared
                .initialize(context, &mut self.initializer)
                .map_err(EffectApplyError::Initialize)?;
            definition
                .validate_clock_shape(prepared.duration, prepared.period)
                .map_err(EffectApplyError::Definition)?;
        }

        pipeline
            .apply_prepared_with_executor(prepared, context, executor)
            .map_err(EffectApplyError::Execution)
    }
}

impl<InitializeError, ExecutionError> fmt::Display
    for EffectApplyError<InitializeError, ExecutionError>
where
    InitializeError: fmt::Display,
    ExecutionError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Definition(error) => write!(formatter, "{error}"),
            Self::RegisteredDefinition(error) => {
                write!(formatter, "registered effect application failed: {error}")
            }
            Self::Application(error) => write!(formatter, "effect application failed: {error}"),
            Self::Initialize(error) => write!(formatter, "effect initialization failed: {error}"),
            Self::Execution(error) => write!(formatter, "effect execution failed: {error}"),
        }
    }
}

impl<InitializeError, ExecutionError> std::error::Error
    for EffectApplyError<InitializeError, ExecutionError>
where
    InitializeError: std::error::Error + 'static,
    ExecutionError: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Definition(error) => Some(error),
            Self::RegisteredDefinition(error) => Some(error),
            Self::Application(error) => Some(error),
            Self::Initialize(error) => Some(error),
            Self::Execution(error) => Some(error),
        }
    }
}

impl EffectTick {
    #[must_use]
    pub const fn new(elapsed_units: ClockUnits) -> Self {
        Self { elapsed_units }
    }

    pub fn run<Tags, Payload>(self, pipeline: &mut EffectPipeline<Tags, Payload>)
    where
        Tags: TagCollection,
    {
        let mut context = ();
        let mut executor = NoEffectExecutor::new();
        self.run_with_executor(pipeline, &mut context, &mut executor)
            .unwrap_or_else(infallible_error);
    }

    pub fn run_with_executor<Context, Tags, Payload, Executor>(
        self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
        context: &mut Context,
        executor: &mut Executor,
    ) -> Result<(), Executor::Error>
    where
        Tags: TagCollection,
        Executor: EffectExecutor<Context, Tags, Payload>,
    {
        pipeline.advance_with_executor(self.elapsed_units, context, executor)
    }
}

impl EffectRemove {
    #[must_use]
    pub const fn new(effect_id: ActiveEffectId) -> Self {
        Self { effect_id }
    }

    pub fn run<Tags, Payload>(
        self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
    ) -> Option<EffectInstance<Tags, Payload>>
    where
        Tags: TagCollection,
    {
        let mut sink = DiscardEffectLifecycleEvents;
        self.run_with_sink(pipeline, &mut sink)
    }

    pub fn run_with_sink<Tags, Payload, Sink>(
        self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
        sink: &mut Sink,
    ) -> Option<EffectInstance<Tags, Payload>>
    where
        Tags: TagCollection,
        Sink: EffectLifecycleSink<Tags, Payload>,
    {
        pipeline.remove_with_sink(self.effect_id, sink)
    }
}

impl EffectRemoveForObject {
    #[must_use]
    pub const fn new(object_id: ObjectId, policy: EffectObjectRemovalPolicy) -> Self {
        Self { object_id, policy }
    }

    #[must_use]
    pub fn run<Tags, Payload>(
        self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
    ) -> Vec<EffectInstance<Tags, Payload>>
    where
        Tags: TagCollection,
    {
        let mut sink = DiscardEffectLifecycleEvents;
        self.run_with_sink(pipeline, &mut sink)
    }

    pub fn run_with_sink<Tags, Payload, Sink>(
        self,
        pipeline: &mut EffectPipeline<Tags, Payload>,
        sink: &mut Sink,
    ) -> Vec<EffectInstance<Tags, Payload>>
    where
        Tags: TagCollection,
        Sink: EffectLifecycleSink<Tags, Payload>,
    {
        pipeline.remove_for_object_with_sink(self.object_id, self.policy, sink)
    }
}

fn infallible_error<T>(error: Infallible) -> T {
    match error {}
}
