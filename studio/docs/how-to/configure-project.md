# Configure A Studio Project

Create `studio.config.ts` at the consumer project root or pass its path with
`--config`.

```ts
import { defineStudioConfig } from "@flexweave/studio/config";
import { defineStudioDataAdapter, defineStudioExtension } from "@flexweave/studio/extensions";

const tableAdapter = defineStudioDataAdapter({
  capabilities: ["read", "schema"],
  id: "local-table",
  load: ({ source }) => ({
    records: [
      {
        id: "sample-row",
        kind: "sample.table",
        location: { column: 1, row: 2, sheet: "balance" },
        value: source.options?.row,
      },
    ],
  }),
});

const projectSources = defineStudioExtension({
  dataAdapters: [tableAdapter],
  id: "project-sources",
});

export default defineStudioConfig({
  app: {
    buildCommand: ["bun", "run", "build"],
    checkCommand: ["bun", "run", "typecheck"],
    root: "studio-host",
  },
  catalogRoot: "content/catalog",
  data: {
    sources: [
      {
        adapterId: "local-table",
        id: "balance-table",
        options: {
          row: {
            id: "sample-row",
            label: "Sample row",
          },
        },
      },
    ],
  },
  extensions: [projectSources],
  codegen: {
    outputDirs: {
      abilities: "runtime/generated/abilities",
      effects: "runtime/generated/effects",
      executions: "runtime/generated/executions",
      modifiers: "runtime/generated/modifiers",
      reference: "content/generated-reference",
      tags: "runtime/generated/tags",
    },
  },
  hooks: {
    dir: "runtime/hooks",
    testStubsDir: "runtime/generated-hook-tests",
  },
  rust: {
    flexweaveModule: "flexweave",
    runtimeVocab: {
      ailments: ["synthetic_ailment"],
      damageTypes: ["synthetic_damage"],
    },
  },
  verify: {
    commands: [
      {
        command: ["bun", "--version"],
        fast: true,
        name: "consumer check",
      },
    ],
  },
});
```

Relative paths resolve from the directory containing the active config file.
Absolute paths remain absolute. Generated output directories and runtime hook
directories must be distinct so Studio has clear ownership boundaries.

`app.root` points at the consumer-owned local host app scaffold.
`app.checkCommand` is used by `flexweave-studio verify`; `app.buildCommand`
is the fallback when no check command is configured.

Use a validate-only config only for validation flows:

```ts
export default defineStudioConfig({
  mode: "validate-only",
  catalogRoot: "content/catalog",
});
```
