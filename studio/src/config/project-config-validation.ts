import type { StudioBuiltInCodegenTarget, StudioCodegenTarget } from "../codegen/types";
import { validateDataConfig } from "./data-config-validation";
import { configError } from "./diagnostics";
import {
  readAllowOverlappingOutputDirs,
  validateBuiltInCodegenTargets,
  validateCodegenConfig,
} from "./generated-target-validation";
import { validateOwnedPathPolicy } from "./owned-path-policy";
import { isObject, normalizeStringArray, readString, resolveConfigPath } from "./primitive-readers";
import {
  validateExtensionRustBindings,
  validateRustConfig,
} from "./rust-codegen-context-validation";
import { validateStudioExtensions } from "./studio-extension-validation";
import type {
  ResolvedStudioProjectConfig,
  StudioConfigValidationResult,
  StudioDiagnostic,
  StudioProjectConfig,
  StudioVerifyCommand,
} from "./types";

interface FullConfigFields {
  allowOverlappingOutputDirs: boolean;
  builtInTargets: StudioBuiltInCodegenTarget[];
  hookDir?: string;
  hookTestStubsDir?: string;
  outputDirs: Partial<Record<string, string>>;
  rust?: ResolvedStudioProjectConfig["rust"];
}

const validateVerifyCommands = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): StudioVerifyCommand[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
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
        configError(
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
        configError(
          "invalid-config-field",
          `${field}.command`,
          `Studio project config field ${field}.command must include at least one argument.`,
        ),
      );
    }

    if (commandValue.fast !== undefined && typeof commandValue.fast !== "boolean") {
      diagnostics.push(
        configError(
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
      configError(
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
      configError(
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
      configError(
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
    configError(
      "missing-config-field",
      "hooks",
      "Full Studio project configs must declare hooks.dir.",
    ),
  );
  return {};
};

const validateFullConfigFields = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
  builtInTargetIds: readonly StudioBuiltInCodegenTarget[],
  extensionTargetIds: readonly string[],
): FullConfigFields => ({
  allowOverlappingOutputDirs: readAllowOverlappingOutputDirs(value, diagnostics),
  builtInTargets: [...builtInTargetIds],
  ...validateHookConfig(value, diagnostics),
  outputDirs: validateCodegenConfig(value, diagnostics, builtInTargetIds, extensionTargetIds),
  rust: validateRustConfig(value, diagnostics),
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
      configError(
        "invalid-config-field",
        "mode",
        'Studio project config field mode must be "full" or "validate-only".',
      ),
    );
  }

  const extensions = validateStudioExtensions(raw.extensions, diagnostics);
  const extensionTargetIds = extensions.flatMap((extension) =>
    (extension.generatedTargets ?? []).map((target) => target.id),
  );
  const builtInTargetIds =
    mode === "full" ? validateBuiltInCodegenTargets(raw.codegen?.builtInTargets, diagnostics) : [];
  const seenTargetIds = new Set<string>(builtInTargetIds);
  for (const targetId of extensionTargetIds) {
    if (seenTargetIds.has(targetId)) {
      diagnostics.push(
        configError(
          "duplicate-generated-target",
          "extensions.generatedTargets",
          `Studio generated target "${targetId}" is registered more than once or shadows an active built-in target.`,
        ),
      );
      continue;
    }
    seenTargetIds.add(targetId);
  }
  const fullFields: FullConfigFields =
    mode === "full"
      ? validateFullConfigFields(raw, diagnostics, builtInTargetIds, extensionTargetIds)
      : { allowOverlappingOutputDirs: false, builtInTargets: [], outputDirs: {} };
  diagnostics.push(...validateExtensionRustBindings(extensions, fullFields.rust));

  const verifyCommands = validateVerifyConfig(raw.verify, diagnostics);
  const appConfig = validateAppConfig(raw.app, diagnostics);
  const extensionAdapters = extensions.flatMap((extension) => extension.dataAdapters ?? []);
  const data = validateDataConfig(raw, diagnostics, extensionAdapters);

  const resolvedOutputDirs = Object.fromEntries(
    [...fullFields.builtInTargets, ...extensionTargetIds].map((target) => [
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
        fullFields.builtInTargets.map((target) => [
          `codegen.outputDirs.${target}`,
          resolvedOutputDirs[target],
        ]),
      ),
      ...Object.fromEntries(
        extensionTargetIds.map((target) => [
          `codegen.outputDirs.${target}`,
          resolvedOutputDirs[target],
        ]),
      ),
      "hooks.dir": resolvedHookDir,
      "hooks.testStubsDir": resolvedHookTestStubsDir,
    };
    validateOwnedPathPolicy(ownedPaths, diagnostics, {
      allowOverlappingOutputDirs: fullFields.allowOverlappingOutputDirs,
    });
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
      codegen: {
        allowOverlappingOutputDirs: fullFields.allowOverlappingOutputDirs,
        builtInTargets: fullFields.builtInTargets,
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
      rust: fullFields.rust,
      verify: {
        commands: verifyCommands,
      },
    },
    diagnostics,
    ok: true,
  };
};
