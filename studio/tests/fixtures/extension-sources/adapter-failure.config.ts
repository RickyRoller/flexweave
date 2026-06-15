import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-file",
        id: "missing-file-source",
        options: {
          path: "sources/missing-file-record.json",
        },
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  mode: "validate-only",
});
