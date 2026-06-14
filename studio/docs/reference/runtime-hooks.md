# Runtime Hook Reference

A runtime hook is a consumer-owned Rust function referenced by an execution
record in the Studio catalog. Generated mechanics definitions refer to hook ids;
the consumer runtime supplies the behavior behind those ids.

Hook file paths resolve from the active Studio project config:

- `hooks.dir` is the root for runtime hook stubs.
- `hooks.testStubsDir` is the optional root for generated hook test stubs.

Studio may create a missing hook stub when a catalog execution references a
hook id. Stub creation is write-if-missing. Existing hook files are never
overwritten by Studio. After creation, hook implementations are owned by the
consumer project.

Studio reports unreferenced hook files as diagnostics and does not delete them
automatically. Removing or retaining an unreferenced hook is a consumer runtime
decision.

The package guarantees deterministic hook file names, write-if-missing stub
creation, generated hook test stub creation when configured, and orphan hook
diagnostics. Runtime semantics belong to the consumer runtime.
