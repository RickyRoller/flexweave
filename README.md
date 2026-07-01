# Flexweave

Flexweave is a Rust workspace for reusable mechanics primitives.

## Product Surfaces

| Surface              | Path   | Purpose                                            |
| -------------------- | ------ | -------------------------------------------------- |
| Flexweave Rust crate | `core` | Rust crate for deterministic mechanics primitives. |

## Commands

| Command          | Purpose                                   |
| ---------------- | ----------------------------------------- |
| `bun run build`  | Build the Rust crate.                     |
| `bun run check`  | Run read-only formatting and Rust checks. |
| `bun run fix`    | Format supported files.                   |
| `bun run test`   | Run Rust crate tests.                     |
| `bun run verify` | Run the full workspace verification gate. |

## Documentation

Documentation uses Diataxis forms so each page has one job:

- Tutorials teach a complete learning path.
- How-to guides solve an operational task.
- Reference pages describe stable contracts.
- Explanation pages capture concepts and rationale.

Start with:

- [Documentation Forms](./docs/reference/documentation-forms.md)
- [Repository Layout](./docs/reference/repository-layout.md)
- [Using Flexweave](./docs/how-to/use-flexweave.md)
- [Product Boundaries](./docs/explanation/product-boundaries.md)
