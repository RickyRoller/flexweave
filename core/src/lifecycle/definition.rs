use std::collections::BTreeMap;
use std::fmt;

use super::kind::LifecycleEventKind;

/// Validation failures for event channel definitions and route wiring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventChannelDefinitionError {
    EmptyChannelName,
    InvalidChannelName {
        channel_name: String,
    },
    EmptyPayloadContract {
        channel_name: String,
    },
    DuplicatePayloadKind {
        channel_name: String,
        kind: LifecycleEventKind,
    },
    DuplicateChannelDefinition {
        channel_name: String,
    },
    MissingChannelDefinition {
        channel_name: String,
    },
    PayloadMismatch {
        channel_name: String,
        kind: LifecycleEventKind,
    },
}

impl fmt::Display for EventChannelDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChannelName => formatter.write_str("event channel name is empty"),
            Self::InvalidChannelName { channel_name } => {
                write!(formatter, "event channel name `{channel_name}` is invalid")
            }
            Self::EmptyPayloadContract { channel_name } => write!(
                formatter,
                "event channel `{channel_name}` has an empty payload contract"
            ),
            Self::DuplicatePayloadKind { channel_name, kind } => write!(
                formatter,
                "event channel `{channel_name}` has duplicate payload kind {kind:?}"
            ),
            Self::DuplicateChannelDefinition { channel_name } => {
                write!(
                    formatter,
                    "event channel `{channel_name}` is defined more than once"
                )
            }
            Self::MissingChannelDefinition { channel_name } => {
                write!(formatter, "event channel `{channel_name}` is not defined")
            }
            Self::PayloadMismatch { channel_name, kind } => write!(
                formatter,
                "event channel `{channel_name}` does not accept payload kind {kind:?}"
            ),
        }
    }
}

impl std::error::Error for EventChannelDefinitionError {}

/// Named, typed routing target for lifecycle facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventChannelDefinition {
    name: String,
    accepted_kinds: Vec<LifecycleEventKind>,
}

impl EventChannelDefinition {
    /// Builds a channel definition and validates its stable name and payload contract.
    pub fn new<I>(
        name: impl Into<String>,
        accepted_kinds: I,
    ) -> Result<Self, EventChannelDefinitionError>
    where
        I: IntoIterator<Item = LifecycleEventKind>,
    {
        let name = validate_channel_name(name.into())?;
        let mut kinds = Vec::new();
        for kind in accepted_kinds {
            if kinds.contains(&kind) {
                return Err(EventChannelDefinitionError::DuplicatePayloadKind {
                    channel_name: name,
                    kind,
                });
            }
            kinds.push(kind);
        }
        if kinds.is_empty() {
            return Err(EventChannelDefinitionError::EmptyPayloadContract { channel_name: name });
        }
        Ok(Self {
            name,
            accepted_kinds: kinds,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn accepted_kinds(&self) -> &[LifecycleEventKind] {
        &self.accepted_kinds
    }

    #[must_use]
    pub fn accepts(&self, kind: LifecycleEventKind) -> bool {
        self.accepted_kinds.contains(&kind)
    }

    /// Validates one lifecycle fact kind against this channel's payload contract.
    pub fn validate_payload_kind(
        &self,
        kind: LifecycleEventKind,
    ) -> Result<(), EventChannelDefinitionError> {
        if self.accepts(kind) {
            Ok(())
        } else {
            Err(EventChannelDefinitionError::PayloadMismatch {
                channel_name: self.name.clone(),
                kind,
            })
        }
    }
}

/// A validated collection of channel definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventChannelDefinitions {
    definitions: Vec<EventChannelDefinition>,
    indexes: BTreeMap<String, usize>,
}

impl EventChannelDefinitions {
    /// Builds a definition collection and rejects duplicate channel names.
    pub fn new<I>(definitions: I) -> Result<Self, EventChannelDefinitionError>
    where
        I: IntoIterator<Item = EventChannelDefinition>,
    {
        let mut collection = Self {
            definitions: Vec::new(),
            indexes: BTreeMap::new(),
        };
        for definition in definitions {
            if collection.indexes.contains_key(definition.name()) {
                return Err(EventChannelDefinitionError::DuplicateChannelDefinition {
                    channel_name: definition.name().to_owned(),
                });
            }
            let index = collection.definitions.len();
            collection
                .indexes
                .insert(definition.name().to_owned(), index);
            collection.definitions.push(definition);
        }
        Ok(collection)
    }

    #[must_use]
    pub fn definitions(&self) -> &[EventChannelDefinition] {
        &self.definitions
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&EventChannelDefinition> {
        self.indexes
            .get(name)
            .and_then(|index| self.definitions.get(*index))
    }

    /// Validates one explicit source-to-channel route.
    pub fn validate_route(
        &self,
        route: &EventChannelRouteDefinition,
    ) -> Result<(), EventChannelDefinitionError> {
        let definition = self.get(route.channel_name()).ok_or_else(|| {
            EventChannelDefinitionError::MissingChannelDefinition {
                channel_name: route.channel_name().to_owned(),
            }
        })?;
        definition.validate_payload_kind(route.event_kind())
    }
}

/// One explicit lifecycle fact route into a named event channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventChannelRouteDefinition {
    channel_name: String,
    event_kind: LifecycleEventKind,
}

impl EventChannelRouteDefinition {
    pub fn new(
        channel_name: impl Into<String>,
        event_kind: LifecycleEventKind,
    ) -> Result<Self, EventChannelDefinitionError> {
        Ok(Self {
            channel_name: validate_channel_name(channel_name.into())?,
            event_kind,
        })
    }

    #[must_use]
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    #[must_use]
    pub fn event_kind(&self) -> LifecycleEventKind {
        self.event_kind
    }
}

fn validate_channel_name(name: String) -> Result<String, EventChannelDefinitionError> {
    if name.is_empty() {
        return Err(EventChannelDefinitionError::EmptyChannelName);
    }
    let valid = name.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | ':' | '/')
    });
    if valid {
        Ok(name)
    } else {
        Err(EventChannelDefinitionError::InvalidChannelName { channel_name: name })
    }
}
