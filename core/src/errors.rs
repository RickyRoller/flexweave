//! Explicit primitive error set shared by Flexweave.

use std::fmt;

/// Domain-neutral primitive errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    MissingRequiredData,
    InvalidObjectId,
    ObjectIdAlreadyExists,
    OutOfMemory,
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingRequiredData => "missing required data",
            Self::InvalidObjectId => "invalid object id",
            Self::ObjectIdAlreadyExists => "object id already exists",
            Self::OutOfMemory => "out of memory",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CoreError {}
