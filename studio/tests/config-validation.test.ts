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

test("config validation resolves generic Rust codegen context", () => {
  const result = validateStudioConfig(
    {
      ...validFullConfig(),
      rust: {
        bindings: {
          synthetic: {
            module: "synthetic_runtime",
          },
        },
        flexweaveModule: "flexweave",
        generatedHeader: "//! Generated {target}",
        macroNames: {
          tag: "tag_ref",
        },
        moduleAliases: {
          core: "flexweave",
        },
        preludeImports: ["core::fmt::Debug"],
        runtimeVocab: {
          ailments: ["burning"],
          damageTypes: ["fire"],
        },
        typePaths: {
          objectId: "flexweave::ObjectId",
        },
      },
    },
    configOptions,
  );

  expect(result.ok).toBe(true);
  expect(result.config?.rust).toMatchObject({
    bindings: {
      synthetic: {
        module: "synthetic_runtime",
      },
    },
    generatedHeader: "//! Generated {target}",
    macroNames: {
      tag: "tag_ref",
    },
    moduleAliases: {
      core: "flexweave",
    },
    preludeImports: ["core::fmt::Debug"],
    typePaths: {
      objectId: "flexweave::ObjectId",
    },
  });
});

test("config validation rejects invalid generic Rust codegen context", () => {
  const result = validateStudioConfig(
    {
      ...validFullConfig(),
      rust: {
        bindings: [],
        flexweaveModule: "flexweave",
        generatedHeader: "",
        macroNames: {
          tag: "",
        },
        preludeImports: [""],
        typePaths: "ObjectId",
      },
    },
    configOptions,
  );

  expect(result.ok).toBe(false);
  expect(result.diagnostics.map((diagnostic) => diagnostic.field)).toEqual(
    expect.arrayContaining([
      "rust.bindings",
      "rust.generatedHeader",
      "rust.macroNames.tag",
      "rust.preludeImports.0",
      "rust.typePaths",
    ]),
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

test("config validation rejects writable data adapters without rollback snapshots", () => {
  const result = validateStudioConfig(
    {
      ...validFullConfig(),
      data: {
        adapters: [
          {
            capabilities: ["read", "write"],
            id: "writable-without-snapshots",
            load: () => ({ records: [] }),
            write: () => ({ records: [] }),
          },
        ],
      },
    },
    configOptions,
  );

  expect(result.ok).toBe(false);
  expect(result.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "invalid-data-adapter",
      field: "data.adapters.0.writeSnapshotPaths",
    }),
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

test("config validation supports extension-only generated target sets", () => {
  const generatedTarget = {
    id: "abilities",
    label: "Consumer abilities",
    plan: () => ({ files: [] }),
  };
  const result = validateStudioConfig(
    {
      ...validFullConfig(),
      codegen: {
        allowOverlappingOutputDirs: true,
        builtInTargets: [],
        outputDirs: {
          abilities: "generated",
          effects: "generated/effects",
        },
      },
      extensions: [
        {
          generatedTargets: [
            generatedTarget,
            {
              id: "effects",
              label: "Consumer effects",
              plan: () => ({ files: [] }),
            },
          ],
          id: "consumer-generated-targets",
        },
      ],
      hooks: {
        dir: "generated/hooks",
      },
    },
    configOptions,
  );

  expect(result.ok).toBe(true);
  expect(result.config?.codegen).toEqual({
    allowOverlappingOutputDirs: true,
    builtInTargets: [],
  });
  expect(result.config?.paths.codegen.outputDirs.abilities).toBe("/workspace/project/generated");

  const activeBuiltInShadow = validateStudioConfig(
    {
      ...validFullConfig(),
      extensions: [
        {
          generatedTargets: [generatedTarget],
          id: "consumer-generated-targets",
        },
      ],
    },
    configOptions,
  );

  expect(activeBuiltInShadow.ok).toBe(false);
  expect(activeBuiltInShadow.diagnostics).toContainEqual(
    expect.objectContaining({
      code: "duplicate-generated-target",
    }),
  );
});

test("config validation resolves local host app metadata", () => {
  const result = validateStudioConfig(
    {
      ...validFullConfig(),
      app: {
        buildCommand: ["bun", "run", "build"],
        checkCommand: ["bun", "run", "typecheck"],
        root: "studio-host",
      },
    },
    configOptions,
  );

  expect(result.ok).toBe(true);
  expect(result.config?.paths.app.root).toBe("/workspace/project/studio-host");
  expect(result.config?.app.checkCommand).toEqual(["bun", "run", "typecheck"]);

  const invalid = validateStudioConfig(
    {
      ...validFullConfig(),
      app: {
        checkCommand: [],
      },
    },
    configOptions,
  );

  expect(invalid.ok).toBe(false);
  expect(invalid.diagnostics.map((diagnostic) => diagnostic.field)).toContain("app.checkCommand");
});
