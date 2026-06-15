import { defineStudioConfig } from "@flexweave/studio/config";

import { syntheticSourceExtension } from "./synthetic-extension";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-file",
        id: "file-backed",
        options: {
          path: "sources/file-record.json",
        },
      },
      {
        adapterId: "synthetic-table",
        id: "table-backed",
        options: {
          rows: [
            {
              id: "table-row",
              label: "Table-backed row",
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
