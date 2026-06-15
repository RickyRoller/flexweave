# Studio Project Config Reference

`@flexweave/studio/config` exports `defineStudioConfig`, input types, resolved
config types, and diagnostic types. The helper is a typed identity function and
does not perform filesystem IO.

`@flexweave/studio/config/load` exports config discovery and loading helpers.
Discovery searches upward from the provided working directory for
`studio.config.ts`.

`@flexweave/studio/extensions` exports `defineStudioExtension`,
`defineStudioDataAdapter`, `defineStudioContentMapper`, source snapshot types,
source record types, source location metadata, and adapter capability helpers.
`@flexweave/studio/codegen` exports `defineStudioGeneratedTarget`. Project
configs register extensions and sources explicitly; Studio does not discover
extension modules from the filesystem.

## Fields

`app.root`:
Optional path to the consumer-owned local host app scaffold.

`app.checkCommand`:
Optional command used by `flexweave-studio verify` to check the local host app.

`app.buildCommand`:
Optional fallback command used by `verify` when `app.checkCommand` is omitted.

`catalogRoot`:
Directory containing the Studio catalog.

`extensions`:
Optional array of Studio extensions. Each extension has an `id` and may provide
data adapters, content mappers, generated targets, source validation, Rust
binding config validators, or local host app contributions.

`extensions[].appContributions`:
Optional extension-owned local host app surfaces. Contributions can declare
navigation sections, authoring areas and editors, workflow actions,
generated-output panels, diagnostics panels, source views, and generated target
display metadata. Studio validates the structural contract and the host app
package composes active contributions into the project adapter.

`data.adapters`:
Optional array of project-local data adapters. Adapters declare capabilities
such as `read`, `write`, `schema`, `watch`, and `diff`.

`data.sources`:
Optional array of source declarations. Each source has an `id`, `adapterId`,
and adapter-owned `options` object. Source records keep provenance through
source locations such as file paths, JSON pointers, sheet names, rows, columns,
cells, and fields.

`mode`:
`"full"` by default. `"validate-only"` allows validation without generated
output, runtime hook, or Rust binding fields.

`codegen.outputDirs`:
Directories for `abilities`, `effects`, `executions`, `modifiers`,
`reference`, and `tags` generated outputs. Extension generated targets may add
their own output directory keys after they are registered by an active
extension. Built-in output directories are required in full configs; extension
target output directories are required only when that target is selected.

`hooks.dir`:
Directory where missing runtime hook stubs may be created.

`hooks.testStubsDir`:
Optional directory for generated hook test stubs.

`rust.flexweaveModule`:
Consumer runtime import binding used by generated Rust definitions.

`rust.generatedHeader`:
Optional generated file header template. `{target}` is replaced with the active
generated target id.

`rust.moduleAliases`, `rust.typePaths`, and `rust.macroNames`:
Optional objects of string aliases used by generated Rust targets.

`rust.preludeImports`:
Optional array of Rust imports available to generated targets.

`rust.runtimeVocab`:
Consumer-owned token lists needed by validation or generated code.

`rust.bindings`:
Optional object for extension-owned Rust binding config. Each extension owns
and validates its own namespace under this object. Studio validates only the
generic object shape.

`verify.commands`:
Structured verification commands. Each command has `name`, `command`, and an
optional `fast` boolean.

Validation reports all practical shape errors in one pass, including missing
fields, invalid command arrays, invalid path values, unknown generated targets,
duplicate owned output paths, malformed extensions, malformed data adapters,
malformed generated targets, malformed host app contributions, missing data
adapters, unknown generated target output directories, and malformed local host
app commands.
