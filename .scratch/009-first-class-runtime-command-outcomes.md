# Make runtime command outcomes first-class and phase-aware

## Validation Verdict

Valid, with scope.

Flexweave already has first-class definition errors and lifecycle facts. The issue is narrower: several state-changing runtime command interfaces still collapse meaningful outcomes into `bool`, `Option`, `String`, or phase-less `Hook(E)`.

This strengthens Flexweave if framed as typed primitive outcomes for state-changing runtime operations. It would muddy the crate if it tried to encode caller-domain reasons such as not enough mana, target immune, or movement blocked.

## Problem

Some runtime command return values are ambiguous:

- `Result<Option<_>, _>` where `None` can mean different lifecycle outcomes.
- `Result<bool, _>` where `false` carries command semantics such as already committed.
- `Option` for cancellation/end paths where missing activation is not explicit.
- `Hook(E)` for failures from different activation phases.
- Effect rejection reason as plain `String`.

Callers can inspect emitted events to recover detail, but command return values should also communicate the primitive outcome directly.

## Evidence

- Core docs say Flexweave owns explicit primitive errors.
- `CoreError`, `ObjectStore::create_with_id`, and query `require_*` helpers already expose explicit primitive errors.
- `AbilityEndResult` is `Result<Option<ActiveAbility>, AbilityActivationError<_>>`.
- `commit_activation_with_events` returns `Result<bool, _>` where `false` means already committed.
- `end_activation_with_events` returns `Ok(None)` for missing activation.
- `cancel_activation` returns `Option`.
- Hook failures from can-activate, cooldown calculation, commit, executor, and end collapse into `AbilityActivationError::Hook(E)`.
- `EffectPipeline::apply_with_events` returns `Result<Option<ActiveEffectId>, EffectDefinitionError>`.
- Accepted instant execution and rejected application can both return `Ok(None)`, requiring events to distinguish them.
- Effect rejection reason is a plain `String`.
- `AttributeMutationResult` already has a stronger shape with `Unchanged`, `Committed`, and `Rejected`.

## What Would Muddy Flexweave

Do not replace simple read/predicate APIs with noisy errors:

- `exists`.
- `has`.
- `get`.
- `is_ready`.
- `contains`.

Do not encode caller-domain rejection reasons into core enums. Core should expose primitive phase/outcome structure and carry caller-provided reason payloads where needed.

## Proposed Scope

Add typed outcome enums for state-changing commands.

Candidate direction:

```rust
pub enum AbilityCommitOutcome {
    Committed { cooldown_units: Option<CooldownUnits> },
    AlreadyCommitted,
}

pub enum AbilityEndOutcome<Tags, Cost, Payload> {
    Ended(ActiveAbility<Tags, Cost, Payload>),
    MissingActivation,
}

pub enum AbilityHookPhase {
    CanActivate,
    CooldownUnits,
    Commit,
    ExecuteInstant,
    End,
}
```

For effects:

```rust
pub enum EffectApplyOutcome {
    Rejected,
    ExecutedInstant,
    ActiveCreated(ActiveEffectId),
}
```

Consider parameterizing effect rejection reason:

```rust
EffectApplicationDecision<Reason>
```

or introduce a caller-owned reason payload while preserving string convenience APIs.

## Design Constraints

- Existing lifecycle events remain available and deterministic.
- Existing convenience APIs can remain as wrappers if useful.
- Outcome enums should not encode product-specific gameplay reasons.
- Hook phase should identify where caller code failed without interpreting why.

## Acceptance Criteria

- Accepted instant effect, rejected effect, and active effect creation are distinguishable from returned outcomes.
- Ability already-committed, missing activation, canceled activation, ended activation, cooldown rejection, and hook phase failure are distinguishable without inspecting emitted events.
- Existing lifecycle facts still emit in deterministic order.
- Tests cover every new outcome variant.
- Docs explain which APIs are convenience wrappers and which return full command outcomes.
