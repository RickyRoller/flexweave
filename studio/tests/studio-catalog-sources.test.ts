import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "bun:test";

import { loadStudioConfig } from "@flexweave/studio/config/load";
import { studioDataAdapterCanWrite } from "@flexweave/studio/extensions";
import {
  describeStudioCatalog,
  listStudioCatalogRecords,
  scaffoldStudioMechanic,
  showStudioCatalogRecord,
  validateStudioCatalog,
} from "@flexweave/studio/workflows";

import {
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
