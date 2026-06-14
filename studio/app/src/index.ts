export interface StudioAppSurface {
  surface: "Flexweave Studio app";
  requiresProjectAdapter: true;
}

export const createStudioAppPlaceholder = (): StudioAppSurface => ({
  requiresProjectAdapter: true,
  surface: "Flexweave Studio app",
});
