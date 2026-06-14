export interface StudioProjectConfig {
  catalogRoot: string;
  generatedOutputRoot: string;
  runtimeHooksRoot: string;
}

export const defineStudioConfig = <const Config extends StudioProjectConfig>(
  config: Config,
): Config => config;

export const minimalStudioProjectConfig = defineStudioConfig({
  catalogRoot: "catalog",
  generatedOutputRoot: "generated",
  runtimeHooksRoot: "runtime-hooks",
});
