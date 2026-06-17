import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "bun:test";

import { loadStudioConfig } from "@flexweave/studio/config/load";
import { studioDataAdapterCanWrite } from "@flexweave/studio/extensions";
import {
  describeStudioCatalog,
  listStudioCatalogRecords,
  planStudioMechanic,
  scaffoldStudioMechanic,
  showStudioCatalogRecord,
  validateStudioCatalog,
} from "@flexweave/studio/workflows";

import {
  copyExtensionFixture,
  copyMinimalFixture,
  extensionFixtureConfigPath,
  extensionFixtureRoot,
  fixtureConfigPath,
} from "./support/studio-fixtures";

test("catalog workflows validate, describe, list, and show fixture records", async () => {
  const validation = await validateStudioCatalog({ configPath: fixtureConfigPath });
  expect(validation.ok).toBe(true);
  expect(validation.recordCount).toBe(6);

  const descriptions = await describeStudioCatalog("abilities", {
    configPath: fixtureConfigPath,
  });
  expect(descriptions.ok).toBe(true);
  expect(descriptions.descriptions[0]?.fields).toContain("effectId");

  const listed = await listStudioCatalogRecords("abilities", {
    configPath: fixtureConfigPath,
  });
  expect(listed.records).toEqual([
    {
      id: "minimal_ability",
      label: "Minimal ability",
      path: "abilities/minimal_ability.json",
    },
  ]);

  const shown = await showStudioCatalogRecord("abilities", "minimal_ability", {
    configPath: fixtureConfigPath,
  });
  expect(shown.record?.effectId).toBe("minimal_effect");
});

test("extension and data adapter contracts load file-backed and table-backed sources", async () => {
  const loaded = await loadStudioConfig({ configPath: extensionFixtureConfigPath });
  expect(loaded.ok).toBe(true);
  expect(loaded.config?.extensions.map((extension) => extension.id)).toEqual([
    "synthetic-source-extension",
  ]);
  expect(
    loaded.config?.extensions[0]?.appContributions?.map((contribution) => contribution.id),
  ).toEqual(["synthetic-host-app"]);
  expect(loaded.config?.data.sources.map((source) => source.id)).toEqual([
    "file-backed",
    "table-backed",
  ]);

  const adapters = loaded.config?.extensions.flatMap((extension) => extension.dataAdapters ?? []);
  expect(adapters?.map((adapter) => adapter.id).toSorted()).toEqual([
    "synthetic-file",
    "synthetic-table",
  ]);
  expect(adapters?.some(studioDataAdapterCanWrite)).toBe(true);

  const validation = await validateStudioCatalog({
    configPath: extensionFixtureConfigPath,
  });
  expect(validation.ok).toBe(true);
  expect(validation.recordCount).toBe(0);
  expect(validation.sourceRecordCount).toBe(2);
  expect(validation.sources).toEqual([
    {
      adapterId: "synthetic-file",
      recordCount: 1,
      sourceId: "file-backed",
    },
    {
      adapterId: "synthetic-table",
      recordCount: 1,
      sourceId: "table-backed",
    },
  ]);
});

test("extension-backed source diagnostics retain file and table provenance", async () => {
  const brokenFile = await validateStudioCatalog({
    configPath: join(extensionFixtureRoot, "broken-file.config.ts"),
  });
  expect(brokenFile.ok).toBe(false);
  expect(brokenFile.diagnostics[0]).toMatchObject({
    code: "synthetic-source-invalid",
    path: "sources/broken-file-record.json",
    source: {
      jsonPointer: "/",
      path: "sources/broken-file-record.json",
    },
  });

  const brokenTable = await validateStudioCatalog({
    configPath: join(extensionFixtureRoot, "broken-table.config.ts"),
  });
  expect(brokenTable.ok).toBe(false);
  expect(brokenTable.diagnostics[0]).toMatchObject({
    code: "synthetic-source-invalid",
    path: "synthetic-table (row 2, column 1, field id)",
    source: {
      column: 1,
      field: "id",
      row: 2,
      sheet: "synthetic-table",
    },
  });
});

test("table-backed source rows map into normalized built-in content", async () => {
  const configPath = join(extensionFixtureRoot, "table-content.config.ts");
  const validation = await validateStudioCatalog({ configPath });
  expect(validation.ok).toBe(true);
  expect(validation.recordCount).toBe(1);
  expect(validation.sourceRecordCount).toBe(1);

  const listed = await listStudioCatalogRecords("tags", { configPath });
  expect(listed.ok).toBe(true);
  expect(listed.records).toEqual([
    {
      id: "table_tag",
      label: "Table-backed tag",
      path: "synthetic-table (row 2, column 1, field id)",
    },
  ]);
});

test("extension source records with built-in raw kinds are not mapped by the built-in JSON mapper", async () => {
  const configPath = join(extensionFixtureRoot, "raw-kind-source.config.ts");
  const validation = await validateStudioCatalog({ configPath });
  expect(validation.ok).toBe(true);
  expect(validation.recordCount).toBe(0);
  expect(validation.sourceRecordCount).toBe(1);
  expect(validation.sources).toEqual([
    {
      adapterId: "synthetic-file",
      recordCount: 1,
      sourceId: "raw-kind-file",
    },
  ]);

  const listed = await listStudioCatalogRecords("tags", { configPath });
  expect(listed.ok).toBe(true);
  expect(listed.records).toEqual([]);

  const mappedConfigPath = join(extensionFixtureRoot, "raw-kind-mapped-source.config.ts");
  const mappedValidation = await validateStudioCatalog({ configPath: mappedConfigPath });
  expect(mappedValidation.ok).toBe(true);
  expect(mappedValidation.recordCount).toBe(1);
  expect(mappedValidation.sourceRecordCount).toBe(1);

  const mappedListed = await listStudioCatalogRecords("tags", { configPath: mappedConfigPath });
  expect(mappedListed.ok).toBe(true);
  expect(mappedListed.records).toEqual([
    {
      id: "extension_raw_tag",
      label: "Extension raw tag",
      path: "sources/raw-kind-file-record.json",
    },
  ]);
});

test("adapter-backed validation diagnostics keep source provenance", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");

  const abilityPath = join(root, "catalog/abilities/minimal_ability.json");
  const ability = JSON.parse(readFileSync(abilityPath, "utf-8")) as Record<string, unknown>;
  writeFileSync(abilityPath, `${JSON.stringify({ ...ability, effectId: "missing_effect" })}\n`);

  const executionPath = join(root, "catalog/executions/minimal_execution.json");
  const execution = JSON.parse(readFileSync(executionPath, "utf-8")) as Record<string, unknown>;
  writeFileSync(executionPath, `${JSON.stringify({ ...execution, hook: "" })}\n`);

  const modifierPath = join(root, "catalog/modifiers/minimal_modifier.json");
  const modifier = JSON.parse(readFileSync(modifierPath, "utf-8")) as Record<string, unknown>;
  writeFileSync(modifierPath, `${JSON.stringify({ ...modifier, value: "not-a-number" })}\n`);

  writeFileSync(
    join(root, "catalog/tags/minimal_tag_duplicate.json"),
    `${JSON.stringify({ id: "minimal_tag", kind: "tag", label: "Duplicate tag" })}\n`,
  );

  const validation = await validateStudioCatalog({ configPath });
  expect(validation.ok).toBe(false);
  expect(validation.diagnostics.map((diagnostic) => diagnostic.code)).toEqual(
    expect.arrayContaining([
      "duplicate-record",
      "invalid-record-field",
      "missing-record-reference",
      "missing-runtime-hook",
    ]),
  );

  for (const code of [
    "duplicate-record",
    "invalid-record-field",
    "missing-record-reference",
    "missing-runtime-hook",
  ]) {
    const diagnostic = validation.diagnostics.find((candidate) => candidate.code === code);
    expect(typeof diagnostic?.path).toBe("string");
    expect(diagnostic?.source).toBeDefined();
  }
});

test("extension loading reports malformed declarations and missing adapters", async () => {
  const malformed = await loadStudioConfig({
    configPath: join(extensionFixtureRoot, "malformed-extension.config.ts"),
  });
  expect(malformed.ok).toBe(false);
  expect(malformed.diagnostics.map((diagnostic) => diagnostic.code)).toContain(
    "invalid-data-adapter",
  );
  expect(malformed.diagnostics.map((diagnostic) => diagnostic.field)).toContain(
    "extensions.0.dataAdapters.0.load",
  );

  const missingAdapter = await loadStudioConfig({
    configPath: join(extensionFixtureRoot, "missing-adapter.config.ts"),
  });
  expect(missingAdapter.ok).toBe(false);
  expect(missingAdapter.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "missing-data-adapter",
      field: "data.sources.0.adapterId",
    }),
  );

  const duplicateProjectExtensionAdapter = await loadStudioConfig({
    configPath: join(extensionFixtureRoot, "duplicate-project-extension-adapter.config.ts"),
  });
  expect(duplicateProjectExtensionAdapter.ok).toBe(false);
  expect(duplicateProjectExtensionAdapter.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "duplicate-data-adapter",
      field: "extensions.0.dataAdapters.0.id",
    }),
  );

  const duplicateExtensionAdapter = await loadStudioConfig({
    configPath: join(extensionFixtureRoot, "duplicate-extension-adapter.config.ts"),
  });
  expect(duplicateExtensionAdapter.ok).toBe(false);
  expect(duplicateExtensionAdapter.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "duplicate-data-adapter",
      field: "extensions.1.dataAdapters.0.id",
    }),
  );

  const malformedAppContribution = await loadStudioConfig({
    configPath: join(extensionFixtureRoot, "malformed-app-contribution.config.ts"),
  });
  expect(malformedAppContribution.ok).toBe(false);
  expect(malformedAppContribution.diagnostics.map((diagnostic) => diagnostic.code)).toContain(
    "invalid-host-app-contribution",
  );
});

test("scaffold rejects source configurations without a writable content adapter", async () => {
  const configPath = join(extensionFixtureRoot, "read-only-content.config.ts");
  const result = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "source_backed_scaffold",
    name: "Source backed scaffold",
  });

  expect(result.ok).toBe(false);
  expect(result.rolledBack).toBe(true);
  expect(result.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "source-write-unsupported",
    }),
  );
  expect(
    existsSync(join(extensionFixtureRoot, "catalog/abilities/source_backed_scaffold.json")),
  ).toBe(false);
});

test("planning rejects source configurations without a writable content adapter", async () => {
  const configPath = join(extensionFixtureRoot, "read-only-content.config.ts");
  const result = await planStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "source_backed_plan",
    name: "Source backed plan",
  });

  expect(result.ok).toBe(false);
  expect(result.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "source-write-unsupported",
    }),
  );
  expect(existsSync(join(extensionFixtureRoot, "catalog/abilities/source_backed_plan.json"))).toBe(
    false,
  );
});

test("planning uses built-in JSON when writable sources are not explicit write targets", async () => {
  const configPath = join(extensionFixtureRoot, "ambiguous-writable-content.config.ts");
  const result = await planStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "json_backed_plan",
    name: "JSON backed plan",
  });

  expect(result.ok).toBe(true);
  expect(result.diagnostics.filter((diagnostic) => diagnostic.severity === "error")).toEqual([]);
  expect(result.plannedFiles).toHaveLength(6);
  expect(result.plannedFiles).toContain("catalog/abilities/json_backed_plan.json");
  expect(existsSync(join(extensionFixtureRoot, "catalog/abilities/json_backed_plan.json"))).toBe(
    false,
  );
});

test("scaffold defaults to JSON catalog writes when sources have no write target", async () => {
  const root = copyExtensionFixture();
  const configPath = join(root, "json-write-with-sources.config.ts");
  const sourcePath = join(root, "sources/writable-table.json");
  const result = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "json_backed_scaffold",
    name: "JSON backed scaffold",
  });

  expect(result.ok).toBe(true);
  expect(result.rolledBack).toBe(false);
  expect(result.diagnostics.filter((diagnostic) => diagnostic.severity === "error")).toEqual([]);
  expect(result.writtenFiles).toContain("catalog/abilities/json_backed_scaffold.json");
  expect(result.writtenFiles).toContain("runtime-hooks/json_backed_scaffold_runtime_hook.rs");
  expect(existsSync(join(root, "catalog/abilities/json_backed_scaffold.json"))).toBe(true);
  expect(JSON.parse(readFileSync(sourcePath, "utf-8"))).toEqual([]);
});

test("scaffold writes mechanics through a writable source adapter", async () => {
  const root = copyExtensionFixture();
  const configPath = join(root, "writable-content.config.ts");
  const result = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "source_backed_scaffold",
    name: "Source backed scaffold",
  });

  expect(result.ok).toBe(true);
  expect(result.rolledBack).toBe(false);
  expect(result.diagnostics.filter((diagnostic) => diagnostic.severity === "error")).toEqual([]);
  expect(result.writtenFiles.filter((path) => path.startsWith("synthetic-table"))).toHaveLength(6);
  expect(result.writtenFiles).toContain("runtime-hooks/source_backed_scaffold_runtime_hook.rs");
  expect(existsSync(join(root, "catalog/abilities/source_backed_scaffold.json"))).toBe(false);
  expect(existsSync(join(root, "catalog/mechanics/source_backed_scaffold.json"))).toBe(false);

  const validation = await validateStudioCatalog({ configPath });
  expect(validation.ok).toBe(true);
  expect(validation.recordCount).toBe(6);
});

test("source-backed scaffold restores adapter snapshots on validation failure", async () => {
  const root = copyExtensionFixture();
  const configPath = join(root, "writable-content.config.ts");
  const sourcePath = join(root, "sources/writable-table.json");
  const result = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "broken_source_backed_scaffold",
    name: "Broken source backed scaffold",
    params: { broken: true },
  });

  expect(result.ok).toBe(false);
  expect(result.rolledBack).toBe(true);
  expect(result.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "missing-record-reference",
    }),
  );
  expect(JSON.parse(readFileSync(sourcePath, "utf-8"))).toEqual([]);
  expect(existsSync(join(root, "catalog/abilities/broken_source_backed_scaffold.json"))).toBe(
    false,
  );
});
