import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

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
  data: {
    sources: [
      {
        adapterId: "synthetic-table",
        id: "table-backed",
        options: {
          path: "sources/writable-table.json",
        },
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  hooks: {
    dir: "runtime-hooks",
    testStubsDir: "generated-hook-tests",
  },
  rust: {
    bindings: {
      synthetic: {
        module: "synthetic_runtime",
      },
    },
    flexweaveModule: "flexweave",
  },
});
