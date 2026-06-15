export const studioCodegenTargets = [
  "abilities",
  "effects",
  "executions",
  "modifiers",
  "reference",
  "tags",
] as const;

export type StudioBuiltInCodegenTarget = (typeof studioCodegenTargets)[number];
export type StudioCodegenTarget = string;

export type StudioGeneratedTargetCleanupPolicy = "managed-files" | "none";

export interface StudioGeneratedTargetFilePlan {
  path: string;
  value: string;
}

export interface StudioGeneratedTargetPlanResult {
  diagnostics?: readonly {
    code: string;
    hint?: string;
    message: string;
    path?: string;
    severity: "error" | "warning";
  }[];
  files: readonly StudioGeneratedTargetFilePlan[];
}

export interface StudioGeneratedTargetPlanContext<Content = unknown> {
  config: unknown;
  content: Content;
  outputDir: string;
  targetId: string;
}

export interface StudioGeneratedTargetDefinition<Content = unknown> {
  cleanup?: StudioGeneratedTargetCleanupPolicy;
  dependencies?: readonly string[];
  id: string;
  label: string;
  plan: (
    context: StudioGeneratedTargetPlanContext<Content>,
  ) => Promise<StudioGeneratedTargetPlanResult> | StudioGeneratedTargetPlanResult;
}

export const defineStudioGeneratedTarget = <const Target extends StudioGeneratedTargetDefinition>(
  target: Target,
): Target => target;

export type StudioGeneratedFileStatus =
  | "created"
  | "deleted"
  | "fresh"
  | "missing"
  | "stale"
  | "unexpected"
  | "updated";

export interface StudioGeneratedFileDiff {
  path: string;
  status: StudioGeneratedFileStatus;
  target: StudioCodegenTarget;
}

export interface StudioCodegenTargetSummary {
  files: StudioGeneratedFileDiff[];
  label: string;
  target: StudioCodegenTarget;
}

export interface RuntimeHookSummary {
  hook: string;
  path: string;
  status: "created" | "existing" | "missing" | "orphan" | "skipped";
}

export const isStudioCodegenTarget = (value: string): value is StudioCodegenTarget =>
  (studioCodegenTargets as readonly string[]).includes(value);

export const isBuiltInStudioCodegenTarget = (value: string): value is StudioBuiltInCodegenTarget =>
  (studioCodegenTargets as readonly string[]).includes(value);
