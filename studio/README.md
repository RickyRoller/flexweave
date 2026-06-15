# Flexweave Studio

`@flexweave/studio` is the reusable authoring package for Flexweave Studio. It
loads a consumer-owned Studio project config, validates a Studio catalog,
plans and scaffolds mechanics, refreshes generated mechanics definitions,
checks generated freshness, reports runtime hook diagnostics, and exposes
server-safe workflow functions for local host apps.

Flexweave Studio does not own consumer runtime semantics, consumer catalog
content, generated output directories, runtime hook implementations after stub
creation, local host app entry points, deployment, or project-specific labels.
Those belong to the consumer project and are declared through `studio.config.ts`.

## Public Entry Points

- `@flexweave/studio/config`
- `@flexweave/studio/config/load`
- `@flexweave/studio/extensions`
- `@flexweave/studio/workflows`
- `@flexweave/studio/codegen`
- `@flexweave/studio-app`
- `flexweave-studio`

`@flexweave/studio/extensions` exposes `defineStudioExtension`,
`defineStudioDataAdapter`, source snapshot types, source record types, source
location metadata, content mapper types, and adapter capability helpers. Data
adapters load source records and preserve provenance; mappers normalize those
records into Studio content. Adapters and mappers do not generate Rust directly.

`@flexweave/studio/codegen` exposes `defineStudioGeneratedTarget` for
extension-owned generated outputs. Built-in and extension generated targets run
through the same registry, dependency resolution, freshness checks, write mode,
and managed-file cleanup.

## Command Family

```bash
flexweave-studio validate
flexweave-studio describe
flexweave-studio list
flexweave-studio show
flexweave-studio plan
flexweave-studio scaffold
flexweave-studio scaffold host-app
flexweave-studio codegen
flexweave-studio verify
flexweave-studio migrate
```

Every project command accepts `--config <path>`. When omitted, Studio discovers
`studio.config.ts` by walking upward from the current working directory.
Consumer paths in the config resolve from the directory containing the active
config file.

`flexweave-studio scaffold host-app` creates a consumer-owned local host app
that imports `@flexweave/studio-app`, records scaffold metadata, and preserves
existing files that differ from the current scaffold template.

## Documentation

- [First Studio workflow](./docs/tutorials/first-workflow.md)
- [Configure a Studio project](./docs/how-to/configure-project.md)
- [Run Studio workflows](./docs/how-to/run-workflows.md)
- [Config reference](./docs/reference/config.md)
- [CLI reference](./docs/reference/cli.md)
- [Workflow and codegen reference](./docs/reference/workflows-and-codegen.md)
- [Runtime hook reference](./docs/reference/runtime-hooks.md)
- [Product boundaries](./docs/explanation/boundaries.md)
- [Generated files and upgrades](./docs/explanation/generated-files-and-upgrades.md)

## Verification

```bash
bun run verify:studio
```
