import { planStudioCatalogWrite, prepareStudioCatalogWrite } from "../internal/catalog";
import type { StudioCatalogRecord } from "../internal/catalog";
import { displayPath } from "../internal/files";
import { validateStudioCatalog } from "./catalog";
import { codegenStudioProject } from "./codegen";
import { hasErrorDiagnostic, resolveWorkflowConfig, workflowError } from "./shared";
import type {
  PlanStudioMechanicOptions,
  PlanStudioMechanicResult,
  ScaffoldStudioMechanicResult,
} from "./types";

const isSourceWriteConfigurationDiagnostic = (code: string) =>
  code === "source-write-ambiguous" || code === "source-write-unsupported";

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
  const writePlan = planStudioCatalogWrite(resolved.config, records, {
    allowExisting: options.allowExisting,
  });

  return {
    diagnostics: writePlan.diagnostics,
    ok: !hasErrorDiagnostic(writePlan.diagnostics),
    plannedFiles: writePlan.plannedPaths,
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
    return {
      ...planned,
      rolledBack: planned.diagnostics.some((diagnostic) =>
        isSourceWriteConfigurationDiagnostic(diagnostic.code),
      ),
      writtenFiles: [],
    };
  }

  const writeSession = prepareStudioCatalogWrite(resolved.config, planned.records, {
    allowExisting: options.allowExisting,
  });
  if (hasErrorDiagnostic(writeSession.diagnostics)) {
    const rollback = writeSession.rollback();
    return {
      diagnostics: [...writeSession.diagnostics, ...rollback.diagnostics],
      ok: false,
      plannedFiles: planned.plannedFiles,
      records: planned.records,
      rolledBack: rollback.rolledBack,
      writtenFiles: [],
    };
  }

  let writtenFiles: string[] = [];

  try {
    const writeResult = await writeSession.write();
    const writeDiagnostics = [...writeSession.diagnostics, ...writeResult.diagnostics];
    writtenFiles = writeResult.writtenPaths;
    if (hasErrorDiagnostic(writeDiagnostics)) {
      const rollback = writeSession.rollback();
      return {
        diagnostics: [...writeDiagnostics, ...rollback.diagnostics],
        ok: false,
        plannedFiles: planned.plannedFiles,
        records: planned.records,
        rolledBack: rollback.rolledBack,
        writtenFiles,
      };
    }

    const validation = await validateStudioCatalog({ config: resolved.config });
    if (!validation.ok) {
      const rollback = writeSession.rollback();
      return {
        diagnostics: [...writeDiagnostics, ...validation.diagnostics, ...rollback.diagnostics],
        ok: false,
        plannedFiles: planned.plannedFiles,
        records: planned.records,
        rolledBack: rollback.rolledBack,
        writtenFiles,
      };
    }

    const codegen = await codegenStudioProject({
      config: resolved.config,
      targets: ["executions"],
    });
    if (!codegen.ok) {
      const rollback = writeSession.rollback();
      return {
        diagnostics: [...writeDiagnostics, ...codegen.diagnostics, ...rollback.diagnostics],
        ok: false,
        plannedFiles: planned.plannedFiles,
        records: planned.records,
        rolledBack: rollback.rolledBack,
        writtenFiles,
      };
    }

    return {
      diagnostics: [...writeDiagnostics, ...codegen.diagnostics],
      ok: true,
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
    const rollback = writeSession.rollback();
    return {
      diagnostics: [
        workflowError(
          "scaffold-failed",
          error instanceof Error
            ? `Failed to scaffold Studio mechanic: ${error.message}`
            : "Failed to scaffold Studio mechanic.",
        ),
        ...writeSession.diagnostics,
        ...rollback.diagnostics,
      ],
      ok: false,
      plannedFiles: planned.plannedFiles,
      records: planned.records,
      rolledBack: rollback.rolledBack,
      writtenFiles,
    };
  }
};
