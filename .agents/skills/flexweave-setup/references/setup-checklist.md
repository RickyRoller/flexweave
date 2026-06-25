# Setup Checklist

Use this checklist to keep Flexweave setup concrete and repeatable.

## Discovery

- Locate the repo root and existing tooling: Cargo, Bun, npm, pnpm, yarn, or a
  mixed workspace. Do not introduce JavaScript tooling solely for Studio CLI
  commands.
- Identify the game runtime package or crate that should import Flexweave Core.
- Identify existing authored-data directories, generated-code directories,
  runtime hook modules, and test conventions.
- Check for existing codegen scripts or generated-file policies.
- Check whether the repo already has `studio.config.json`, `studio.config.ts`,
  `@flexweave/studio`, `@flexweave/studio-app`, or `flexweave` dependencies.
- Check whether `flexweave-studio --help` is available. If it is missing, tell
  the user to install the CLI directly instead of vendoring it into the repo.

## Integration Mode

- Core only: add the Rust crate and runtime tests around primitive usage.
- Studio codegen: add Studio config, catalog root, generated output dirs,
  runtime hook dirs, Rust binding config, and verification commands.
- Studio host app: scaffold the local host app and wire app check/build scripts.

## Initial Studio Config Shape

Use repo-specific paths, but prefer this ownership split:

```json
{
  "catalogRoot": "content/catalog",
  "codegen": {
    "outputDirs": {
      "abilities": "runtime/generated/abilities",
      "effects": "runtime/generated/effects",
      "executions": "runtime/generated/executions",
      "modifiers": "runtime/generated/modifiers",
      "reference": "content/generated-reference",
      "tags": "runtime/generated/tags"
    }
  },
  "hooks": {
    "dir": "runtime/hooks",
    "testStubsDir": "runtime/generated-hook-tests"
  },
  "rust": {
    "flexweaveModule": "flexweave"
  },
  "verify": {
    "commands": [
      {
        "command": ["cargo", "test", "-p", "game-runtime"],
        "fast": true,
        "name": "runtime tests"
      }
    ]
  }
}
```

Use `studio.config.ts` only when the project needs executable extensions, data
adapters, content mappers, generated targets, or host app contributions that
cannot be represented in JSON.

## Command Names

Record direct CLI commands in `FLEXWEAVE.md`. Add package-manager scripts only
when the repo already has a matching convention:

```bash
flexweave-studio validate --config studio.config.json
flexweave-studio codegen --config studio.config.json
flexweave-studio codegen --check --config studio.config.json
flexweave-studio migrate --config studio.config.json
flexweave-studio verify --fast --config studio.config.json
flexweave-studio verify --config studio.config.json
```

If the repo already has a package-manager convention, wrappers such as
`pnpm exec flexweave-studio` or `npx flexweave-studio` are acceptable. Do not
prefer Bun unless the consumer repo already uses Bun.

## Validation Order

1. Dependency install or workspace link succeeds.
2. Config loads.
3. `validate` succeeds.
4. `codegen` writes only under configured output dirs.
5. `codegen --check` is clean.
6. `verify --fast` runs configured fast checks.
7. Host app scaffold verifies when host app is enabled.

## Starter Content

Do not create sample catalog records during setup. Setup may create empty
directories and generated outputs from an empty catalog. If the user wants sample
or starter content, complete setup first and then use the mechanic authoring
skill to plan and scaffold that content explicitly.
