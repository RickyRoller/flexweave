import { defineStudioConfig } from "@flexweave/studio/config";

export default defineStudioConfig({
  catalogRoot: "catalog",
  extensions: [
    {
      dataAdapters: [
        {
          capabilities: ["read"],
          id: "malformed-adapter",
        },
      ],
      id: "malformed-extension",
    },
  ],
  mode: "validate-only",
});
