import { isAbsolute, normalize, relative, resolve } from "node:path";

import { isStudioCodegenTarget, studioCodegenTargets } from "../codegen/types";
import type { StudioCodegenTarget } from "../codegen/types";
import type {
  StudioDataAdapter,
  StudioDataAdapterCapability,
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
    outputDirs?: Partial<Record<StudioCodegenTarget, string>>;
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
    flexweaveModule?: string;
    runtimeVocab?: {
      ailments?: readonly string[];
      damageTypes?: readonly string[];
    };
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
    flexweaveModule: string;
    runtimeVocab: {
      ailments: string[];
      damageTypes: string[];
    };
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

export const defineStudioConfig = <const Config extends StudioProjectConfig>(
  config: Config,
): Config => config;

const error = (code: string, field: string, message: string, hint?: string): StudioDiagnostic => ({
  code,
  field,
  hint,
  message,
  severity: "error",
});

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const readString = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): string | undefined => {
  if (typeof value === "string" && value.trim().length > 0) {
    return value;
  }

  diagnostics.push(
    error(
      "invalid-config-field",
      field,
      `Studio project config field ${field} must be a non-empty string.`,
    ),
  );
  return undefined;
};

const resolveConfigPath = (configDir: string, value: string) =>
  normalize(isAbsolute(value) ? value : resolve(configDir, value));

const normalizeStringArray = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): string[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        field,
        `Studio project config field ${field} must be an array of strings.`,
      ),
    );
    return [];
  }

  const normalized: string[] = [];
  for (const [index, item] of value.entries()) {
    const itemField = `${field}.${index}`;
    if (typeof item !== "string" || item.trim().length === 0) {
      diagnostics.push(
        error(
          "invalid-config-field",
          itemField,
          `Studio project config field ${itemField} must be a non-empty string.`,
        ),
      );
      continue;
    }
    normalized.push(item);
  }

  return normalized;
};

const validAdapterCapabilities: readonly StudioDataAdapterCapability[] = [
  "diff",
  "read",
  "schema",
  "watch",
  "write",
];

const hasObjectShape = (value: unknown): value is Record<string, unknown> => isObject(value);

const validateCapabilities = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioDataAdapterCapability[] => {
  if (!Array.isArray(value)) {
    diagnostics.push(
      error(
        "invalid-data-adapter",
        field,
        `Studio data adapter field ${field} must be an array of capabilities.`,
        `Expected one or more of: ${validAdapterCapabilities.join(", ")}.`,
      ),
    );
    return [];
  }

  const capabilities: StudioDataAdapterCapability[] = [];
  for (const [index, item] of value.entries()) {
    const itemField = `${field}.${index}`;
    if (
      typeof item !== "string" ||
      !(validAdapterCapabilities as readonly string[]).includes(item)
    ) {
      diagnostics.push(
        error(
          "invalid-data-adapter",
          itemField,
          `Studio data adapter capability ${itemField} is not supported.`,
          `Expected one of: ${validAdapterCapabilities.join(", ")}.`,
        ),
      );
      continue;
    }
    if (!capabilities.includes(item as StudioDataAdapterCapability)) {
      capabilities.push(item as StudioDataAdapterCapability);
    }
  }

  if (capabilities.length === 0) {
    diagnostics.push(
      error(
        "invalid-data-adapter",
        field,
        `Studio data adapter field ${field} must include at least one capability.`,
      ),
    );
  }

  return capabilities;
};

const validateDataAdapter = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioDataAdapter | undefined => {
  if (!hasObjectShape(value)) {
    diagnostics.push(
      error(
        "invalid-data-adapter",
        field,
        `Studio data adapter ${field} must be an object returned by defineStudioDataAdapter.`,
      ),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const label =
    value.label === undefined ? undefined : readString(value.label, `${field}.label`, diagnostics);
  const capabilities = validateCapabilities(
    value.capabilities,
    `${field}.capabilities`,
    diagnostics,
  );

  if (typeof value.load !== "function") {
    diagnostics.push(
      error(
        "invalid-data-adapter",
        `${field}.load`,
        `Studio data adapter ${field} must provide a load function.`,
      ),
    );
  }

  if (value.write !== undefined && typeof value.write !== "function") {
    diagnostics.push(
      error(
        "invalid-data-adapter",
        `${field}.write`,
        `Studio data adapter ${field}.write must be a function when provided.`,
      ),
    );
  }

  if (capabilities.includes("write") && typeof value.write !== "function") {
    diagnostics.push(
      error(
        "invalid-data-adapter",
        `${field}.write`,
        `Writable Studio data adapter ${field} must provide a write function.`,
      ),
    );
  }

  if (!id || typeof value.load !== "function" || capabilities.length === 0) {
    return undefined;
  }

  return {
    ...(value as unknown as StudioDataAdapter),
    capabilities,
    id,
    label,
  };
};

const validateDataAdapters = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioDataAdapter[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        field,
        `Studio project config field ${field} must be an array of data adapters.`,
      ),
    );
    return [];
  }

  const adapters: StudioDataAdapter[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const adapter = validateDataAdapter(item, `${field}.${index}`, diagnostics);
    if (!adapter) {
      continue;
    }
    if (seen.has(adapter.id)) {
      diagnostics.push(
        error(
          "duplicate-data-adapter",
          `${field}.${index}.id`,
          `Studio data adapter "${adapter.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(adapter.id);
    adapters.push(adapter);
  }

  return adapters;
};

const validateStudioExtension = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioExtension | undefined => {
  if (!hasObjectShape(value)) {
    diagnostics.push(
      error(
        "invalid-studio-extension",
        field,
        `Studio extension ${field} must be an object returned by defineStudioExtension.`,
      ),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const label =
    value.label === undefined ? undefined : readString(value.label, `${field}.label`, diagnostics);
  const dataAdapters = validateDataAdapters(
    value.dataAdapters,
    `${field}.dataAdapters`,
    diagnostics,
  );

  if (value.validateSources !== undefined && typeof value.validateSources !== "function") {
    diagnostics.push(
      error(
        "invalid-studio-extension",
        `${field}.validateSources`,
        `Studio extension ${field}.validateSources must be a function when provided.`,
      ),
    );
  }

  if (!id) {
    return undefined;
  }

  return {
    ...(value as unknown as StudioExtension),
    dataAdapters,
    id,
    label,
  };
};

const validateStudioExtensions = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): StudioExtension[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "extensions",
        "Studio project config field extensions must be an array of Studio extensions.",
      ),
    );
    return [];
  }

  const extensions: StudioExtension[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const extension = validateStudioExtension(item, `extensions.${index}`, diagnostics);
    if (!extension) {
      continue;
    }
    if (seen.has(extension.id)) {
      diagnostics.push(
        error(
          "duplicate-studio-extension",
          `extensions.${index}.id`,
          `Studio extension "${extension.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(extension.id);
    extensions.push(extension);
  }

  return extensions;
};

const validateSourceConfig = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioSourceConfig | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      error("invalid-source-config", field, `Studio source ${field} must be an object.`),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const adapterId = readString(value.adapterId, `${field}.adapterId`, diagnostics);
  const label =
    value.label === undefined ? undefined : readString(value.label, `${field}.label`, diagnostics);
  if (value.options !== undefined && !isObject(value.options)) {
    diagnostics.push(
      error(
        "invalid-source-config",
        `${field}.options`,
        `Studio source field ${field}.options must be an object when provided.`,
      ),
    );
  }

  if (!id || !adapterId) {
    return undefined;
  }

  return {
    adapterId,
    id,
    label,
    options: isObject(value.options) ? value.options : undefined,
  };
};

const validateDataConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
  extensionAdapters: StudioDataAdapter[],
): { adapters: StudioDataAdapter[]; sources: StudioSourceConfig[] } => {
  if (value.data === undefined) {
    return {
      adapters: [],
      sources: [],
    };
  }

  if (!isObject(value.data)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "data",
        "Studio project config field data must be an object when provided.",
      ),
    );
    return {
      adapters: [],
      sources: [],
    };
  }

  const adapters = validateDataAdapters(value.data.adapters, "data.adapters", diagnostics);
  const sourcesValue = value.data.sources;
  const sources: StudioSourceConfig[] = [];
  if (sourcesValue !== undefined && !Array.isArray(sourcesValue)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "data.sources",
        "Studio project config field data.sources must be an array of source declarations.",
      ),
    );
  } else {
    for (const [index, item] of (sourcesValue ?? []).entries()) {
      const source = validateSourceConfig(item, `data.sources.${index}`, diagnostics);
      if (source) {
        sources.push(source);
      }
    }
  }

  const availableAdapters = new Set(
    [...adapters, ...extensionAdapters].map((adapter) => adapter.id),
  );
  for (const [index, source] of sources.entries()) {
    if (!availableAdapters.has(source.adapterId)) {
      diagnostics.push(
        error(
          "missing-data-adapter",
          `data.sources.${index}.adapterId`,
          `Studio source "${source.id}" references missing data adapter "${source.adapterId}".`,
          "Register the adapter in data.adapters or through an active Studio extension.",
        ),
      );
    }
  }

  return { adapters, sources };
};

const validateVerifyCommands = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): StudioVerifyCommand[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "verify.commands",
        "Studio project config field verify.commands must be an array.",
      ),
    );
    return [];
  }

  const commands: StudioVerifyCommand[] = [];
  for (const [index, commandValue] of value.entries()) {
    const field = `verify.commands.${index}`;
    if (!isObject(commandValue)) {
      diagnostics.push(
        error(
          "invalid-config-field",
          field,
          `Studio project config field ${field} must be an object.`,
        ),
      );
      continue;
    }

    const name = readString(commandValue.name, `${field}.name`, diagnostics);
    const command = normalizeStringArray(commandValue.command, `${field}.command`, diagnostics);
    if (command.length === 0) {
      diagnostics.push(
        error(
          "invalid-config-field",
          `${field}.command`,
          `Studio project config field ${field}.command must include at least one argument.`,
        ),
      );
    }

    if (commandValue.fast !== undefined && typeof commandValue.fast !== "boolean") {
      diagnostics.push(
        error(
          "invalid-config-field",
          `${field}.fast`,
          `Studio project config field ${field}.fast must be a boolean when provided.`,
        ),
      );
    }

    if (name && command.length > 0) {
      commands.push({
        command,
        fast: commandValue.fast === true,
        name,
      });
    }
  }

  return commands;
};

const validateVerifyConfig = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): StudioVerifyCommand[] => {
  if (value === undefined) {
    return [];
  }

  if (!isObject(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "verify",
        "Studio project config field verify must be an object when provided.",
      ),
    );
    return [];
  }

  return validateVerifyCommands(value.commands, diagnostics);
};

const validateOptionalCommand = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): string[] | undefined => {
  if (value === undefined) {
    return undefined;
  }

  const command = normalizeStringArray(value, field, diagnostics);
  if (command.length === 0) {
    diagnostics.push(
      error(
        "invalid-config-field",
        field,
        `Studio project config field ${field} must include at least one argument when provided.`,
      ),
    );
    return undefined;
  }

  return command;
};

const validateAppConfig = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): { buildCommand?: string[]; checkCommand?: string[]; root?: string } => {
  if (value === undefined) {
    return {};
  }

  if (!isObject(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "app",
        "Studio project config field app must be an object when provided.",
      ),
    );
    return {};
  }

  return {
    buildCommand: validateOptionalCommand(value.buildCommand, "app.buildCommand", diagnostics),
    checkCommand: validateOptionalCommand(value.checkCommand, "app.checkCommand", diagnostics),
    root: value.root === undefined ? undefined : readString(value.root, "app.root", diagnostics),
  };
};

const validateRuntimeVocabConfig = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): { ailments: string[]; damageTypes: string[] } => {
  if (value === undefined) {
    return {
      ailments: [],
      damageTypes: [],
    };
  }

  if (!isObject(value)) {
    diagnostics.push(
      error(
        "invalid-config-field",
        "rust.runtimeVocab",
        "Studio project config field rust.runtimeVocab must be an object when provided.",
      ),
    );
    return {
      ailments: [],
      damageTypes: [],
    };
  }

  return {
    ailments: normalizeStringArray(value.ailments, "rust.runtimeVocab.ailments", diagnostics),
    damageTypes: normalizeStringArray(
      value.damageTypes,
      "rust.runtimeVocab.damageTypes",
      diagnostics,
    ),
  };
};

const validateDuplicateOwnedPaths = (
  paths: Record<string, string | undefined>,
  diagnostics: StudioDiagnostic[],
) => {
  const byPath: Record<string, string | undefined> = {};
  for (const [field, value] of Object.entries(paths)) {
    if (!value) {
      continue;
    }

    const existing = byPath[value];
    if (existing) {
      diagnostics.push(
        error(
          "duplicate-owned-path",
          field,
          `Studio project config fields ${existing} and ${field} resolve to the same owned path.`,
          "Use distinct directories for generated targets and runtime hook roots.",
        ),
      );
      continue;
    }

    byPath[value] = field;
  }
};

const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

const validateAmbiguousOwnedPaths = (
  paths: Record<string, string | undefined>,
  diagnostics: StudioDiagnostic[],
) => {
  const entries = Object.entries(paths).filter(
    (entry): entry is [string, string] => typeof entry[1] === "string" && entry[1].length > 0,
  );

  for (let leftIndex = 0; leftIndex < entries.length; leftIndex += 1) {
    for (let rightIndex = leftIndex + 1; rightIndex < entries.length; rightIndex += 1) {
      const [leftField, leftPath] = entries[leftIndex];
      const [rightField, rightPath] = entries[rightIndex];
      if (leftPath === rightPath) {
        continue;
      }

      if (pathContains(leftPath, rightPath) || pathContains(rightPath, leftPath)) {
        diagnostics.push(
          error(
            "ambiguous-owned-path",
            rightField,
            `Studio project config fields ${leftField} and ${rightField} overlap owned paths.`,
            "Use sibling directories instead of nesting generated targets or runtime hook roots.",
          ),
        );
      }
    }
  }
};

interface FullConfigFields {
  flexweaveModule?: string;
  hookDir?: string;
  hookTestStubsDir?: string;
  outputDirs: Partial<Record<StudioCodegenTarget, string>>;
}

const validateCodegenConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): Partial<Record<StudioCodegenTarget, string>> => {
  const outputDirs: Partial<Record<StudioCodegenTarget, string>> = {};
  const codegenValue = value.codegen;

  if (isObject(codegenValue) && isObject(codegenValue.outputDirs)) {
    for (const target of studioCodegenTargets) {
      const configuredPath = readString(
        codegenValue.outputDirs[target],
        `codegen.outputDirs.${target}`,
        diagnostics,
      );
      if (configuredPath) {
        outputDirs[target] = configuredPath;
      }
    }

    for (const key of Object.keys(codegenValue.outputDirs)) {
      if (!isStudioCodegenTarget(key)) {
        diagnostics.push(
          error(
            "unknown-codegen-target",
            `codegen.outputDirs.${key}`,
            `Unknown Studio generated output target "${key}".`,
            `Expected one of: ${studioCodegenTargets.join(", ")}.`,
          ),
        );
      }
    }

    return outputDirs;
  }

  diagnostics.push(
    error(
      "missing-config-field",
      "codegen.outputDirs",
      "Full Studio project configs must declare codegen.outputDirs.",
    ),
  );
  return outputDirs;
};

const validateHookConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): Pick<FullConfigFields, "hookDir" | "hookTestStubsDir"> => {
  if (isObject(value.hooks)) {
    return {
      hookDir: readString(value.hooks.dir, "hooks.dir", diagnostics),
      hookTestStubsDir:
        value.hooks.testStubsDir === undefined
          ? undefined
          : readString(value.hooks.testStubsDir, "hooks.testStubsDir", diagnostics),
    };
  }

  diagnostics.push(
    error("missing-config-field", "hooks", "Full Studio project configs must declare hooks.dir."),
  );
  return {};
};

const validateRustConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): string | undefined => {
  if (isObject(value.rust)) {
    return readString(value.rust.flexweaveModule, "rust.flexweaveModule", diagnostics);
  }

  diagnostics.push(
    error(
      "missing-config-field",
      "rust",
      "Full Studio project configs must declare rust.flexweaveModule.",
    ),
  );
  return undefined;
};

const validateFullConfigFields = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): FullConfigFields => ({
  ...validateHookConfig(value, diagnostics),
  flexweaveModule: validateRustConfig(value, diagnostics),
  outputDirs: validateCodegenConfig(value, diagnostics),
});

export const validateStudioConfig = (
  value: unknown,
  options: { configDir: string; configPath: string },
): StudioConfigValidationResult => {
  const diagnostics: StudioDiagnostic[] = [];

  if (!isObject(value)) {
    return {
      diagnostics: [
        {
          code: "invalid-config",
          message: "Studio project config must export an object.",
          severity: "error",
        },
      ],
      ok: false,
    };
  }

  const raw = value as unknown as StudioProjectConfig;
  const catalogRoot = readString(raw.catalogRoot, "catalogRoot", diagnostics);
  const mode = raw.mode ?? "full";
  if (mode !== "full" && mode !== "validate-only") {
    diagnostics.push(
      error(
        "invalid-config-field",
        "mode",
        'Studio project config field mode must be "full" or "validate-only".',
      ),
    );
  }

  const fullFields =
    mode === "full" ? validateFullConfigFields(raw, diagnostics) : { outputDirs: {} };

  const verifyCommands = validateVerifyConfig(raw.verify, diagnostics);
  const appConfig = validateAppConfig(raw.app, diagnostics);
  const extensions = validateStudioExtensions(raw.extensions, diagnostics);
  const extensionAdapters = extensions.flatMap((extension) => extension.dataAdapters ?? []);
  const data = validateDataConfig(raw, diagnostics, extensionAdapters);
  const runtimeVocab = validateRuntimeVocabConfig(
    isObject(raw.rust) ? raw.rust.runtimeVocab : undefined,
    diagnostics,
  );

  const resolvedOutputDirs = Object.fromEntries(
    studioCodegenTargets.map((target) => [
      target,
      fullFields.outputDirs[target]
        ? resolveConfigPath(options.configDir, fullFields.outputDirs[target])
        : "",
    ]),
  ) as Record<StudioCodegenTarget, string>;
  const resolvedHookDir = fullFields.hookDir
    ? resolveConfigPath(options.configDir, fullFields.hookDir)
    : undefined;
  const resolvedHookTestStubsDir = fullFields.hookTestStubsDir
    ? resolveConfigPath(options.configDir, fullFields.hookTestStubsDir)
    : undefined;
  const resolvedAppRoot = appConfig.root
    ? resolveConfigPath(options.configDir, appConfig.root)
    : undefined;

  if (mode === "full") {
    const ownedPaths = {
      ...Object.fromEntries(
        studioCodegenTargets.map((target) => [
          `codegen.outputDirs.${target}`,
          resolvedOutputDirs[target],
        ]),
      ),
      "hooks.dir": resolvedHookDir,
      "hooks.testStubsDir": resolvedHookTestStubsDir,
    };
    validateDuplicateOwnedPaths(ownedPaths, diagnostics);
    validateAmbiguousOwnedPaths(ownedPaths, diagnostics);
  }

  if (!catalogRoot || diagnostics.some((diagnostic) => diagnostic.severity === "error")) {
    return { diagnostics, ok: false };
  }

  return {
    config: {
      app: {
        buildCommand: appConfig.buildCommand,
        checkCommand: appConfig.checkCommand,
      },
      configDir: options.configDir,
      configPath: options.configPath,
      data,
      extensions,
      mode,
      paths: {
        app: {
          root: resolvedAppRoot,
        },
        catalogRoot: resolveConfigPath(options.configDir, catalogRoot),
        codegen: {
          outputDirs: resolvedOutputDirs,
        },
        hooks: {
          dir: resolvedHookDir,
          testStubsDir: resolvedHookTestStubsDir,
        },
      },
      raw,
      rust: fullFields.flexweaveModule
        ? {
            flexweaveModule: fullFields.flexweaveModule,
            runtimeVocab,
          }
        : undefined,
      verify: {
        commands: verifyCommands,
      },
    },
    diagnostics,
    ok: true,
  };
};
