# Studio Project Config Reference

`@flexweave/studio/config` exports `defineStudioConfig`, input types, resolved
config types, and diagnostic types. The helper is a typed identity function and
does not perform filesystem IO.

`@flexweave/studio/config/load` exports config discovery and loading helpers.
Discovery searches upward from the provided working directory for
`studio.config.ts`.

`@flexweave/studio/extensions` exports `defineStudioExtension`,
`defineStudioDataAdapter`, source snapshot types, source record types, source
location metadata, and adapter capability helpers. Project configs register
extensions and sources explicitly; Studio does not discover extension modules
from the filesystem.

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
data adapters or source validation.

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
`reference`, and `tags` generated outputs.

`hooks.dir`:
Directory where missing runtime hook stubs may be created.

`hooks.testStubsDir`:
Optional directory for generated hook test stubs.

`rust.flexweaveModule`:
Consumer runtime import binding used by generated Rust definitions.

`rust.runtimeVocab`:
Consumer-owned token lists needed by validation or generated code.

`verify.commands`:
Structured verification commands. Each command has `name`, `command`, and an
optional `fast` boolean.

Validation reports all practical shape errors in one pass, including missing
fields, invalid command arrays, invalid path values, unknown generated targets,
duplicate owned output paths, malformed extensions, malformed data adapters,
missing data adapters, and malformed local host app commands.
