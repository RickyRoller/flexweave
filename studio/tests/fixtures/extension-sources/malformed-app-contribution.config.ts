import { defineStudioConfig } from "@flexweave/studio/config";

export default defineStudioConfig({
  catalogRoot: "catalog",
  extensions: [
    {
      appContributions: [
        {
          authoring: "not-an-authoring-object",
          id: "malformed-host-app",
          navigation: [
            {
              id: "broken-navigation",
              label: "Broken navigation",
              links: [
                {
                  href: "/broken",
                  id: "missing-label",
                },
              ],
            },
          ],
          workflowActions: [
            {
              commandName: "validate",
              id: "broken-action",
              label: "Broken action",
              variant: "loud",
            },
          ],
        },
      ],
      id: "malformed-host-app-extension",
    },
  ],
  mode: "validate-only",
});
