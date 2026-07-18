# Flexweave

**Deterministic gameplay mechanics primitives for Rust.**

Flexweave is a toolkit for building gameplay mechanics: the rules that decide
what game objects can do, how their attributes change, when abilities commit,
how effects unfold over time, and which outcomes a runtime can react to. It
provides reusable primitives for objects, attached data, attributes, tags,
queries, abilities, effects, clocks, lifecycle facts, and signals without
requiring a particular engine or game architecture.

## Gameplay mechanics as a distinct layer

A game's mechanics are related to its engine, content, and presentation, but
they are not the same thing. A mechanic answers questions such as whether an
ability may start, what happens when it commits, how long an effect remains
active, or which objects match a targeting rule. The surrounding runtime
decides when to ask those questions, where authored definitions come from, and
how the answers reach animation, networking, persistence, or UI.

Those concerns often grow together in a game codebase. An ability becomes tied
to an engine scheduler, a status effect publishes directly to a product event
bus, or targeting depends on the layout of an ECS world. That can be a sensible
local choice, but it makes the rules harder to test, simulate, reuse, or run in
a different environment.

Flexweave treats gameplay mechanics as a layer worth modeling on its own. It
owns deterministic mechanics state and explicit state transitions. The
consumer owns orchestration and product meaning. The result is not a complete
gameplay system; it is a stable foundation from which one can be built.

## The boundary between mechanics and meaning

Flexweave deliberately uses domain-neutral concepts. An `Object` might be a
combatant, item, card, encounter, or something with no visual representation.
An ability might mean an attack, spell, interaction, or command. An effect
might represent damage, healing, a temporary modifier, or a rule that exists
only to enforce a cooldown.

This separation is useful because the lifecycle shape is often reusable even
when the meaning is not. Different games can share the ideas of activation,
commitment, cancellation, periodic execution, expiration, and deterministic
selection without sharing content schemas or genre terminology.

The boundary also keeps policy visible. Flexweave does not assume that an
ability cost is mana, that a cooldown is stored on an ability, or that an
effect must modify an attribute. Callers compose costs and cooldowns from
attributes, tags, effects, gates, and commit actions. Flexweave supplies the
mechanics vocabulary and enforces its invariants; the game decides what those
compositions mean.

## Why determinism belongs in the mechanics layer

Gameplay rules are easier to trust when identical inputs produce identical
results. Flexweave therefore preserves object allocation order, public
iteration and query order, lifecycle fact order, and primitive outcomes where
ordering is part of the contract.

That property is valuable beyond lockstep simulation. It makes focused tests
repeatable, allows servers and tools to share rule behavior, and gives replays,
debuggers, content validation, and AI-driven simulations a stable mechanics
surface.

Flexweave does not claim to make an entire application deterministic. A runtime
can still introduce nondeterminism through scheduling, concurrency, external
state, or unordered collections. The crate's guarantee is narrower: its
primitives behave predictably when the caller gives them the same ordered
inputs.

## Facts rather than an event architecture

Mechanics produce facts: an attribute changed, an activation was rejected, an
ability committed, an effect executed, or an active effect expired. Flexweave
exposes those lifecycle facts without deciding what the application must do
with them.

This distinction prevents the mechanics layer from quietly becoming the
application's event architecture. A lifecycle fact is not automatically a UI
event, network message, audit record, or engine signal. Callers may publish it
to a typed `EventChannel`, map it into their own event model, retain it for
diagnostics, or discard it after a synchronous reaction.

Signals follow the same principle. They are projected, exportable facts derived
from lifecycle activity, not a second hidden orchestration system. Projection
and publication remain explicit so the runtime boundary stays observable.

## A toolkit rather than an engine

Flexweave does not own a game loop, ECS world, async runtime, content loader,
serialization format, network protocol, or rendering model. It also does not
automatically discover definitions or route lifecycle facts. Those omissions
are design choices: taking control of those systems would make the crate easier
to adopt in one environment and harder to reuse everywhere else.

Instead, Flexweave provides synchronous operations, explicit outcomes, checked
object-reference paths, caller-constructed definition bundles, and opaque clock
units. An engine adapter can drive those primitives from frames, fixed ticks,
or turns. A server can drive the same primitives from commands. A test can
exercise them without either environment.

Flexweave is a good fit when gameplay rules need to remain portable across
runtimes, testable without a running engine, or understandable independently
of product infrastructure. It is not intended to replace a full game engine or
provide an authored gameplay framework out of the box.

## Install

```sh
cargo add flexweave
```

See [flexweave.dev](https://flexweave.dev) for tutorials, runtime patterns,
conceptual guides, and API documentation.

## License

Flexweave is licensed under either of the following, at your option:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))
- MIT License ([LICENSE-MIT](./LICENSE-MIT))
