import { existsSync } from "node:fs";
import { join } from "node:path";

import { normalizeRecordKind, writeStudioCatalogRecord } from "../internal/catalog";
import type { StudioCatalogRecord, StudioRecordKind } from "../internal/catalog";
import { displayPath, restoreSnapshots, snapshotPaths } from "../internal/files";
import { validateStudioCatalog } from "./catalog";
import { codegenStudioProject } from "./codegen";
import { resolveWorkflowConfig, workflowError } from "./shared";
import type {
  PlanStudioMechanicOptions,
  PlanStudioMechanicResult,
  ScaffoldStudioMechanicResult,
} from "./types";

const mechanicRecords = (
  id: string,
  label: string,
  params: Record<string, unknown> = {},
): StudioCatalogRecord[] => {
  const broken = params.broken === true;
  return [
    { id, kind: "tag", label: `${label} tag` },
    { id, kind: "modifier", label: `${label} modifier`, value: 1 },
    {
      hook: `${id}_runtime_hook`,
      id,
      kind: "execution",
      label: `${label} execution`,
    },
    {
      executionId: id,
      id,
      kind: "effect",
      label: `${label} effect`,
      modifierId: id,
      tagIds: [id],
    },
    {
      effectId: broken ? `${id}_missing_effect` : id,
      id,
      kind: "ability",
      label: `${label} ability`,
    },
    {
      id,
      kind: "mechanic",
      label,
      recordIds: [
        `tag:${id}`,
        `modifier:${id}`,
        `execution:${id}`,
        `effect:${id}`,
        `ability:${id}`,
      ],
    },
  ];
};

const kindForRecord = (record: StudioCatalogRecord): StudioRecordKind => {
  const kind = normalizeRecordKind(record.kind);
  if (!kind) {
    throw new Error(`Unsupported Studio catalog record kind ${record.kind}.`);
  }
  return kind;
};

export const planStudioMechanic = async (
  options: PlanStudioMechanicOptions,
): Promise<PlanStudioMechanicResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { diagnostics: resolved.diagnostics, ok: false, plannedFiles: [], records: [] };
  }

  if (options.archetype !== "mechanic") {
    return {
      diagnostics: [
        workflowError(
          "unknown-mechanic-archetype",
          `Unknown Studio mechanic archetype "${options.archetype}".`,
          undefined,
          'Use "mechanic" for the built-in synthetic archetype.',
        ),
      ],
      ok: false,
      plannedFiles: [],
      records: [],
    };
  }

  const records = mechanicRecords(options.id, options.name, options.params);
  const plannedFiles = records.map((record) =>
    join(resolved.config.paths.catalogRoot, kindForRecord(record), `${record.id}.json`),
  );
  const diagnostics =
    options.allowExisting === true
      ? []
      : plannedFiles
          .filter((path) => existsSync(path))
          .map((path) =>
            workflowError(
              "planned-file-exists",
              `Planned Studio catalog file already exists: ${displayPath(resolved.config.configDir, path)}`,
              displayPath(resolved.config.configDir, path),
            ),
          );

  return {
    diagnostics,
    ok: diagnostics.length === 0,
    plannedFiles: plannedFiles.map((path) => displayPath(resolved.config.configDir, path)),
    records,
  };
};

export const scaffoldStudioMechanic = async (
  options: PlanStudioMechanicOptions,
): Promise<ScaffoldStudioMechanicResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      diagnostics: resolved.diagnostics,
      ok: false,
      plannedFiles: [],
      records: [],
      rolledBack: false,
      writtenFiles: [],
    };
  }

  const planned = await planStudioMechanic({ ...options, config: resolved.config });
  if (!planned.ok) {
    return { ...planned, rolledBack: false, writtenFiles: [] };
  }

  const absolutePlannedFiles = planned.records.map((record) =>
    join(resolved.config.paths.catalogRoot, kindForRecord(record), `${record.id}.json`),
  );
  const snapshots = snapshotPaths(absolutePlannedFiles);
  const writtenFiles: string[] = [];

  try {
    for (const record of planned.records) {
      const writeResult = writeStudioCatalogRecord(resolved.config, kindForRecord(record), record);
      if (writeResult.diagnostics.length > 0 || !writeResult.path) {
        restoreSnapshots(snapshots);
        return {
          diagnostics: writeResult.diagnostics,
          ok: false,
          plannedFiles: planned.plannedFiles,
          records: planned.records,
          rolledBack: true,
          writtenFiles,
        };
      }
      const { path } = writeResult;
      writtenFiles.push(displayPath(resolved.config.configDir, path));
    }

    const validation = await validateStudioCatalog({ config: resolved.config });
    if (!validation.ok) {
      restoreSnapshots(snapshots);
      return {
        diagnostics: validation.diagnostics,
        ok: false,
        plannedFiles: planned.plannedFiles,
        records: planned.records,
        rolledBack: true,
        writtenFiles,
      };
    }

    const codegen = await codegenStudioProject({
      config: resolved.config,
      targets: ["executions"],
    });
    return {
      diagnostics: codegen.diagnostics,
      ok: codegen.ok,
      plannedFiles: planned.plannedFiles,
      records: planned.records,
      rolledBack: false,
      writtenFiles: [
        ...writtenFiles,
        ...codegen.hooks
          .filter((hook) => hook.status === "created")
          .map((hook) => displayPath(resolved.config.configDir, hook.path)),
      ],
    };
  } catch (error) {
    restoreSnapshots(snapshots);
    return {
      diagnostics: [
        workflowError(
          "scaffold-failed",
          error instanceof Error
            ? `Failed to scaffold Studio mechanic: ${error.message}`
            : "Failed to scaffold Studio mechanic.",
        ),
      ],
      ok: false,
      plannedFiles: planned.plannedFiles,
      records: planned.records,
      rolledBack: true,
      writtenFiles,
    };
  }
};
