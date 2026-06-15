# Flexweave Studio

Flexweave Studio covers reusable authoring and build-time workflows for
consumer projects.

## Language

**Studio package**:
The `@flexweave/studio` package that owns project config loading, catalog
contracts, validation, migrations, generated output checks, workflow APIs, and
runtime contract docs.

**Studio app package**:
The `@flexweave/studio-app` package that owns the reusable application shell.

**Studio extension**:
A project-neutral contribution object registered through `studio.config.ts`.
Extensions can provide data adapters, source validation, content mappers,
generated targets, Rust binding validation, and local host app contributions
without importing Studio internals.

**Host app contribution**:
Extension-owned local Studio app metadata for navigation, authoring areas,
editor definitions, workflow actions, generated-output panels, diagnostics
panels, and source views.

**Data adapter**:
A pluggable source integration that loads authored records into a source
snapshot. Adapters declare capabilities such as read, write, schema, watch, and
diff.

**Source snapshot**:
The result of loading a configured source through a data adapter. Snapshots
contain source records and diagnostics.

**Source record**:
One authored source item loaded by an adapter before it is normalized into
Studio content or project-owned models.

**Source location**:
Diagnostic provenance for a source record. Locations may include file paths,
JSON pointers, sheet names, row numbers, column numbers, cells, or fields.

**Mapper**:
Extension code that converts source snapshots into normalized Studio content.
Mappers keep source storage formats separate from validation and generation.

**Normalized Studio content**:
Source-agnostic records consumed by Studio validators and generators.

**Generated target**:
A registered unit of generated output with an id, label, dependencies, cleanup
policy, configured output directory, and plan function.

**Rust codegen context**:
Resolved Rust generation settings from project config, including the Flexweave
module path, generated header template, aliases, type paths, macro names,
prelude imports, runtime vocabulary, and extension binding config.

**Extension Rust binding config**:
A namespaced Rust config object validated by the extension that owns it.

**Extension-owned migration**:
A namespaced migration declared by a Studio extension. The extension owns its
schema version check, file writes, skipped state, and unsupported-version
diagnostics.

**Verify check**:
A structured health check reported by `flexweave-studio verify` for config,
extensions, sources, mappers, validation, generated targets, runtime hooks,
local host app state, and project commands.

**Adapter capability**:
An explicit data adapter feature declaration. Studio workflows use capabilities
to distinguish read-only adapters from writable adapters.

**Studio catalog**:
Project-owned authored content read by Studio workflows.

**Studio project config**:
Project-owned configuration that declares catalog roots, data sources,
extensions, generated output roots, runtime hook roots, and verification
commands.

**Generated mechanics definitions**:
Output written by Studio workflows to consumer-declared paths.

**Runtime hooks**:
Consumer-owned functions that connect generated definitions to a consumer
runtime.

**Local host app**:
A small consumer-owned app entry point that imports the versioned Studio app
package and provides a project adapter.

**Project adapter**:
Consumer-owned wiring for labels, config, workflow calls, and deployment needs.

**Host app scaffold**:
Files created by `flexweave-studio scaffold host-app` for the consumer-owned
local host app package, entry point, project adapter, and scaffold metadata.
Metadata distinguishes scaffold-managed files from project-owned adapter code.

## Relationships

- Studio builds on Core concepts but remains optional.
- Studio packages provide reusable workflows and shell behavior.
- Projects provide content, config, data adapters, mappers, runtime
  hooks, generated output paths, branding, and deployment.
- Package updates should be followed by migrate and verify commands.
- Host app scaffolds are updated through migrate and checked through verify.
