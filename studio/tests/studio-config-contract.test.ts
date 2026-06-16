import { mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";

import { defineStudioConfig, validateStudioConfig } from "@flexweave/studio/config";
import { findStudioConfig, loadStudioConfig } from "@flexweave/studio/config/load";

import { fixtureConfigPath, fixtureRoot, linkWorkspacePackage } from "./support/studio-fixtures";

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
