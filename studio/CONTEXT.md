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

**Studio catalog**:
Consumer-owned authored content read by Studio workflows.

**Studio project config**:
Consumer-owned configuration that declares catalog roots, generated output
roots, runtime hook roots, and verification commands.

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

## Relationships

- Studio builds on Core concepts but remains optional.
- Studio packages provide reusable workflows and shell behavior.
- Consumer projects provide content, config, runtime hooks, generated output
  paths, adapters, branding, and deployment.
- Package updates should be followed by migrate and verify commands.
- Host app scaffolds are updated through migrate and checked through verify.
