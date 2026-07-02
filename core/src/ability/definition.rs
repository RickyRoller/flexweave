use std::collections::BTreeMap;
use std::fmt;

use crate::registry::{DefinitionRegistryEntry, RegistryEntry};

/// Authorable ability definition metadata.
///
/// `emitted_channel_keys` are metadata for validation and caller-owned adapter
/// wiring. Ability activation APIs emit lifecycle facts through return values or
/// callbacks; they do not publish to channels automatically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityDefinition<PayloadSchema = ()> {
    pub key: String,
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
        }
    }
}

impl std::error::Error for AbilityDefinitionError {}

/// Validated ability definition registry failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbilityDefinitionRegistryError {
    InvalidDefinition { error: AbilityDefinitionError },
    DuplicateKey { key: String },
    MissingDefinition { key: String },
}

impl fmt::Display for AbilityDefinitionRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDefinition { error } => {
                write!(formatter, "invalid ability definition: {error}")
            }
            Self::DuplicateKey { key } => {
                write!(
                    formatter,
                    "ability definition `{key}` is defined more than once"
                )
            }
            Self::MissingDefinition { key } => {
                write!(formatter, "ability definition `{key}` is not registered")
            }
        }
    }
}

impl std::error::Error for AbilityDefinitionRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidDefinition { error } => Some(error),
            Self::DuplicateKey { .. } | Self::MissingDefinition { .. } => None,
        }
    }
}

impl<PayloadSchema> AbilityDefinition<PayloadSchema> {
    /// Creates ability definition metadata with no routing metadata.
    #[must_use]
    pub fn new(key: impl Into<String>, payload_schema: PayloadSchema) -> Self {
        Self {
            key: key.into(),
            emits_lifecycle: false,
            emitted_channel_keys: Vec::new(),
            payload_schema,
        }
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

impl<PayloadSchema> RegistryEntry for AbilityDefinition<PayloadSchema> {
    fn key(&self) -> &str {
        &self.key
    }
}

impl<PayloadSchema> DefinitionRegistryEntry for AbilityDefinition<PayloadSchema>
where
    PayloadSchema: Clone,
{
    type Definition = Self;

    fn build_definition(&self) -> Self::Definition {
        self.clone()
    }
}

/// Validated ability definitions in deterministic declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityDefinitions<PayloadSchema = ()> {
    definitions: Vec<AbilityDefinition<PayloadSchema>>,
    indexes: BTreeMap<String, usize>,
}

impl<PayloadSchema> AbilityDefinitions<PayloadSchema> {
    /// Builds a validated definition collection and rejects duplicate keys.
    pub fn new<I>(definitions: I) -> Result<Self, AbilityDefinitionRegistryError>
    where
        I: IntoIterator<Item = AbilityDefinition<PayloadSchema>>,
    {
        let mut collection = Self {
            definitions: Vec::new(),
            indexes: BTreeMap::new(),
        };
        for definition in definitions {
            definition
                .validate()
                .map_err(|error| AbilityDefinitionRegistryError::InvalidDefinition { error })?;
            if collection.indexes.contains_key(&definition.key) {
                return Err(AbilityDefinitionRegistryError::DuplicateKey {
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
    pub fn definitions(&self) -> &[AbilityDefinition<PayloadSchema>] {
        &self.definitions
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&AbilityDefinition<PayloadSchema>> {
        self.indexes
            .get(key)
            .and_then(|index| self.definitions.get(*index))
    }

    pub fn require(
        &self,
        key: &str,
    ) -> Result<&AbilityDefinition<PayloadSchema>, AbilityDefinitionRegistryError> {
        self.get(key)
            .ok_or_else(|| AbilityDefinitionRegistryError::MissingDefinition {
                key: key.to_owned(),
            })
    }

    /// Validates emitted channel references against caller-provided channel keys.
    pub fn validate_channels(&self, known_channels: &[&str]) -> Result<(), AbilityDefinitionError> {
        for definition in &self.definitions {
            definition.validate_channels(known_channels)?;
        }
        Ok(())
    }
}
