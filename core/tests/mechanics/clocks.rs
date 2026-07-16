use super::support::*;

#[test]
fn turn_based_clock_advances_effect_lifetimes_in_turns() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Shield,
    }

    let turn_clock = FixedStepClock::new(1);
    let source = ObjectId::new(1);
    let target = ObjectId::new(2);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut effects,
        &FlexEffectDefinition {
            key: "turn_shield".to_owned(),
            kind: EffectKind::Periodic,
            duration: Some(EffectClockPolicy::from_clock(&turn_clock, 3)),
            period: Some(EffectClockPolicy::from_clock(&turn_clock, 1)),
            routing: EffectRouting::default(),
            payload_schema: (),
        },
        EffectApplicationInput {
            source_id: Some(source),
            target_id: target,
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Shield,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();

    let events = MechanicsTick::from_clock(&turn_clock, 1).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );

    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(pulse),
    ] = events.as_slice()
    else {
        panic!("one turn should advance the effect and execute one period");
    };
    assert_eq!(advanced.elapsed_units, 1);
    assert_eq!(advanced.effect.remaining_units, Some(2));
    assert_eq!(pulse.elapsed_units, Some(1));

    let events = MechanicsTick::from_clock(&turn_clock, 2).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );

    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("two more turns should execute remaining periods and expire");
    };
    assert_eq!(expiring_advance.elapsed_units, 2);
    assert_eq!(expired.remaining_units, Some(0));
    assert_eq!(effects.count(), 0);
}

#[test]
fn realtime_clock_lets_callers_choose_duration_to_unit_scale() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Pulse,
    }

    let realtime = RealtimeClock::new(1000);
    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut effects,
        &FlexEffectDefinition {
            key: "realtime_pulse".to_owned(),
            kind: EffectKind::Periodic,
            duration: Some(EffectClockPolicy::from_clock(
                &realtime,
                Duration::from_secs(2),
            )),
            period: Some(EffectClockPolicy::from_clock(
                &realtime,
                Duration::from_millis(500),
            )),
            routing: EffectRouting::default(),
            payload_schema: (),
        },
        EffectApplicationInput {
            source_id: None,
            target_id: ObjectId::new(20),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Pulse,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();

    let events = MechanicsTick::from_clock(&realtime, Duration::from_millis(250)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );

    let [EffectLifecycleEvent::Advanced(advanced)] = events.as_slice() else {
        panic!("quarter-second tick should only advance when period is 500 ms");
    };
    assert_eq!(advanced.elapsed_units, 250);
    assert_eq!(advanced.effect.remaining_units, Some(1750));

    let events = MechanicsTick::from_clock(&realtime, Duration::from_millis(250)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(pulse),
    ] = events.as_slice()
    else {
        panic!("second quarter-second tick should complete one period");
    };
    assert_eq!(advanced.elapsed_units, 250);
    assert_eq!(pulse.elapsed_units, Some(500));

    let events = MechanicsTick::from_clock(&realtime, Duration::from_millis(1500)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    let [
        EffectLifecycleEvent::Advanced(expiring_advance),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::PeriodicExecuted(_),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("remaining realtime duration should emit final periods before expiration");
    };
    assert_eq!(expiring_advance.elapsed_units, 1500);
    assert_eq!(expired.remaining_units, Some(0));
}

#[test]
fn realtime_accumulator_matches_aggregate_elapsed_time_for_fractional_frames() {
    let realtime = RealtimeClock::new(1000);
    let frame = Duration::from_nanos(16_666_667);
    let mut accumulator = RealtimeClockAccumulator::from_clock(realtime);

    let accumulated_units: ClockUnits = (0..60).map(|_| accumulator.advance(frame)).sum();

    assert_eq!(
        accumulated_units,
        realtime.units_for(Duration::from_nanos(1_000_000_020))
    );
    assert_eq!(
        (0..60)
            .map(|_| realtime.units_for(frame))
            .sum::<ClockUnits>(),
        960
    );
}

#[test]
fn realtime_accumulator_expires_effect_duration_from_repeated_sub_unit_deltas() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Brief,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut effects,
        &duration_effect_definition("brief", 1),
        EffectApplicationInput {
            source_id: None,
            target_id: ObjectId::new(1),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Brief,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();

    let mut accumulator = RealtimeClockAccumulator::new(60);
    let frame = Duration::from_millis(16);

    let events = MechanicsTick::new(accumulator.advance(frame)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    assert!(events.is_empty());
    assert_eq!(effects.count(), 1);

    let events = MechanicsTick::new(accumulator.advance(frame)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::Expired(expired),
    ] = events.as_slice()
    else {
        panic!("second sub-unit frame should advance and expire the effect");
    };
    assert_eq!(advanced.elapsed_units, 1);
    assert_eq!(advanced.effect.remaining_units, Some(0));
    assert_eq!(expired.remaining_units, Some(0));
    assert_eq!(effects.count(), 0);
}

#[test]
fn realtime_accumulator_executes_periodic_effects_from_repeated_sub_unit_deltas() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Payload {
        Pulse,
    }

    let mut effects = EffectPipeline::<TagSet<TestAtom>, Payload>::new();
    apply_effect(
        &mut effects,
        &FlexEffectDefinition {
            key: "pulse".to_owned(),
            kind: EffectKind::Periodic,
            duration: Some(EffectClockPolicy::new(3)),
            period: Some(EffectClockPolicy::new(1)),
            routing: EffectRouting::default(),
            payload_schema: (),
        },
        EffectApplicationInput {
            source_id: None,
            target_id: ObjectId::new(1),
            tags: TagSet::new([Tag::new([TestAtom::Category])]),
            payload: Payload::Pulse,
            decision: EffectApplicationDecision::Accept,
        },
    )
    .unwrap();

    let mut accumulator = RealtimeClockAccumulator::new(60);
    let frame = Duration::from_millis(16);

    let events = MechanicsTick::new(accumulator.advance(frame)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    assert!(events.is_empty());

    let events = MechanicsTick::new(accumulator.advance(frame)).run(
        MechanicsDriver::<EffectLifecycleEvent<TagSet<TestAtom>, Payload>>::new()
            .with_store(&mut effects),
    );
    let [
        EffectLifecycleEvent::Advanced(advanced),
        EffectLifecycleEvent::PeriodicExecuted(pulse),
    ] = events.as_slice()
    else {
        panic!("second sub-unit frame should complete one periodic interval");
    };
    assert_eq!(advanced.elapsed_units, 1);
    assert_eq!(advanced.effect.remaining_units, Some(2));
    assert_eq!(pulse.elapsed_units, Some(1));
    assert_eq!(effects.count(), 1);
}
