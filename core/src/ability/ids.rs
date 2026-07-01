use std::fmt;

/// Granted ability id assigned by `AbilityStore`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbilityId(u64);

impl AbilityId {
    pub const INVALID: Self = Self(0);

    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn is_invalid(self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for AbilityId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<AbilityId> for u64 {
    fn from(value: AbilityId) -> Self {
        value.get()
    }
}

impl fmt::Display for AbilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Active ability activation id assigned by `AbilityStore`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbilityActivationId(u64);

impl AbilityActivationId {
    pub const INVALID: Self = Self(0);

    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn is_invalid(self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for AbilityActivationId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<AbilityActivationId> for u64 {
    fn from(value: AbilityActivationId) -> Self {
        value.get()
    }
}

impl fmt::Display for AbilityActivationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
