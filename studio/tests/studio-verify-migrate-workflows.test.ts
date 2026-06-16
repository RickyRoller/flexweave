import { readFileSync, writeFileSync } from "node:fs";
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
