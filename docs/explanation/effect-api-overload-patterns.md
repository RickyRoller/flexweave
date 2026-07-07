# Effect Operation Builder Pattern

Flexweave used to encode independent runtime choices into method suffixes:
direct versus registered definitions, checked versus unchecked references,
initialized versus non-initialized effects, executor-backed callbacks, and
owned versus borrowed lifecycle events. Adding author callbacks made that shape
multiply too quickly.

The public API now uses operation builders plus executors instead of overload
families.

## What Changed

- `EffectPipeline` no longer exposes public `apply*` or `tick*` method families.
  Use `EffectApply` and `EffectTick`.
- `AbilityStore` no longer exposes public `begin_activation*`,
  `begin_registered_activation*`, or `commit_activation*` method families. Use
  `AbilityActivation` and `AbilityCommit`.
- Effect application failures are consolidated under `EffectApplyError`.
- Callback behavior and lifecycle event sinks live in executors:
  `NoEffectExecutor`, `EffectActionExecutor`, `NoAbilityActivationExecutor`,
  `AbilityGateExecutor`, `NoAbilityCommitExecutor`, and
  `AbilityCommitActionExecutor`.
- Owned event emission is an explicit adapter over borrowed lifecycle facts
  through `with_owned_events`; borrowed event emission uses
  `with_borrowed_events`.

The store and pipeline still own deterministic state transitions. Operation
builders own command composition. Executors own caller code and lifecycle event
delivery.

## Rust Pattern Rationale

The Rust API Guidelines recommend builders when construction has many inputs,
optional configuration, or several flavors that would otherwise become many
constructors or functions. They also call out builders as especially appropriate
when the terminal operation has side effects. Flexweave operations fit that
shape because applying an effect, ticking effects, activating an ability, and
committing an ability all mutate store state and may run caller code.
Source: [Rust API Guidelines, C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder).

The Rust Design Patterns builder note makes the same tradeoff explicit:
builders prevent constructor proliferation and support both one-line and
incremental configuration, at the cost of a slightly larger API concept.
Source: [Rust Design Patterns, Builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html).

Flexweave uses consuming command builders because operation configuration is
normally one-shot and includes owned runtime input. That keeps call sites compact
and keeps the terminal `run*` method responsible for validation and side
effects.

## Effect Examples

Apply a direct definition with the default no-op executor:

```rust
let outcome = EffectApply::definition(&definition, input).run(&mut effects)?;
```

Apply a registered definition and retain owned lifecycle facts:

```rust
let mut context = ();
let mut executor =
    NoEffectExecutor::new().with_owned_events(|event| lifecycle_events.push(event));

let outcome = EffectApply::registered(&effect_definitions, "enemy/wasp/poison", input)
    .run_with_executor(&mut effects, &mut context, &mut executor)?;
```

Apply an initialized, checked effect with caller-owned execution:

```rust
let mut executor =
    EffectActionExecutor::new(&mut action).with_owned_events(|event| events.push(event));

let outcome = EffectApply::definition(&definition, input)
    .checked(&objects, EffectSourcePolicy::RequireLiveSource)
    .initialized(&mut initializer)
    .run_with_executor(&mut effects, &mut runtime, &mut executor)?;
```

Advance active effects with caller-owned periodic execution:

```rust
let mut executor =
    EffectActionExecutor::new(&mut action).with_borrowed_events(|event| publish(event));

EffectTick::new(elapsed_units)
    .run_with_executor(&mut effects, &mut runtime, &mut executor)?;
```

## Ability Examples

Activate a granted ability:

```rust
let activation_id = AbilityActivation::new(ability_id).run(&mut abilities)?;
```

Activate a registered ability for an expected owner while running a caller-owned
gate and retaining lifecycle facts:

```rust
let mut executor =
    AbilityGateExecutor::new(&mut gate).with_owned_events(|event| events.push(event));

let activation_id = AbilityActivation::registered(&ability_definitions, ability_id)
    .for_owner(owner_id)
    .run_with_executor(&mut abilities, &runtime, &mut executor)?;
```

Commit an activation with caller-owned side effects:

```rust
let mut executor =
    AbilityCommitActionExecutor::new(&mut commit_action)
        .with_owned_events(|event| events.push(event));

let outcome = AbilityCommit::new(activation_id)
    .run_with_executor(&mut abilities, &mut runtime, &mut executor)?;
```

## Design Rules Going Forward

- Add new runtime axes to operation builders, not to `EffectPipeline` or
  `AbilityStore` method suffixes.
- Keep callback contracts as named traits with closure blanket impls when the
  contract is meaningful, such as `EffectExecutionAction`,
  `AbilityActivationGate`, and `AbilityCommitAction`.
- Keep borrowed lifecycle views as the primitive event shape. Use explicit sink
  adapters for owned events, borrowed events, and discarded events.
- Keep no-op executors for the default path so the implementation has one
  generic execution route.
- Use specific domain types for meaningful choices, such as
  `EffectSourcePolicy`, instead of boolean flags.
