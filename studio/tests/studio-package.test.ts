import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { isAbsolute, join, relative, resolve } from "node:path";
import { expect, test } from "bun:test";

import { studioCodegenTargets } from "@flexweave/studio/codegen";
import { defineStudioConfig, validateStudioConfig } from "@flexweave/studio/config";
import { findStudioConfig, loadStudioConfig } from "@flexweave/studio/config/load";
import {
  codegenStudioProject,
  describeStudioCatalog,
  listStudioCatalogRecords,
  migrateStudioProject,
  planStudioMechanic,
  scaffoldStudioMechanic,
  showStudioCatalogRecord,
  validateStudioCatalog,
  verifyStudioProject,
} from "@flexweave/studio/workflows";

const studioRoot = resolve(import.meta.dirname, "..");
const repoRoot = resolve(studioRoot, "..");
const fixtureRoot = join(studioRoot, "tests/fixtures/minimal");
const fixtureConfigPath = join(fixtureRoot, "studio.config.ts");

const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

const linkWorkspacePackage = (root: string) => {
  const scopeRoot = join(root, "node_modules/@flexweave");
  mkdirSync(scopeRoot, { recursive: true });
  const linkPath = join(scopeRoot, "studio");
  if (!existsSync(linkPath)) {
    symlinkSync(studioRoot, linkPath, "dir");
  }
};

const copyFixture = () => {
  const root = join(tmpdir(), `studio-fixture-${crypto.randomUUID()}`);
  mkdirSync(root, { recursive: true });
  cpSync(fixtureRoot, root, { recursive: true });
  linkWorkspacePackage(root);
  return root;
};

const runCli = async (args: string[], cwd = studioRoot) => {
  const proc = Bun.spawn(["bun", join(studioRoot, "src/cli/main.ts"), ...args], {
    cwd,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return { exitCode, stderr, stdout };
};

test("package metadata exposes only the Studio public contract", () => {
  const packageJson = JSON.parse(readFileSync(join(studioRoot, "package.json"), "utf-8"));

  expect(packageJson.name).toBe("@flexweave/studio");
  expect(Object.keys(packageJson.bin)).toEqual(["flexweave-studio"]);
  expect(Object.keys(packageJson.exports).toSorted()).toEqual([
    "./codegen",
    "./config",
    "./config/load",
    "./workflows",
  ]);
  expect(studioCodegenTargets).toEqual([
    "abilities",
    "effects",
    "executions",
    "modifiers",
    "reference",
    "tags",
  ]);
  expect(existsSync(join(studioRoot, "examples"))).toBe(false);
  expect(
    readdirSync(join(studioRoot, "tests/fixtures"), { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name),
  ).toEqual(["minimal"]);
});

test("config loading supports explicit paths, discovery, relative paths, and validate-only configs", async () => {
  const loaded = await loadStudioConfig({ configPath: fixtureConfigPath });

  expect(loaded.ok).toBe(true);
  expect(loaded.config?.paths.catalogRoot).toBe(join(fixtureRoot, "catalog"));
  expect(loaded.config?.paths.codegen.outputDirs.abilities).toBe(
    join(fixtureRoot, "generated/abilities"),
  );
  expect(loaded.config?.verify.commands[0]).toEqual({
    command: ["bun", "--version"],
    fast: true,
    name: "fixture command",
  });

  const nested = join(fixtureRoot, "catalog/abilities");
  expect(findStudioConfig(nested).configPath).toBe(fixtureConfigPath);
  const discovered = await loadStudioConfig({ cwd: nested });
  expect(discovered.config?.configPath).toBe(fixtureConfigPath);

  const validateOnlyRoot = join(tmpdir(), `studio-validate-only-${crypto.randomUUID()}`);
  mkdirSync(join(validateOnlyRoot, "catalog"), { recursive: true });
  linkWorkspacePackage(validateOnlyRoot);
  writeFileSync(
    join(validateOnlyRoot, "studio.config.ts"),
    [
      'import { defineStudioConfig } from "@flexweave/studio/config";',
      "export default defineStudioConfig({",
      '  mode: "validate-only",',
      '  catalogRoot: "catalog",',
      "});",
      "",
    ].join("\n"),
  );

  const validateOnly = await loadStudioConfig({
    configPath: join(validateOnlyRoot, "studio.config.ts"),
  });
  expect(validateOnly.ok).toBe(true);
  expect(validateOnly.config?.mode).toBe("validate-only");
});

test("config validation reports shape errors and duplicate owned paths", () => {
  const config = defineStudioConfig({
    catalogRoot: "catalog",
    codegen: {
      outputDirs: {
        abilities: "generated/shared",
        effects: "generated/shared",
        executions: "generated/executions",
        modifiers: "generated/modifiers",
        reference: "generated/reference",
        tags: "generated/tags",
      },
    },
    hooks: {
      dir: "runtime-hooks",
    },
    rust: {
      flexweaveModule: "flexweave",
    },
    verify: {
      commands: [
        {
          command: [],
          name: "",
        },
      ],
    },
  });

  const result = validateStudioConfig(config, {
    configDir: fixtureRoot,
    configPath: fixtureConfigPath,
  });
  expect(result.ok).toBe(false);
  expect(result.diagnostics.map((diagnostic) => diagnostic.code)).toContain("duplicate-owned-path");
  expect(result.diagnostics.map((diagnostic) => diagnostic.field)).toContain(
    "verify.commands.0.command",
  );

  const nestedConfig = defineStudioConfig({
    catalogRoot: "catalog",
    codegen: {
      outputDirs: {
        abilities: "generated/abilities",
        effects: "generated/effects",
        executions: "generated/executions",
        modifiers: "generated/modifiers",
        reference: "generated/reference",
        tags: "generated/tags",
      },
    },
    hooks: {
      dir: "generated",
    },
    rust: {
      flexweaveModule: "flexweave",
    },
  });
  const nestedResult = validateStudioConfig(nestedConfig, {
    configDir: fixtureRoot,
    configPath: fixtureConfigPath,
  });
  expect(nestedResult.ok).toBe(false);
  expect(nestedResult.diagnostics.map((diagnostic) => diagnostic.code)).toContain(
    "ambiguous-owned-path",
  );
});

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

test("codegen check detects stale, missing, and unexpected managed files without writing", async () => {
  const root = copyFixture();
  const configPath = join(root, "studio.config.ts");
  const stalePath = join(root, "generated/abilities/generated.rs");
  const missingPath = join(root, "generated/tags/generated.rs");
  const unexpectedPath = join(root, "generated/effects/unused.rs");
  const originalStale = readFileSync(stalePath, "utf-8");

  writeFileSync(stalePath, `${originalStale}// stale\n`);
  rmSync(missingPath);
  writeFileSync(
    unexpectedPath,
    '//! Generated by Flexweave Studio for effects.\n\npub const OLD: &str = "old";\n',
  );

  const result = await codegenStudioProject({ check: true, configPath });

  expect(result.ok).toBe(false);
  expect(result.diagnostics.map((diagnostic) => diagnostic.code).toSorted()).toEqual([
    "generated-missing",
    "generated-stale",
    "generated-unexpected",
  ]);
  expect(readFileSync(stalePath, "utf-8")).toBe(`${originalStale}// stale\n`);
  expect(existsSync(missingPath)).toBe(false);
  expect(existsSync(unexpectedPath)).toBe(true);
});

test("codegen writes only configured outputs, preserves hooks, and reports orphan hooks", async () => {
  const root = copyFixture();
  const configPath = join(root, "studio.config.ts");
  const hookPath = join(root, "runtime-hooks/minimal_execution.rs");
  const hookValue = "//! consumer-owned hook\n\npub fn minimal_execution() {}\n";
  const orphanPath = join(root, "runtime-hooks/orphan.rs");

  rmSync(join(root, "generated/abilities/generated.rs"));
  writeFileSync(hookPath, hookValue);
  writeFileSync(orphanPath, "//! orphan runtime hook\n");

  const result = await codegenStudioProject({ configPath });
  const loaded = await loadStudioConfig({ configPath });
  expect(loaded.ok).toBe(true);
  const outputDirs = Object.values(loaded.config?.paths.codegen.outputDirs ?? {});
  const hookDirs = [loaded.config?.paths.hooks.dir, loaded.config?.paths.hooks.testStubsDir].filter(
    (path): path is string => typeof path === "string",
  );

  expect(result.ok).toBe(true);
  expect(existsSync(join(root, "generated/abilities/generated.rs"))).toBe(true);
  expect(readFileSync(hookPath, "utf-8")).toBe(hookValue);
  expect(result.diagnostics.map((diagnostic) => diagnostic.code)).toContain("orphan-runtime-hook");
  expect(existsSync(orphanPath)).toBe(true);
  for (const target of studioCodegenTargets) {
    expect(
      result.targets
        .find((summary) => summary.target === target)
        ?.files.every((file) => outputDirs.some((outputDir) => pathContains(outputDir, file.path))),
    ).toBe(true);
  }
  expect(
    result.hooks.every((hook) => hookDirs.some((hookDir) => pathContains(hookDir, hook.path))),
  ).toBe(true);
});

test("mechanic planning and scaffolding are transactional", async () => {
  const root = copyFixture();
  const configPath = join(root, "studio.config.ts");

  const planned = await planStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "planned_mechanic",
    name: "Planned mechanic",
  });
  expect(planned.ok).toBe(true);
  expect(planned.plannedFiles).toHaveLength(6);

  const scaffolded = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "scaffolded_mechanic",
    name: "Scaffolded mechanic",
  });
  expect(scaffolded.ok).toBe(true);
  expect(existsSync(join(root, "catalog/abilities/scaffolded_mechanic.json"))).toBe(true);
  expect(existsSync(join(root, "runtime-hooks/scaffolded_mechanic_runtime_hook.rs"))).toBe(true);

  const failed = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "broken_mechanic",
    name: "Broken mechanic",
    params: { broken: true },
  });
  expect(failed.ok).toBe(false);
  expect(failed.rolledBack).toBe(true);
  expect(existsSync(join(root, "catalog/abilities/broken_mechanic.json"))).toBe(false);
});

test("verify and migrate expose stable package workflows", async () => {
  const verified = await verifyStudioProject({ configPath: fixtureConfigPath });
  expect(verified.ok).toBe(true);
  expect(verified.validation.ok).toBe(true);
  expect(verified.codegen.ok).toBe(true);
  expect(verified.commands[0]?.name).toBe("fixture command");

  const migrated = await migrateStudioProject({ configPath: fixtureConfigPath });
  expect(migrated.ok).toBe(true);
  expect(migrated.applied).toEqual([]);
});

test("CLI help and JSON output use the Studio command contract", async () => {
  const help = await runCli(["--help"]);
  expect(help.exitCode).toBe(0);
  for (const command of [
    "validate",
    "describe",
    "list",
    "show",
    "plan",
    "scaffold",
    "codegen",
    "verify",
    "migrate",
  ]) {
    expect(help.stdout).toContain(command);
  }

  const validate = await runCli(["validate", "--json", "--config", fixtureConfigPath]);
  expect(validate.exitCode).toBe(0);
  expect(JSON.parse(validate.stdout).recordCount).toBe(6);

  const missing = await runCli(["validate", "--json"], join(repoRoot, "docs"));
  expect(missing.exitCode).not.toBe(0);
  expect(JSON.parse(missing.stdout).diagnostics[0].code).toBe("missing-config");

  const badTarget = await runCli([
    "codegen",
    "--json",
    "--target",
    "unknown",
    "--config",
    fixtureConfigPath,
  ]);
  expect(badTarget.exitCode).not.toBe(0);
  expect(JSON.parse(badTarget.stdout).diagnostics[0].code).toBe("unknown-codegen-target");
});
