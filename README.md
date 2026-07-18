<p align="center">
  <img src="./assets/flexweave.svg" alt="Flexweave logo" width="120" />
</p>

# Flexweave

[Documentation](https://flexweave.dev)

## Install

```sh
cargo add flexweave
```

## What Flexweave Is

Flexweave is a Rust library for building deterministic mechanics without tying
an application to a particular genre, engine, content format, or scheduling
model. It provides the domain-neutral pieces that mechanics systems tend to
share: stable objects, typed attached data, numeric attributes, tags and
queries, ability and effect lifecycles, clocks, lifecycle facts, and signals.

The central design choice is a boundary between mechanics and meaning.
Flexweave owns the shape of mechanics state and its transitions; the consuming
runtime decides what those pieces represent and how they are orchestrated. An
object might be a character, item, or encounter. An ability might represent an
attack, spell, card, or command. A clock unit might be a turn, tick, or slice of
real time. The caller chooses those meanings, loads the active definitions,
applies consequences at lifecycle boundaries, and maps Flexweave's facts into
its own events and adapters.

Determinism is part of that foundation. Given identical primitive inputs,
Flexweave preserves object ids, iteration and query order, lifecycle facts, and
results. That makes the library useful for tests, simulations, replays,
debugging, content validation, and server-side command processing. It does not
attempt to make an entire application deterministic; scheduling, concurrency,
external state, and ordering outside the library remain the runtime's
responsibility.

Flexweave is therefore a mechanics toolkit rather than a game engine or content
system. It does not prescribe a game loop, load authored content, or
automatically route events. Instead, it gives higher-level runtimes predictable
building blocks while leaving product-specific policy at the boundary.

## Product Surfaces

| Surface              | Path   | Purpose                                             |
| -------------------- | ------ | --------------------------------------------------- |
| Flexweave Rust crate | `core` | Rust crate for deterministic mechanics primitives.  |
| Documentation site   | `docs` | Hostable Fumadocs site for Flexweave documentation. |

## Commands

| Command            | Purpose                                                    |
| ------------------ | ---------------------------------------------------------- |
| `bun run build`    | Build the Rust crate and docs site.                        |
| `bun run check`    | Run read-only formatting, Rust checks, and docs typecheck. |
| `bun run docs:dev` | Start the local documentation site.                        |
| `bun run fix`      | Format workspace files.                                    |
| `bun run test`     | Run Rust crate tests.                                      |
| `bun run verify`   | Run the CI verification gate locally.                      |
