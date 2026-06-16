import { defineStudioConfig } from "@flexweave/studio/config";
import { defineStudioDataAdapter } from "@flexweave/studio/extensions";

import { syntheticSourceExtension } from "./synthetic-extension";

const projectSyntheticFileAdapter = defineStudioDataAdapter({
  capabilities: ["read"],
  id: "synthetic-file",
  label: "Project synthetic file adapter",
  load: () => ({ records: [] }),
});

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    adapters: [projectSyntheticFileAdapter],
    sources: [
      {
        adapterId: "synthetic-file",
        id: "project-conflict",
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  mode: "validate-only",
});
