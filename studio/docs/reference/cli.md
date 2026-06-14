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
- `codegen`: refresh generated mechanics definitions.
- `codegen --check`: fail when generated mechanics definitions are missing,
  stale, or unexpectedly present.
- `verify`: run validation, generated freshness checks, and configured
  verification commands.
- `migrate`: run package migrations after updates. The initial registry is
  empty and reports an up-to-date project.

Failures exit non-zero. JSON failures include structured diagnostics and do not
require parsing human output.
