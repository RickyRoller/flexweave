import type { StudioBuiltInCodegenTarget, StudioCodegenTarget } from "../codegen/types";
import type {
  StudioDataAdapter,
  StudioExtension,
  StudioSourceConfig,
  StudioSourceLocation,
} from "../extensions";

export const STUDIO_CONFIG_FILE_NAME = "studio.config.ts";

export interface StudioDiagnostic {
  code: string;
  source?: StudioSourceLocation;
  field?: string;
  hint?: string;
  message: string;
  path?: string;
  severity: "error" | "warning";
}

export interface StudioVerifyCommandInput {
  command: readonly string[];
  fast?: boolean;
  name: string;
}

export interface StudioVerifyCommand {
  command: string[];
  fast: boolean;
  name: string;
}

export interface StudioProjectConfig {
  app?: {
    buildCommand?: readonly string[];
    checkCommand?: readonly string[];
    root?: string;
  };
  catalogRoot: string;
  codegen?: {
    allowOverlappingOutputDirs?: boolean;
    builtInTargets?: readonly StudioBuiltInCodegenTarget[];
    outputDirs?: Partial<Record<string, string>>;
  };
  data?: {
    adapters?: readonly StudioDataAdapter[];
    sources?: readonly StudioSourceConfig[];
  };
  extensions?: readonly StudioExtension[];
  hooks?: {
    dir?: string;
    testStubsDir?: string;
  };
  mode?: "full" | "validate-only";
  rust?: {
    bindings?: Record<string, unknown>;
    flexweaveModule?: string;
    generatedHeader?: string;
    macroNames?: Record<string, string>;
    moduleAliases?: Record<string, string>;
    preludeImports?: readonly string[];
    runtimeVocab?: {
      ailments?: readonly string[];
      damageTypes?: readonly string[];
    };
    typePaths?: Record<string, string>;
  };
  verify?: {
    commands?: readonly StudioVerifyCommandInput[];
  };
}

export interface ResolvedStudioProjectConfig {
  app: {
    buildCommand?: string[];
    checkCommand?: string[];
  };
  configDir: string;
  configPath: string;
  mode: "full" | "validate-only";
  codegen: {
    allowOverlappingOutputDirs: boolean;
    builtInTargets: StudioBuiltInCodegenTarget[];
  };
  data: {
    adapters: StudioDataAdapter[];
    sources: StudioSourceConfig[];
  };
  extensions: StudioExtension[];
  paths: {
    app: {
      root?: string;
    };
    catalogRoot: string;
    codegen: {
      outputDirs: Record<StudioCodegenTarget, string>;
    };
    hooks: {
      dir?: string;
      testStubsDir?: string;
    };
  };
  raw: StudioProjectConfig;
  rust?: {
    bindings: Record<string, unknown>;
    flexweaveModule: string;
    generatedHeader?: string;
    macroNames: Record<string, string>;
    moduleAliases: Record<string, string>;
    preludeImports: string[];
    runtimeVocab: {
      ailments: string[];
      damageTypes: string[];
    };
    typePaths: Record<string, string>;
  };
  verify: {
    commands: StudioVerifyCommand[];
  };
}

export interface StudioConfigValidationResult {
  config?: ResolvedStudioProjectConfig;
  diagnostics: StudioDiagnostic[];
  ok: boolean;
}
