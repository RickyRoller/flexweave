# Flexweave

Flexweave is a standalone product workspace for reusable mechanics primitives
and the Studio authoring surface.

## Product Surfaces

| Surface                  | Path         | Purpose                                                                                                             |
| ------------------------ | ------------ | ------------------------------------------------------------------------------------------------------------------- |
| Flexweave Core           | `core`       | Rust crate for deterministic mechanics primitives.                                                                  |
| Flexweave Studio package | `studio`     | TypeScript package for catalog contracts, validation, migrations, generated output checks, and authoring workflows. |
| Flexweave Studio app     | `studio/app` | Reusable application shell imported by project-local Studio host apps.                                              |

## Local Host App Model

Consumer projects run Studio through a small local host app backed by versioned
Flexweave packages. The consumer project owns its Studio project config,
catalog content, generated output directories, runtime hooks, local host app
entry point, adapter, branding, and deployment.

The shared Flexweave packages own the reusable workflows and app shell. Package
updates use this flow:

```bash
bun update @flexweave/studio @flexweave/studio-app
bun run flexweave-studio migrate
bun run flexweave-studio verify
```

## Commands

| Command          | Purpose                                                              |
| ---------------- | -------------------------------------------------------------------- |
| `bun run build`  | Build Core and available Studio surfaces.                            |
| `bun run check`  | Run read-only format, structure, term-scan, Core, and Studio checks. |
| `bun run fix`    | Format supported files.                                              |
| `bun run test`   | Run Core and Studio tests.                                           |
| `bun run verify` | Run the full workspace verification gate.                            |

## Documentation

Documentation uses Diataxis forms so each page has one job:

- Tutorials teach a complete learning path.
- How-to guides solve an operational task.
- Reference pages describe stable contracts.
- Explanation pages capture concepts and rationale.

Start with:

- [Documentation Forms](./docs/reference/documentation-forms.md)
- [Repository Layout](./docs/reference/repository-layout.md)
- [Using Core](./docs/how-to/use-core.md)
- [Using Studio](./docs/how-to/use-studio.md)
- [Updating Studio Packages](./docs/how-to/update-studio-packages.md)
- [Product Boundaries](./docs/explanation/product-boundaries.md)
