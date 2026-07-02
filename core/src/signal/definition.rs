use crate::lifecycle::LifecycleEventKind;
use crate::tag::{TagSet, TagSetQuery};
use std::fmt;

/// Signal lifecycle categories emitted by projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalKind {
    ActiveStart,
    WhileActive,
    Executed,
    Recurring,
    Removed,
}

/// Signal retention metadata.
///
/// This is authoring/export metadata. It does not configure `EventChannel`
/// retention unless caller code maps it to a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalRetentionPolicy {
    Drop,
    Retain,
}

/// Signal export metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalExportPolicy {
    Internal,
    Export,
}

/// Tag matching policy for a Signal definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalTagMatch<Atom> {
    Any,
    Query(TagSetQuery<Atom>),
}

impl<Atom> SignalTagMatch<Atom>
where
    Atom: Eq,
{
    pub(super) fn matches(&self, tags: &TagSet<Atom>) -> bool {
        match self {
            Self::Any => true,
            Self::Query(query) => tags.matches(query),
        }
    }
}

impl<Atom> SignalTagMatch<Atom> {
    fn is_valid(&self) -> bool {
        match self {
            Self::Any => true,
            Self::Query(query) => {
                !query.all.is_empty() || !query.any.is_empty() || !query.none.is_empty()
            }
        }
    }
}

/// Authorable Signal definition data.
///
/// `channel_key` names the caller-owned channel or adapter target that should
/// receive projected facts. The key is metadata and a validation hint; it does
/// not cause automatic routing.
///
/// `lifecycle_event_kinds` is the complete source-kind whitelist for this
/// definition. While-active reinvocation requires
/// [`LifecycleEventKind::SignalReinvoked`] in this list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalDefinition<Atom, PayloadSchema = ()> {
    pub key: String,
    pub signal_kind: SignalKind,
    pub lifecycle_event_kinds: Vec<LifecycleEventKind>,
    pub tag_match: SignalTagMatch<Atom>,
    pub payload_schema: String,
    pub signal_payload: PayloadSchema,
    pub channel_key: String,
    pub category: String,
    pub retention: SignalRetentionPolicy,
    pub export: SignalExportPolicy,
    pub debug_label: String,
    pub description: String,
}

/// Signal definition validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalDefinitionError {
    EmptyKey,
    DuplicateKey { key: String },
    MissingLifecycleEventKinds { key: String },
    InvalidTagQuery { key: String },
    MissingPayloadSchema { key: String },
    MissingChannelKey { key: String },
    MissingCategory { key: String },
    UnknownChannelKey { key: String, channel_key: String },
}

impl fmt::Display for SignalDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("signal definition key is empty"),
            Self::DuplicateKey { key } => {
                write!(
                    formatter,
                    "signal definition `{key}` is defined more than once"
                )
            }
            Self::MissingLifecycleEventKinds { key } => write!(
                formatter,
                "signal definition `{key}` must declare at least one lifecycle event kind"
            ),
            Self::InvalidTagQuery { key } => {
                write!(
                    formatter,
                    "signal definition `{key}` has an invalid tag query"
                )
            }
            Self::MissingPayloadSchema { key } => {
                write!(
                    formatter,
                    "signal definition `{key}` is missing a payload schema"
                )
            }
            Self::MissingChannelKey { key } => {
                write!(
                    formatter,
                    "signal definition `{key}` is missing a channel key"
                )
            }
            Self::MissingCategory { key } => {
                write!(formatter, "signal definition `{key}` is missing a category")
            }
            Self::UnknownChannelKey { key, channel_key } => write!(
                formatter,
                "signal definition `{key}` references unknown channel `{channel_key}`"
            ),
        }
    }
}

impl std::error::Error for SignalDefinitionError {}

impl<Atom, PayloadSchema> SignalDefinition<Atom, PayloadSchema> {
    /// Validates one Signal definition.
    pub fn validate(&self) -> Result<(), SignalDefinitionError> {
        if self.key.is_empty() {
            return Err(SignalDefinitionError::EmptyKey);
        }
        if self.lifecycle_event_kinds.is_empty() {
            return Err(SignalDefinitionError::MissingLifecycleEventKinds {
                key: self.key.clone(),
            });
        }
        if !self.tag_match.is_valid() {
            return Err(SignalDefinitionError::InvalidTagQuery {
                key: self.key.clone(),
            });
        }
        if self.payload_schema.is_empty() {
            return Err(SignalDefinitionError::MissingPayloadSchema {
                key: self.key.clone(),
            });
        }
        if self.channel_key.is_empty() {
            return Err(SignalDefinitionError::MissingChannelKey {
                key: self.key.clone(),
            });
        }
        if self.category.is_empty() {
            return Err(SignalDefinitionError::MissingCategory {
                key: self.key.clone(),
            });
        }
        Ok(())
    }
}

/// Validated Signal definitions in deterministic declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalDefinitions<Atom, PayloadSchema> {
    definitions: Vec<SignalDefinition<Atom, PayloadSchema>>,
}

impl<Atom, PayloadSchema> SignalDefinitions<Atom, PayloadSchema> {
    /// Builds a validated definition collection and rejects duplicate keys.
    pub fn new<I>(definitions: I) -> Result<Self, SignalDefinitionError>
    where
        I: IntoIterator<Item = SignalDefinition<Atom, PayloadSchema>>,
    {
        let mut ordered = Vec::new();
        for definition in definitions {
            definition.validate()?;
            if ordered
                .iter()
                .any(|existing: &SignalDefinition<Atom, PayloadSchema>| {
                    existing.key == definition.key
                })
            {
                return Err(SignalDefinitionError::DuplicateKey {
                    key: definition.key,
                });
            }
            ordered.push(definition);
        }
        Ok(Self {
            definitions: ordered,
        })
    }

    #[must_use]
    pub fn definitions(&self) -> &[SignalDefinition<Atom, PayloadSchema>] {
        &self.definitions
    }

    /// Validates channel references against caller-provided channel keys.
    ///
    /// Validation proves the declared keys are known. Caller code still owns
    /// projecting facts and publishing them into the matching channel or bus.
    pub fn validate_channels(&self, known_channels: &[&str]) -> Result<(), SignalDefinitionError> {
        for definition in &self.definitions {
            if !known_channels
                .iter()
                .any(|known| *known == definition.channel_key)
            {
                return Err(SignalDefinitionError::UnknownChannelKey {
                    key: definition.key.clone(),
                    channel_key: definition.channel_key.clone(),
                });
            }
        }
        Ok(())
    }
}
