import { isAbsolute, normalize, resolve } from "node:path";

import { configError } from "./diagnostics";
import type { StudioDiagnostic } from "./types";

export const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export const readString = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): string | undefined => {
  if (typeof value === "string" && value.trim().length > 0) {
    return value;
  }

  diagnostics.push(
    configError(
      "invalid-config-field",
      field,
      `Studio project config field ${field} must be a non-empty string.`,
    ),
  );
  return undefined;
};

export const readOptionalString = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): string | undefined => (value === undefined ? undefined : readString(value, field, diagnostics));

export const readNonNegativeInteger = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): number | undefined => {
  if (typeof value === "number" && Number.isInteger(value) && value >= 0) {
    return value;
  }

  diagnostics.push(
    configError(
      "invalid-config-field",
      field,
      `Studio project config field ${field} must be a non-negative integer.`,
    ),
  );
  return undefined;
};

export const resolveConfigPath = (configDir: string, value: string) =>
  normalize(isAbsolute(value) ? value : resolve(configDir, value));

export const normalizeStringArray = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): string[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio project config field ${field} must be an array of strings.`,
      ),
    );
    return [];
  }

  const normalized: string[] = [];
  for (const [index, item] of value.entries()) {
    const itemField = `${field}.${index}`;
    if (typeof item !== "string" || item.trim().length === 0) {
      diagnostics.push(
        configError(
          "invalid-config-field",
          itemField,
          `Studio project config field ${itemField} must be a non-empty string.`,
        ),
      );
      continue;
    }
    normalized.push(item);
  }

  return normalized;
};
