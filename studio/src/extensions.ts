import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "./config/schema";
import type { StudioGeneratedTargetDefinition } from "./codegen/types";

export type { StudioDiagnostic } from "./config/schema";

export type StudioDataAdapterCapability = "diff" | "read" | "schema" | "watch" | "write";

export interface StudioSourceLocation {
  cell?: string;
  column?: number;
  display?: string;
  field?: string;
  jsonPointer?: string;
  line?: number;
  path?: string;
  row?: number;
  sheet?: string;
  uri?: string;
}

export interface StudioSourceRecord {
  id: string;
  kind: string;
  location?: StudioSourceLocation;
  value: unknown;
}

export interface StudioSourceSnapshot {
  adapterId?: string;
  diagnostics?: readonly StudioDiagnostic[];
  records: readonly StudioSourceRecord[];
  sourceId?: string;
}

export interface StudioMappedContentRecord {
  expectedKind?: string;
  location?: StudioSourceLocation;
  path?: string;
  sourceRecord?: StudioSourceRecord;
  value: unknown;
}

export interface StudioContentMapperContext {
  config: ResolvedStudioProjectConfig;
  snapshots: readonly StudioSourceSnapshot[];
}

export interface StudioContentMapperResult {
  diagnostics?: readonly StudioDiagnostic[];
  records: readonly StudioMappedContentRecord[];
}

export interface StudioContentMapper {
  id: string;
  label?: string;
  map: (
    context: StudioContentMapperContext,
  ) => Promise<StudioContentMapperResult> | StudioContentMapperResult;
}

export interface StudioSourceConfig {
  adapterId: string;
  id: string;
  label?: string;
  options?: Record<string, unknown>;
}

export interface StudioDataAdapterLoadContext {
  config: ResolvedStudioProjectConfig;
  source: StudioSourceConfig;
}

export interface StudioDataAdapterWriteContext extends StudioDataAdapterLoadContext {
  records: readonly StudioSourceRecord[];
}

export interface StudioDataAdapter {
  capabilities: readonly StudioDataAdapterCapability[];
  id: string;
  label?: string;
  load: (
    context: StudioDataAdapterLoadContext,
  ) => Promise<StudioSourceSnapshot> | StudioSourceSnapshot;
  write?: (
    context: StudioDataAdapterWriteContext,
  ) => Promise<StudioSourceSnapshot> | StudioSourceSnapshot;
}

export interface StudioSourceValidationContext {
  config: ResolvedStudioProjectConfig;
  snapshots: readonly StudioSourceSnapshot[];
}

export interface StudioRustBindingValidationContext {
  namespace: string;
  value: unknown;
}

export interface StudioRustBindingConfigValidator {
  namespace: string;
  validate: (context: StudioRustBindingValidationContext) => readonly StudioDiagnostic[];
}

export type StudioHostAppRouteKind =
  | "authoring-editor"
  | "diagnostics-panel"
  | "generated-output"
  | "overview"
  | "source-view";

export type StudioHostAppWorkflowCommandName =
  | "codegen"
  | "describe"
  | "list"
  | "migrate"
  | "plan"
  | "scaffold"
  | "show"
  | "validate"
  | "verify";

export type StudioHostAppActionVariant = "primary" | "secondary";

export interface StudioHostAppNavigationLink {
  href: string;
  icon?: string;
  id: string;
  label: string;
}

export interface StudioHostAppNavigationSection {
  id: string;
  label: string;
  links: readonly StudioHostAppNavigationLink[];
}

export interface StudioHostAppAuthoringAreaDefinition {
  description?: string;
  editorId?: string;
  icon?: string;
  id: string;
  label: string;
}

export interface StudioHostAppAuthoringEditorDefinition {
  areaId: string;
  commandName?: StudioHostAppWorkflowCommandName;
  description?: string;
  id: string;
  label: string;
  recordKind?: string;
}

export interface StudioHostAppWorkflowActionDefinition {
  commandName: StudioHostAppWorkflowCommandName;
  id: string;
  label: string;
  variant: StudioHostAppActionVariant;
}

export interface StudioHostAppCodegenTargetDefinition {
  description?: string;
  label: string;
  outputLabel?: string;
  target: string;
}

export interface StudioHostAppGeneratedOutputPanelDefinition {
  description?: string;
  id: string;
  label: string;
  target?: string;
}

export interface StudioHostAppDiagnosticsPanelDefinition {
  commandName?: StudioHostAppWorkflowCommandName;
  description?: string;
  id: string;
  label: string;
}

export interface StudioHostAppSourceViewDefinition {
  adapterId?: string;
  description?: string;
  id: string;
  label: string;
  sourceId?: string;
}

export interface StudioHostAppAuthoringContribution {
  areas?: readonly StudioHostAppAuthoringAreaDefinition[];
  editors?: readonly StudioHostAppAuthoringEditorDefinition[];
}

export interface StudioHostAppContribution {
  authoring?: StudioHostAppAuthoringContribution;
  codegenTargets?: readonly StudioHostAppCodegenTargetDefinition[];
  diagnosticsPanels?: readonly StudioHostAppDiagnosticsPanelDefinition[];
  generatedOutputPanels?: readonly StudioHostAppGeneratedOutputPanelDefinition[];
  id: string;
  label?: string;
  navigation?: readonly StudioHostAppNavigationSection[];
  sourceViews?: readonly StudioHostAppSourceViewDefinition[];
  workflowActions?: readonly StudioHostAppWorkflowActionDefinition[];
}

export interface StudioExtension {
  appContributions?: readonly StudioHostAppContribution[];
  contentMappers?: readonly StudioContentMapper[];
  dataAdapters?: readonly StudioDataAdapter[];
  generatedTargets?: readonly StudioGeneratedTargetDefinition[];
  id: string;
  label?: string;
  rustBindingConfigs?: readonly StudioRustBindingConfigValidator[];
  validateSources?: (
    context: StudioSourceValidationContext,
  ) => Promise<readonly StudioDiagnostic[]> | readonly StudioDiagnostic[];
}

export interface StudioSourceLoadResult {
  diagnostics: StudioDiagnostic[];
  ok: boolean;
  snapshots: StudioSourceSnapshot[];
}

export const defineStudioDataAdapter = <const Adapter extends StudioDataAdapter>(
  adapter: Adapter,
): Adapter => adapter;

export const defineStudioExtension = <const Extension extends StudioExtension>(
  extension: Extension,
): Extension => extension;

export const defineStudioHostAppContribution = <
  const Contribution extends StudioHostAppContribution,
>(
  contribution: Contribution,
): Contribution => contribution;

export const defineStudioContentMapper = <const Mapper extends StudioContentMapper>(
  mapper: Mapper,
): Mapper => mapper;

export const studioDataAdapterCanWrite = (adapter: StudioDataAdapter) =>
  adapter.capabilities.includes("write") && typeof adapter.write === "function";

export const studioSourceLocationLabel = (location: StudioSourceLocation | undefined) => {
  if (!location) {
    return;
  }

  if (location.display) {
    return location.display;
  }

  const base = location.path ?? location.uri ?? location.sheet;
  const parts = [
    location.row === undefined ? undefined : `row ${location.row}`,
    location.column === undefined ? undefined : `column ${location.column}`,
    location.cell === undefined ? undefined : `cell ${location.cell}`,
    location.field === undefined ? undefined : `field ${location.field}`,
  ].filter((part): part is string => typeof part === "string");

  if (base && parts.length > 0) {
    return `${base} (${parts.join(", ")})`;
  }

  return base ?? (parts.join(", ") || undefined);
};

const extensionError = (
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

export const loadStudioSourceSnapshots = async (
  config: ResolvedStudioProjectConfig,
): Promise<StudioSourceLoadResult> => {
  const adapters: Record<string, StudioDataAdapter | undefined> = {};
  for (const adapter of config.data.adapters) {
    adapters[adapter.id] = adapter;
  }
  for (const extension of config.extensions) {
    for (const adapter of extension.dataAdapters ?? []) {
      adapters[adapter.id] = adapter;
    }
  }

  const diagnostics: StudioDiagnostic[] = [];
  const snapshots: StudioSourceSnapshot[] = [];

  for (const source of config.data.sources) {
    const adapter = adapters[source.adapterId];
    if (!adapter) {
      diagnostics.push(
        extensionError(
          "missing-data-adapter",
          `Studio source "${source.id}" references missing data adapter "${source.adapterId}".`,
          config.configPath,
          "Register the adapter in data.adapters or through an active Studio extension.",
        ),
      );
      continue;
    }

    if (!adapter.capabilities.includes("read")) {
      diagnostics.push(
        extensionError(
          "adapter-read-unsupported",
          `Studio data adapter "${adapter.id}" cannot read source "${source.id}".`,
          config.configPath,
          "Use an adapter with the read capability for source loading.",
        ),
      );
      continue;
    }

    try {
      const snapshot = await adapter.load({ config, source });
      snapshots.push({
        ...snapshot,
        adapterId: snapshot.adapterId ?? adapter.id,
        sourceId: snapshot.sourceId ?? source.id,
      });
      diagnostics.push(...(snapshot.diagnostics ?? []));
    } catch (error) {
      diagnostics.push(
        extensionError(
          "data-adapter-load-failed",
          error instanceof Error
            ? `Studio data adapter "${adapter.id}" failed to load source "${source.id}": ${error.message}`
            : `Studio data adapter "${adapter.id}" failed to load source "${source.id}".`,
          config.configPath,
        ),
      );
    }
  }

  for (const extension of config.extensions) {
    if (!extension.validateSources) {
      continue;
    }
    try {
      diagnostics.push(...(await extension.validateSources({ config, snapshots })));
    } catch (error) {
      diagnostics.push(
        extensionError(
          "extension-source-validation-failed",
          error instanceof Error
            ? `Studio extension "${extension.id}" failed source validation: ${error.message}`
            : `Studio extension "${extension.id}" failed source validation.`,
          config.configPath,
        ),
      );
    }
  }

  return {
    diagnostics,
    ok: diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    snapshots,
  };
};
