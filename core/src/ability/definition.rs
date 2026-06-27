use std::fmt;

/// Ability activation mode declared by an ability definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityActivationMode {
    Instant,
    Active,
}

/// When cooldown or caller-owned commit policy should be applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityCommitTiming {
    OnStart,
    OnEnd,
    Manual,
}

/// Whether an activation can be canceled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityCancelPolicy {
    CannotCancel,
    CanCancel,
}

/// Authorable ability definition metadata.
///
/// `emitted_channel_keys` are metadata for validation and caller-owned adapter
/// wiring. Ability activation APIs emit lifecycle facts through return values or
/// callbacks; they do not publish to channels automatically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityDefinition<PayloadSchema = ()> {
    pub key: String,
    pub activation_mode: AbilityActivationMode,
    pub commit_timing: AbilityCommitTiming,
    pub cancel_policy: AbilityCancelPolicy,
    pub tag_requirement_keys: Vec<String>,
    pub activation_tag_keys: Vec<String>,
    pub emits_lifecycle: bool,
    pub emitted_channel_keys: Vec<String>,
    pub payload_schema: PayloadSchema,
}

/// Ability definition validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityDefinitionError {
    EmptyKey,
    MissingEmittedChannelKey { key: String },
    EmptyEmittedChannelKey { key: String },
    UnknownEmittedChannelKey { key: String, channel_key: String },
    EmptyTagRequirementKey { key: String },
    EmptyActivationTagKey { key: String },
    InstantCannotBeCanceled { key: String },
    InstantCannotCommitOnEnd { key: String },
}

impl fmt::Display for AbilityDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("ability definition key is empty"),
            Self::MissingEmittedChannelKey { key } => write!(
                formatter,
                "ability definition `{key}` emits lifecycle but has no emitted channel keys"
            ),
            Self::EmptyEmittedChannelKey { key } => write!(
                formatter,
                "ability definition `{key}` has an empty emitted channel key"
            ),
            Self::UnknownEmittedChannelKey { key, channel_key } => write!(
                formatter,
                "ability definition `{key}` references unknown emitted channel `{channel_key}`"
            ),
            Self::EmptyTagRequirementKey { key } => write!(
                formatter,
                "ability definition `{key}` has an empty tag requirement key"
            ),
            Self::EmptyActivationTagKey { key } => write!(
                formatter,
                "ability definition `{key}` has an empty activation tag key"
            ),
            Self::InstantCannotBeCanceled { key } => {
                write!(
                    formatter,
                    "instant ability definition `{key}` cannot be canceled"
                )
            }
            Self::InstantCannotCommitOnEnd { key } => write!(
                formatter,
                "instant ability definition `{key}` cannot commit on end"
            ),
        }
    }
}

impl std::error::Error for AbilityDefinitionError {}

impl<PayloadSchema> AbilityDefinition<PayloadSchema> {
    /// Creates instant ability definition metadata with no routing or tag metadata.
    #[must_use]
    pub fn instant(key: impl Into<String>, payload_schema: PayloadSchema) -> Self {
        Self {
            key: key.into(),
            activation_mode: AbilityActivationMode::Instant,
            commit_timing: AbilityCommitTiming::OnStart,
            cancel_policy: AbilityCancelPolicy::CannotCancel,
            tag_requirement_keys: Vec::new(),
            activation_tag_keys: Vec::new(),
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema,
        }
    }

    /// Creates active ability definition metadata with no routing or tag metadata.
    #[must_use]
    pub fn active(key: impl Into<String>, payload_schema: PayloadSchema) -> Self {
        Self {
            key: key.into(),
            activation_mode: AbilityActivationMode::Active,
            commit_timing: AbilityCommitTiming::OnStart,
            cancel_policy: AbilityCancelPolicy::CanCancel,
            tag_requirement_keys: Vec::new(),
            activation_tag_keys: Vec::new(),
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema,
        }
    }

    #[must_use]
    pub fn with_commit_timing(mut self, commit_timing: AbilityCommitTiming) -> Self {
        self.commit_timing = commit_timing;
        self
    }

    #[must_use]
    pub fn with_cancel_policy(mut self, cancel_policy: AbilityCancelPolicy) -> Self {
        self.cancel_policy = cancel_policy;
        self
    }

    #[must_use]
    pub fn with_tag_requirement_keys<I, K>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.tag_requirement_keys = keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        self
    }

    #[must_use]
    pub fn with_activation_tag_keys<I, K>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.activation_tag_keys = keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        self
    }

    /// Enables lifecycle publication metadata and replaces emitted channel keys.
    ///
    /// Caller code still owns publishing emitted ability lifecycle facts.
    #[must_use]
    pub fn with_lifecycle_channels<I, K>(mut self, channel_keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.emits_lifecycle = true;
        self.emitted_channel_keys = channel_keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        self
    }

    /// Validates authorable ability metadata before granting or activation.
    pub fn validate(&self) -> Result<(), AbilityDefinitionError> {
        if self.key.is_empty() {
            return Err(AbilityDefinitionError::EmptyKey);
        }
        if self.emits_lifecycle && self.emitted_channel_keys.is_empty() {
            return Err(AbilityDefinitionError::MissingEmittedChannelKey {
                key: self.key.clone(),
            });
        }
        if self.emitted_channel_keys.iter().any(|key| key.is_empty()) {
            return Err(AbilityDefinitionError::EmptyEmittedChannelKey {
                key: self.key.clone(),
            });
        }
        if self.tag_requirement_keys.iter().any(|key| key.is_empty()) {
            return Err(AbilityDefinitionError::EmptyTagRequirementKey {
                key: self.key.clone(),
            });
        }
        if self.activation_tag_keys.iter().any(|key| key.is_empty()) {
            return Err(AbilityDefinitionError::EmptyActivationTagKey {
                key: self.key.clone(),
            });
        }
        if self.activation_mode == AbilityActivationMode::Instant
            && self.cancel_policy == AbilityCancelPolicy::CanCancel
        {
            return Err(AbilityDefinitionError::InstantCannotBeCanceled {
                key: self.key.clone(),
            });
        }
        if self.activation_mode == AbilityActivationMode::Instant
            && self.commit_timing == AbilityCommitTiming::OnEnd
        {
            return Err(AbilityDefinitionError::InstantCannotCommitOnEnd {
                key: self.key.clone(),
            });
        }
        Ok(())
    }

    /// Validates emitted channel references against caller-provided channel keys.
    ///
    /// Validation does not wire automatic publication.
    pub fn validate_channels(&self, known_channels: &[&str]) -> Result<(), AbilityDefinitionError> {
        for channel_key in &self.emitted_channel_keys {
            if !known_channels.iter().any(|known| *known == channel_key) {
                return Err(AbilityDefinitionError::UnknownEmittedChannelKey {
                    key: self.key.clone(),
                    channel_key: channel_key.clone(),
                });
            }
        }
        Ok(())
    }
}
