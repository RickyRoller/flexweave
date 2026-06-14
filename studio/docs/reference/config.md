# Studio Project Config Reference

`@flexweave/studio/config` exports `defineStudioConfig`, input types, resolved
config types, and diagnostic types. The helper is a typed identity function and
does not perform filesystem IO.

`@flexweave/studio/config/load` exports config discovery and loading helpers.
Discovery searches upward from the provided working directory for
`studio.config.ts`.

## Fields

`catalogRoot`:
Directory containing the Studio catalog.

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
and duplicate owned output paths.
