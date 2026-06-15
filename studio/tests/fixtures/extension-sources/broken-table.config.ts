import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-table",
        id: "table-backed",
        options: {
          rows: [
            {
              id: "broken-table-row",
              label: "Broken table-backed row",
              valid: false,
            },
          ],
        },
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  mode: "validate-only",
});
