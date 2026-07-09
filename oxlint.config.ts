import { defineConfig } from "oxlint";

import core from "ultracite/oxlint/core";

export default defineConfig({
  extends: [core],
  ignorePatterns: ["docs/src/routeTree.gen.ts"],
});
