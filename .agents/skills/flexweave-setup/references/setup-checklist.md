# Setup Checklist

Use this checklist to keep Flexweave setup concrete and repeatable.

## Discovery

- Locate the repo root and package manager: Bun, npm, pnpm, yarn, Cargo, or a
  mixed workspace.
- Identify the game runtime package or crate that should import Flexweave Core.
- Identify existing authored-data directories, generated-code directories,
  runtime hook modules, and test conventions.
- Check for existing codegen scripts or generated-file policies.
- Check whether the repo already has `studio.config.ts`,
  `@flexweave/studio`, `@flexweave/studio-app`, or `flexweave` dependencies.

## Integration Mode

- Core only: add the Rust crate and runtime tests around primitive usage.
- Studio codegen: add Studio config, catalog root, generated output dirs,
  runtime hook dirs, Rust binding config, and verification commands.
- Studio host app: scaffold the local host app and wire app check/build scripts.

## Initial Studio Config Shape

Use repo-specific paths, but prefer this ownership split:

```ts
import { defineStudioConfig } from "@flexweave/studio/config";

export default defineStudioConfig({
  catalogRoot: "content/catalog",
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
      ailments: [],
      damageTypes: [],
    },
  },
  verify: {
    commands: [
      {
        command: ["cargo", "test", "-p", "game-runtime"],
        fast: true,
        name: "runtime tests",
      },
    ],
  },
});
```

## Script Names

Adapt names to the repo, but preserve this command coverage:

```json
{
  "studio:validate": "flexweave-studio validate --config studio.config.ts",
  "studio:codegen": "flexweave-studio codegen --config studio.config.ts",
  "studio:check-generated": "flexweave-studio codegen --check --config studio.config.ts",
  "studio:migrate": "flexweave-studio migrate --config studio.config.ts",
  "studio:verify:fast": "flexweave-studio verify --fast --config studio.config.ts",
  "studio:verify": "flexweave-studio verify --config studio.config.ts"
}
```

If the package manager requires an executor, use the repo convention:
`bun x flexweave-studio`, `pnpm exec flexweave-studio`,
`npx flexweave-studio`, or a direct workspace bin.

## Validation Order

1. Dependency install or workspace link succeeds.
2. Config typechecks or loads.
3. `validate` succeeds.
4. `codegen` writes only under configured output dirs.
5. `codegen --check` is clean.
6. `verify --fast` runs configured fast checks.
7. Host app scaffold verifies when host app is enabled.
