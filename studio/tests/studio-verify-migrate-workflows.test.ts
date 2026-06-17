import { randomUUID } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";

import {
  codegenStudioProject,
  migrateStudioProject,
  verifyStudioProject,
} from "@flexweave/studio/workflows";

import {
  copyExtensionFixture,
  copyFixtureTree,
  copyMinimalFixture,
  extensionFixtureRoot,
  fixtureConfigPath,
  linkWorkspacePackage,
} from "./support/studio-fixtures";

test("extension-owned migrations are explicit, idempotent, and reject unsupported versions", async () => {
  const root = copyExtensionFixture();
  const configPath = join(root, "studio.config.ts");
  const statePath = join(root, "sources/migration-state.json");

  const migrated = await migrateStudioProject({ configPath });
  expect(migrated.ok).toBe(true);
  expect(migrated.applied).toEqual(["synthetic-source-extension: synthetic-source-schema 0 -> 1"]);
  expect(migrated.changedFiles).toEqual([statePath]);
  expect(migrated.checks).toContainEqual(
    expect.objectContaining({
      extensionId: "synthetic-source-extension",
      name: "extension:synthetic-source-extension:synthetic-source-schema",
      status: "applied",
    }),
  );
  expect(JSON.parse(readFileSync(statePath, "utf-8"))).toMatchObject({ version: 1 });

  const second = await migrateStudioProject({ configPath });
  expect(second.ok).toBe(true);
  expect(second.applied).toEqual([]);
  expect(second.changedFiles).toEqual([]);
  expect(second.skipped).toContain("Synthetic source schema is current.");

  writeFileSync(statePath, `${JSON.stringify({ version: 99 }, null, 2)}\n`);
  const unsupported = await migrateStudioProject({ configPath });
  expect(unsupported.ok).toBe(false);
  expect(unsupported.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "unsupported-extension-migration",
      path: statePath,
    }),
  );
  expect(unsupported.manualFollowUps[0]).toContain(
    "Unsupported synthetic source schema version 99",
  );
});

test("verify reports extension-aware checks for fast, full, stale, adapter, and command failures", async () => {
  const fixtureTreeRoot = copyFixtureTree();
  const generatedRoot = join(fixtureTreeRoot, "minimal");
  const generatedConfigPath = join(generatedRoot, "generated-target.config.ts");
  const refreshed = await codegenStudioProject({ configPath: generatedConfigPath });
  expect(refreshed.ok).toBe(true);

  const full = await verifyStudioProject({ configPath: generatedConfigPath });
  expect(full.ok).toBe(true);
  expect(full.checks).toContainEqual(
    expect.objectContaining({
      name: "extension:synthetic-source-extension",
      status: "passed",
    }),
  );
  expect(full.checks).toContainEqual(
    expect.objectContaining({
      name: "generated-target:synthetic-summary",
      status: "passed",
      targetId: "synthetic-summary",
    }),
  );

  const fastRoot = copyMinimalFixture();
  const fastConfigPath = join(fastRoot, "studio.config.ts");
  const fastConfig = readFileSync(fastConfigPath, "utf-8").replace(
    "commands: [",
    [
      "commands: [",
      "      {",
      '        command: ["bun", "--version"],',
      "        fast: false,",
      '        name: "slow fixture command",',
      "      },",
    ].join("\n"),
  );
  writeFileSync(fastConfigPath, fastConfig);
  const fast = await verifyStudioProject({ configPath: fastConfigPath, fast: true });
  expect(fast.ok).toBe(true);
  expect(fast.commands.map((command) => command.name)).toEqual(["fixture command"]);
  expect(fast.checks.every((check) => check.mode === "fast")).toBe(true);

  const fullWithSlow = await verifyStudioProject({ configPath: fastConfigPath });
  expect(fullWithSlow.commands.map((command) => command.name)).toEqual([
    "slow fixture command",
    "fixture command",
  ]);

  writeFileSync(join(generatedRoot, "generated/synthetic/summary.txt"), "stale\n");
  const stale = await verifyStudioProject({ configPath: generatedConfigPath });
  expect(stale.ok).toBe(false);
  expect(stale.checks).toContainEqual(
    expect.objectContaining({
      name: "generated-target:synthetic-summary",
      status: "failed",
      targetId: "synthetic-summary",
    }),
  );

  const adapterFailure = await verifyStudioProject({
    configPath: join(extensionFixtureRoot, "adapter-failure.config.ts"),
  });
  expect(adapterFailure.ok).toBe(false);
  expect(adapterFailure.checks).toContainEqual(
    expect.objectContaining({
      adapterId: "synthetic-file",
      name: "source:missing-file-source",
      sourceId: "missing-file-source",
      status: "failed",
    }),
  );

  const commandRoot = copyMinimalFixture();
  const commandConfigPath = join(commandRoot, "studio.config.ts");
  writeFileSync(
    commandConfigPath,
    readFileSync(commandConfigPath, "utf-8").replace(
      '["bun", "--version"]',
      '["bun", "-e", "process.exit(7)"]',
    ),
  );
  const commandFailure = await verifyStudioProject({ configPath: commandConfigPath });
  expect(commandFailure.ok).toBe(false);
  expect(commandFailure.checks).toContainEqual(
    expect.objectContaining({
      command: ["bun", "-e", "process.exit(7)"],
      exitCode: 7,
      name: "command:fixture command",
      status: "failed",
    }),
  );
});

test("verify attributes source and mapper diagnostics without substring ownership", async () => {
  const sourceOwned = await verifyStudioProject({
    configPath: join(extensionFixtureRoot, "broken-file.config.ts"),
  });
  const sourceCheck = sourceOwned.checks.find((check) => check.name === "source:file-backed");
  expect(sourceCheck).toMatchObject({
    adapterId: "synthetic-file",
    sourceId: "file-backed",
    status: "failed",
  });
  expect(sourceCheck?.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "synthetic-source-invalid",
      path: "sources/broken-file-record.json",
    }),
  );
  expect(
    sourceCheck?.diagnostics.some(
      (diagnostic) =>
        diagnostic.message.includes("file-backed") ||
        diagnostic.message.includes("synthetic-file") ||
        diagnostic.path?.includes("file-backed") ||
        diagnostic.path?.includes("synthetic-file") ||
        diagnostic.field?.includes("file-backed") ||
        diagnostic.field?.includes("synthetic-file"),
    ),
  ).toBe(false);

  const mapperRoot = join(tmpdir(), `studio-mapper-attribution-${randomUUID()}`);
  mkdirSync(join(mapperRoot, "catalog"), { recursive: true });
  linkWorkspacePackage(mapperRoot);
  const mapperConfigPath = join(mapperRoot, "studio.config.ts");
  writeFileSync(
    mapperConfigPath,
    [
      'import { defineStudioConfig } from "@flexweave/studio/config";',
      'import { defineStudioContentMapper, defineStudioExtension } from "@flexweave/studio/extensions";',
      "",
      "const structuredAttributionMapper = defineStudioContentMapper({",
      '  id: "structured-attribution-mapper",',
      "  map: () => ({",
      "    diagnostics: [",
      "      {",
      '        code: "detached-mapping-diagnostic",',
      '        message: "Mapped content is not usable.",',
      '        path: "sourceless-output.json",',
      '        severity: "error",',
      "      },",
      "    ],",
      "    records: [],",
      "  }),",
      "});",
      "",
      "export default defineStudioConfig({",
      '  catalogRoot: "catalog",',
      "  extensions: [",
      "    defineStudioExtension({",
      "      contentMappers: [structuredAttributionMapper],",
      '      id: "structured-attribution-extension",',
      "    }),",
      "  ],",
      '  mode: "validate-only",',
      "});",
      "",
    ].join("\n"),
  );

  const mapperOwned = await verifyStudioProject({ configPath: mapperConfigPath });
  const mapperCheck = mapperOwned.checks.find(
    (check) => check.name === "mapper:structured-attribution-mapper",
  );
  expect(mapperCheck).toMatchObject({
    extensionId: "structured-attribution-extension",
    status: "failed",
  });
  expect(mapperCheck?.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "detached-mapping-diagnostic",
      message: "Mapped content is not usable.",
      path: "sourceless-output.json",
    }),
  );
  expect(
    mapperCheck?.diagnostics.some(
      (diagnostic) =>
        diagnostic.message.includes("structured-attribution-mapper") ||
        diagnostic.path?.includes("structured-attribution-mapper") ||
        diagnostic.field?.includes("structured-attribution-mapper"),
    ),
  ).toBe(false);
});

test("verify reuses one loaded catalog for validation and codegen checks", async () => {
  const root = join(tmpdir(), `studio-single-catalog-verify-${randomUUID()}`);
  mkdirSync(join(root, "catalog"), { recursive: true });
  mkdirSync(join(root, "generated/single-load"), { recursive: true });
  mkdirSync(join(root, "runtime-hooks"), { recursive: true });
  linkWorkspacePackage(root);

  const configPath = join(root, "studio.config.ts");
  const counterPath = join(root, "source-load-count.txt");
  const summaryPath = join(root, "generated/single-load/summary.txt");
  const expectedSummary = [
    "Generated by Flexweave Studio volatile summary.",
    "Records: load-1",
    "",
  ].join("\n");
  writeFileSync(summaryPath, expectedSummary);
  writeFileSync(
    configPath,
    [
      'import { existsSync, readFileSync, writeFileSync } from "node:fs";',
      'import { join } from "node:path";',
      'import { defineStudioGeneratedTarget } from "@flexweave/studio/codegen";',
      'import { defineStudioConfig } from "@flexweave/studio/config";',
      'import { defineStudioContentMapper, defineStudioDataAdapter, defineStudioExtension } from "@flexweave/studio/extensions";',
      "",
      "const volatileAdapter = defineStudioDataAdapter({",
      '  capabilities: ["read"],',
      '  id: "volatile-source",',
      "  load: ({ config }) => {",
      '    const counterPath = join(config.configDir, "source-load-count.txt");',
      '    const current = existsSync(counterPath) ? Number(readFileSync(counterPath, "utf-8").trim()) : 0;',
      "    const next = current + 1;",
      `    writeFileSync(counterPath, \`\${next}\\n\`);`,
      "    return {",
      "      records: [",
      "        {",
      `          id: \`load-\${next}\`,`,
      '          kind: "volatile.raw",',
      `          value: { id: \`load-\${next}\`, label: \`Load \${next}\` },`,
      "        },",
      "      ],",
      "    };",
      "  },",
      "});",
      "",
      "const volatileMapper = defineStudioContentMapper({",
      '  id: "volatile-mapper",',
      "  map: ({ snapshots }) => ({",
      "    records: snapshots.flatMap((snapshot) =>",
      "      snapshot.records",
      '        .filter((record) => record.kind === "volatile.raw")',
      "        .map((record) => ({",
      '          expectedKind: "tags",',
      "          path: record.id,",
      "          sourceRecord: record,",
      "          value: {",
      "            id: record.id,",
      '            kind: "tag",',
      "            label: record.value.label,",
      "          },",
      "        })),",
      "    ),",
      "  }),",
      "});",
      "",
      "const volatileSummaryTarget = defineStudioGeneratedTarget({",
      '  cleanup: "managed-files",',
      '  id: "single-load-summary",',
      '  label: "Single-load summary",',
      "  plan: ({ content, outputDir }) => ({",
      "    files: [",
      "      {",
      '        path: join(outputDir, "summary.txt"),',
      "        value: [",
      '          "Generated by Flexweave Studio volatile summary.",',
      `          \`Records: \${content.records.map((record) => record.id).join(", ")}\`,`,
      '          "",',
      '        ].join("\\n"),',
      "      },",
      "    ],",
      "  }),",
      "});",
      "",
      "export default defineStudioConfig({",
      '  catalogRoot: "catalog",',
      "  codegen: {",
      "    builtInTargets: [],",
      "    outputDirs: {",
      '      "single-load-summary": "generated/single-load",',
      "    },",
      "  },",
      "  data: {",
      "    sources: [",
      "      {",
      '        adapterId: "volatile-source",',
      '        id: "volatile",',
      "      },",
      "    ],",
      "  },",
      "  extensions: [",
      "    defineStudioExtension({",
      "      contentMappers: [volatileMapper],",
      "      dataAdapters: [volatileAdapter],",
      "      generatedTargets: [volatileSummaryTarget],",
      '      id: "volatile-extension",',
      "    }),",
      "  ],",
      "  hooks: {",
      '    dir: "runtime-hooks",',
      "  },",
      '  mode: "full",',
      "  rust: {",
      '    flexweaveModule: "flexweave",',
      "  },",
      "  verify: {",
      "    commands: [],",
      "  },",
      "});",
      "",
    ].join("\n"),
  );

  const verified = await verifyStudioProject({ configPath });

  expect(verified.ok).toBe(true);
  expect(readFileSync(counterPath, "utf-8")).toBe("1\n");
  expect(verified.checks).toContainEqual(
    expect.objectContaining({
      name: "generated-target:single-load-summary",
      status: "passed",
    }),
  );
  expect(readFileSync(summaryPath, "utf-8")).toBe(expectedSummary);
});

test("verify and migrate expose stable package workflows", async () => {
  const verified = await verifyStudioProject({ configPath: fixtureConfigPath });
  expect(verified.ok).toBe(true);
  expect(verified.validation.ok).toBe(true);
  expect(verified.codegen.ok).toBe(true);
  expect(verified.commands[0]?.name).toBe("fixture command");
  expect(verified.hostApp.status).toBe("not-configured");
  expect(verified.checks).toContainEqual(
    expect.objectContaining({
      name: "host-app",
      status: "skipped",
    }),
  );

  const migrated = await migrateStudioProject({ configPath: fixtureConfigPath });
  expect(migrated.ok).toBe(true);
  expect(migrated.applied).toEqual([]);
  expect(migrated.changedFiles).toEqual([]);
});
