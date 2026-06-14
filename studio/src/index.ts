export { defineStudioConfig, minimalStudioProjectConfig } from "./config";
export type { StudioProjectConfig } from "./config";
export { listReservedStudioWorkflows } from "./workflows";
export type { StudioWorkflowName } from "./workflows";

export interface StudioPackageStatus {
  surface: "Flexweave Studio package";
  message: string;
}

export const studioPackageStatus = (): StudioPackageStatus => ({
  message: "phase-one placeholder ready for workspace verification",
  surface: "Flexweave Studio package",
});
