import { existsSync, mkdirSync } from "node:fs";
import { join, relative } from "node:path";

import { studioCodegenTargets } from "../codegen/types";
import type { ResolvedStudioProjectConfig } from "../config/schema";
import { readTextIfExists, writeTextFile } from "../internal/files";
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
    ? join(config.configDir, appRoot)
    : (config.paths.app.root ?? join(config.configDir, "studio-host"));

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

const hostAppProjectOwnedFiles = ["src/project-adapter.ts"];

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

const hostAppScaffoldFiles = (
  config: ResolvedStudioProjectConfig,
  root: string,
  existingMetadata?: HostAppScaffoldMetadata,
) => {
  const configPath = hostAppConfigPath(config, join(root, "src"));
  const codegenTargets = studioCodegenTargets
    .map(
      (target) =>
        `    { label: "Generated ${target}", outputLabel: "${target}", target: "${target}" },`,
    )
    .join("\n");

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
      'import { projectAdapter } from "./project-adapter";',
      "",
      "export const app = createStudioApp(projectAdapter);",
      "export default app;",
      "",
    ].join("\n"),
    "src/project-adapter.ts": [
      "import {",
      "  codegenStudioProject,",
      "  describeStudioCatalog,",
      "  listStudioCatalogRecords,",
      "  migrateStudioProject,",
      "  planStudioMechanic,",
      "  scaffoldStudioMechanic,",
      "  showStudioCatalogRecord,",
      "  validateStudioCatalog,",
      "  verifyStudioProject,",
      '} from "@flexweave/studio/workflows";',
      'import { loadStudioConfig } from "@flexweave/studio/config/load";',
      'import { fileURLToPath } from "node:url";',
      "import {",
      "  collectStudioAppContributions,",
      "  composeStudioAppContributions,",
      "  defineStudioAppAdapter,",
      '} from "@flexweave/studio-app";',
      "",
      `const workflowOptions = { configPath: fileURLToPath(new URL("${configPath}", import.meta.url)) };`,
      "const loadedConfig = await loadStudioConfig(workflowOptions);",
      "const extensionContributions = loadedConfig.config",
      "  ? collectStudioAppContributions(loadedConfig.config.extensions)",
      "  : [];",
      "",
      "const asMechanicInput = (input: Record<string, unknown>) => ({",
      '  archetype: typeof input.archetype === "string" ? input.archetype : "mechanic",',
      '  id: typeof input.id === "string" ? input.id : "",',
      '  name: typeof input.name === "string" ? input.name : "",',
      "  params:",
      '    typeof input.params === "object" && input.params !== null && !Array.isArray(input.params)',
      "      ? (input.params as Record<string, unknown>)",
      "      : undefined,",
      "});",
      "",
      "const baseProjectAdapter = defineStudioAppAdapter({",
      "  authoring: {",
      "    areas: [",
      '      { editorId: "tags", id: "tags", label: "Tags" },',
      '      { editorId: "abilities", id: "abilities", label: "Abilities" },',
      '      { editorId: "effects", id: "effects", label: "Effects" },',
      "    ],",
      "    editors: [",
      '      { areaId: "tags", commandName: "list", id: "tags", label: "Tags", recordKind: "tags" },',
      '      { areaId: "abilities", commandName: "list", id: "abilities", label: "Abilities", recordKind: "abilities" },',
      '      { areaId: "effects", commandName: "list", id: "effects", label: "Effects", recordKind: "effects" },',
      "    ],",
      "  },",
      "  codegenTargets: [",
      codegenTargets,
      "  ],",
      '  id: "local-studio-host",',
      "  labels: {",
      '    productName: "Flexweave Studio",',
      '    projectName: "Consumer project",',
      '    shellSubtitle: "Catalog authoring",',
      '    workspaceTitle: "Authoring workspace",',
      '    workflowTrail: ["Studio catalog", "Generated mechanics definitions", "Consumer runtime"],',
      "  },",
      "  navigation: [",
      '    { id: "workspace", label: "Workspace", links: [{ href: "/", id: "overview", label: "Overview" }] },',
      '    { id: "generated", label: "Generated", links: [{ href: "/#generated-output", id: "generated-output", label: "Generated output" }] },',
      "  ],",
      "  serverFunctions: {",
      "    codegen: (input) => codegenStudioProject({ ...workflowOptions, ...input }),",
      "    describe: (input) => describeStudioCatalog(input?.kind, workflowOptions),",
      "    list: (input) => listStudioCatalogRecords(input.kind, { ...workflowOptions, filter: input.filter }),",
      "    migrate: () => migrateStudioProject(workflowOptions),",
      "    plan: (input) => planStudioMechanic({ ...workflowOptions, ...asMechanicInput(input) }),",
      "    scaffold: (input) => scaffoldStudioMechanic({ ...workflowOptions, ...asMechanicInput(input) }),",
      "    show: (input) => showStudioCatalogRecord(input.kind, input.id, workflowOptions),",
      "    validate: () => validateStudioCatalog(workflowOptions),",
      "    verify: (input) => verifyStudioProject({ ...workflowOptions, ...input }),",
      "  },",
      "  workflowActions: [",
      '    { commandName: "validate", id: "validate", label: "Validate", variant: "secondary" },',
      '    { commandName: "codegen", id: "codegen", label: "Generate", variant: "secondary" },',
      '    { commandName: "verify", id: "verify", label: "Verify", variant: "primary" },',
      "  ],",
      "});",
      "",
      "const composedProjectAdapter = composeStudioAppContributions(",
      "  baseProjectAdapter,",
      "  extensionContributions,",
      ");",
      "",
      "export const projectAdapter = composedProjectAdapter.adapter;",
      "export const projectAdapterDiagnostics = [",
      "  ...(loadedConfig.ok ? [] : loadedConfig.diagnostics),",
      "  ...composedProjectAdapter.diagnostics,",
      "];",
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

const scaffoldStatus = (
  path: string,
  expected: string,
  allowMetadataUpdate: boolean,
  projectOwned: boolean,
): StudioHostAppFileResult => {
  const current = readTextIfExists(path);
  if (current === undefined) {
    writeTextFile(path, expected);
    return { path, status: "created" };
  }

  if (current === expected) {
    return { path, status: "unchanged" };
  }

  if (path.endsWith(".flexweave-studio-app.json") && jsonTextMatches(current, expected)) {
    return { path, status: "unchanged" };
  }

  if (allowMetadataUpdate && path.endsWith(".flexweave-studio-app.json")) {
    writeTextFile(path, expected);
    return { path, status: "updated" };
  }

  if (projectOwned) {
    return {
      path,
      reason: "Project-owned host app file preserved.",
      status: "project-owned",
    };
  }

  return {
    path,
    reason: "Existing file differs from the current scaffold template.",
    status: "manual-follow-up",
  };
};

export const writeHostAppScaffold = (
  config: ResolvedStudioProjectConfig,
  root: string,
  options: { updateMetadata: boolean },
): StudioHostAppFileResult[] => {
  mkdirSync(root, { recursive: true });
  const metadata = readHostAppMetadata(root);
  const templates = hostAppScaffoldFiles(config, root, metadata);
  return Object.entries(templates).map(([relativePath, value]) =>
    scaffoldStatus(
      join(root, relativePath),
      value,
      options.updateMetadata,
      isHostAppProjectOwnedFile(relativePath, metadata),
    ),
  );
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
  const files = writeHostAppScaffold(resolved.config, root, {
    updateMetadata: false,
  });
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
): StudioHostAppFileResult[] => {
  const metadata = readHostAppMetadata(root);
  return Object.entries(hostAppScaffoldFiles(config, root, metadata)).map(
    ([relativePath, expected]) => {
      const path = join(root, relativePath);
      const current = readTextIfExists(path);
      if (current === undefined) {
        return {
          path,
          reason: "Required host app scaffold file is missing.",
          status: "manual-follow-up",
        };
      }
      if (isHostAppProjectOwnedFile(relativePath, metadata)) {
        return {
          path,
          reason: "Project-owned host app file preserved.",
          status: "project-owned",
        };
      }
      if (
        current !== expected &&
        !(path.endsWith(".flexweave-studio-app.json") && jsonTextMatches(current, expected))
      ) {
        return {
          path,
          reason: "Host app scaffold file differs from the current scaffold template.",
          status: "manual-follow-up",
        };
      }
      return { path, status: "unchanged" };
    },
  );
};

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
