# First Studio Workflow

This tutorial creates the smallest useful Flexweave Studio flow in a consumer
project: define config, add a few synthetic catalog records, validate them, and
check generated mechanics definitions.

1. Add a `studio.config.ts` file.

```ts
import { defineStudioConfig } from "@flexweave/studio/config";

export default defineStudioConfig({
  catalogRoot: "catalog",
  codegen: {
    outputDirs: {
      abilities: "generated/abilities",
      effects: "generated/effects",
      executions: "generated/executions",
      modifiers: "generated/modifiers",
      reference: "generated/reference",
      tags: "generated/tags",
    },
  },
  hooks: {
    dir: "runtime-hooks",
    testStubsDir: "generated-hook-tests",
  },
  rust: {
    flexweaveModule: "flexweave",
    runtimeVocab: {
      ailments: [],
      damageTypes: [],
    },
  },
});
```

2. Add synthetic records under the configured `catalog` directory. Keep ids
   stable and connect records through explicit fields such as `effectId`,
   `executionId`, and `hook`.

3. Validate the catalog.

```bash
flexweave-studio validate --config studio.config.ts
```

4. Refresh generated mechanics definitions.

```bash
flexweave-studio codegen --config studio.config.ts
```

5. Confirm the generated outputs are fresh.

```bash
flexweave-studio codegen --check --config studio.config.ts
```

The generated files live only under configured output directories. Runtime hook
stubs are created only when missing and become consumer-owned immediately.
