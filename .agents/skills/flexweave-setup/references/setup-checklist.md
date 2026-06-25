# Setup Checklist

Use this checklist to keep Flexweave setup concrete and repeatable.

## Discovery

- Locate the repo root and existing tooling: Cargo workspace files, runtime
  crate manifests, agent startup docs, and build/test commands.
- Locate the owning Rust runtime crate for Flexweave. If there is no Rust
  runtime crate, stop and ask where Flexweave should be integrated.
- Inspect current runtime mechanics state before editing: object identity,
  attributes, abilities/cooldowns, effects/lifecycle, tags, ticking, and events.
  Record whether each is Flexweave-backed, manual, or not adopted yet.
- Identify existing runtime modules, state containers, scheduling/ticking code,
  event flow, and test conventions.
- Check whether the runtime crate already has a `flexweave` dependency and how
  it is imported.

## Integration Mode

- Flexweave availability: required. Add the `flexweave` crate to the owning runtime
  crate and verify with existing compile/check commands.
- Flexweave adoption map: required. Document which Flexweave primitives the runtime
  already uses, which manual systems must be preserved, and which primitives are
  not adopted yet.

## Flexweave Adoption Map

Record the current status in `FLEXWEAVE.md` instead of forcing a migration during
setup:

- Object identity: `Flexweave-backed`, `manual`, or `not adopted yet`.
- Attributes: `Flexweave-backed`, `manual`, or `not adopted yet`.
- Abilities/cooldowns: `Flexweave-backed`, `manual`, or `not adopted yet`.
- Effects/lifecycle: `Flexweave-backed`, `manual`, or `not adopted yet`.
- Tags/queries: `Flexweave-backed`, `manual`, or `not adopted yet`.
- Mechanics ticking/events: `Flexweave-backed`, `manual`, or `not adopted yet`.

If an existing manual system owns one of these responsibilities, preserve it and
document the gap. Do not replace manual systems during setup unless the user
explicitly asks for that migration.

## FLEXWEAVE.md Content

- Runtime crate/package/module that owns Flexweave.
- Flexweave dependency source and version/path if discoverable.
- Existing runtime paths for object identity, attributes, abilities, effects,
  tags, ticking, events, and tests.
- Commands for runtime compile/check and focused mechanics tests.
- Manual systems that should stay manual until a user asks for migration.
- Open decisions that future mechanic work should resolve.

## Validation Order

1. Flexweave dependency install succeeds in the owning runtime crate.
2. The runtime crate's existing compile/check command succeeds.
3. `FLEXWEAVE.md` records the adoption map and verification commands.
4. The repo's agent startup file points future agents at `FLEXWEAVE.md` when
   such a file exists.
