use crate::clock::{Clock, ClockUnits};
use crate::registry::{DefinitionRegistryEntry, RegistryEntry};
use std::collections::BTreeMap;
use std::fmt;

/// Effect definition kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectKind {
    Instant,
    Duration,
    Periodic,
    Indefinite,
}

/// Duration or period clock policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectClockPolicy {
    pub units: ClockUnits,
}

impl EffectClockPolicy {
    #[must_use]
    pub const fn new(units: ClockUnits) -> Self {
        Self { units }
    }

    #[must_use]
    pub fn from_clock<C>(clock: &C, step: C::Step) -> Self
    where
        C: Clock,
    {
        Self {
            units: clock.units_for(step),
        }
    }
}

/// Named lifecycle and Signal routing keys declared by an effect definition.
///
/// These keys are authoring metadata for validation and caller-owned adapter
/// wiring. `EffectPipeline` emits lifecycle facts through callbacks; it does
/// not publish to these channels or project Signal facts automatically.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectRouting {
    pub requires_lifecycle_channel: bool,
    pub lifecycle_channel_keys: Vec<String>,
    pub signal_channel_keys: Vec<String>,
}

/// Authorable, reusable effect definition data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition<PayloadSchema = ()> {
    pub key: String,
    pub kind: EffectKind,
    pub duration: Option<EffectClockPolicy>,
    pub period: Option<EffectClockPolicy>,
    pub routing: EffectRouting,
    pub payload_schema: PayloadSchema,
}

/// Authoring-time effect definition validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectDefinitionError {
    EmptyKey,
    DurationRequired { key: String },
    DurationNotAllowed { key: String },
    PeriodRequired { key: String },
    PeriodNotAllowed { key: String },
    InvalidDuration { key: String },
    InvalidPeriod { key: String },
    MissingLifecycleChannelKey { key: String },
    EmptyLifecycleChannelKey { key: String },
    EmptySignalChannelKey { key: String },
}

impl fmt::Display for EffectDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("effect definition key is empty"),
            Self::DurationRequired { key } => {
                write!(formatter, "effect definition `{key}` requires a duration")
            }
            Self::DurationNotAllowed { key } => write!(
                formatter,
                "effect definition `{key}` must not define a duration"
            ),
            Self::PeriodRequired { key } => {
                write!(formatter, "effect definition `{key}` requires a period")
            }
            Self::PeriodNotAllowed { key } => {
                write!(
                    formatter,
                    "effect definition `{key}` must not define a period"
                )
            }
            Self::InvalidDuration { key } => {
                write!(
                    formatter,
                    "effect definition `{key}` has an invalid duration"
                )
            }
            Self::InvalidPeriod { key } => {
                write!(formatter, "effect definition `{key}` has an invalid period")
            }
            Self::MissingLifecycleChannelKey { key } => write!(
                formatter,
                "effect definition `{key}` requires a lifecycle channel key"
            ),
            Self::EmptyLifecycleChannelKey { key } => write!(
                formatter,
                "effect definition `{key}` has an empty lifecycle channel key"
            ),
            Self::EmptySignalChannelKey { key } => write!(
                formatter,
                "effect definition `{key}` has an empty signal channel key"
            ),
        }
    }
}

impl std::error::Error for EffectDefinitionError {}

/// Validated effect definition registry failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectDefinitionRegistryError {
    InvalidDefinition { error: EffectDefinitionError },
    DuplicateKey { key: String },
    MissingDefinition { key: String },
}

impl fmt::Display for EffectDefinitionRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDefinition { error } => {
                write!(formatter, "invalid effect definition: {error}")
            }
            Self::DuplicateKey { key } => {
                write!(
                    formatter,
                    "effect definition `{key}` is defined more than once"
                )
            }
            Self::MissingDefinition { key } => {
                write!(formatter, "effect definition `{key}` is not registered")
            }
        }
    }
}

impl std::error::Error for EffectDefinitionRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidDefinition { error } => Some(error),
            Self::DuplicateKey { .. } | Self::MissingDefinition { .. } => None,
        }
    }
}

impl<PayloadSchema> EffectDefinition<PayloadSchema> {
    /// Creates an instant effect definition with no routing.
    #[must_use]
    pub fn instant(key: impl Into<String>, payload_schema: PayloadSchema) -> Self {
        Self {
            key: key.into(),
            kind: EffectKind::Instant,
            duration: None,
            period: None,
            routing: EffectRouting::default(),
            payload_schema,
        }
    }

    /// Creates a duration effect definition with no routing.
    #[must_use]
    pub fn duration(
        key: impl Into<String>,
        units: ClockUnits,
        payload_schema: PayloadSchema,
    ) -> Self {
        Self {
            key: key.into(),
            kind: EffectKind::Duration,
            duration: Some(EffectClockPolicy::new(units)),
            period: None,
            routing: EffectRouting::default(),
            payload_schema,
        }
    }

    /// Creates a periodic effect definition with no routing.
    #[must_use]
    pub fn periodic(
        key: impl Into<String>,
        duration_units: ClockUnits,
        period_units: ClockUnits,
        payload_schema: PayloadSchema,
    ) -> Self {
        Self {
            key: key.into(),
            kind: EffectKind::Periodic,
            duration: Some(EffectClockPolicy::new(duration_units)),
            period: Some(EffectClockPolicy::new(period_units)),
            routing: EffectRouting::default(),
            payload_schema,
        }
    }

    /// Creates an indefinite effect definition with no routing.
    #[must_use]
    pub fn indefinite(key: impl Into<String>, payload_schema: PayloadSchema) -> Self {
        Self {
            key: key.into(),
            kind: EffectKind::Indefinite,
            duration: None,
            period: None,
            routing: EffectRouting::default(),
            payload_schema,
        }
    }

    #[must_use]
    pub fn with_routing(mut self, routing: EffectRouting) -> Self {
        self.routing = routing;
        self
    }

    /// Marks lifecycle publication as required on `channel_key` metadata.
    ///
    /// This validates that the definition names a lifecycle channel, but caller
    /// code must still publish emitted lifecycle facts to that channel.
    #[must_use]
    pub fn requiring_lifecycle_channel(mut self, channel_key: impl Into<String>) -> Self {
        self.routing.requires_lifecycle_channel = true;
        self.routing.lifecycle_channel_keys.push(channel_key.into());
        self
    }

    /// Replaces lifecycle channel keys while preserving the lifecycle requirement flag.
    #[must_use]
    pub fn with_lifecycle_channels<I, K>(mut self, channel_keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.routing.lifecycle_channel_keys = channel_keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        self
    }

    #[must_use]
    pub fn with_signal_channels<I, K>(mut self, channel_keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.routing.signal_channel_keys = channel_keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        self
    }

    /// Validates authorable definition shape before runtime application.
    pub fn validate(&self) -> Result<(), EffectDefinitionError> {
        if self.key.is_empty() {
            return Err(EffectDefinitionError::EmptyKey);
        }

        self.validate_clock_shape(self.duration, self.period)?;

        if self.routing.requires_lifecycle_channel && self.routing.lifecycle_channel_keys.is_empty()
        {
            return Err(EffectDefinitionError::MissingLifecycleChannelKey {
                key: self.key.clone(),
            });
        }
        if self
            .routing
            .lifecycle_channel_keys
            .iter()
            .any(|key| key.is_empty())
        {
            return Err(EffectDefinitionError::EmptyLifecycleChannelKey {
                key: self.key.clone(),
            });
        }
        if self
            .routing
            .signal_channel_keys
            .iter()
            .any(|key| key.is_empty())
        {
            return Err(EffectDefinitionError::EmptySignalChannelKey {
                key: self.key.clone(),
            });
        }

        Ok(())
    }

    pub(crate) fn validate_clock_shape(
        &self,
        duration: Option<EffectClockPolicy>,
        period: Option<EffectClockPolicy>,
    ) -> Result<(), EffectDefinitionError> {
        match self.kind {
            EffectKind::Instant => {
                self.reject_duration(duration)?;
                self.reject_period(period)?;
            }
            EffectKind::Duration => {
                self.require_duration(duration)?;
                self.reject_period(period)?;
            }
            EffectKind::Periodic => {
                self.require_duration(duration)?;
                self.require_period(period)?;
            }
            EffectKind::Indefinite => {
                self.reject_duration(duration)?;
                self.reject_period(period)?;
            }
        }

        Ok(())
    }

    fn require_duration(
        &self,
        duration: Option<EffectClockPolicy>,
    ) -> Result<EffectClockPolicy, EffectDefinitionError> {
        let duration = duration.ok_or_else(|| EffectDefinitionError::DurationRequired {
            key: self.key.clone(),
        })?;
        if duration.units == 0 {
            return Err(EffectDefinitionError::InvalidDuration {
                key: self.key.clone(),
            });
        }
        Ok(duration)
    }

    fn require_period(
        &self,
        period: Option<EffectClockPolicy>,
    ) -> Result<EffectClockPolicy, EffectDefinitionError> {
        let period = period.ok_or_else(|| EffectDefinitionError::PeriodRequired {
            key: self.key.clone(),
        })?;
        if period.units == 0 {
            return Err(EffectDefinitionError::InvalidPeriod {
                key: self.key.clone(),
            });
        }
        Ok(period)
    }

    fn reject_duration(
        &self,
        duration: Option<EffectClockPolicy>,
    ) -> Result<(), EffectDefinitionError> {
        if duration.is_some() {
            Err(EffectDefinitionError::DurationNotAllowed {
                key: self.key.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn reject_period(
        &self,
        period: Option<EffectClockPolicy>,
    ) -> Result<(), EffectDefinitionError> {
        if period.is_some() {
            Err(EffectDefinitionError::PeriodNotAllowed {
                key: self.key.clone(),
            })
        } else {
            Ok(())
        }
    }
}

impl<PayloadSchema> RegistryEntry for EffectDefinition<PayloadSchema> {
    fn key(&self) -> &str {
        &self.key
    }
}

impl<PayloadSchema> DefinitionRegistryEntry for EffectDefinition<PayloadSchema>
where
    PayloadSchema: Clone,
{
    type Definition = Self;

    fn build_definition(&self) -> Self::Definition {
        self.clone()
    }
}

/// Validated effect definitions in deterministic declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinitions<PayloadSchema = ()> {
    definitions: Vec<EffectDefinition<PayloadSchema>>,
    indexes: BTreeMap<String, usize>,
}

impl<PayloadSchema> EffectDefinitions<PayloadSchema> {
    /// Builds a validated definition collection and rejects duplicate keys.
    pub fn new<I>(definitions: I) -> Result<Self, EffectDefinitionRegistryError>
    where
        I: IntoIterator<Item = EffectDefinition<PayloadSchema>>,
    {
        let mut collection = Self {
            definitions: Vec::new(),
            indexes: BTreeMap::new(),
        };
        for definition in definitions {
            definition
                .validate()
                .map_err(|error| EffectDefinitionRegistryError::InvalidDefinition { error })?;
            if collection.indexes.contains_key(&definition.key) {
                return Err(EffectDefinitionRegistryError::DuplicateKey {
                    key: definition.key,
                });
            }
            let index = collection.definitions.len();
            collection.indexes.insert(definition.key.clone(), index);
            collection.definitions.push(definition);
        }
        Ok(collection)
    }

    #[must_use]
    pub fn definitions(&self) -> &[EffectDefinition<PayloadSchema>] {
        &self.definitions
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&EffectDefinition<PayloadSchema>> {
        self.indexes
            .get(key)
            .and_then(|index| self.definitions.get(*index))
    }

    pub fn require(
        &self,
        key: &str,
    ) -> Result<&EffectDefinition<PayloadSchema>, EffectDefinitionRegistryError> {
        self.get(key)
            .ok_or_else(|| EffectDefinitionRegistryError::MissingDefinition {
                key: key.to_owned(),
            })
    }
}
