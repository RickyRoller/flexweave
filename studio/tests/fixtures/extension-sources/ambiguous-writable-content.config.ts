import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-table",
        id: "writable-table-a",
        options: {
          rows: [],
        },
      },
      {
        adapterId: "synthetic-table",
        id: "writable-table-b",
        options: {
          rows: [],
        },
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  mode: "validate-only",
});
