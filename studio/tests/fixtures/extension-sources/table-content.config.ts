import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-table",
        id: "table-content",
        options: {
          rows: [
            {
              contentKind: "tag",
              id: "table_tag",
              label: "Table-backed tag",
              valid: true,
            },
          ],
        },
      },
    ],
  },
  extensions: [syntheticSourceExtension],
  mode: "validate-only",
});
