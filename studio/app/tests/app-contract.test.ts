import { expect, test } from "bun:test";

import {
  collectStudioAppContributions,
  composeStudioAppContributions,
  createDefaultStudioProjectAdapter,
  createStudioApp,
  createStudioOverviewPanel,
  defineStudioAppAdapter,
} from "../src";
import { syntheticSourceExtension } from "../../tests/fixtures/extension-sources/synthetic-extension";
import { generatedTargetFixtureConfigPath } from "../../tests/support/studio-fixtures";

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

test("app shell composes extension host app contributions", () => {
  const contributions = collectStudioAppContributions([syntheticSourceExtension]);
  const composed = composeStudioAppContributions(adapter, contributions);

  expect(composed.ok).toBe(true);
  expect(composed.diagnostics).toEqual([]);
  expect(composed.adapter.navigation.map((section) => section.id)).toContain("synthetic");
  expect(composed.adapter.authoring.editors.map((editor) => editor.id)).toContain(
    "synthetic-source-records",
  );
  expect(composed.adapter.workflowActions.map((action) => action.id)).toContain(
    "validate-synthetic-sources",
  );

  const app = createStudioApp(composed.adapter);
  const panel = createStudioOverviewPanel(composed.adapter);

  expect(app.codegenTargets.map((target) => target.target)).toContain("synthetic-summary");
  expect(app.generatedOutputPanels.map((generatedPanel) => generatedPanel.id)).toEqual([
    "synthetic-summary-output",
  ]);
  expect(app.diagnosticsPanels.map((diagnosticsPanel) => diagnosticsPanel.id)).toEqual([
    "synthetic-source-diagnostics",
  ]);
  expect(app.sourceViews.map((sourceView) => sourceView.id)).toEqual(["synthetic-table-source"]);
  expect(app.routes.map((route) => route.id)).toEqual(
    expect.arrayContaining([
      "authoring.synthetic-source-records",
      "generated.synthetic-summary-output",
      "diagnostics.synthetic-source-diagnostics",
      "source.synthetic-table-source",
    ]),
  );
  expect(panel.sourceViews[0]?.adapterId).toBe("synthetic-table");
});

test("app contribution composition reports duplicate app surface ids", () => {
  const composed = composeStudioAppContributions(adapter, [
    {
      authoring: {
        editors: [
          {
            areaId: "tags",
            id: "tags",
            label: "Duplicate tags",
          },
        ],
      },
      id: "duplicate-tags",
    },
  ]);

  expect(composed.ok).toBe(false);
  expect(composed.diagnostics[0]).toMatchObject({
    code: "duplicate-host-app-contribution",
    field: "authoring.editors.1",
  });
});

test("default project adapter composes configured targets and extension surfaces", async () => {
  const result = await createDefaultStudioProjectAdapter({
    configPath: generatedTargetFixtureConfigPath,
  });

  expect(result.ok).toBe(true);
  expect(result.diagnostics).toEqual([]);
  expect(result.adapter.codegenTargets.map((target) => target.target)).toEqual(
    expect.arrayContaining(["abilities", "synthetic-rust", "synthetic-summary"]),
  );
  expect(result.adapter.generatedOutputPanels?.map((panel) => panel.id)).toEqual([
    "synthetic-summary-output",
  ]);
});

test("server function bindings stay project-provided", async () => {
  const result = await adapter.serverFunctions.validate?.();

  expect(result).toEqual({
    diagnostics: [],
    ok: true,
    recordCount: 0,
  });
});
