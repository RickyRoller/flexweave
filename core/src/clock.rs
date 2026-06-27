//! Domain-neutral clock unit adapters.

use std::time::Duration;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

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
///
/// This conversion is stateless and floors fractional units for each call. Use
/// [`RealtimeClockAccumulator`] when advancing mechanics from repeated realtime
/// frame deltas that may be smaller than one clock unit.
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
            / NANOS_PER_SECOND;
        units.min(u128::from(ClockUnits::MAX)) as ClockUnits
    }
}

/// Stateful realtime converter that preserves fractional clock units.
///
/// Use this at the boundary between a caller-owned realtime loop and
/// Flexweave's integer mechanics ticks. Each [`Self::advance`] call returns the
/// whole units available for the elapsed duration while retaining the fractional
/// remainder for later calls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RealtimeClockAccumulator {
    clock: RealtimeClock,
    remainder_nanos_times_units: u128,
}

impl RealtimeClockAccumulator {
    /// Creates an accumulator using the same scale as [`RealtimeClock::new`].
    #[must_use]
    pub const fn new(units_per_second: ClockUnits) -> Self {
        Self::from_clock(RealtimeClock::new(units_per_second))
    }

    /// Creates an accumulator from an existing realtime clock scale.
    #[must_use]
    pub const fn from_clock(clock: RealtimeClock) -> Self {
        Self {
            clock,
            remainder_nanos_times_units: 0,
        }
    }

    /// Returns the stateless realtime clock scale used by this accumulator.
    #[must_use]
    pub const fn clock(self) -> RealtimeClock {
        self.clock
    }

    /// Returns the configured number of mechanics units per second.
    #[must_use]
    pub const fn units_per_second(self) -> ClockUnits {
        self.clock.units_per_second()
    }

    /// Converts elapsed realtime into whole mechanics units and retains any
    /// fractional remainder for the next call.
    pub fn advance(&mut self, elapsed: Duration) -> ClockUnits {
        let accumulated_nanos_times_units = elapsed
            .as_nanos()
            .saturating_mul(u128::from(self.clock.units_per_second()))
            .saturating_add(self.remainder_nanos_times_units);
        let units = accumulated_nanos_times_units / NANOS_PER_SECOND;
        self.remainder_nanos_times_units = accumulated_nanos_times_units % NANOS_PER_SECOND;
        units.min(u128::from(ClockUnits::MAX)) as ClockUnits
    }

    /// Drops any retained fractional remainder.
    pub fn reset(&mut self) {
        self.remainder_nanos_times_units = 0;
    }
}
