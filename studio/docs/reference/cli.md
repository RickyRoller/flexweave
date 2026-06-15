# CLI Reference

The package exposes one bin: `flexweave-studio`.

Every project command supports:

- `--config <path>` to load an explicit Studio project config.
- `--json` for machine-readable output.
- `--quiet` to suppress human success output.

Commands:

- `validate`: validate the configured Studio catalog.
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
- `verify`: run validation, generated freshness checks, and configured
  verification commands. When a host app is configured, it also checks scaffold
  health and runs the host app check or build command.
- `migrate`: run package migrations after updates. It reads local host app
  scaffold metadata when present and reports changed files plus manual
  follow-ups.

Failures exit non-zero. JSON failures include structured diagnostics and do not
require parsing human output.
