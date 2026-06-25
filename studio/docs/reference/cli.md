# CLI Reference

The package exposes one bin: `flexweave-studio`.

Every project command supports:

- `--config <path>` to load an explicit Studio project config.
- `--json` for machine-readable output.
- `--quiet` to suppress human success output.

Commands:

- `validate`: load configured data sources and validate the configured Studio
  catalog.
- `describe`: describe record schemas.
- `list`: list records for one kind.
- `show`: show one record.
- `plan`: preview mechanic scaffolding writes.
- `scaffold`: write mechanic records and runtime hook stubs transactionally.
- `scaffold host-app`: create a local host app package manifest, entry point,
  project adapter, TypeScript config, and scaffold metadata.
- `codegen`: refresh generated mechanics definitions.
- `codegen --check`: fail when generated mechanics definitions are missing,
  stale, or unexpectedly present.
- `verify`: run config, extension, source, mapper, validation, generated target,
  runtime hook, host app, and project command checks. When `--fast` is passed,
  project commands are limited to commands marked `fast`.
- `migrate`: run package and extension-owned migrations after updates. It reads
  local host app scaffold metadata when present and reports changed files,
  skipped work, unsupported versions, and manual follow-ups.

When omitted, project commands discover `studio.config.json` before
`studio.config.ts`.

Failures exit non-zero. JSON output includes structured diagnostics plus
per-check status records and does not require parsing human output.
