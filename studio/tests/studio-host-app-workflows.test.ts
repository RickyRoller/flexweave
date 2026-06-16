import { readFileSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { expect, test } from "bun:test";

import {
  migrateStudioProject,
  scaffoldStudioHostApp,
  verifyStudioHostApp,
} from "@flexweave/studio/workflows";
import { defineStudioGeneratedTarget } from "@flexweave/studio/codegen";
import { defineStudioConfig, validateStudioConfig } from "@flexweave/studio/config";
import { defineStudioExtension } from "@flexweave/studio/extensions";

import {
  copyExtensionFixture,
  copyMinimalFixture,
  extensionFixtureRoot,
  linkHostAppPackages,
} from "./support/studio-fixtures";

test("host app scaffold is idempotent and preserves consumer-owned edits", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");
  const appRoot = join(root, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(scaffolded.ok).toBe(true);
  expect(scaffolded.changedFiles.map((path) => relative(appRoot, path)).toSorted()).toEqual([
    ".flexweave-studio-app.json",
    "package.json",
    "src/main.ts",
    "src/project-adapter.ts",
    "tsconfig.json",
  ]);
  expect(scaffolded.manualFollowUps).toEqual([]);
  const metadata = JSON.parse(
    readFileSync(join(appRoot, ".flexweave-studio-app.json"), "utf-8"),
  ) as {
    managedFiles?: string[];
    packageRefs?: Record<string, string>;
    projectOwnedFiles?: string[];
    version?: number;
  };
  expect(metadata.version).toBe(2);
  expect(metadata.managedFiles).toEqual([
    ".flexweave-studio-app.json",
    "package.json",
    "src/main.ts",
    "tsconfig.json",
  ]);
  expect(metadata.projectOwnedFiles).toEqual(["src/project-adapter.ts"]);
  expect(metadata.packageRefs).toEqual({
    studio: "@flexweave/studio",
    studioApp: "@flexweave/studio-app",
  });
  expect(readFileSync(join(appRoot, "src/project-adapter.ts"), "utf-8")).toContain(
    "createDefaultStudioProjectAdapter",
  );

  const second = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(second.ok).toBe(true);
  expect(second.changedFiles).toEqual([]);
  expect(second.manualFollowUps).toEqual([]);

  const adapterPath = join(appRoot, "src/project-adapter.ts");
  writeFileSync(
    adapterPath,
    `${readFileSync(adapterPath, "utf-8")}\nexport const localAdapterCustomization = true;\n`,
  );
  const preservedAdapter = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(preservedAdapter.ok).toBe(true);
  expect(preservedAdapter.changedFiles).toEqual([]);
  expect(preservedAdapter.manualFollowUps).toEqual([]);
  expect(preservedAdapter.files).toContainEqual(
    expect.objectContaining({
      path: adapterPath,
      status: "project-owned",
    }),
  );

  const entryPath = join(appRoot, "src/main.ts");
  writeFileSync(
    entryPath,
    `${readFileSync(entryPath, "utf-8")}\nexport const localValue = true;\n`,
  );
  const preserved = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(preserved.ok).toBe(true);
  expect(preserved.changedFiles).toEqual([]);
  expect(preserved.manualFollowUps[0]).toContain("src/main.ts");
});

test("host app scaffold honors metadata-declared project-owned files", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");
  const appRoot = join(root, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(scaffolded.ok).toBe(true);

  const metadataPath = join(appRoot, ".flexweave-studio-app.json");
  const metadata = JSON.parse(readFileSync(metadataPath, "utf-8")) as {
    files: string[];
    managedFiles: string[];
    packageName: string;
    packageRefs: Record<string, string>;
    projectOwnedFiles: string[];
    scaffold: string;
    studioPackageName: string;
    version: number;
  };
  writeFileSync(
    metadataPath,
    `${JSON.stringify(
      {
        files: metadata.files,
        managedFiles: metadata.managedFiles.filter((file) => file !== "src/main.ts"),
        packageName: metadata.packageName,
        packageRefs: metadata.packageRefs,
        projectOwnedFiles: [...metadata.projectOwnedFiles, "src/main.ts"],
        scaffold: metadata.scaffold,
        studioPackageName: metadata.studioPackageName,
        version: metadata.version,
      },
      null,
      2,
    )}\n`,
  );

  const entryPath = join(appRoot, "src/main.ts");
  writeFileSync(
    entryPath,
    `${readFileSync(entryPath, "utf-8")}\nexport const atlasOwnedEntryCustomization = true;\n`,
  );

  const preserved = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(preserved.ok).toBe(true);
  expect(preserved.changedFiles).toEqual([]);
  expect(preserved.manualFollowUps).toEqual([]);
  expect(preserved.files).toContainEqual(
    expect.objectContaining({
      path: entryPath,
      status: "project-owned",
    }),
  );

  linkHostAppPackages(appRoot);
  const verified = await verifyStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(verified.ok).toBe(true);
});

test("host app scaffold composes extension contributions and verifies extension fixture", async () => {
  const root = copyExtensionFixture();
  const configPath = join(root, "studio.config.ts");
  const appRoot = join(root, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(scaffolded.ok).toBe(true);
  expect(readFileSync(join(appRoot, "src/project-adapter.ts"), "utf-8")).toContain(
    "createDefaultStudioProjectAdapter",
  );

  linkHostAppPackages(appRoot);
  const verified = await verifyStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(verified.ok).toBe(true);
  expect(verified.command?.exitCode).toBe(0);

  const moduleUrl = `${pathToFileURL(join(appRoot, "src/main.ts")).href}?${crypto.randomUUID()}`;
  const imported = (await import(moduleUrl)) as {
    app: {
      diagnosticsPanels: { id: string }[];
      generatedOutputPanels: { id: string }[];
      sourceViews: { id: string }[];
    };
  };
  expect(imported.app.generatedOutputPanels.map((panel) => panel.id)).toEqual([
    "synthetic-summary-output",
  ]);
  expect(imported.app.diagnosticsPanels.map((panel) => panel.id)).toEqual([
    "synthetic-source-diagnostics",
  ]);
  expect(imported.app.sourceViews.map((view) => view.id)).toEqual(["synthetic-table-source"]);
});

test("host app scaffold derives codegen target metadata from active generated targets", async () => {
  const root = copyMinimalFixture();
  const appRoot = join(root, "studio-host");
  const generatedTarget = defineStudioGeneratedTarget({
    id: "tags",
    label: "Consumer tags",
    plan: () => ({ files: [] }),
  });
  const validated = validateStudioConfig(
    defineStudioConfig({
      catalogRoot: "catalog",
      codegen: {
        builtInTargets: [],
        outputDirs: {
          tags: "generated/shadow-tags",
        },
      },
      extensions: [
        defineStudioExtension({
          generatedTargets: [generatedTarget],
          id: "consumer-generated-targets",
        }),
      ],
      hooks: {
        dir: "runtime-hooks",
      },
      rust: {
        flexweaveModule: "flexweave",
      },
    }),
    {
      configDir: root,
      configPath: join(root, "studio.config.ts"),
    },
  );

  expect(validated.ok).toBe(true);

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    config: validated.config,
  });

  expect(scaffolded.ok).toBe(true);
  const adapter = readFileSync(join(appRoot, "src/project-adapter.ts"), "utf-8");
  expect(adapter).toContain("createDefaultStudioProjectAdapter");
});

test("host app verification reports malformed extension app contributions", async () => {
  const verified = await verifyStudioHostApp({
    appRoot: "studio-host",
    configPath: join(extensionFixtureRoot, "malformed-app-contribution.config.ts"),
  });

  expect(verified.ok).toBe(false);
  expect(verified.diagnostics.map((diagnostic) => diagnostic.code)).toContain(
    "invalid-host-app-contribution",
  );
});

test("host app migrate and verify cover scaffold metadata and typecheck", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");
  const appRoot = join(root, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(scaffolded.ok).toBe(true);

  const metadataPath = join(appRoot, ".flexweave-studio-app.json");
  const metadata = JSON.parse(readFileSync(metadataPath, "utf-8")) as Record<string, unknown>;
  writeFileSync(metadataPath, `${JSON.stringify({ ...metadata, version: 0 }, null, 2)}\n`);

  const migrated = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  expect(migrated.ok).toBe(true);
  expect(migrated.applied).toEqual(["host app scaffold 0 -> 2"]);
  expect(migrated.changedFiles.map((path) => relative(appRoot, path))).toEqual([
    ".flexweave-studio-app.json",
  ]);

  const secondMigration = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  expect(secondMigration.ok).toBe(true);
  expect(secondMigration.applied).toEqual([]);
  expect(secondMigration.changedFiles).toEqual([]);

  const adapterPath = join(appRoot, "src/project-adapter.ts");
  const currentAdapter = readFileSync(adapterPath, "utf-8");
  writeFileSync(adapterPath, "export const projectAdapter = { legacy: true };\n");
  writeFileSync(metadataPath, `${JSON.stringify({ ...metadata, version: 0 }, null, 2)}\n`);
  const legacyMigration = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  expect(legacyMigration.ok).toBe(true);
  expect(legacyMigration.manualFollowUps[0]).toContain("src/project-adapter.ts");
  writeFileSync(adapterPath, currentAdapter);

  linkHostAppPackages(appRoot);
  const verified = await verifyStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(verified.ok).toBe(true);
  expect(verified.status).toBe("checked");
  expect(verified.command?.exitCode).toBe(0);
});

test("migrate rejects unsupported host app scaffold versions", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");
  const appRoot = join(root, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(scaffolded.ok).toBe(true);

  const metadataPath = join(appRoot, ".flexweave-studio-app.json");
  const metadata = JSON.parse(readFileSync(metadataPath, "utf-8")) as Record<string, unknown>;
  writeFileSync(metadataPath, `${JSON.stringify({ ...metadata, version: 99 }, null, 2)}\n`);

  const migrated = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  expect(migrated.ok).toBe(false);
  expect(migrated.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "unsupported-host-app-scaffold-version",
      path: metadataPath,
    }),
  );
  expect(migrated.manualFollowUps[0]).toContain("Unsupported local host app scaffold version 99");
});

test("migrate rejects unsupported host app package refs", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");
  const appRoot = join(root, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  expect(scaffolded.ok).toBe(true);

  const metadataPath = join(appRoot, ".flexweave-studio-app.json");
  const metadata = JSON.parse(readFileSync(metadataPath, "utf-8")) as {
    packageRefs: Record<string, string>;
  };
  writeFileSync(
    metadataPath,
    `${JSON.stringify(
      {
        ...metadata,
        packageRefs: {
          ...metadata.packageRefs,
          studioApp: "@flexweave/studio-app-next",
        },
      },
      null,
      2,
    )}\n`,
  );

  const migrated = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  expect(migrated.ok).toBe(false);
  expect(migrated.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "unsupported-host-app-package-ref",
      path: join(appRoot, "package.json"),
    }),
  );
  expect(migrated.manualFollowUps[0]).toContain("@flexweave/studio-app-next");
});
