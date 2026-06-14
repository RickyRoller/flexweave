# Domain Docs

Flexweave domain docs are scoped by product surface.

## Startup Order

1. Read `CONTEXT-MAP.md`.
2. Read `core/CONTEXT.md` when changing Core.
3. Read `studio/CONTEXT.md` when changing Studio package, app, runtime
   contract, or local host app guidance.
4. Read the closest ADR when changing a boundary or update flow.

## Vocabulary Rules

Use product terms directly:

- Flexweave
- Flexweave Core
- Flexweave Studio
- Studio catalog
- Studio project config
- Generated mechanics definitions
- Runtime hooks
- Local host app
- Consumer project
- Consumer runtime

Avoid importing consumer-specific vocabulary into reusable Core or Studio
contracts.
