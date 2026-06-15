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
      tags: "generated/tags",
    },
  },
  extensions: [syntheticSourceExtension],
  hooks: {
    dir: "runtime-hooks",
  },
  rust: {
    bindings: {
      synthetic: {},
    },
    flexweaveModule: "flexweave",
  },
});
