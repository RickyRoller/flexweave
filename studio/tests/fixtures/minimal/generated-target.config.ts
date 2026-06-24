import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "../extension-sources/synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  codegen: {
    outputDirs: {
      abilities: "generated/abilities",
      effects: "generated/effects",
      executions: "generated/executions",
      modifiers: "generated/modifiers",
      reference: "generated/reference",
      "synthetic-rust": "generated/synthetic-rust",
      "synthetic-summary": "generated/synthetic",
      tags: "generated/tags",
    },
  },
  extensions: [syntheticSourceExtension],
  hooks: {
    dir: "runtime-hooks",
    testStubsDir: "generated-hook-tests",
  },
  rust: {
    bindings: {
      synthetic: {
        module: "minimal_synthetic_runtime",
      },
    },
    flexweaveModule: "flexweave",
    preludeImports: ["core::fmt::Debug"],
  },
});
