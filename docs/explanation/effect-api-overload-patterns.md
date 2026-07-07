# Effect API Overload Patterns

This note looks at the effect pipeline overload growth in the
`effect-execution-actions` worktree and summarizes Rust API patterns that can
control it without losing the synchronous author-callback boundary.

## Current Shape After Cleanup

The effect pipeline previously exposed separate methods for several independent
axes. The action-support cleanup now keeps the existing event conveniences but
does not add another action/event cross-product:

- Event retention: no event callback, owned lifecycle events, or borrowed event
  views. The borrowed path remains the implementation primitive, with owned
  methods converting through `to_owned_event`.
  Source: [`core/src/effect/pipeline.rs`](../../core/src/effect/pipeline.rs),
  [`core/src/effect/events.rs`](../../core/src/effect/events.rs).
- Execution behavior: common paths use `NoEffectExecutor`; action-backed paths
  configure `EffectActionExecutor` with a caller-owned `EffectExecutionAction`
  implemented by a named type or closure. Executors can also carry lifecycle
  sinks for owned, borrowed, or discarded facts.
  Source: [`core/src/effect/application.rs`](../../core/src/effect/application.rs).
- Application preparation: direct definition, initialized application, registered
  definition lookup, and checked object-reference validation. These still have
  convenience methods because they predate the executor cleanup.
  Source: [`core/src/effect/pipeline.rs`](../../core/src/effect/pipeline.rs).
- Periodic execution on ticking uses `tick_with_executor` for action-backed
  execution and keeps `tick_with_events` / `tick_with_borrowed_events` as
  convenience wrappers.
  Source: [`core/src/effect/pipeline.rs`](../../core/src/effect/pipeline.rs).
- Operation builders now provide the preferred composition point for the
  application axes. `EffectApply` carries direct versus registered definitions,
  optional checked reference validation, optional initialization, and executor
  selection. `EffectTick` carries tick execution through the same executor
  shape.
  Source: [`core/src/effect/operation.rs`](../../core/src/effect/operation.rs).

The ability system already uses the better part of this design: a small named
trait, a no-op implementation, and a blanket closure implementation for the
caller-owned behavior. ADR 0007 explicitly records that ability gates and commit
actions use named public traits plus closure blanket impls.
Source: [`core/src/ability/hooks.rs`](../../core/src/ability/hooks.rs),
[`core/docs/adr/0007-split-ability-gate-from-commit-action.md`](../../core/docs/adr/0007-split-ability-gate-from-commit-action.md).

The same operation-builder shape now exists for abilities. `AbilityActivation`
composes direct versus registered activation, optional owner checking, gates,
and lifecycle sinks through an activation executor. `AbilityCommit` composes
commit actions and lifecycle sinks through a commit executor.
Source: [`core/src/ability/operation.rs`](../../core/src/ability/operation.rs),
[`core/src/ability/hooks.rs`](../../core/src/ability/hooks.rs).

The weak point was not the callback trait. It was encoding every independent
choice into the method name, so each new axis multiplied the public surface.

## Rust Patterns

### Use One Borrowed Event Primitive

Rust's API Guidelines say callers should decide where data is copied, and
functions should not clone when a borrow is enough.
Source: [Rust API Guidelines, C-CALLER-CONTROL](https://rust-lang.github.io/api-guidelines/flexibility.html#caller-decides-where-to-copy-and-place-data-c-caller-control).

That supports making borrowed lifecycle views the canonical callback shape:

```rust
for<'event> FnMut(EffectLifecycleEventView<'event, Tags, Payload>)
```

Higher-ranked trait bounds are the right language feature when a callback must
accept a value borrowed for any short callback lifetime.
Source: [Rust Reference, higher-ranked trait bounds](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds).

Owned events should be an explicit adapter over the borrowed primitive, not a
separate method suffix everywhere. This keeps clone requirements local to the
owned adapter instead of spreading `Tags: Clone, Payload: Clone` across multiple
operation variants.

### Keep Named Behavior Traits

The current `EffectExecutionAction` mirrors `AbilityCommitAction` well.
`FnMut` is the standard callback bound when a function-like value may be called
repeatedly and mutate captured state.
Source: [std::ops::FnMut](https://doc.rust-lang.org/std/ops/trait.FnMut.html).

The API Guidelines also prefer generic bounds when they express the minimum
assumptions needed by the function; this preserves static dispatch and lets
closures remain lightweight.
Source: [Rust API Guidelines, C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generics-c-generic).

So the action trait is not the thing to remove. It should remain the named
contract for author code, with closure support as an ergonomic implementation.

### Move Combinations Into Builders Or Command Objects

The Rust API Guidelines recommend builders when construction involves many
inputs, optional configuration, or a choice between several flavors, because
otherwise APIs drift toward many constructors with many arguments.
Source: [Rust API Guidelines, C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder).

The standard library uses this shape for APIs with multiple independent options.
`std::process::Command` is a process builder with required data in `new`,
configuration methods, and terminal methods such as `spawn`/`output`.
Source: [std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html).
`std::fs::OpenOptions` similarly chains independent file-open options and then
calls `open`.
Source: [std::fs::OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html).

For Flexweave, this points to operation builders or command objects rather than
more method suffixes:

```rust
EffectApply::definition(definition, input)
    .checked(objects, EffectSourcePolicy::RequireLiveSource)
    .initialized(context, initializer)
    .execute_with(context, action)
    .events(effect_events::owned(|event| events.push(event)))
```

and:

```rust
EffectTick::new(elapsed_units)
    .execute_with(context, action)
    .events(effect_events::borrowed(|event| publish(event)))
```

The implemented API names differ slightly from the sketch because executors
carry both behavior and event sinks:

```rust
let mut executor =
    EffectActionExecutor::new(&mut action).with_owned_events(|event| events.push(event));

EffectApply::definition(definition, input)
    .checked(objects, EffectSourcePolicy::RequireLiveSource)
    .initialized(&mut initializer)
    .run_with_executor(&mut pipeline, &mut context, &mut executor)?;
```

For abilities:

```rust
let mut executor =
    AbilityGateExecutor::new(&mut gate).with_owned_events(|event| events.push(event));

let activation_id = AbilityActivation::registered(definitions, ability_id)
    .for_owner(owner_id)
    .run_with_executor(&mut abilities, &runtime, &mut executor)?;
```

The important shape is one terminal execution path per operation family, with
independent configuration represented as builder state.

### Use Custom Types For Meaningful Choices

The API Guidelines prefer deliberate types over ambiguous primitive flags.
Source: [Rust API Guidelines, C-CUSTOM-TYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#arguments-convey-meaning-through-types-not-bool-or-option-c-custom-type).

Flexweave is already doing this with `EffectSourcePolicy`. Keep that style for
new axes: event retention should be an event sink adapter type, and application
source should be a definition source type, not a pile of boolean parameters.

If a builder or command object is public, keep fields private unless it is truly
a passive data record. Private fields preserve invariants and leave room for
future validation.
Source: [Rust API Guidelines, C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private).

### Avoid Conversion Traits For Callback Ownership

`Borrow` is for owned and borrowed representations that behave equivalently for
traits such as `Eq`, `Ord`, and `Hash`; `AsRef` is for cheap reference
conversion.
Source: [std::borrow::Borrow](https://doc.rust-lang.org/std/borrow/trait.Borrow.html),
[Rust API Guidelines, C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv-traits).

Those traits do not model "this callback wants an owned event" versus "this
callback wants a borrowed event view". Use explicit sink adapters such as
`OwnedEffectLifecycleEvents`, `DiscardEffectLifecycleEvents`, or a borrowed-view
`FnMut` instead of trying to infer that choice through conversion traits.

### Keep No-Op Defaults

No-op executors and initializer types are useful because they let the core
implementation share one generic path. `Infallible` is the standard error type
for errors that cannot happen in a generic `Result`.
Source: [std::convert::Infallible](https://doc.rust-lang.org/std/convert/enum.Infallible.html).

For option-like configuration structs, `Default` is the standard way to expose a
useful baseline and override individual options.
Source: [std::default::Default](https://doc.rust-lang.org/std/default/trait.Default.html).

### Use Trait Objects Only As An Escape Hatch

Trait objects can reduce monomorphization and store heterogeneous callbacks, but
they add dynamic dispatch and restrict generic methods.
Source: [Rust API Guidelines, C-OBJECT](https://rust-lang.github.io/api-guidelines/flexibility.html#traits-are-object-safe-if-they-may-be-useful-as-a-trait-object-c-object).

Flexweave core is small, deterministic, and hot-path oriented, and action error
types are currently generic. Static generic traits remain the better default.
Consider trait objects only for a higher-level runtime adapter that wants to
store heterogeneous effect handlers behind one erased error type.

## Options For Flexweave

### Option A: Document Borrowed Methods As Canonical

Keep the current API, but document borrowed event methods as the primitive and
owned event methods as convenience wrappers.

Tradeoff: low migration cost, but it does not stop the next axis from adding
another set of suffixes.

### Option B: Add Executor And Event Sink Adapters

Introduce one executor and event sink concept with explicit adapters:

- `EffectActionExecutor::new(action)` adapts an execution action.
- `NoEffectExecutor::new()` covers no-action paths.
- `with_borrowed_events(f)` calls `f` with `EffectLifecycleEventView`.
- `with_owned_events(f)` converts each view with `to_owned_event`.
- `DiscardEffectLifecycleEvents` ignores events.

Then new APIs accept an executor parameter rather than encoding
action/no-action and owned/borrowed/no events in the method name.

Tradeoff: this removes one major multiplier and keeps clone costs explicit, but
it still leaves direct/checked/registered/initialized as method-name axes unless
paired with a command object.

### Option C: Add Operation Builders

Add `EffectApply` and `EffectTick` command objects that carry the optional axes
and execute through one pipeline entry point per operation family.

Keep small convenience methods only for common default cases:

- `apply`
- `apply_checked`
- `apply_registered`
- `apply_initialized`
- `tick`

Move action, event sink, checked validation, registered lookup, and
initialization composition into the command object.

Tradeoff: this is the strongest fix for API growth, but generic builders that
borrow context, initializers, actions, and event sinks will need careful lifetime
and error-type design. A staged implementation can start with event sink
adapters and add builders after the desired call shape is proven in tests.

## Recommendation

Use Option C as the long-term public shape, with Option B's executor and sink
adapters as the shared implementation mechanism.

Flexweave should treat borrowed lifecycle views as the canonical event stream,
provide explicit executor/event-sink adapters, and stop adding public methods
for every action/no-action and owned-vs-borrowed event combination. New
multi-axis operations should be exposed as command builders, not suffix-heavy
method families.

This fits the existing Flexweave boundary:

- It preserves synchronous author-owned actions.
- It keeps static dispatch and caller-owned error types.
- It keeps clone costs under caller control.
- It aligns effects and abilities around named action traits plus operation
  builders.
- It limits the public surface before effects gain more runtime hooks.

Suggested migration:

1. Add executor and event sink adapters and make new APIs use borrowed views internally.
2. Add `EffectApply`, `EffectTick`, `AbilityActivation`, and `AbilityCommit`
   command objects as additive APIs.
3. Update examples/tests to prefer command objects for action-backed,
   gate-backed, or event-backed flows.
4. Keep existing suffix methods during transition, then deprecate the most
   combinatorial variants once the command APIs are stable.
