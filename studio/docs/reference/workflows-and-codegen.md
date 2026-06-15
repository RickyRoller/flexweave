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
configurations without a writable content adapter fail before writing files.

Codegen resolves selected target ids through the active registry. Target
dependencies are included before the selected target and deduplicated
deterministically. Unknown target diagnostics list target ids available for the
active project.

Codegen write mode writes managed files only under configured output
directories. Check mode compares expected files to disk and does not create,
modify, or delete files. A target plan that writes outside its configured output
directory fails before write mode touches disk. Target summaries contain file
statuses that a local host app can render without importing package internals.

Host app scaffold results include created or updated files and manual
follow-ups for existing files that differ from the current scaffold template.
Host app verification checks scaffold metadata, required files, and the
configured host app check or build command.
