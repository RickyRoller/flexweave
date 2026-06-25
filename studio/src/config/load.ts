import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { STUDIO_CONFIG_FILE_NAME, STUDIO_CONFIG_FILE_NAMES, validateStudioConfig } from "./schema";
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
    for (const fileName of STUDIO_CONFIG_FILE_NAMES) {
      const candidate = resolve(current, fileName);
      searched.push(candidate);
      if (existsSync(candidate)) {
        return { configPath: candidate, searched };
      }
    }

    const parent = dirname(current);
    if (parent === current) {
      return { searched };
    }
    current = parent;
  }
};

const loadConfigValue = async (configPath: string): Promise<unknown> => {
  if (configPath.endsWith(".json")) {
    return JSON.parse(readFileSync(configPath, "utf-8"));
  }

  const href = `${pathToFileURL(configPath).href}?studioConfigLoad=${Date.now()}`;
  const module = await import(href);
  return module.default ?? module.config;
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
    const configValue = await loadConfigValue(discovery.configPath);
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
