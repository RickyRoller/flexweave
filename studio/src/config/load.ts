import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { STUDIO_CONFIG_FILE_NAME, validateStudioConfig } from "./schema";
import type {
  ResolvedStudioProjectConfig,
  StudioConfigValidationResult,
  StudioDiagnostic,
} from "./schema";

export interface StudioConfigDiscoveryResult {
  configPath?: string;
  searched: string[];
}

export interface LoadStudioConfigOptions {
  configPath?: string;
  cwd?: string;
}

export interface LoadStudioConfigResult {
  config?: ResolvedStudioProjectConfig;
  diagnostics: StudioDiagnostic[];
  ok: boolean;
}

const failure = (diagnostics: StudioDiagnostic[]): LoadStudioConfigResult => ({
  diagnostics,
  ok: false,
});

export const findStudioConfig = (cwd = process.cwd()): StudioConfigDiscoveryResult => {
  const searched: string[] = [];
  let current = resolve(cwd);

  while (true) {
    const candidate = resolve(current, STUDIO_CONFIG_FILE_NAME);
    searched.push(candidate);
    if (existsSync(candidate)) {
      return { configPath: candidate, searched };
    }

    const parent = dirname(current);
    if (parent === current) {
      return { searched };
    }
    current = parent;
  }
};

export const loadStudioConfig = async (
  options: LoadStudioConfigOptions = {},
): Promise<LoadStudioConfigResult> => {
  const cwd = options.cwd ?? process.cwd();
  const discovery = options.configPath
    ? {
        configPath: resolve(cwd, options.configPath),
        searched: [resolve(cwd, options.configPath)],
      }
    : findStudioConfig(cwd);

  if (!discovery.configPath || !existsSync(discovery.configPath)) {
    return failure([
      {
        code: "missing-config",
        hint: `Create ${STUDIO_CONFIG_FILE_NAME} or pass --config <path>.`,
        message: `No Studio project config found. Searched: ${discovery.searched.join(", ")}`,
        severity: "error",
      },
    ]);
  }

  try {
    const href = `${pathToFileURL(discovery.configPath).href}?studioConfigLoad=${Date.now()}`;
    const module = await import(href);
    const configValue = module.default ?? module.config;
    return validateStudioConfig(configValue, {
      configDir: dirname(discovery.configPath),
      configPath: discovery.configPath,
    });
  } catch (error) {
    return failure([
      {
        code: "config-load-failed",
        message:
          error instanceof Error
            ? `Failed to load Studio project config: ${error.message}`
            : "Failed to load Studio project config.",
        path: discovery.configPath,
        severity: "error",
      },
    ]);
  }
};

export type { ResolvedStudioProjectConfig, StudioConfigValidationResult };
