import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import { loadStudioCatalog } from "../internal/catalog";
import { validateLoadedStudioCatalog } from "./catalog";
import { codegenLoadedStudioProject } from "./codegen";
import { verifyStudioHostApp } from "./host-app";
import { hasErrorDiagnostic, resolveWorkflowConfig, workflowError } from "./shared";
import type {
  CodegenStudioResult,
  StudioVerifyCheckResult,
  StudioVerifyCheckStatus,
  StudioVerifyCommandResult,
  StudioVerifyMode,
  StudioWorkflowOptions,
  ValidateStudioCatalogResult,
  VerifyStudioHostAppResult,
  VerifyStudioProjectResult,
} from "./types";

const verifyCheck = (
  input: Omit<StudioVerifyCheckResult, "diagnostics" | "mode" | "status"> & {
    diagnostics?: readonly StudioDiagnostic[];
    mode: StudioVerifyMode;
    passed: boolean;
    skipped?: boolean;
  },
): StudioVerifyCheckResult => {
  const { diagnostics: inputDiagnostics, passed, skipped, ...rest } = input;
  const diagnostics = [...(inputDiagnostics ?? [])];
  let status: StudioVerifyCheckStatus = "failed";
  if (skipped) {
    status = "skipped";
  } else if (passed && !hasErrorDiagnostic(diagnostics)) {
    status = "passed";
  }

  return {
    ...rest,
    diagnostics,
    status,
  };
};

const diagnosticsMatching = (
  diagnostics: readonly StudioDiagnostic[],
  patterns: readonly string[],
) =>
  diagnostics.filter((diagnostic) =>
    patterns.some(
      (pattern) =>
        diagnostic.message.includes(pattern) ||
        diagnostic.path?.includes(pattern) ||
        diagnostic.field?.includes(pattern),
    ),
  );

const sourceCheckDiagnostics = (
  validation: ValidateStudioCatalogResult,
  source: ResolvedStudioProjectConfig["data"]["sources"][number],
) =>
  validation.sourceDiagnostics
    .filter(
      (group) =>
        group.sourceId === source.id &&
        (group.adapterId === undefined || group.adapterId === source.adapterId),
    )
    .flatMap((group) => group.diagnostics);

const mapperCheckDiagnostics = (
  validation: ValidateStudioCatalogResult,
  extensionId: string,
  mapperId: string,
) =>
  validation.mapperDiagnostics
    .filter((group) => group.extensionId === extensionId && group.mapperId === mapperId)
    .flatMap((group) => group.diagnostics);

const buildVerifyChecks = (
  config: ResolvedStudioProjectConfig,
  validation: ValidateStudioCatalogResult,
  codegen: CodegenStudioResult,
  hostApp: VerifyStudioHostAppResult,
  commands: readonly StudioVerifyCommandResult[],
  mode: StudioVerifyMode,
): StudioVerifyCheckResult[] => {
  const checks: StudioVerifyCheckResult[] = [
    verifyCheck({
      diagnostics: [],
      mode,
      name: "config",
      passed: true,
    }),
    ...config.extensions.map((extension) =>
      verifyCheck({
        diagnostics: [],
        extensionId: extension.id,
        mode,
        name: `extension:${extension.id}`,
        passed: true,
      }),
    ),
    ...config.data.sources.map((source) => {
      const diagnostics = sourceCheckDiagnostics(validation, source);
      return verifyCheck({
        adapterId: source.adapterId,
        diagnostics,
        mode,
        name: `source:${source.id}`,
        passed: !hasErrorDiagnostic(diagnostics),
        sourceId: source.id,
      });
    }),
    ...config.extensions.flatMap((extension) =>
      (extension.contentMappers ?? []).map((mapper) => {
        const diagnostics = mapperCheckDiagnostics(validation, extension.id, mapper.id);
        return verifyCheck({
          diagnostics,
          extensionId: extension.id,
          mode,
          name: `mapper:${mapper.id}`,
          passed: !hasErrorDiagnostic(diagnostics),
        });
      }),
    ),
    verifyCheck({
      diagnostics: validation.diagnostics,
      mode,
      name: "validation",
      passed: validation.ok,
    }),
    ...codegen.targets.map((target) =>
      verifyCheck({
        diagnostics: diagnosticsMatching(codegen.diagnostics, [target.target]),
        mode,
        name: `generated-target:${target.target}`,
        passed: target.files.every((file) => file.status === "fresh"),
        targetId: target.target,
      }),
    ),
    verifyCheck({
      diagnostics: codegen.diagnostics,
      mode,
      name: "generated-freshness",
      passed: codegen.ok,
    }),
    verifyCheck({
      diagnostics: codegen.diagnostics.filter((diagnostic) =>
        diagnostic.code.includes("runtime-hook"),
      ),
      mode,
      name: "runtime-hooks",
      passed: codegen.hooks.every(
        (hook) => hook.status === "existing" || hook.status === "skipped",
      ),
    }),
    verifyCheck({
      diagnostics: hostApp.diagnostics,
      mode,
      name: "host-app",
      passed: hostApp.ok,
      skipped: hostApp.status === "not-configured",
    }),
    ...commands.map((command) =>
      verifyCheck({
        command: command.command,
        diagnostics:
          command.exitCode === 0
            ? []
            : [
                workflowError(
                  "verify-command-failed",
                  `Studio verify command failed: ${command.name}.`,
                  undefined,
                  command.command.join(" "),
                ),
              ],
        exitCode: command.exitCode,
        mode,
        name: `command:${command.name}`,
        passed: command.exitCode === 0,
        stderr: command.stderr,
        stdout: command.stdout,
      }),
    ),
  ];

  return checks;
};

const runStudioVerifyCommand = async (
  config: ResolvedStudioProjectConfig,
  commandConfig: { command: string[]; fast: boolean; name: string },
): Promise<StudioVerifyCommandResult> => {
  const proc = Bun.spawn(commandConfig.command, {
    cwd: config.configDir,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return {
    command: commandConfig.command,
    exitCode,
    fast: commandConfig.fast,
    name: commandConfig.name,
    stderr,
    stdout,
  };
};

export const verifyStudioProject = async (
  options: StudioWorkflowOptions & { appRoot?: string; fast?: boolean } = {},
): Promise<VerifyStudioProjectResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    const mode = options.fast ? "fast" : "full";
    const checks = [
      verifyCheck({
        diagnostics: resolved.diagnostics,
        mode,
        name: "config",
        passed: false,
      }),
    ];
    const emptyValidation: ValidateStudioCatalogResult = {
      diagnostics: resolved.diagnostics,
      mapperDiagnostics: [],
      ok: false,
      recordCount: 0,
      sourceDiagnostics: [],
      sourceRecordCount: 0,
      sources: [],
    };
    const emptyCodegen: CodegenStudioResult = {
      checked: true,
      diagnostics: resolved.diagnostics,
      hooks: [],
      ok: false,
      targets: [],
    };
    const emptyHostApp: VerifyStudioHostAppResult = {
      diagnostics: resolved.diagnostics,
      files: [],
      manualFollowUps: [],
      ok: false,
      status: "missing",
    };
    return {
      checks,
      codegen: emptyCodegen,
      commands: [],
      diagnostics: resolved.diagnostics,
      hostApp: emptyHostApp,
      ok: false,
      validation: emptyValidation,
    };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const validation = validateLoadedStudioCatalog(resolved.config, catalog);
  const codegen = await codegenLoadedStudioProject(resolved.config, catalog, { check: true });
  const hostApp = await verifyStudioHostApp({
    appRoot: options.appRoot,
    config: resolved.config,
  });
  const commandConfigs = options.fast
    ? resolved.config.verify.commands.filter((command) => command.fast)
    : resolved.config.verify.commands;
  const commands: StudioVerifyCommandResult[] = [];

  for (const commandConfig of commandConfigs) {
    commands.push(await runStudioVerifyCommand(resolved.config, commandConfig));
  }

  const commandDiagnostics = commands
    .filter((command) => command.exitCode !== 0)
    .map((command) =>
      workflowError(
        "verify-command-failed",
        `Studio verify command failed: ${command.name}.`,
        undefined,
        command.command.join(" "),
      ),
    );
  const diagnostics = [
    ...validation.diagnostics,
    ...codegen.diagnostics,
    ...hostApp.diagnostics,
    ...commandDiagnostics,
  ];
  const checks = buildVerifyChecks(
    resolved.config,
    validation,
    codegen,
    hostApp,
    commands,
    options.fast ? "fast" : "full",
  );

  return {
    checks,
    codegen,
    commands,
    diagnostics,
    hostApp,
    ok:
      validation.ok &&
      codegen.ok &&
      hostApp.ok &&
      commands.every((command) => command.exitCode === 0) &&
      diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    validation,
  };
};
