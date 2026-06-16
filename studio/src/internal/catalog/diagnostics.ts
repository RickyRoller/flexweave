import type { StudioDiagnostic } from "../../config/schema";
import type { StudioSourceLocation } from "../../extensions";

export const catalogDiagnostic = (
  code: string,
  message: string,
  path?: string,
  field?: string,
  source?: StudioSourceLocation,
  severity: StudioDiagnostic["severity"] = "error",
): StudioDiagnostic => ({
  code,
  field,
  message,
  path,
  severity,
  source,
});
