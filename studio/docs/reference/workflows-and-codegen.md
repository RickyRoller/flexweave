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

`@flexweave/studio/codegen` exports generated target types and summaries. The
generated targets are `abilities`, `effects`, `executions`, `modifiers`,
`reference`, and `tags`.

Validation loads configured data sources through active Studio extensions and
data adapters before reporting workflow results. Source-backed diagnostics keep
adapter-provided source locations so callers can point users at file paths,
JSON pointers, sheet names, row numbers, column numbers, cells, or fields.
Source snapshots are ingestion results only; mapping and generated output remain
separate workflow stages.

Codegen write mode writes managed files only under configured output
directories. Check mode compares expected files to disk and does not create,
modify, or delete files. Target summaries contain file statuses that a local
host app can render without importing package internals.

Host app scaffold results include created or updated files and manual
follow-ups for existing files that differ from the current scaffold template.
Host app verification checks scaffold metadata, required files, and the
configured host app check or build command.
