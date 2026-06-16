import type { StudioExtension } from "../extensions";
import { configError } from "./diagnostics";
import { isObject, normalizeStringArray, readString } from "./primitive-readers";
import type { ResolvedStudioProjectConfig, StudioDiagnostic, StudioProjectConfig } from "./types";

const validateRuntimeVocabConfig = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): { ailments: string[]; damageTypes: string[] } => {
  if (value === undefined) {
    return {
      ailments: [],
      damageTypes: [],
    };
  }

  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        "rust.runtimeVocab",
        "Studio project config field rust.runtimeVocab must be an object when provided.",
      ),
    );
    return {
      ailments: [],
      damageTypes: [],
    };
  }

  return {
    ailments: normalizeStringArray(value.ailments, "rust.runtimeVocab.ailments", diagnostics),
    damageTypes: normalizeStringArray(
      value.damageTypes,
      "rust.runtimeVocab.damageTypes",
      diagnostics,
    ),
  };
};

const validateStringRecord = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): Record<string, string> => {
  if (value === undefined) {
    return {};
  }

  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio project config field ${field} must be an object of strings when provided.`,
      ),
    );
    return {};
  }

  const result: Record<string, string> = {};
  for (const [key, item] of Object.entries(value)) {
    if (typeof item !== "string" || item.trim().length === 0) {
      diagnostics.push(
        configError(
          "invalid-config-field",
          `${field}.${key}`,
          `Studio project config field ${field}.${key} must be a non-empty string.`,
        ),
      );
      continue;
    }
    result[key] = item;
  }

  return result;
};

const validateBindingsRecord = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): Record<string, unknown> => {
  if (value === undefined) {
    return {};
  }

  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        "rust.bindings",
        "Studio project config field rust.bindings must be an object when provided.",
      ),
    );
    return {};
  }

  return value;
};

export const validateRustConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): ResolvedStudioProjectConfig["rust"] | undefined => {
  if (!isObject(value.rust)) {
    diagnostics.push(
      configError(
        "missing-config-field",
        "rust",
        "Full Studio project configs must declare rust.flexweaveModule.",
      ),
    );
    return undefined;
  }

  const flexweaveModule = readString(
    value.rust.flexweaveModule,
    "rust.flexweaveModule",
    diagnostics,
  );
  const generatedHeader =
    value.rust.generatedHeader === undefined
      ? undefined
      : readString(value.rust.generatedHeader, "rust.generatedHeader", diagnostics);

  if (!flexweaveModule) {
    return undefined;
  }

  return {
    bindings: validateBindingsRecord(value.rust.bindings, diagnostics),
    flexweaveModule,
    generatedHeader,
    macroNames: validateStringRecord(value.rust.macroNames, "rust.macroNames", diagnostics),
    moduleAliases: validateStringRecord(
      value.rust.moduleAliases,
      "rust.moduleAliases",
      diagnostics,
    ),
    preludeImports: normalizeStringArray(
      value.rust.preludeImports,
      "rust.preludeImports",
      diagnostics,
    ),
    runtimeVocab: validateRuntimeVocabConfig(value.rust.runtimeVocab, diagnostics),
    typePaths: validateStringRecord(value.rust.typePaths, "rust.typePaths", diagnostics),
  };
};

export const validateExtensionRustBindings = (
  extensions: readonly StudioExtension[],
  rust: ResolvedStudioProjectConfig["rust"] | undefined,
): StudioDiagnostic[] => {
  if (!rust) {
    return [];
  }

  const diagnostics: StudioDiagnostic[] = [];
  for (const extension of extensions) {
    for (const validator of extension.rustBindingConfigs ?? []) {
      try {
        diagnostics.push(
          ...validator.validate({
            namespace: validator.namespace,
            value: rust.bindings[validator.namespace],
          }),
        );
      } catch (error) {
        diagnostics.push(
          configError(
            "extension-rust-config-failed",
            `extensions.${extension.id}.rustBindingConfigs.${validator.namespace}`,
            error instanceof Error
              ? `Studio extension "${extension.id}" failed Rust binding config validation: ${error.message}`
              : `Studio extension "${extension.id}" failed Rust binding config validation.`,
          ),
        );
      }
    }
  }

  return diagnostics;
};
