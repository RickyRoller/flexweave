<p align="center">
  <img src="./assets/flexweave.svg" alt="Flexweave logo" width="120" />
</p>

# Flexweave

Flexweave is a Rust workspace for reusable mechanics primitives.

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
| `bun run fix`      | Format supported files.                                    |
| `bun run test`     | Run Rust crate tests.                                      |
| `bun run verify`   | Run the full workspace verification gate.                  |

## Documentation

The hosted documentation is organized as a book-style guide with a separate
top-level API Reference:

Start with:

- [What Flexweave Is](./docs/content/docs/getting-started/what-is-flexweave.mdx)
- [Create Combatants](./docs/content/docs/rpg-combat/01-create-combatants.mdx)
- [Core Concepts](./docs/content/docs/core-concepts/objects-and-attached-data.mdx)
- [API Reference](./docs/content/docs/api-reference/index.mdx)
