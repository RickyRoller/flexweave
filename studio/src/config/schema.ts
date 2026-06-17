import type { StudioProjectConfig } from "./types";

export { validateStudioConfig } from "./project-config-validation";
export { STUDIO_CONFIG_FILE_NAME } from "./types";
export type { StudioDataAdapterRegistry } from "./data-adapter-registry";
export {
  composeHostAppContributionModel,
  hostAppContributionModelValues,
  mergeHostAppContributionModels,
  normalizeHostAppContributionModel,
  normalizeHostAppContributions,
  validateHostAppContributionModel,
} from "./host-app-contribution-validation";
export type {
  StudioHostAppContributionModel,
  StudioHostAppContributionModelEntry,
  StudioHostAppContributionModelInput,
  StudioHostAppContributionModelValidationContext,
  StudioHostAppContributionModelValues,
  StudioHostAppContributionSourceReference,
} from "./host-app-contribution-validation";
export type {
  ResolvedStudioProjectConfig,
  StudioConfigValidationResult,
  StudioDiagnostic,
  StudioProjectConfig,
  StudioVerifyCommand,
  StudioVerifyCommandInput,
} from "./types";

export const defineStudioConfig = <const Config extends StudioProjectConfig>(
  config: Config,
): Config => config;
