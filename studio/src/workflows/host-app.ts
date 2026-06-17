import { existsSync, mkdirSync, rmdirSync } from "node:fs";
import { dirname, join, relative } from "node:path";

import { resolveConfigPath } from "../config/primitive-readers";
import type { ResolvedStudioProjectConfig } from "../config/schema";
import { readTextIfExists, restoreSnapshots, snapshotPaths, writeTextFile } from "../internal/files";
import { STUDIO_HOST_APP_SCAFFOLD_VERSION } from "./constants";
import { resolveWorkflowConfig, workflowError, workflowWarning } from "./shared";
import type {
  ScaffoldStudioHostAppOptions,
  ScaffoldStudioHostAppResult,
  StudioHostAppFileResult,
  StudioVerifyCommandResult,
  VerifyStudioHostAppResult,
} from "./types";

export const hostAppRoot = (config: ResolvedStudioProjectConfig, appRoot?: string) =>
  appRoot
    ? resolveConfigPath(config.configDir, appRoot)
    : (config.paths.app.root ?? resolveConfigPath(config.configDir, "studio-host"));

const hostAppConfigPath = (config: ResolvedStudioProjectConfig, fromDir: string) => {
  const path = relative(fromDir, config.configPath).replaceAll("\\", "/");
  return path.startsWith(".") ? path : `./${path}`;
};

export const hostAppMetadataPath = (root: string) => join(root, ".flexweave-studio-app.json");

export const hostAppPackagePath = (root: string) => join(root, "package.json");

const hostAppManagedFiles = [
  ".flexweave-studio-app.json",
  "package.json",
  "src/main.ts",
  "tsconfig.json",
];

const hostAppProjectAdapterPath = "src/project-adapter.ts";
const hostAppDefaultAdapterFactoryName = "createDefaultStudioProjectAdapter";
const hostAppProjectAdapterResultName = "projectAdapterResult";
const hostAppProjectOwnedFiles = [hostAppProjectAdapterPath];

type HostAppScaffoldPlanMode = "verify" | "write";
type HostAppWriteStatus = Extract<StudioHostAppFileResult["status"], "created" | "updated">;

interface HostAppScaffoldMetadata {
  files?: string[];
  managedFiles?: string[];
  packageName?: string;
  packageRefs?: Record<string, string>;
  projectOwnedFiles?: string[];
  scaffold?: string;
  studioPackageName?: string;
  version?: number;
}

interface HostAppScaffoldPlanOptions {
  mode: HostAppScaffoldPlanMode;
  requireDefaultProjectAdapter?: boolean;
  updateMetadata: boolean;
}

interface PlannedHostAppWrite {
  path: string;
  status: HostAppWriteStatus;
  value: string;
}

interface HostAppScaffoldPlan {
  files: StudioHostAppFileResult[];
  writes: PlannedHostAppWrite[];
}

interface PreparedHostAppScaffoldWrite extends HostAppScaffoldPlan {
  rollback: () => void;
  write: () => StudioHostAppFileResult[];
}

const stringArrayField = (value: unknown): string[] | undefined =>
  Array.isArray(value) && value.every((item) => typeof item === "string") ? [...value] : undefined;

const stringRecordField = (value: unknown): Record<string, string> | undefined =>
  typeof value === "object" &&
  value !== null &&
  !Array.isArray(value) &&
  Object.values(value).every((item) => typeof item === "string")
    ? { ...(value as Record<string, string>) }
    : undefined;

export const readHostAppMetadata = (root: string): HostAppScaffoldMetadata | undefined => {
  const value = readTextIfExists(hostAppMetadataPath(root));
  if (!value) {
    return undefined;
  }

  try {
    const parsed = JSON.parse(value) as Record<string, unknown>;
    return {
      files: stringArrayField(parsed.files),
      managedFiles: stringArrayField(parsed.managedFiles),
      packageName: typeof parsed.packageName === "string" ? parsed.packageName : undefined,
      packageRefs: stringRecordField(parsed.packageRefs),
      projectOwnedFiles: stringArrayField(parsed.projectOwnedFiles),
      scaffold: typeof parsed.scaffold === "string" ? parsed.scaffold : undefined,
      studioPackageName:
        typeof parsed.studioPackageName === "string" ? parsed.studioPackageName : undefined,
      version: typeof parsed.version === "number" ? parsed.version : undefined,
    };
  } catch {
    return undefined;
  }
};

export const hostAppMetadataForScaffold = (
  existing?: HostAppScaffoldMetadata,
): Required<HostAppScaffoldMetadata> => {
  const managedFiles = existing?.managedFiles ?? hostAppManagedFiles;
  const projectOwnedFiles = existing?.projectOwnedFiles ?? hostAppProjectOwnedFiles;
  return {
    files: existing?.files ?? [...managedFiles, ...projectOwnedFiles],
    managedFiles,
    packageName: existing?.packageName ?? "@flexweave/studio-app",
    packageRefs: existing?.packageRefs ?? {
      studio: "@flexweave/studio",
      studioApp: "@flexweave/studio-app",
    },
    projectOwnedFiles,
    scaffold: existing?.scaffold ?? "flexweave-studio-host-app",
    studioPackageName: existing?.studioPackageName ?? "@flexweave/studio",
    version: STUDIO_HOST_APP_SCAFFOLD_VERSION,
  };
};

const hostAppPackageDependencyScopes = [
  "dependencies",
  "devDependencies",
  "peerDependencies",
  "optionalDependencies",
] as const;

export const readHostAppPackageDependencies = (
  root: string,
): Record<string, string> | undefined => {
  const value = readTextIfExists(hostAppPackagePath(root));
  if (!value) {
    return undefined;
  }

  try {
    const parsed = JSON.parse(value) as Record<string, unknown>;
    const dependencies: Record<string, string> = {};
    for (const scope of hostAppPackageDependencyScopes) {
      const scoped = stringRecordField(parsed[scope]);
      if (scoped) {
        Object.assign(dependencies, scoped);
      }
    }
    return dependencies;
  } catch {
    return undefined;
  }
};

const isHostAppProjectOwnedFile = (relativePath: string, metadata?: HostAppScaffoldMetadata) =>
  (metadata?.projectOwnedFiles ?? hostAppProjectOwnedFiles).includes(relativePath);

const usesDefaultProjectAdapterResultBoundary = (current: string) =>
  current.includes(hostAppDefaultAdapterFactoryName) &&
  current.includes(hostAppProjectAdapterResultName) &&
  !current.includes("defaultProjectAdapter.adapter") &&
  !current.includes("projectAdapterDiagnostics");

const needsDefaultProjectAdapterMigration = (path: string, current: string | undefined) =>
  path.endsWith(hostAppProjectAdapterPath) &&
  current !== undefined &&
  !usesDefaultProjectAdapterResultBoundary(current);

const hostAppScaffoldFiles = (
  config: ResolvedStudioProjectConfig,
  root: string,
  existingMetadata?: HostAppScaffoldMetadata,
) => {
  const configPath = hostAppConfigPath(config, join(root, "src"));
  const metadata = hostAppMetadataForScaffold(existingMetadata);

  return {
    ".flexweave-studio-app.json": `${JSON.stringify(metadata, null, 2)}\n`,
    "package.json": `${JSON.stringify(
      {
        dependencies: {
          "@flexweave/studio": "0.0.0",
          "@flexweave/studio-app": "0.0.0",
        },
        devDependencies: {
          "bun-types": "^1.3.2",
          typescript: "^6.0.3",
        },
        name: "flexweave-studio-host",
        private: true,
        scripts: {
          build: "bun run typecheck",
          typecheck: "tsc -p tsconfig.json --noEmit",
        },
        type: "module",
      },
      null,
      2,
    )}\n`,
    "src/main.ts": [
      'import { createStudioApp } from "@flexweave/studio-app";',
      "",
      'import { projectAdapterResult } from "./project-adapter";',
      "",
      "export const app = createStudioApp(projectAdapterResult);",
      "export default app;",
      "",
    ].join("\n"),
    "src/project-adapter.ts": [
      'import { fileURLToPath } from "node:url";',
      "",
      'import { createDefaultStudioProjectAdapter } from "@flexweave/studio-app";',
      "",
      "export const projectAdapterResult = await createDefaultStudioProjectAdapter({",
      `  configPath: fileURLToPath(new URL("${configPath}", import.meta.url)),`,
      "});",
      "",
    ].join("\n"),
    "tsconfig.json": `${JSON.stringify(
      {
        compilerOptions: {
          allowSyntheticDefaultImports: true,
          esModuleInterop: true,
          forceConsistentCasingInFileNames: true,
          lib: ["ES2023"],
          module: "ESNext",
          moduleResolution: "Bundler",
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
          types: ["bun-types"],
        },
        include: ["src/**/*.ts"],
      },
      null,
      2,
    )}\n`,
  } satisfies Record<string, string>;
};

const jsonTextMatches = (current: string, expected: string) => {
  try {
    return (
      `${JSON.stringify(JSON.parse(current), null, 2)}\n` ===
      `${JSON.stringify(JSON.parse(expected), null, 2)}\n`
    );
  } catch {
    return false;
  }
};

const isHostAppMetadataFile = (path: string) => path.endsWith(".flexweave-studio-app.json");

const hostAppFileTextMatches = (path: string, current: string, expected: string) =>
  current === expected || (isHostAppMetadataFile(path) && jsonTextMatches(current, expected));

const plannedHostAppWrite = (
  path: string,
  status: HostAppWriteStatus,
  value: string,
): { file: StudioHostAppFileResult; write: PlannedHostAppWrite } => ({
  file: { path, status },
  write: { path, status, value },
});

const planHostAppFile = (
  path: string,
  expected: string,
  current: string | undefined,
  projectOwned: boolean,
  options: HostAppScaffoldPlanOptions,
): { file: StudioHostAppFileResult; write?: PlannedHostAppWrite } => {
  if (current === undefined) {
    if (options.mode === "verify") {
      return {
        file: {
          path,
          reason: "Required host app scaffold file is missing.",
          status: "manual-follow-up",
        },
      };
    }

    return plannedHostAppWrite(path, "created", expected);
  }

  if (options.mode === "verify" && projectOwned) {
    return {
      file: {
        path,
        reason: "Project-owned host app file preserved.",
        status: "project-owned",
      },
    };
  }

  if (hostAppFileTextMatches(path, current, expected)) {
    return { file: { path, status: "unchanged" } };
  }

  if (options.mode === "verify") {
    return {
      file: {
        path,
        reason: "Host app scaffold file differs from the current scaffold template.",
        status: "manual-follow-up",
      },
    };
  }

  if (options.updateMetadata && isHostAppMetadataFile(path)) {
    return plannedHostAppWrite(path, "updated", expected);
  }

  if (projectOwned) {
    if (
      options.requireDefaultProjectAdapter === true &&
      needsDefaultProjectAdapterMigration(path, current)
    ) {
      return {
        file: {
          path,
          reason:
            "Project adapter must export the createDefaultStudioProjectAdapter result as projectAdapterResult so the host app entry point receives adapter diagnostics.",
          status: "manual-follow-up",
        },
      };
    }

    return {
      file: {
        path,
        reason: "Project-owned host app file preserved.",
        status: "project-owned",
      },
    };
  }

  return {
    file: {
      path,
      reason: "Existing file differs from the current scaffold template.",
      status: "manual-follow-up",
    },
  };
};

export const planHostAppScaffold = (
  config: ResolvedStudioProjectConfig,
  root: string,
  options: HostAppScaffoldPlanOptions,
): HostAppScaffoldPlan => {
  const metadata = readHostAppMetadata(root);
  const templates = hostAppScaffoldFiles(config, root, metadata);
  const files: StudioHostAppFileResult[] = [];
  const writes: PlannedHostAppWrite[] = [];

  for (const [relativePath, value] of Object.entries(templates)) {
    const path = join(root, relativePath);
    const planned = planHostAppFile(
      path,
      value,
      readTextIfExists(path),
      isHostAppProjectOwnedFile(relativePath, metadata),
      options,
    );
    files.push(planned.file);
    if (planned.write) {
      writes.push(planned.write);
    }
  }

  return { files, writes };
};

export const prepareHostAppScaffoldWrite = (
  config: ResolvedStudioProjectConfig,
  root: string,
  options: Omit<HostAppScaffoldPlanOptions, "mode">,
): PreparedHostAppScaffoldWrite => {
  const plan = planHostAppScaffold(config, root, { ...options, mode: "write" });
  const snapshots = snapshotPaths(plan.writes.map((write) => write.path));
  const createdDirectoryCandidates = [
    ...new Set(plan.writes.map((write) => dirname(write.path)).filter((path) => !existsSync(path))),
  ].toSorted((left, right) => right.length - left.length);

  const rollback = () => {
    restoreSnapshots(snapshots);
    for (const path of createdDirectoryCandidates) {
      try {
        rmdirSync(path);
      } catch {
        // Leave non-empty or otherwise unavailable directories intact.
      }
    }
  };

  return {
    ...plan,
    rollback,
    write: () => {
      try {
        mkdirSync(root, { recursive: true });
        for (const write of plan.writes) {
          writeTextFile(write.path, write.value);
        }
        return plan.files;
      } catch (error) {
        rollback();
        throw error;
      }
    },
  };
};

export const writeHostAppScaffold = (
  config: ResolvedStudioProjectConfig,
  root: string,
  options: { requireDefaultProjectAdapter?: boolean; updateMetadata: boolean },
): StudioHostAppFileResult[] => {
  const session = prepareHostAppScaffoldWrite(config, root, options);
  return session.write();
};

export const manualFollowUps = (files: StudioHostAppFileResult[]) =>
  files
    .filter((file) => file.status === "manual-follow-up")
    .map((file) => `${file.path}: ${file.reason ?? "manual review required"}`);

export const changedFiles = (files: StudioHostAppFileResult[]) =>
  files
    .filter((file) => file.status === "created" || file.status === "updated")
    .map((file) => file.path);

export const readHostAppMetadataVersion = (root: string): number | undefined =>
  readHostAppMetadata(root)?.version;

export const scaffoldStudioHostApp = async (
  options: ScaffoldStudioHostAppOptions = {},
): Promise<ScaffoldStudioHostAppResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      changedFiles: [],
      diagnostics: resolved.diagnostics,
      files: [],
      manualFollowUps: [],
      ok: false,
    };
  }

  const root = hostAppRoot(resolved.config, options.appRoot);
  const writeSession = prepareHostAppScaffoldWrite(resolved.config, root, {
    updateMetadata: false,
  });
  const files = writeSession.write();
  const followUps = manualFollowUps(files);

  return {
    appRoot: root,
    changedFiles: changedFiles(files),
    diagnostics: followUps.map((followUp) =>
      workflowWarning("host-app-manual-follow-up", followUp),
    ),
    files,
    manualFollowUps: followUps,
    metadataVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    ok: true,
  };
};

const verifyHostAppFiles = (
  config: ResolvedStudioProjectConfig,
  root: string,
): StudioHostAppFileResult[] =>
  planHostAppScaffold(config, root, {
    mode: "verify",
    updateMetadata: false,
  }).files;

const runHostAppCommand = async (
  config: ResolvedStudioProjectConfig,
  root: string,
): Promise<StudioVerifyCommandResult> => {
  const command = config.app.checkCommand ?? config.app.buildCommand ?? ["bun", "run", "typecheck"];
  const proc = Bun.spawn(command, {
    cwd: root,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return {
    command,
    exitCode,
    fast: false,
    name: "local host app check",
    stderr,
    stdout,
  };
};

export const verifyStudioHostApp = async (
  options: ScaffoldStudioHostAppOptions = {},
): Promise<VerifyStudioHostAppResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      diagnostics: resolved.diagnostics,
      files: [],
      manualFollowUps: [],
      ok: false,
      status: "missing",
    };
  }

  const root = hostAppRoot(resolved.config, options.appRoot);
  const configured = resolved.config.paths.app.root !== undefined || options.appRoot !== undefined;
  const metadataExists = existsSync(hostAppMetadataPath(root));
  if (!existsSync(root) || !metadataExists) {
    if (!configured) {
      return {
        appRoot: root,
        diagnostics: [],
        files: [],
        manualFollowUps: [],
        ok: true,
        status: "not-configured",
      };
    }

    const message = `Local host app scaffold metadata is missing at ${hostAppMetadataPath(root)}.`;
    return {
      appRoot: root,
      diagnostics: [workflowError("host-app-metadata-missing", message)],
      files: [],
      manualFollowUps: [message],
      ok: false,
      status: "missing",
    };
  }

  const files = verifyHostAppFiles(resolved.config, root);
  const followUps = manualFollowUps(files);
  const command = await runHostAppCommand(resolved.config, root);
  const diagnostics = [
    ...followUps.map((followUp) => workflowError("host-app-manual-follow-up", followUp)),
    ...(command.exitCode === 0
      ? []
      : [
          workflowError(
            "host-app-check-failed",
            "Local host app check command failed.",
            undefined,
            command.command.join(" "),
          ),
        ]),
  ];

  return {
    appRoot: root,
    command,
    diagnostics,
    files,
    manualFollowUps: followUps,
    ok: diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    status: "checked",
  };
};
