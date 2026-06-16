import { loadStudioConfig } from "../config/load";
import type { LoadStudioConfigOptions } from "../config/load";
import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import type { StudioWorkflowOptions } from "./types";

export const workflowError = (
  code: string,
  message: string,
  path?: string,
  hint?: string,
): StudioDiagnostic => ({
  code,
  hint,
  message,
  path,
  severity: "error",
});

export const workflowWarning = (
  code: string,
  message: string,
  path?: string,
): StudioDiagnostic => ({
  code,
  message,
  path,
  severity: "warning",
});

export const resolveWorkflowConfig = async (
  options: StudioWorkflowOptions,
): Promise<
  | { config: ResolvedStudioProjectConfig; diagnostics: StudioDiagnostic[]; ok: true }
  | { diagnostics: StudioDiagnostic[]; ok: false }
> => {
  if (options.config) {
    return { config: options.config, diagnostics: [], ok: true };
  }

  const loaded = await loadStudioConfig({
    configPath: options.configPath,
    cwd: options.cwd,
  } satisfies LoadStudioConfigOptions);
  if (!loaded.ok || !loaded.config) {
    return { diagnostics: loaded.diagnostics, ok: false };
  }

  return { config: loaded.config, diagnostics: loaded.diagnostics, ok: true };
};

export const fullConfigRequired = (config: ResolvedStudioProjectConfig): StudioDiagnostic[] =>
  config.mode === "full"
    ? []
    : [
        workflowError(
          "full-config-required",
          "This Studio workflow requires a full Studio project config.",
          config.configPath,
        ),
      ];

export const hasErrorDiagnostic = (diagnostics: readonly StudioDiagnostic[]) =>
  diagnostics.some((diagnostic) => diagnostic.severity === "error");
