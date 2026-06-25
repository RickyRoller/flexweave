# Configure A Studio Project

Create `studio.config.json` at the consumer project root or pass its path with
`--config`. Use JSON for normal game repos so Studio does not require a
project-local JavaScript package install.

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
        "command": ["cargo", "test"],
        "fast": true,
        "name": "runtime tests"
      },
      {
        "command": ["cargo", "check"],
        "fast": true,
        "name": "runtime check"
      }
    ]
  }
}
```

Relative paths resolve from the directory containing the active config file.
Absolute paths remain absolute. Generated output directories and runtime hook
directories must be distinct so Studio has clear ownership boundaries.

By default, scaffold commands write catalog records to JSON files under
`catalogRoot`. If a source adapter should own scaffold writes, set
`data.writeSourceId` to that declared source id. Studio will not infer write
ownership from the number of configured sources.

`app.root` points at the consumer-owned local host app scaffold.
`app.checkCommand` is used by `flexweave-studio verify`; `app.buildCommand`
is the fallback when no check command is configured.

Use a validate-only config only for validation flows:

```json
{
  "mode": "validate-only",
  "catalogRoot": "content/catalog"
}
```

Use a TypeScript config only when the project needs executable extensions,
custom data adapters, content mappers, generated targets, or host app
contributions. TypeScript configs can still import helpers from
`@flexweave/studio`, which means the consumer repo must make that package
resolvable through its chosen JavaScript tooling.
