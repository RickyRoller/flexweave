import { expect, test } from "bun:test";

import { createStudioApp, createStudioOverviewPanel, defineStudioAppAdapter } from "../src";

const adapter = defineStudioAppAdapter({
  authoring: {
    areas: [
      {
        editorId: "tags",
        id: "tags",
        label: "Tags",
      },
    ],
    editors: [
      {
        areaId: "tags",
        commandName: "list",
        id: "tags",
        label: "Tags",
        recordKind: "tags",
      },
    ],
  },
  codegenTargets: [
    {
      label: "Generated tags",
      outputLabel: "tags output",
      target: "tags",
    },
  ],
  id: "synthetic-project",
  labels: {
    productName: "Synthetic Studio",
    projectName: "Synthetic project",
    shellSubtitle: "Catalog authoring",
    workflowTrail: ["Studio catalog", "Generated mechanics definitions"],
    workspaceTitle: "Authoring workspace",
  },
  navigation: [
    {
      id: "workspace",
      label: "Workspace",
      links: [
        {
          href: "/",
          id: "overview",
          label: "Overview",
        },
      ],
    },
  ],
  serverFunctions: {
    validate: () => ({ diagnostics: [], ok: true, recordCount: 0 }),
  },
  workflowActions: [
    {
      commandName: "validate",
      id: "validate",
      label: "Validate",
      variant: "primary",
    },
  ],
});

test("app shell derives adapter-neutral routes and panel metadata", () => {
  const app = createStudioApp(adapter);
  const panel = createStudioOverviewPanel(adapter);

  expect(app.adapterId).toBe("synthetic-project");
  expect(app.routes).toEqual([
    {
      href: "/",
      id: "overview",
      kind: "overview",
      label: "Authoring workspace",
    },
    {
      href: "/#generated-output",
      id: "generated-output",
      kind: "generated-output",
      label: "Generated output",
    },
    {
      editorId: "tags",
      href: "/authoring/tags",
      id: "authoring.tags",
      kind: "authoring-editor",
      label: "Tags",
    },
  ]);
  expect(panel.codegenTargets[0]?.target).toBe("tags");
  expect(panel.workflowActions[0]?.commandName).toBe("validate");
});

test("server function bindings stay project-provided", async () => {
  const result = await adapter.serverFunctions.validate?.();

  expect(result).toEqual({
    diagnostics: [],
    ok: true,
    recordCount: 0,
  });
});
