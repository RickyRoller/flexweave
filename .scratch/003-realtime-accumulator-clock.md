# Add realtime accumulation to avoid fractional clock-unit drift

## Validation Verdict

Valid.

This is an API gap in realtime ticking rather than a bug in cooldown or effect storage. Flexweave stores consume integer `ClockUnits`; the drift is introduced when `RealtimeClock` converts each `Duration` independently and floors fractional units.

This strengthens Flexweave because caller-defined clocks are part of the public primitive model.

## Problem

`RealtimeClock::units_for(Duration)` converts elapsed time into integer `ClockUnits` using integer division. Any fractional unit is discarded each call.

That is fine for one-shot conversion, but not for frame-driven realtime updates. Repeated sub-unit or fractional-unit ticks can make cooldowns and effects expire late, fire late, or never advance.

Concrete examples:

- `RealtimeClock::new(60)` with repeated `Duration::from_millis(16)` returns `0` units per frame, so mechanics never advance.
- `RealtimeClock::new(1000)` with 16.666 ms frame deltas floors to 16 ms each tick, steadily losing fractional time.

## Evidence

- `ClockUnits` is `u64`, so no fractional remainder exists once time enters mechanics: `core/src/clock.rs`.
- `RealtimeClock::units_for` converts nanoseconds to units and floors via integer division: `core/src/clock.rs`.
- `MechanicsDriver::tick_clock` immediately passes the integer result into stores: `core/src/mechanics.rs`.
- Ability cooldowns only subtract integer elapsed units: `core/src/ability/store.rs`.
- Effect ticking only consumes integer elapsed units, and periodic effects accumulate integer period elapsed units: `core/src/effect/pipeline.rs`.
- Existing realtime tests cover exact conversions such as 250 ms, 500 ms, and 1500 ms. They do not expose fractional drift: `core/tests/mechanics.rs`.

## What Would Muddy Flexweave

Do not make Flexweave own the game loop or wall-clock source.

Flexweave should not schedule frames, sleep threads, decide timestep policy for a specific engine, or depend on Bevy/Tokio/winit. It should provide deterministic adapters that callers can drive.

## Proposed Scope

Add a stateful realtime accumulator and/or fixed-step driver.

Candidate API:

```rust
pub struct RealtimeClockAccumulator {
    clock: RealtimeClock,
    remainder_nanos_times_units: u128,
}

impl RealtimeClockAccumulator {
    pub fn new(units_per_second: ClockUnits) -> Self;
    pub fn advance(&mut self, elapsed: Duration) -> ClockUnits;
    pub fn reset(&mut self);
}
```

Alternative or complement:

```rust
pub struct FixedStepRealtimeDriver {
    step: Duration,
    accumulated: Duration,
}

impl FixedStepRealtimeDriver {
    pub fn advance(&mut self, elapsed: Duration) -> ClockUnits;
}
```

Keep `RealtimeClock::units_for` as a stateless conversion for one-shot use.

## Design Constraints

- Accumulated conversion must be deterministic for identical duration sequences.
- Accumulator should saturate safely on extreme durations.
- Existing `Clock` trait behavior should remain compatible.
- Direct integer ticking should remain the fastest path for turn-based or fixed-step simulations.

## Acceptance Criteria

- Tests show repeated sub-unit realtime deltas eventually advance mechanics.
- Tests show repeated fractional deltas produce the same total units as equivalent aggregate time, up to documented integer precision.
- Cooldown tests cover repeated small realtime deltas.
- Effect duration tests cover repeated small realtime deltas.
- Periodic effect tests cover repeated small realtime deltas.
- Existing `RealtimeClock::units_for` behavior remains available and documented as stateless/flooring.

