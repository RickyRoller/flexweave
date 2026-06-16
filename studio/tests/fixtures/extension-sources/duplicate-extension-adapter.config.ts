import { defineStudioConfig } from "@flexweave/studio/config";
import { defineStudioDataAdapter, defineStudioExtension } from "@flexweave/studio/extensions";

import { syntheticSourceExtension } from "./synthetic-extension";

const conflictingExtension = defineStudioExtension({
  dataAdapters: [
    defineStudioDataAdapter({
      capabilities: ["read"],
      id: "synthetic-file",
      label: "Conflicting synthetic file adapter",
      load: () => ({ records: [] }),
    }),
  ],
  id: "conflicting-source-extension",
  label: "Conflicting source extension",
});

export default defineStudioConfig({
  catalogRoot: "catalog",
  extensions: [syntheticSourceExtension, conflictingExtension],
  mode: "validate-only",
});
