import { isBuiltInStudioCodegenTarget, studioCodegenTargets } from "../codegen/types";
import type {
  StudioBuiltInCodegenTarget,
  StudioGeneratedTargetDefinition,
  StudioGeneratedTargetId,
} from "../codegen/types";
import { configError } from "./diagnostics";
import { isObject, normalizeStringArray, readString } from "./primitive-readers";
import type { StudioDiagnostic, StudioProjectConfig } from "./types";

const validateGeneratedTarget = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioGeneratedTargetDefinition | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-generated-target",
        field,
        `Studio generated target ${field} must be an object.`,
      ),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const label = readString(value.label, `${field}.label`, diagnostics);
  const dependencies = normalizeStringArray(
    value.dependencies,
    `${field}.dependencies`,
    diagnostics,
  );

  if (
    value.cleanup !== undefined &&
    value.cleanup !== "managed-files" &&
    value.cleanup !== "none"
  ) {
    diagnostics.push(
      configError(
        "invalid-generated-target",
        `${field}.cleanup`,
        `Studio generated target ${field}.cleanup must be "managed-files" or "none" when provided.`,
      ),
    );
  }

  if (typeof value.plan !== "function") {
    diagnostics.push(
      configError(
        "invalid-generated-target",
        `${field}.plan`,
        `Studio generated target ${field} must provide a plan function.`,
      ),
    );
  }

  if (!id || !label || typeof value.plan !== "function") {
    return undefined;
  }

  return {
    ...(value as unknown as StudioGeneratedTargetDefinition),
    dependencies,
    id,
    label,
  };
};

export const validateGeneratedTargets = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioGeneratedTargetDefinition[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio extension field ${field} must be an array of generated targets.`,
      ),
    );
    return [];
  }

  const targets: StudioGeneratedTargetDefinition[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const target = validateGeneratedTarget(item, `${field}.${index}`, diagnostics);
    if (!target) {
      continue;
    }
    if (seen.has(target.id)) {
      diagnostics.push(
        configError(
          "duplicate-generated-target",
          `${field}.${index}.id`,
          `Studio generated target "${target.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(target.id);
    targets.push(target);
  }

  return targets;
};

export const validateBuiltInCodegenTargets = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): StudioBuiltInCodegenTarget[] => {
  if (value === undefined) {
    return [...studioCodegenTargets];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        "codegen.builtInTargets",
        "Studio project config field codegen.builtInTargets must be an array of built-in generated target ids.",
        `Expected zero or more of: ${studioCodegenTargets.join(", ")}.`,
      ),
    );
    return [];
  }

  const targets: StudioBuiltInCodegenTarget[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const field = `codegen.builtInTargets.${index}`;
    if (typeof item !== "string" || !isBuiltInStudioCodegenTarget(item)) {
      diagnostics.push(
        configError(
          "invalid-config-field",
          field,
          `Studio project config field ${field} must be a built-in generated target id.`,
          `Expected one of: ${studioCodegenTargets.join(", ")}.`,
        ),
      );
      continue;
    }
    if (seen.has(item)) {
      diagnostics.push(
        configError(
          "duplicate-generated-target",
          field,
          `Studio built-in generated target "${item}" is listed more than once.`,
        ),
      );
      continue;
    }
    seen.add(item);
    targets.push(item);
  }

  return targets;
};

export const readAllowOverlappingOutputDirs = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
) => {
  const rawValue = value.codegen?.allowOverlappingOutputDirs;
  if (rawValue === undefined) {
    return false;
  }
  if (typeof rawValue === "boolean") {
    return rawValue;
  }
  diagnostics.push(
    configError(
      "invalid-config-field",
      "codegen.allowOverlappingOutputDirs",
      "Studio project config field codegen.allowOverlappingOutputDirs must be a boolean when provided.",
    ),
  );
  return false;
};

export const validateCodegenConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
  builtInTargetIds: readonly StudioBuiltInCodegenTarget[],
  extensionTargetIds: readonly StudioGeneratedTargetId[],
): Partial<Record<StudioGeneratedTargetId, string>> => {
  const outputDirs: Partial<Record<StudioGeneratedTargetId, string>> = {};
  const codegenValue = value.codegen;
  const activeTargetIds = [...builtInTargetIds, ...extensionTargetIds];
  const activeTargetIdSet = new Set<StudioGeneratedTargetId>(activeTargetIds);
  const builtInTargetIdSet = new Set<StudioGeneratedTargetId>(builtInTargetIds);

  if (isObject(codegenValue) && isObject(codegenValue.outputDirs)) {
    for (const target of builtInTargetIds) {
      const configuredPath = readString(
        codegenValue.outputDirs[target],
        `codegen.outputDirs.${target}`,
        diagnostics,
      );
      if (configuredPath) {
        outputDirs[target] = configuredPath;
      }
    }

    for (const key of Object.keys(codegenValue.outputDirs)) {
      if (!activeTargetIdSet.has(key)) {
        diagnostics.push(
          configError(
            "unknown-codegen-target",
            `codegen.outputDirs.${key}`,
            `Unknown Studio generated output target "${key}".`,
            `Expected one of: ${activeTargetIds.join(", ")}.`,
          ),
        );
        continue;
      }

      if (!builtInTargetIdSet.has(key)) {
        const configuredPath = readString(
          codegenValue.outputDirs[key],
          `codegen.outputDirs.${key}`,
          diagnostics,
        );
        if (configuredPath) {
          outputDirs[key] = configuredPath;
        }
      }
    }

    return outputDirs;
  }

  diagnostics.push(
    configError(
      "missing-config-field",
      "codegen.outputDirs",
      "Full Studio project configs must declare codegen.outputDirs.",
    ),
  );
  return outputDirs;
};
