import { expect, test } from "bun:test";

import { validateStudioConfig } from "@flexweave/studio/config";

const configOptions = {
  configDir: "/workspace/project",
  configPath: "/workspace/project/studio.config.ts",
};

const validFullConfig = () => ({
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
    dir: "runtime/hooks",
    testStubsDir: "runtime/test-stubs",
  },
  rust: {
    flexweaveModule: "flexweave",
  },
});

test("config validation rejects invalid runtime vocabulary", () => {
  const result = validateStudioConfig(
    {
      ...validFullConfig(),
      rust: {
        flexweaveModule: "flexweave",
        runtimeVocab: {
          ailments: ["burning", ""],
          damageTypes: "fire",
        },
      },
    },
    configOptions,
  );

  expect(result.ok).toBe(false);
  expect(result.config).toBeUndefined();
  expect(result.diagnostics.map((diagnostic) => diagnostic.field)).toEqual(
    expect.arrayContaining(["rust.runtimeVocab.ailments.1", "rust.runtimeVocab.damageTypes"]),
  );
});

test("config validation rejects invalid verify shape", () => {
  const nonObjectVerify = validateStudioConfig(
    {
      ...validFullConfig(),
      verify: "bun test",
    },
    configOptions,
  );

  expect(nonObjectVerify.ok).toBe(false);
  expect(nonObjectVerify.config).toBeUndefined();
  expect(nonObjectVerify.diagnostics.map((diagnostic) => diagnostic.field)).toContain("verify");

  const invalidCommands = validateStudioConfig(
    {
      ...validFullConfig(),
      verify: {
        commands: "bun test",
      },
    },
    configOptions,
  );

  expect(invalidCommands.ok).toBe(false);
  expect(invalidCommands.config).toBeUndefined();
  expect(invalidCommands.diagnostics.map((diagnostic) => diagnostic.field)).toContain(
    "verify.commands",
  );
});

test("config validation preserves validate-only support", () => {
  const result = validateStudioConfig(
    {
      catalogRoot: "catalog",
      mode: "validate-only",
    },
    configOptions,
  );

  expect(result.ok).toBe(true);
  expect(result.config?.mode).toBe("validate-only");
  expect(result.config?.paths.catalogRoot).toBe("/workspace/project/catalog");
  expect(result.config?.verify.commands).toEqual([]);
});
