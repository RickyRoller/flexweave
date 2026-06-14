use std::fmt;

/// Active effect instance id assigned by `EffectPipeline`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActiveEffectId(u64);

impl ActiveEffectId {
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

impl From<u64> for ActiveEffectId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<ActiveEffectId> for u64 {
    fn from(value: ActiveEffectId) -> Self {
        value.get()
    }
}

impl fmt::Display for ActiveEffectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
