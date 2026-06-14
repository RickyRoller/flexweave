//! Domain-neutral clock unit adapters.

use std::time::Duration;

/// Caller-defined mechanics clock units.
///
/// Flexweave treats these as opaque units. A game may map one unit to a turn,
/// millisecond, server tick, phase, or any other deterministic clock boundary.
pub type ClockUnits = u64;

/// Converts caller-owned clock steps into Flexweave clock units.
pub trait Clock {
    type Step;

    fn units_for(&self, step: Self::Step) -> ClockUnits;
}

/// Fixed-step clock for turn, phase, tick, or other count-based games.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FixedStepClock {
    units_per_step: ClockUnits,
}

impl FixedStepClock {
    #[must_use]
    pub const fn new(units_per_step: ClockUnits) -> Self {
        Self { units_per_step }
    }

    #[must_use]
    pub const fn units_per_step(self) -> ClockUnits {
        self.units_per_step
    }
}

impl Clock for FixedStepClock {
    type Step = ClockUnits;

    fn units_for(&self, step: Self::Step) -> ClockUnits {
        self.units_per_step.saturating_mul(step)
    }
}

/// Realtime clock that maps `std::time::Duration` into caller-selected units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RealtimeClock {
    units_per_second: ClockUnits,
}

impl RealtimeClock {
    #[must_use]
    pub const fn new(units_per_second: ClockUnits) -> Self {
        Self { units_per_second }
    }

    #[must_use]
    pub const fn units_per_second(self) -> ClockUnits {
        self.units_per_second
    }
}

impl Clock for RealtimeClock {
    type Step = Duration;

    fn units_for(&self, step: Self::Step) -> ClockUnits {
        let units = step
            .as_nanos()
            .saturating_mul(u128::from(self.units_per_second))
            / 1_000_000_000;
        units.min(u128::from(ClockUnits::MAX)) as ClockUnits
    }
}
