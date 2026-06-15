import { defineStudioConfig } from "@flexweave/studio/config";

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "missing-adapter",
        id: "missing-adapter-source",
      },
    ],
  },
  mode: "validate-only",
});
