import type { RuntimeHookSummary, StudioCodegenTargetSummary } from "../codegen/types";
import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import type {
  StudioMapperDiagnosticAttribution,
  StudioSourceDiagnosticAttribution,
} from "../extensions";
import type { StudioCatalogRecord, StudioRecordKind } from "../internal/catalog";

export interface StudioWorkflowOptions {
  config?: ResolvedStudioProjectConfig;
  configPath?: string;
  cwd?: string;
}

export interface StudioWorkflowResult {
  diagnostics: StudioDiagnostic[];
  ok: boolean;
}

export interface ValidateStudioCatalogResult extends StudioWorkflowResult {
  configPath?: string;
  mapperDiagnostics: StudioMapperDiagnosticAttribution[];
  recordCount: number;
  sourceDiagnostics: StudioSourceDiagnosticAttribution[];
  sourceRecordCount: number;
  sources: StudioSourceSummary[];
}

export interface StudioSourceSummary {
  adapterId?: string;
  recordCount: number;
  sourceId?: string;
}

export interface StudioRecordDescription {
  fields: string[];
  kind: StudioRecordKind;
  summary: string;
}

export interface DescribeStudioCatalogResult extends StudioWorkflowResult {
  descriptions: StudioRecordDescription[];
}

export interface ListStudioCatalogRecordsResult extends StudioWorkflowResult {
  kind: StudioRecordKind;
  records: { id: string; label: string; path: string }[];
}

export interface ShowStudioCatalogRecordResult extends StudioWorkflowResult {
  record?: StudioCatalogRecord & { path: string };
}

export interface CodegenStudioResult extends StudioWorkflowResult {
  checked: boolean;
  configPath?: string;
  hooks: RuntimeHookSummary[];
  targets: StudioCodegenTargetSummary[];
}

export interface PlanStudioMechanicOptions extends StudioWorkflowOptions {
  allowExisting?: boolean;
  archetype: string;
  id: string;
  name: string;
  params?: Record<string, unknown>;
}

export interface PlanStudioMechanicResult extends StudioWorkflowResult {
  plannedFiles: string[];
  records: StudioCatalogRecord[];
}

export interface ScaffoldStudioMechanicResult extends PlanStudioMechanicResult {
  rolledBack: boolean;
  writtenFiles: string[];
}

export interface ScaffoldStudioHostAppOptions extends StudioWorkflowOptions {
  appRoot?: string;
}

export type StudioHostAppFileStatus =
  | "created"
  | "manual-follow-up"
  | "project-owned"
  | "unchanged"
  | "updated";

export interface StudioHostAppFileResult {
  path: string;
  reason?: string;
  status: StudioHostAppFileStatus;
}

export interface ScaffoldStudioHostAppResult extends StudioWorkflowResult {
  appRoot?: string;
  changedFiles: string[];
  files: StudioHostAppFileResult[];
  manualFollowUps: string[];
  metadataVersion?: number;
}

export interface VerifyStudioHostAppResult extends StudioWorkflowResult {
  appRoot?: string;
  command?: StudioVerifyCommandResult;
  files: StudioHostAppFileResult[];
  manualFollowUps: string[];
  status: "checked" | "missing" | "not-configured";
}

export interface StudioVerifyCommandResult {
  command: string[];
  exitCode: number | null;
  fast: boolean;
  name: string;
  stderr: string;
  stdout: string;
}

export type StudioVerifyMode = "fast" | "full";

export type StudioVerifyCheckStatus = "failed" | "passed" | "skipped";

export interface StudioVerifyCheckResult {
  adapterId?: string;
  command?: string[];
  diagnostics: StudioDiagnostic[];
  exitCode?: number | null;
  extensionId?: string;
  mode: StudioVerifyMode;
  name: string;
  sourceId?: string;
  status: StudioVerifyCheckStatus;
  stdout?: string;
  stderr?: string;
  targetId?: string;
}

export interface VerifyStudioProjectResult extends StudioWorkflowResult {
  checks: StudioVerifyCheckResult[];
  codegen: CodegenStudioResult;
  commands: StudioVerifyCommandResult[];
  hostApp: VerifyStudioHostAppResult;
  validation: ValidateStudioCatalogResult;
}

export interface MigrateStudioProjectResult extends StudioWorkflowResult {
  applied: string[];
  changedFiles: string[];
  checks: StudioMigrationCheckResult[];
  manualFollowUps: string[];
  skipped: string[];
}

export type StudioMigrationCheckStatus = "applied" | "failed" | "skipped";

export interface StudioMigrationCheckResult {
  applied: string[];
  changedFiles: string[];
  currentVersion?: number;
  diagnostics: StudioDiagnostic[];
  extensionId?: string;
  manualFollowUps: string[];
  name: string;
  skipped: string[];
  status: StudioMigrationCheckStatus;
  targetVersion?: number;
}
