use crate::clock::{Clock, ClockUnits};
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

        match self.kind {
            EffectKind::Instant => {
                self.reject_duration()?;
                self.reject_period()?;
            }
            EffectKind::Duration => {
                self.require_duration()?;
                self.reject_period()?;
            }
            EffectKind::Periodic => {
                self.require_duration()?;
                self.require_period()?;
            }
            EffectKind::Indefinite => {
                self.reject_duration()?;
                self.reject_period()?;
            }
        }

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

    fn require_duration(&self) -> Result<EffectClockPolicy, EffectDefinitionError> {
        let duration = self
            .duration
            .ok_or_else(|| EffectDefinitionError::DurationRequired {
                key: self.key.clone(),
            })?;
        if duration.units == 0 {
            return Err(EffectDefinitionError::InvalidDuration {
                key: self.key.clone(),
            });
        }
        Ok(duration)
    }

    fn require_period(&self) -> Result<EffectClockPolicy, EffectDefinitionError> {
        let period = self
            .period
            .ok_or_else(|| EffectDefinitionError::PeriodRequired {
                key: self.key.clone(),
            })?;
        if period.units == 0 {
            return Err(EffectDefinitionError::InvalidPeriod {
                key: self.key.clone(),
            });
        }
        Ok(period)
    }

    fn reject_duration(&self) -> Result<(), EffectDefinitionError> {
        if self.duration.is_some() {
            Err(EffectDefinitionError::DurationNotAllowed {
                key: self.key.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn reject_period(&self) -> Result<(), EffectDefinitionError> {
        if self.period.is_some() {
            Err(EffectDefinitionError::PeriodNotAllowed {
                key: self.key.clone(),
            })
        } else {
            Ok(())
        }
    }
}
