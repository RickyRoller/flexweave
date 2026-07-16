# Context Index

Flexweave uses a small set of product contexts. Read this file first, then read
only the context files that match the area you are touching.

## Contexts

- [Flexweave](./core/CONTEXT.md) covers domain-agnostic mechanics
  primitives such as object identity, attached data, attributes, effects,
  abilities, tags, queries, registries, signals, and primitive errors.

## Relationships

- Flexweave provides primitive mechanics building blocks while caller code owns
  application meaning.
- Consumer projects consume the Flexweave crate and provide their own content,
  runtime bindings, and deployment.

## Design Records

- `core/docs/adr/` contains durable core decisions.
- `core/docs/models/` contains implementation-facing system models.
