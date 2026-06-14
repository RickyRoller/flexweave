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
- `codegenStudioProject`
- `verifyStudioProject`
- `migrateStudioProject`

`@flexweave/studio/codegen` exports generated target types and summaries. The
generated targets are `abilities`, `effects`, `executions`, `modifiers`,
`reference`, and `tags`.

Codegen write mode writes managed files only under configured output
directories. Check mode compares expected files to disk and does not create,
modify, or delete files. Target summaries contain file statuses that a local
host app can render without importing package internals.
