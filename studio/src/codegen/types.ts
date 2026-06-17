export const studioCodegenTargets = [
  "abilities",
  "effects",
  "executions",
  "modifiers",
  "reference",
  "tags",
] as const;

export type StudioBuiltInCodegenTarget = (typeof studioCodegenTargets)[number];
export type StudioCodegenTarget = StudioBuiltInCodegenTarget;
export type StudioGeneratedTargetId = string;

const studioCodegenTargetSet: ReadonlySet<string> = new Set(studioCodegenTargets);

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

export interface StudioGeneratedTargetPlanContext<Content = unknown, Config = unknown> {
  config: Config;
  content: Content;
  outputDir: string;
  targetId: StudioGeneratedTargetId;
}

export interface StudioGeneratedTargetDefinition<
  Content = unknown,
  Config = unknown,
  TargetId extends StudioGeneratedTargetId = StudioGeneratedTargetId,
> {
  cleanup?: StudioGeneratedTargetCleanupPolicy;
  dependencies?: readonly StudioGeneratedTargetId[];
  id: TargetId;
  label: string;
  plan: (
    context: StudioGeneratedTargetPlanContext<Content, Config>,
  ) => Promise<StudioGeneratedTargetPlanResult> | StudioGeneratedTargetPlanResult;
}

export const defineStudioGeneratedTarget = <
  Content = unknown,
  Config = unknown,
  const TargetId extends StudioGeneratedTargetId = StudioGeneratedTargetId,
>(
  target: StudioGeneratedTargetDefinition<Content, Config, TargetId>,
): StudioGeneratedTargetDefinition<Content, Config, TargetId> => target;

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
  target: StudioGeneratedTargetId;
}

export interface StudioCodegenTargetSummary {
  files: StudioGeneratedFileDiff[];
  label: string;
  target: StudioGeneratedTargetId;
}

export interface RuntimeHookSummary {
  hook: string;
  path: string;
  status: "created" | "existing" | "missing" | "orphan" | "skipped";
}

export const isBuiltInStudioCodegenTarget = (value: string): value is StudioBuiltInCodegenTarget =>
  studioCodegenTargetSet.has(value);
