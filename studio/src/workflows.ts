const reservedStudioWorkflows = ["validate", "migrate", "verify"] as const;

export type StudioWorkflowName = (typeof reservedStudioWorkflows)[number];

export const listReservedStudioWorkflows = (): readonly StudioWorkflowName[] =>
  reservedStudioWorkflows;
