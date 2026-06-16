import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-file",
        id: "raw-kind-file",
        options: {
          path: "sources/raw-kind-file-record.json",
          recordKind: "tags",
        },
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  mode: "validate-only",
});
