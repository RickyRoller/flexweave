# Flexweave

Flexweave is the Rust crate for deterministic mechanics primitives. The
crate package name is `flexweave`.

Use Flexweave when a caller needs a domain-neutral foundation for objects, attached
data, attributes, tags, abilities, effects, signals, lifecycle clocks, and
explicit primitive errors.

## Primitive Model

An Object is a stable mechanics handle. Flexweave allocates Object ids in
deterministic creation order, and Object stores preserve deterministic
iteration order.

Attached data adds caller-owned meaning to an Object id. A Data store holds one
attached value type and reports primitive errors for invalid Object ids or
missing required data.

Attributes expose signed numeric channels on Objects. Derived attributes are
calculated from caller-owned state. Attribute changes report previous and
current values without assigning product meaning to either value.

Tags attach deterministic labels to Objects and support repeatable tag queries.
Queries preserve Object iteration order so identical inputs produce identical
selection order.

Abilities describe activation lifecycle, cooldown units, commit timing, grants,
and cancellation policy. Caller-owned hooks decide whether an activation is
accepted and what payload is committed. Use `grant_checked` and
`begin_activation_for_owner_with_events` in common runtime flows so ability
owners are validated against live objects and expected owners before hooks run.

Effects describe application, execution, active lifetime, advancement,
removal, and expiration. Active effect instances carry runtime effect state for
a finite or indefinite lifetime. Use `apply_checked_with_events` with an
explicit `EffectSourcePolicy` when an `ObjectStore` is available; the raw
`apply_with_events` path is reserved for callers that intentionally manage
object-reference validity themselves.

Signals and event channels record lifecycle facts that callers can project into
their own runtime model. Retention policies make the exported facts explicit.

Clock units are opaque `u64` mechanics units. Callers map their own clocks into
those units through fixed-step or real-time adapters. `RealtimeClock` is a
stateless flooring conversion for one-shot durations; use
`RealtimeClockAccumulator` when repeated frame deltas need to preserve
fractional clock units across ticks.

## Determinism

Flexweave avoids unordered public iteration where ordering is part of the contract.
Object ids, Data store scans, tag queries, and mechanics-store queries are
designed to produce repeatable results from identical inputs.

## Commands

```bash
cargo build -p flexweave
cargo test -p flexweave
cargo clippy -p flexweave --all-targets -- -D warnings
cargo doc -p flexweave --no-deps
```

## Boundary

Flexweave owns object identity, attached data, attributes, derived attributes, tags,
queries, abilities, effects, registries, signals, caller-defined clock units,
deterministic mechanics stores, and primitive errors.

Flexweave does not own authored content storage, generated output paths, design UI,
or caller runtime bindings.
