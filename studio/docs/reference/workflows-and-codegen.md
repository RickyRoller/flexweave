# Workflow And Codegen Reference

`@flexweave/studio/workflows` exports server-safe functions for CLI and local
host app integration. Workflow functions return structured results and do not
call `process.exit` or write to stdout or stderr.

Exports include:

- `validateStudioCatalog`
- `describeStudioCatalog`
- `listStudioCatalogRecords`
- `showStudioCatalogRecord`
- `planStudioMechanic`
- `scaffoldStudioMechanic`
- `scaffoldStudioHostApp`
- `codegenStudioProject`
- `verifyStudioHostApp`
- `verifyStudioProject`
- `migrateStudioProject`

`@flexweave/studio/codegen` exports generated target types, summaries, and
`defineStudioGeneratedTarget`. The built-in generated targets are `abilities`,
`effects`, `executions`, `modifiers`, `reference`, and `tags`. Studio
extensions may register additional generated targets with ids, labels,
dependencies, cleanup policies, and plan functions.

Validation loads the built-in JSON catalog and configured data sources through
data adapters, then runs content mappers to produce normalized Studio content.
Source-backed diagnostics keep adapter-provided source locations so callers can
point users at file paths, JSON pointers, sheet names, row numbers, column
numbers, cells, or fields.

Scaffold writes go through the active writable content adapter. The built-in
JSON catalog adapter supports transactional scaffold writes and rollback. Source
adapters can provide `writeSnapshotPaths` for filesystem-backed rollback;
otherwise rollback diagnostics report that source writes could not be restored
automatically. Source configurations without a writable content adapter fail
before writing files.

Codegen resolves selected target ids through the active registry. Target
dependencies are included before the selected target and deduplicated
deterministically. Unknown target diagnostics list target ids available for the
active project.

Generated Rust targets receive the resolved Rust codegen context. Built-in
targets use the generic subset owned by Studio, including generated headers.
Extension targets may read their own namespaced `rust.bindings` config after
their extension validates it.

Codegen write mode writes managed files only under configured output
directories. Check mode compares expected files to disk and does not create,
modify, or delete files. A target plan that writes outside its configured output
directory fails before write mode touches disk. Target summaries contain file
statuses that a local host app can render without importing package internals.

Host app scaffold results include created or updated files and manual
follow-ups for scaffold-managed files that differ from the current scaffold
template. Scaffold metadata records managed files, project-owned files, the
scaffold version, and package refs. Project-owned adapter files are preserved
on rerun and migrate.

Generated host app adapters load the active Studio config through public
`@flexweave/studio/config/load`, call public `@flexweave/studio/workflows`
functions for server bindings, and compose active extension host app
contributions through `@flexweave/studio-app`.

Host app verification checks scaffold metadata, required managed files,
project-owned file presence, contribution contract diagnostics, and the
configured host app check or build command.

Verify results include a `checks` array for unattended callers. Checks are
reported for config load, active extensions, configured sources and adapters,
extension mappers, validation, each generated target, runtime hooks, host app
state, and project commands. Each check includes a name, mode, status, owning
extension/adapter/target/command fields when applicable, diagnostics, and
command output for failed project commands.

`verify --fast` still runs the built-in health checks and host app checks. It
filters project-declared verification commands to those marked `fast: true`.
Full verify runs every project-declared command.

Migrate results also include a `checks` array. The host app scaffold migration
detects the current scaffold version, rejects unsupported future versions with
manual follow-ups, and preserves project-owned adapter files. Extension-owned
migrations run in deterministic extension id and migration id order; each
extension owns its schema version detection, writes, skipped result, and
unsupported-version diagnostics.
