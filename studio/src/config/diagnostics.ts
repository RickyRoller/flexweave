import type { StudioDiagnostic } from "./types";

export const configError = (
  code: string,
  field: string,
  message: string,
  hint?: string,
): StudioDiagnostic => ({
  code,
  field,
  hint,
  message,
  severity: "error",
});
