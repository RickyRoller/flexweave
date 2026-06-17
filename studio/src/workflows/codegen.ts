import { existsSync, rmSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";

import type {
  RuntimeHookSummary,
  StudioCodegenTargetSummary,
  StudioGeneratedFileDiff,
  StudioGeneratedTargetId,
} from "../codegen/types";
import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import { loadStudioCatalog } from "../internal/catalog";
import type { StudioCatalog } from "../internal/catalog";
import {
  displayPath,
  listFilesRecursive,
  readTextIfExists,
  restoreSnapshots,
  snapshotPaths,
  writeTextFile,
} from "../internal/files";
import {
  fullConfigRequired,
  resolveWorkflowConfig,
  workflowError,
  workflowWarning,
} from "./shared";
import type { CodegenStudioResult, StudioWorkflowOptions } from "./types";
import {
  activeGeneratedTargets,
  defaultSelectedGeneratedTargets,
} from "./generated-target-registry";
import type { RegisteredGeneratedTarget, StudioCatalogContent } from "./generated-target-registry";

const rustIdentifier = (value: string) =>
  value
    .toLowerCase()
    .replaceAll(/[^a-z0-9_]+/g, "_")
    .replace(/^([0-9])/, "_$1")
    .replaceAll(/_+/g, "_")
    .replaceAll(/^_|_$/g, "");

interface PlannedGeneratedFile {
  path: string;
  target: StudioGeneratedTargetId;
  value: string;
}

interface PlannedRuntimeHookFile {
  hook: string;
  path: string;
  value: string;
}

interface CodegenStudioProjectOptions {
  check?: boolean;
  targets?: readonly string[];
}

const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

const selectTargets = (
  registeredTargets: RegisteredGeneratedTarget[],
  defaultTargetIds: readonly string[],
  requestedTargets?: readonly string[],
): { diagnostics: StudioDiagnostic[]; targets: RegisteredGeneratedTarget[] } => {
  const byId: Record<string, RegisteredGeneratedTarget | undefined> = {};
  for (const target of registeredTargets) {
    byId[target.id] = target;
  }

  const availableTargetIds = registeredTargets.map((target) => target.id);
  const rootTargetIds =
    requestedTargets && requestedTargets.length > 0 ? [...requestedTargets] : [...defaultTargetIds];
  const diagnostics: StudioDiagnostic[] = [];
  const orderedTargets: RegisteredGeneratedTarget[] = [];
  const visiting = new Set<string>();
  const visited = new Set<string>();

  const visit = (targetId: string, dependencyOf?: string) => {
    const target = byId[targetId];
    if (!target) {
      diagnostics.push(
        workflowError(
          dependencyOf ? "missing-generated-target-dependency" : "unknown-codegen-target",
          dependencyOf
            ? `Generated target "${dependencyOf}" depends on missing target "${targetId}".`
            : `Unknown Studio generated output target "${targetId}".`,
          undefined,
          `Expected one of: ${availableTargetIds.join(", ")}.`,
        ),
      );
      return;
    }

    if (visited.has(target.id)) {
      return;
    }

    if (visiting.has(target.id)) {
      diagnostics.push(
        workflowError(
          "generated-target-cycle",
          `Generated target dependency cycle includes "${target.id}".`,
        ),
      );
      return;
    }

    visiting.add(target.id);
    for (const dependency of target.dependencies ?? []) {
      visit(dependency, target.id);
    }
    visiting.delete(target.id);
    visited.add(target.id);
    orderedTargets.push(target);
  };

  for (const targetId of rootTargetIds) {
    visit(targetId);
  }

  return { diagnostics, targets: orderedTargets };
};

const plannedGeneratedFiles = async (
  config: ResolvedStudioProjectConfig,
  catalog: StudioCatalogContent,
  targets: RegisteredGeneratedTarget[],
): Promise<{ diagnostics: StudioDiagnostic[]; files: PlannedGeneratedFile[] }> => {
  const diagnostics: StudioDiagnostic[] = [];
  const files: PlannedGeneratedFile[] = [];

  for (const target of targets) {
    const outputDir = config.paths.codegen.outputDirs[target.id];
    if (!outputDir) {
      diagnostics.push(
        workflowError(
          "missing-generated-output-root",
          `Generated target "${target.id}" does not have a configured output directory.`,
          config.configPath,
          `Add codegen.outputDirs.${target.id} to the Studio project config.`,
        ),
      );
      continue;
    }

    try {
      const result = await target.plan({
        config,
        content: catalog,
        outputDir,
        targetId: target.id,
      });
      diagnostics.push(...((result.diagnostics ?? []) as StudioDiagnostic[]));
      for (const file of result.files) {
        if (!pathContains(outputDir, file.path)) {
          diagnostics.push(
            workflowError(
              "generated-output-out-of-bounds",
              `Generated target "${target.id}" planned a file outside its configured output directory.`,
              file.path,
              `Target output directory: ${outputDir}`,
            ),
          );
          continue;
        }
        files.push({ path: file.path, target: target.id, value: file.value });
      }
    } catch (error) {
      diagnostics.push(
        workflowError(
          "generated-target-plan-failed",
          error instanceof Error
            ? `Generated target "${target.id}" failed to plan files: ${error.message}`
            : `Generated target "${target.id}" failed to plan files.`,
          config.configPath,
        ),
      );
    }
  }

  return { diagnostics, files };
};

const managedFileHeader = "Generated by Flexweave Studio";

const detectUnexpectedManagedFiles = (
  config: ResolvedStudioProjectConfig,
  expected: PlannedGeneratedFile[],
  targets: RegisteredGeneratedTarget[],
): StudioGeneratedFileDiff[] => {
  const expectedPaths = new Set(expected.map((file) => file.path));
  const diffs: StudioGeneratedFileDiff[] = [];

  for (const target of targets) {
    if (target.cleanup === "none") {
      continue;
    }
    const outputDir = config.paths.codegen.outputDirs[target.id];
    if (!outputDir) {
      continue;
    }
    for (const path of listFilesRecursive(outputDir)) {
      if (expectedPaths.has(path)) {
        continue;
      }
      const value = readTextIfExists(path);
      if (value?.includes(managedFileHeader)) {
        diffs.push({ path, status: "unexpected", target: target.id });
      }
    }
  }

  return diffs;
};

const summarizeGeneratedFiles = (
  config: ResolvedStudioProjectConfig,
  expected: PlannedGeneratedFile[],
  targets: RegisteredGeneratedTarget[],
): StudioGeneratedFileDiff[] => {
  const diffs = expected.map((file): StudioGeneratedFileDiff => {
    const current = readTextIfExists(file.path);
    if (current === undefined) {
      return { path: file.path, status: "missing", target: file.target };
    }
    if (current !== file.value) {
      return { path: file.path, status: "stale", target: file.target };
    }
    return { path: file.path, status: "fresh", target: file.target };
  });

  return [...diffs, ...detectUnexpectedManagedFiles(config, expected, targets)];
};

const hookFileName = (hook: string) => `${rustIdentifier(hook)}.rs`;

const hookModuleIndexFiles = new Set(["lib.rs", "mod.rs"]);

const hookStub = (hook: string) =>
  `//! Runtime hook stub created by Flexweave Studio. Consumer-owned after creation.\n\npub fn ${rustIdentifier(hook)}() {}\n`;

const hookTestStub = (hook: string) =>
  `//! Runtime hook test stub created by Flexweave Studio.\n\n#[test]\nfn ${rustIdentifier(hook)}_is_declared() {\n    assert!(true);\n}\n`;

const runtimeHookStatus = (exists: boolean, write: boolean) => {
  if (exists) {
    return "existing";
  }
  return write ? "created" : "missing";
};

const expectedRuntimeHooks = (catalog: StudioCatalog) =>
  new Set(
    catalog.byKind.executions
      .map((record) => record.hook)
      .filter((hook): hook is string => typeof hook === "string" && hook.length > 0),
  );

const plannedRuntimeHookFiles = (
  config: ResolvedStudioProjectConfig,
  catalog: StudioCatalog,
): PlannedRuntimeHookFile[] => {
  const hookDir = config.paths.hooks.dir;
  if (!hookDir) {
    return [];
  }

  const files: PlannedRuntimeHookFile[] = [];
  for (const hook of [...expectedRuntimeHooks(catalog)].toSorted()) {
    const path = join(hookDir, hookFileName(hook));
    if (!existsSync(path)) {
      files.push({ hook, path, value: hookStub(hook) });
    }

    if (config.paths.hooks.testStubsDir) {
      const testPath = join(config.paths.hooks.testStubsDir, hookFileName(hook));
      if (!existsSync(testPath)) {
        files.push({ hook, path: testPath, value: hookTestStub(hook) });
      }
    }
  }

  return files;
};

const summarizeHooks = (
  config: ResolvedStudioProjectConfig,
  catalog: StudioCatalog,
  options: { write: boolean },
): RuntimeHookSummary[] => {
  const hookDir = config.paths.hooks.dir;
  if (!hookDir) {
    return [];
  }

  const expectedHooks = expectedRuntimeHooks(catalog);
  const summaries: RuntimeHookSummary[] = [];

  for (const hook of [...expectedHooks].toSorted()) {
    const path = join(hookDir, hookFileName(hook));
    const exists = existsSync(path);
    summaries.push({
      hook,
      path,
      status: runtimeHookStatus(exists, options.write),
    });

    if (config.paths.hooks.testStubsDir) {
      const testPath = join(config.paths.hooks.testStubsDir, hookFileName(hook));
      if (existsSync(testPath)) {
        summaries.push({ hook, path: testPath, status: "existing" });
      } else if (options.write) {
        summaries.push({ hook, path: testPath, status: "created" });
      } else {
        summaries.push({ hook, path: testPath, status: "missing" });
      }
    }
  }

  for (const path of listFilesRecursive(hookDir)) {
    const fileName = path.split("/").at(-1) ?? "";
    if (!path.endsWith(".rs") || hookModuleIndexFiles.has(fileName)) {
      continue;
    }
    const hook = fileName.replace(/\.rs$/, "");
    if (!expectedHooks.has(hook)) {
      summaries.push({ hook, path, status: "orphan" });
    }
  }

  return summaries.toSorted((left, right) => left.path.localeCompare(right.path));
};

const generatedWriteStatus = (before: string | undefined, value: string) => {
  if (before === undefined) {
    return "created";
  }
  return before === value ? "fresh" : "updated";
};

const prepareCodegenTargets = (
  config: ResolvedStudioProjectConfig,
  options: CodegenStudioProjectOptions,
): { diagnostics: StudioDiagnostic[]; targets: RegisteredGeneratedTarget[] } => {
  const fullConfigDiagnostics = fullConfigRequired(config);
  const registeredTargets = activeGeneratedTargets(config);
  const selected = selectTargets(
    registeredTargets,
    defaultSelectedGeneratedTargets(config).map((target) => target.id),
    options.targets,
  );
  return {
    diagnostics: [...fullConfigDiagnostics, ...selected.diagnostics],
    targets: selected.targets,
  };
};

const failedCodegenResult = (
  config: ResolvedStudioProjectConfig,
  options: CodegenStudioProjectOptions,
  diagnostics: StudioDiagnostic[],
): CodegenStudioResult => ({
  checked: options.check === true,
  configPath: config.configPath,
  diagnostics,
  hooks: [],
  ok: false,
  targets: [],
});

const codegenPreparedStudioProject = async (
  config: ResolvedStudioProjectConfig,
  catalog: StudioCatalog,
  selectedTargets: RegisteredGeneratedTarget[],
  options: CodegenStudioProjectOptions,
): Promise<CodegenStudioResult> => {
  if (catalog.diagnostics.some((diagnostic) => diagnostic.severity === "error")) {
    return failedCodegenResult(config, options, catalog.diagnostics);
  }

  const planned = await plannedGeneratedFiles(config, catalog, selectedTargets);
  if (planned.diagnostics.some((diagnostic) => diagnostic.severity === "error")) {
    return failedCodegenResult(config, options, planned.diagnostics);
  }

  const expected = planned.files;
  const diffs = summarizeGeneratedFiles(config, expected, selectedTargets);
  const hooks = summarizeHooks(config, catalog, {
    write: options.check !== true,
  });
  let finalDiffs = diffs;

  if (options.check !== true) {
    const hookFiles = plannedRuntimeHookFiles(config, catalog);
    const snapshots = snapshotPaths([
      ...expected.map((file) => file.path),
      ...hookFiles.map((file) => file.path),
    ]);
    try {
      for (const file of expected) {
        const before = readTextIfExists(file.path);
        writeTextFile(file.path, file.value);
        const status = generatedWriteStatus(before, file.value);
        finalDiffs = finalDiffs.map((diff) =>
          diff.path === file.path ? { ...diff, status } : diff,
        );
      }

      for (const diff of diffs.filter((candidate) => candidate.status === "unexpected")) {
        if (existsSync(diff.path)) {
          snapshots.push(...snapshotPaths([diff.path]));
          rmSync(diff.path);
          finalDiffs = finalDiffs.map((candidate) =>
            candidate.path === diff.path ? { ...candidate, status: "deleted" } : candidate,
          );
        }
      }

      for (const file of hookFiles) {
        writeTextFile(file.path, file.value);
      }
    } catch (error) {
      restoreSnapshots(snapshots);
      return {
        checked: false,
        configPath: config.configPath,
        diagnostics: [
          workflowError(
            "codegen-write-failed",
            error instanceof Error
              ? `Failed to write generated mechanics definitions or runtime hook stubs: ${error.message}`
              : "Failed to write generated mechanics definitions or runtime hook stubs.",
          ),
        ],
        hooks: summarizeHooks(config, catalog, { write: false }),
        ok: false,
        targets: [],
      };
    }
  }

  const staleDiagnostics =
    options.check === true
      ? diffs
          .filter((diff) => diff.status !== "fresh")
          .map((diff) =>
            workflowError(
              `generated-${diff.status}`,
              `Generated mechanics definition is ${diff.status}: ${displayPath(config.configDir, diff.path)}`,
              displayPath(config.configDir, diff.path),
            ),
          )
      : [];
  const hookDiagnostics = hooks
    .filter((hook) => hook.status === "orphan")
    .map((hook) =>
      workflowWarning(
        "orphan-runtime-hook",
        `Runtime hook file is not referenced by the Studio catalog: ${displayPath(config.configDir, hook.path)}`,
        displayPath(config.configDir, hook.path),
      ),
    );

  const targetSummaries = selectedTargets.map(
    (target): StudioCodegenTargetSummary => ({
      files: finalDiffs.filter((diff) => diff.target === target.id),
      label: target.label,
      target: target.id,
    }),
  );

  const diagnostics = [...planned.diagnostics, ...staleDiagnostics, ...hookDiagnostics];
  return {
    checked: options.check === true,
    configPath: config.configPath,
    diagnostics,
    hooks,
    ok: diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    targets: targetSummaries,
  };
};

export const codegenLoadedStudioProject = (
  config: ResolvedStudioProjectConfig,
  catalog: StudioCatalog,
  options: CodegenStudioProjectOptions = {},
): Promise<CodegenStudioResult> => {
  const prepared = prepareCodegenTargets(config, options);
  if (prepared.diagnostics.length > 0) {
    return Promise.resolve(failedCodegenResult(config, options, prepared.diagnostics));
  }

  return codegenPreparedStudioProject(config, catalog, prepared.targets, options);
};

export const codegenStudioProject = async (
  options: StudioWorkflowOptions & CodegenStudioProjectOptions = {},
): Promise<CodegenStudioResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      checked: options.check === true,
      diagnostics: resolved.diagnostics,
      hooks: [],
      ok: false,
      targets: [],
    };
  }

  const prepared = prepareCodegenTargets(resolved.config, options);
  if (prepared.diagnostics.length > 0) {
    return failedCodegenResult(resolved.config, options, prepared.diagnostics);
  }

  const catalog = await loadStudioCatalog(resolved.config);
  return codegenPreparedStudioProject(resolved.config, catalog, prepared.targets, options);
};
