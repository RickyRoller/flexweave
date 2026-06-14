# Context Index

Flexweave uses a small set of product contexts. Read this file first, then read
only the context files that match the area you are touching.

## Contexts

- [Flexweave Core](./core/CONTEXT.md) covers domain-agnostic mechanics
  primitives such as object identity, attached data, attributes, effects,
  abilities, tags, queries, registries, signals, and primitive errors.
- [Flexweave Studio](./studio/CONTEXT.md) covers Studio catalog contracts,
  Studio project config, generated mechanics definitions, runtime hooks, local
  host apps, package update flow, and reusable app shell boundaries.

## Relationships

- Flexweave Core is usable without Flexweave Studio.
- Flexweave Studio builds on Core concepts as an optional authoring and
  build-time system.
- Consumer projects consume versioned Flexweave packages and provide their own
  content, runtime bindings, runtime hooks, and local host app entry point.

## ADRs

- `docs/adr/` contains product-wide decisions.
- Context-local ADRs may be added near the affected surface when a decision is
  difficult to reverse and needs permanent rationale.
