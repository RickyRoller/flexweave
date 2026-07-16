# Effects System Models

These models describe the current Flexweave effect primitive as an
implementation resource. D2 sources are the editable artifacts, and peer SVGs
are tracked beside them for review. The diagrams use D2's built-in Dark Mauve
theme (`theme-id: 200`) for consistent contrast.

## Scope

The effects system covers authorable effect definitions, runtime application
inputs, source and target validation, instant execution, active effect storage,
duration advancement, periodic execution, expiration, manual removal,
object-keyed cleanup, lifecycle facts, event publication, and signal projection.

Flexweave owns deterministic primitive state and lifecycle facts. Caller code
owns application decisions, payload meaning, clock semantics, event-channel
publication, signal publication, and runtime effects derived from lifecycle
facts or signals.

## Source Paths

- `core/src/effect.rs`
- `core/src/effect/definition.rs`
- `core/src/effect/application.rs`
- `core/src/effect/operation.rs`
- `core/src/effect/pipeline.rs`
- `core/src/effect/events.rs`
- `core/src/effect/ids.rs`
- `core/src/signal/projection.rs`
- `core/src/object_lifecycle.rs`
- `core/src/lifecycle/kind.rs`
- `core/tests/effects.rs`
- `core/tests/signals.rs`
- `core/tests/object_destruction.rs`
- `core/README.md`
- `docs/content/docs/core-concepts/effects-and-active-instances.mdx`
- `docs/content/docs/api-reference/effect.mdx`

## Model Files

- `data-model.d2` shows the static definition, application input, active
  instance, pipeline, event, object cleanup, and caller boundary relationships.
- `lifecycle.d2` shows checked application, caller rejection, instant execution,
  active creation, ticking, periodic execution, expiration, manual removal, and
  object-keyed cleanup.
- `event-publication.d2` shows borrowed versus owned lifecycle facts, caller
  event-channel publication, and caller-owned signal projection/publication.

Each D2 file has a same-name `.svg` render beside it.

## Implementation Notes

- `EffectDefinition` validates kind, duration, period, and non-empty routing
  metadata before application.
- `EffectDefinitions` validates definitions, rejects duplicate keys, and
  preserves declaration order for registered lookups.
- `EffectPipeline` emits lifecycle facts through callbacks. Routing keys do not
  auto-publish lifecycle events or projected signal facts.
- `EffectApply::checked` validates `source_id` and `target_id` against
  `ObjectStore` before any lifecycle fact is emitted. Unchecked application
  paths copy references as-is.
- `EffectApplicationDecision` is caller-owned: rejection emits a rejection fact
  and stores no active effect.
- Instant effects emit `ApplicationAccepted` and `Executed` without active
  storage. Duration, periodic, and indefinite effects create `EffectInstance`
  state.
- `EffectTick` advances active effects in deterministic application order.
  Periodic executions are capped to the active lifetime, and natural timeout
  emits `Expired` after the final advance and periodic executions.
- Manual removal and object-keyed cleanup emit `Removed`; natural timeout emits
  `Expired`.

## Open Questions

- Should effects get a combined registered-and-checked application helper, or
  should callers continue composing registered lookup and checked application
  manually when both are needed?
- Should effect routing metadata get known-channel validation like signal
  definitions already have?
- Should signal projection remain inside the effects model, or should signals
  get a separate core-system model with effects as one source?
