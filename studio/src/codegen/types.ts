export const studioCodegenTargets = [
  "abilities",
  "effects",
  "executions",
  "modifiers",
  "reference",
  "tags",
] as const;

export type StudioCodegenTarget = (typeof studioCodegenTargets)[number];

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
