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
  },
  verify: {
    commands: [
      {
        command: ["bun", "--version"],
        fast: true,
        name: "fixture command",
      },
    ],
  },
});
