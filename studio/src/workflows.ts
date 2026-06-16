import { existsSync, mkdirSync, rmSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";

import { defineStudioGeneratedTarget, studioCodegenTargets } from "./codegen/types";
import type {
  RuntimeHookSummary,
  StudioCodegenTarget,
  StudioCodegenTargetSummary,
  StudioGeneratedTargetDefinition,
  StudioGeneratedFileDiff,
} from "./codegen/types";
import { loadStudioConfig } from "./config/load";
import type { LoadStudioConfigOptions } from "./config/load";
import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "./config/schema";
import {
  loadStudioCatalog,
  normalizeRecordKind,
  studioRecordKinds,
  writeStudioCatalogRecord,
} from "./internal/catalog";
import type { StudioCatalogRecord, StudioRecordKind } from "./internal/catalog";
import {
  displayPath,
  listFilesRecursive,
  readTextIfExists,
  restoreSnapshots,
  snapshotPaths,
  writeTextFile,
} from "./internal/files";

export interface StudioWorkflowOptions {
  config?: ResolvedStudioProjectConfig;
  configPath?: string;
  cwd?: string;
}

export interface StudioWorkflowResult {
  diagnostics: StudioDiagnostic[];
  ok: boolean;
}

export interface ValidateStudioCatalogResult extends StudioWorkflowResult {
  configPath?: string;
  recordCount: number;
  sourceRecordCount: number;
  sources: StudioSourceSummary[];
}

export interface StudioSourceSummary {
  adapterId?: string;
  recordCount: number;
  sourceId?: string;
}

export interface StudioRecordDescription {
  fields: string[];
  kind: StudioRecordKind;
  summary: string;
}

export interface DescribeStudioCatalogResult extends StudioWorkflowResult {
  descriptions: StudioRecordDescription[];
}

export interface ListStudioCatalogRecordsResult extends StudioWorkflowResult {
  kind: StudioRecordKind;
  records: { id: string; label: string; path: string }[];
}

export interface ShowStudioCatalogRecordResult extends StudioWorkflowResult {
  record?: StudioCatalogRecord & { path: string };
}

export interface CodegenStudioResult extends StudioWorkflowResult {
  checked: boolean;
  configPath?: string;
  hooks: RuntimeHookSummary[];
  targets: StudioCodegenTargetSummary[];
}

export interface PlanStudioMechanicOptions extends StudioWorkflowOptions {
  allowExisting?: boolean;
  archetype: string;
  id: string;
  name: string;
  params?: Record<string, unknown>;
}

export interface PlanStudioMechanicResult extends StudioWorkflowResult {
  plannedFiles: string[];
  records: StudioCatalogRecord[];
}

export interface ScaffoldStudioMechanicResult extends PlanStudioMechanicResult {
  rolledBack: boolean;
  writtenFiles: string[];
}

export interface ScaffoldStudioHostAppOptions extends StudioWorkflowOptions {
  appRoot?: string;
}

export type StudioHostAppFileStatus =
  | "created"
  | "manual-follow-up"
  | "project-owned"
  | "unchanged"
  | "updated";

export interface StudioHostAppFileResult {
  path: string;
  reason?: string;
  status: StudioHostAppFileStatus;
}

export interface ScaffoldStudioHostAppResult extends StudioWorkflowResult {
  appRoot?: string;
  changedFiles: string[];
  files: StudioHostAppFileResult[];
  manualFollowUps: string[];
  metadataVersion?: number;
}

export interface VerifyStudioHostAppResult extends StudioWorkflowResult {
  appRoot?: string;
  command?: StudioVerifyCommandResult;
  files: StudioHostAppFileResult[];
  manualFollowUps: string[];
  status: "checked" | "missing" | "not-configured";
}

export interface StudioVerifyCommandResult {
  command: string[];
  exitCode: number | null;
  fast: boolean;
  name: string;
  stderr: string;
  stdout: string;
}

export type StudioVerifyMode = "fast" | "full";

export type StudioVerifyCheckStatus = "failed" | "passed" | "skipped";

export interface StudioVerifyCheckResult {
  adapterId?: string;
  command?: string[];
  diagnostics: StudioDiagnostic[];
  exitCode?: number | null;
  extensionId?: string;
  mode: StudioVerifyMode;
  name: string;
  sourceId?: string;
  status: StudioVerifyCheckStatus;
  stdout?: string;
  stderr?: string;
  targetId?: string;
}

export interface VerifyStudioProjectResult extends StudioWorkflowResult {
  checks: StudioVerifyCheckResult[];
  codegen: CodegenStudioResult;
  commands: StudioVerifyCommandResult[];
  hostApp: VerifyStudioHostAppResult;
  validation: ValidateStudioCatalogResult;
}

export interface MigrateStudioProjectResult extends StudioWorkflowResult {
  applied: string[];
  changedFiles: string[];
  checks: StudioMigrationCheckResult[];
  manualFollowUps: string[];
  skipped: string[];
}

export type StudioMigrationCheckStatus = "applied" | "failed" | "skipped";

export interface StudioMigrationCheckResult {
  applied: string[];
  changedFiles: string[];
  currentVersion?: number;
  diagnostics: StudioDiagnostic[];
  extensionId?: string;
  manualFollowUps: string[];
  name: string;
  skipped: string[];
  status: StudioMigrationCheckStatus;
  targetVersion?: number;
}

export const STUDIO_HOST_APP_SCAFFOLD_VERSION = 2;

const schemaDescriptions: StudioRecordDescription[] = [
  {
    fields: ["kind", "id", "label", "effectId"],
    kind: "abilities",
    summary: "Ability records name callable mechanics and may reference effects.",
  },
  {
    fields: ["kind", "id", "label", "executionId", "modifierId", "tagIds"],
    kind: "effects",
    summary: "Effect records connect generated definitions to executions and tags.",
  },
  {
    fields: ["kind", "id", "label", "hook"],
    kind: "executions",
    summary: "Execution records name runtime hooks declared by the consumer runtime.",
  },
  {
    fields: ["kind", "id", "label", "recordIds"],
    kind: "mechanics",
    summary: "Mechanic manifests record files created by Studio scaffolding.",
  },
  {
    fields: ["kind", "id", "label", "value"],
    kind: "modifiers",
    summary: "Modifier records provide reusable generated definition data.",
  },
  {
    fields: ["kind", "id", "label"],
    kind: "tags",
    summary: "Tag records provide stable grouping tokens for generated definitions.",
  },
];

const workflowError = (
  code: string,
  message: string,
  path?: string,
  hint?: string,
): StudioDiagnostic => ({
  code,
  hint,
  message,
  path,
  severity: "error",
});

const workflowWarning = (code: string, message: string, path?: string): StudioDiagnostic => ({
  code,
  message,
  path,
  severity: "warning",
});

const resolveWorkflowConfig = async (
  options: StudioWorkflowOptions,
): Promise<
  | { config: ResolvedStudioProjectConfig; diagnostics: StudioDiagnostic[]; ok: true }
  | { diagnostics: StudioDiagnostic[]; ok: false }
> => {
  if (options.config) {
    return { config: options.config, diagnostics: [], ok: true };
  }

  const loaded = await loadStudioConfig({
    configPath: options.configPath,
    cwd: options.cwd,
  } satisfies LoadStudioConfigOptions);
  if (!loaded.ok || !loaded.config) {
    return { diagnostics: loaded.diagnostics, ok: false };
  }

  return { config: loaded.config, diagnostics: loaded.diagnostics, ok: true };
};

const hostAppRoot = (config: ResolvedStudioProjectConfig, appRoot?: string) =>
  appRoot
    ? join(config.configDir, appRoot)
    : (config.paths.app.root ?? join(config.configDir, "studio-host"));

const hostAppConfigPath = (config: ResolvedStudioProjectConfig, fromDir: string) => {
  const path = relative(fromDir, config.configPath).replaceAll("\\", "/");
  return path.startsWith(".") ? path : `./${path}`;
};

const hostAppMetadataPath = (root: string) => join(root, ".flexweave-studio-app.json");

const hostAppManagedFiles = [
  ".flexweave-studio-app.json",
  "package.json",
  "src/main.ts",
  "tsconfig.json",
];

const hostAppProjectOwnedFiles = ["src/project-adapter.ts"];

const isHostAppProjectOwnedFile = (relativePath: string) =>
  hostAppProjectOwnedFiles.includes(relativePath);

const hostAppScaffoldFiles = (config: ResolvedStudioProjectConfig, root: string) => {
  const configPath = hostAppConfigPath(config, join(root, "src"));
  const codegenTargets = studioCodegenTargets
    .map(
      (target) =>
        `    { label: "Generated ${target}", outputLabel: "${target}", target: "${target}" },`,
    )
    .join("\n");

  const metadata = {
    files: [...hostAppManagedFiles, ...hostAppProjectOwnedFiles],
    managedFiles: hostAppManagedFiles,
    packageName: "@flexweave/studio-app",
    packageRefs: {
      studio: "@flexweave/studio",
      studioApp: "@flexweave/studio-app",
    },
    projectOwnedFiles: hostAppProjectOwnedFiles,
    scaffold: "flexweave-studio-host-app",
    studioPackageName: "@flexweave/studio",
    version: STUDIO_HOST_APP_SCAFFOLD_VERSION,
  };

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

const writeHostAppScaffold = (
  config: ResolvedStudioProjectConfig,
  root: string,
  options: { updateMetadata: boolean },
): StudioHostAppFileResult[] => {
  mkdirSync(root, { recursive: true });
  const templates = hostAppScaffoldFiles(config, root);
  return Object.entries(templates).map(([relativePath, value]) =>
    scaffoldStatus(
      join(root, relativePath),
      value,
      options.updateMetadata,
      isHostAppProjectOwnedFile(relativePath),
    ),
  );
};

const manualFollowUps = (files: StudioHostAppFileResult[]) =>
  files
    .filter((file) => file.status === "manual-follow-up")
    .map((file) => `${file.path}: ${file.reason ?? "manual review required"}`);

const changedFiles = (files: StudioHostAppFileResult[]) =>
  files
    .filter((file) => file.status === "created" || file.status === "updated")
    .map((file) => file.path);

const readHostAppMetadataVersion = (root: string): number | undefined => {
  const value = readTextIfExists(hostAppMetadataPath(root));
  if (!value) {
    return undefined;
  }

  try {
    const parsed = JSON.parse(value) as { version?: unknown };
    return typeof parsed.version === "number" ? parsed.version : undefined;
  } catch {
    return undefined;
  }
};

const fullConfigRequired = (config: ResolvedStudioProjectConfig): StudioDiagnostic[] =>
  config.mode === "full"
    ? []
    : [
        workflowError(
          "full-config-required",
          "This Studio workflow requires a full Studio project config.",
          config.configPath,
        ),
      ];

export const validateStudioCatalog = async (
  options: StudioWorkflowOptions = {},
): Promise<ValidateStudioCatalogResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      diagnostics: resolved.diagnostics,
      ok: false,
      recordCount: 0,
      sourceRecordCount: 0,
      sources: [],
    };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const sourceSnapshots = catalog.sourceSnapshots.filter((snapshot) => snapshot.records.length > 0);
  return {
    configPath: resolved.config.configPath,
    diagnostics: catalog.diagnostics,
    ok: catalog.diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    recordCount: catalog.records.length,
    sourceRecordCount: catalog.sourceSnapshots.reduce(
      (total, snapshot) => total + snapshot.records.length,
      0,
    ),
    sources: sourceSnapshots.map((snapshot) => ({
      adapterId: snapshot.adapterId,
      recordCount: snapshot.records.length,
      sourceId: snapshot.sourceId,
    })),
  };
};

export const describeStudioCatalog = async (
  kind: string | undefined,
  options: StudioWorkflowOptions = {},
): Promise<DescribeStudioCatalogResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { descriptions: [], diagnostics: resolved.diagnostics, ok: false };
  }

  if (!kind) {
    return { descriptions: schemaDescriptions, diagnostics: [], ok: true };
  }

  const normalized = normalizeRecordKind(kind);
  if (!normalized) {
    return {
      descriptions: [],
      diagnostics: [
        workflowError(
          "unknown-record-kind",
          `Unknown Studio catalog record kind "${kind}".`,
          undefined,
          `Expected one of: ${studioRecordKinds.join(", ")}.`,
        ),
      ],
      ok: false,
    };
  }

  return {
    descriptions: schemaDescriptions.filter((description) => description.kind === normalized),
    diagnostics: [],
    ok: true,
  };
};

export const listStudioCatalogRecords = async (
  kind: string,
  options: StudioWorkflowOptions & { filter?: string } = {},
): Promise<ListStudioCatalogRecordsResult> => {
  const normalized = normalizeRecordKind(kind);
  if (!normalized) {
    return {
      diagnostics: [
        workflowError(
          "unknown-record-kind",
          `Unknown Studio catalog record kind "${kind}".`,
          undefined,
          `Expected one of: ${studioRecordKinds.join(", ")}.`,
        ),
      ],
      kind: "abilities",
      ok: false,
      records: [],
    };
  }

  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { diagnostics: resolved.diagnostics, kind: normalized, ok: false, records: [] };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const filter = options.filter?.toLowerCase();
  const records = catalog.byKind[normalized]
    .filter(
      (record) =>
        !filter ||
        record.id.toLowerCase().includes(filter) ||
        record.label.toLowerCase().includes(filter),
    )
    .map((record) => ({
      id: record.id,
      label: record.label,
      path: record.path,
    }));

  return {
    diagnostics: catalog.diagnostics,
    kind: normalized,
    ok: catalog.diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    records,
  };
};

export const showStudioCatalogRecord = async (
  kind: string,
  id: string,
  options: StudioWorkflowOptions = {},
): Promise<ShowStudioCatalogRecordResult> => {
  const normalized = normalizeRecordKind(kind);
  if (!normalized) {
    return {
      diagnostics: [
        workflowError("unknown-record-kind", `Unknown Studio catalog record kind "${kind}".`),
      ],
      ok: false,
    };
  }

  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { diagnostics: resolved.diagnostics, ok: false };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const record = catalog.byKind[normalized].find((candidate) => candidate.id === id);
  if (!record) {
    return {
      diagnostics: [
        workflowError(
          "missing-record",
          `No ${normalized} record with id "${id}" exists in the Studio catalog.`,
        ),
      ],
      ok: false,
    };
  }

  return { diagnostics: catalog.diagnostics, ok: true, record };
};

const rustIdentifier = (value: string) =>
  value
    .toLowerCase()
    .replaceAll(/[^a-z0-9_]+/g, "_")
    .replace(/^([0-9])/, "_$1")
    .replaceAll(/_+/g, "_")
    .replaceAll(/^_|_$/g, "");

const generatedHeader = (target: StudioCodegenTarget, config: ResolvedStudioProjectConfig) => {
  const template = config.rust?.generatedHeader;
  if (template) {
    return `${template.replaceAll("{target}", target)}\n\n`;
  }

  return `//! Generated by Flexweave Studio for ${target}.\n//! Do not edit manually. Run \`flexweave-studio codegen --target ${target}\` to refresh.\n\n`;
};

const renderRustDefinitions = (
  target: Exclude<StudioCodegenTarget, "reference">,
  records: { id: string; label: string }[],
  config: ResolvedStudioProjectConfig,
) => {
  const entries =
    records.length === 0
      ? "pub const DEFINITIONS: &[(&str, &str)] = &[];\n"
      : [
          "pub const DEFINITIONS: &[(&str, &str)] = &[",
          ...records.map(
            (record) => `    ("${record.id}", "${record.label.replaceAll('"', '\\"')}"),`,
          ),
          "];",
          "",
        ].join("\n");

  return `${generatedHeader(target, config)}${entries}`;
};

const renderReference = (
  config: ResolvedStudioProjectConfig,
  catalog: Awaited<ReturnType<typeof loadStudioCatalog>>,
) => {
  const sections = studioRecordKinds.flatMap((kind) => [
    `## ${kind}`,
    "",
    ...(catalog.byKind[kind].length === 0
      ? ["- No records."]
      : catalog.byKind[kind].map((record) => `- ${record.id}: ${record.label}`)),
    "",
  ]);

  return [
    "<!-- Generated by Flexweave Studio. Do not edit manually. -->",
    "",
    "# Studio Catalog Reference",
    "",
    `Config: ${displayPath(config.configDir, config.configPath)}`,
    "",
    ...sections,
  ].join("\n");
};

interface PlannedGeneratedFile {
  path: string;
  target: StudioCodegenTarget;
  value: string;
}

type StudioCatalogContent = Awaited<ReturnType<typeof loadStudioCatalog>>;
type RegisteredGeneratedTarget = StudioGeneratedTargetDefinition<StudioCatalogContent>;

const generatedTargetLabel = (target: StudioCodegenTarget) =>
  target === "reference" ? "Generated catalog reference" : `Generated ${target} definitions`;

const builtInGeneratedTargets = (
  projectConfig: ResolvedStudioProjectConfig,
): RegisteredGeneratedTarget[] =>
  projectConfig.codegen.builtInTargets.map((target) =>
    defineStudioGeneratedTarget({
      cleanup: "managed-files",
      id: target,
      label: generatedTargetLabel(target),
      plan: ({ config, content, outputDir }) => {
        const resolvedConfig = config as ResolvedStudioProjectConfig;
        const catalog = content as StudioCatalogContent;
        const path =
          target === "reference"
            ? join(outputDir, "studio-reference.md")
            : join(outputDir, "generated.rs");
        const value =
          target === "reference"
            ? renderReference(resolvedConfig, catalog)
            : renderRustDefinitions(target, catalog.byKind[target], resolvedConfig);
        return {
          files: [{ path, value }],
        };
      },
    }),
  );

const activeGeneratedTargets = (
  config: ResolvedStudioProjectConfig,
): RegisteredGeneratedTarget[] => [
  ...builtInGeneratedTargets(config),
  ...(config.extensions.flatMap(
    (extension) => extension.generatedTargets ?? [],
  ) as RegisteredGeneratedTarget[]),
];

const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

const selectTargets = (
  registeredTargets: RegisteredGeneratedTarget[],
  requestedTargets?: readonly string[],
): { diagnostics: StudioDiagnostic[]; targets: RegisteredGeneratedTarget[] } => {
  const byId: Record<string, RegisteredGeneratedTarget | undefined> = {};
  for (const target of registeredTargets) {
    byId[target.id] = target;
  }

  const availableTargetIds = registeredTargets.map((target) => target.id);
  const rootTargetIds =
    requestedTargets && requestedTargets.length > 0 ? [...requestedTargets] : availableTargetIds;
  const diagnostics: StudioDiagnostic[] = [];
  const orderedTargets: RegisteredGeneratedTarget[] = [];
  const visiting = new Set<string>();
  const visited = new Set<string>();

  const visit = (targetId: string, dependencyOf?: string) => {
    const target = byId[targetId];
    if (!target) {
      diagnostics.push(
        workflowError(
          dependencyOf ? "missing-generated-target-dependency" : "unknown-codegen-target",
          dependencyOf
            ? `Generated target "${dependencyOf}" depends on missing target "${targetId}".`
            : `Unknown Studio generated output target "${targetId}".`,
          undefined,
          `Expected one of: ${availableTargetIds.join(", ")}.`,
        ),
      );
      return;
    }

    if (visited.has(target.id)) {
      return;
    }

    if (visiting.has(target.id)) {
      diagnostics.push(
        workflowError(
          "generated-target-cycle",
          `Generated target dependency cycle includes "${target.id}".`,
        ),
      );
      return;
    }

    visiting.add(target.id);
    for (const dependency of target.dependencies ?? []) {
      visit(dependency, target.id);
    }
    visiting.delete(target.id);
    visited.add(target.id);
    orderedTargets.push(target);
  };

  for (const targetId of rootTargetIds) {
    visit(targetId);
  }

  return { diagnostics, targets: orderedTargets };
};

const plannedGeneratedFiles = async (
  config: ResolvedStudioProjectConfig,
  catalog: StudioCatalogContent,
  targets: RegisteredGeneratedTarget[],
): Promise<{ diagnostics: StudioDiagnostic[]; files: PlannedGeneratedFile[] }> => {
  const diagnostics: StudioDiagnostic[] = [];
  const files: PlannedGeneratedFile[] = [];

  for (const target of targets) {
    const outputDir = config.paths.codegen.outputDirs[target.id];
    if (!outputDir) {
      diagnostics.push(
        workflowError(
          "missing-generated-output-root",
          `Generated target "${target.id}" does not have a configured output directory.`,
          config.configPath,
          `Add codegen.outputDirs.${target.id} to the Studio project config.`,
        ),
      );
      continue;
    }

    try {
      const result = await target.plan({
        config,
        content: catalog,
        outputDir,
        targetId: target.id,
      });
      diagnostics.push(...((result.diagnostics ?? []) as StudioDiagnostic[]));
      for (const file of result.files) {
        if (!pathContains(outputDir, file.path)) {
          diagnostics.push(
            workflowError(
              "generated-output-out-of-bounds",
              `Generated target "${target.id}" planned a file outside its configured output directory.`,
              file.path,
              `Target output directory: ${outputDir}`,
            ),
          );
          continue;
        }
        files.push({ path: file.path, target: target.id, value: file.value });
      }
    } catch (error) {
      diagnostics.push(
        workflowError(
          "generated-target-plan-failed",
          error instanceof Error
            ? `Generated target "${target.id}" failed to plan files: ${error.message}`
            : `Generated target "${target.id}" failed to plan files.`,
          config.configPath,
        ),
      );
    }
  }

  return { diagnostics, files };
};

const managedFileHeader = "Generated by Flexweave Studio";

const detectUnexpectedManagedFiles = (
  config: ResolvedStudioProjectConfig,
  expected: PlannedGeneratedFile[],
  targets: RegisteredGeneratedTarget[],
): StudioGeneratedFileDiff[] => {
  const expectedPaths = new Set(expected.map((file) => file.path));
  const diffs: StudioGeneratedFileDiff[] = [];

  for (const target of targets) {
    if (target.cleanup === "none") {
      continue;
    }
    const outputDir = config.paths.codegen.outputDirs[target.id];
    for (const path of listFilesRecursive(outputDir)) {
      if (expectedPaths.has(path)) {
        continue;
      }
      const value = readTextIfExists(path);
      if (value?.includes(managedFileHeader)) {
        diffs.push({ path, status: "unexpected", target: target.id });
      }
    }
  }

  return diffs;
};

const summarizeGeneratedFiles = (
  config: ResolvedStudioProjectConfig,
  expected: PlannedGeneratedFile[],
  targets: RegisteredGeneratedTarget[],
): StudioGeneratedFileDiff[] => {
  const diffs = expected.map((file): StudioGeneratedFileDiff => {
    const current = readTextIfExists(file.path);
    if (current === undefined) {
      return { path: file.path, status: "missing", target: file.target };
    }
    if (current !== file.value) {
      return { path: file.path, status: "stale", target: file.target };
    }
    return { path: file.path, status: "fresh", target: file.target };
  });

  return [...diffs, ...detectUnexpectedManagedFiles(config, expected, targets)];
};

const hookFileName = (hook: string) => `${rustIdentifier(hook)}.rs`;

const hookStub = (hook: string) =>
  `//! Runtime hook stub created by Flexweave Studio. Consumer-owned after creation.\n\npub fn ${rustIdentifier(hook)}() {}\n`;

const hookTestStub = (hook: string) =>
  `//! Runtime hook test stub created by Flexweave Studio.\n\n#[test]\nfn ${rustIdentifier(hook)}_is_declared() {\n    assert!(true);\n}\n`;

const runtimeHookStatus = (exists: boolean, write: boolean) => {
  if (exists) {
    return "existing";
  }
  return write ? "created" : "missing";
};

const summarizeHooks = (
  config: ResolvedStudioProjectConfig,
  catalog: Awaited<ReturnType<typeof loadStudioCatalog>>,
  options: { write: boolean },
): RuntimeHookSummary[] => {
  const hookDir = config.paths.hooks.dir;
  if (!hookDir) {
    return [];
  }

  if (options.write) {
    mkdirSync(hookDir, { recursive: true });
  }
  if (options.write && config.paths.hooks.testStubsDir) {
    mkdirSync(config.paths.hooks.testStubsDir, { recursive: true });
  }

  const expectedHooks = new Set(
    catalog.byKind.executions
      .map((record) => record.hook)
      .filter((hook): hook is string => typeof hook === "string" && hook.length > 0),
  );
  const summaries: RuntimeHookSummary[] = [];

  for (const hook of [...expectedHooks].toSorted()) {
    const path = join(hookDir, hookFileName(hook));
    const exists = existsSync(path);
    if (!exists && options.write) {
      writeTextFile(path, hookStub(hook));
    }
    summaries.push({
      hook,
      path,
      status: runtimeHookStatus(exists, options.write),
    });

    if (config.paths.hooks.testStubsDir) {
      const testPath = join(config.paths.hooks.testStubsDir, hookFileName(hook));
      if (existsSync(testPath)) {
        summaries.push({ hook, path: testPath, status: "existing" });
      } else if (options.write) {
        writeTextFile(testPath, hookTestStub(hook));
        summaries.push({ hook, path: testPath, status: "created" });
      } else {
        summaries.push({ hook, path: testPath, status: "missing" });
      }
    }
  }

  for (const path of listFilesRecursive(hookDir)) {
    if (!path.endsWith(".rs")) {
      continue;
    }
    const hook = path.split("/").at(-1)?.replace(/\.rs$/, "") ?? "";
    if (!expectedHooks.has(hook)) {
      summaries.push({ hook, path, status: "orphan" });
    }
  }

  return summaries.toSorted((left, right) => left.path.localeCompare(right.path));
};

const generatedWriteStatus = (before: string | undefined, value: string) => {
  if (before === undefined) {
    return "created";
  }
  return before === value ? "fresh" : "updated";
};

export const codegenStudioProject = async (
  options: StudioWorkflowOptions & {
    check?: boolean;
    targets?: readonly string[];
  } = {},
): Promise<CodegenStudioResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      checked: options.check === true,
      diagnostics: resolved.diagnostics,
      hooks: [],
      ok: false,
      targets: [],
    };
  }

  const fullConfigDiagnostics = fullConfigRequired(resolved.config);
  const registeredTargets = activeGeneratedTargets(resolved.config);
  const selected = selectTargets(registeredTargets, options.targets);
  if (fullConfigDiagnostics.length > 0 || selected.diagnostics.length > 0) {
    return {
      checked: options.check === true,
      configPath: resolved.config.configPath,
      diagnostics: [...fullConfigDiagnostics, ...selected.diagnostics],
      hooks: [],
      ok: false,
      targets: [],
    };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  if (catalog.diagnostics.some((diagnostic) => diagnostic.severity === "error")) {
    return {
      checked: options.check === true,
      configPath: resolved.config.configPath,
      diagnostics: catalog.diagnostics,
      hooks: [],
      ok: false,
      targets: [],
    };
  }

  const planned = await plannedGeneratedFiles(resolved.config, catalog, selected.targets);
  if (planned.diagnostics.some((diagnostic) => diagnostic.severity === "error")) {
    return {
      checked: options.check === true,
      configPath: resolved.config.configPath,
      diagnostics: planned.diagnostics,
      hooks: [],
      ok: false,
      targets: [],
    };
  }

  const expected = planned.files;
  const diffs = summarizeGeneratedFiles(resolved.config, expected, selected.targets);
  const hooks = summarizeHooks(resolved.config, catalog, {
    write: options.check !== true,
  });
  let finalDiffs = diffs;

  if (options.check !== true) {
    const snapshots = snapshotPaths(expected.map((file) => file.path));
    try {
      for (const file of expected) {
        const before = readTextIfExists(file.path);
        writeTextFile(file.path, file.value);
        const status = generatedWriteStatus(before, file.value);
        finalDiffs = finalDiffs.map((diff) =>
          diff.path === file.path ? { ...diff, status } : diff,
        );
      }

      for (const diff of diffs.filter((candidate) => candidate.status === "unexpected")) {
        if (existsSync(diff.path)) {
          snapshots.push(...snapshotPaths([diff.path]));
          rmSync(diff.path);
          finalDiffs = finalDiffs.map((candidate) =>
            candidate.path === diff.path ? { ...candidate, status: "deleted" } : candidate,
          );
        }
      }
    } catch (error) {
      restoreSnapshots(snapshots);
      return {
        checked: false,
        configPath: resolved.config.configPath,
        diagnostics: [
          workflowError(
            "codegen-write-failed",
            error instanceof Error
              ? `Failed to write generated mechanics definitions: ${error.message}`
              : "Failed to write generated mechanics definitions.",
          ),
        ],
        hooks,
        ok: false,
        targets: [],
      };
    }
  }

  const staleDiagnostics =
    options.check === true
      ? diffs
          .filter((diff) => diff.status !== "fresh")
          .map((diff) =>
            workflowError(
              `generated-${diff.status}`,
              `Generated mechanics definition is ${diff.status}: ${displayPath(resolved.config.configDir, diff.path)}`,
              displayPath(resolved.config.configDir, diff.path),
            ),
          )
      : [];
  const hookDiagnostics = hooks
    .filter((hook) => hook.status === "orphan")
    .map((hook) =>
      workflowWarning(
        "orphan-runtime-hook",
        `Runtime hook file is not referenced by the Studio catalog: ${displayPath(resolved.config.configDir, hook.path)}`,
        displayPath(resolved.config.configDir, hook.path),
      ),
    );

  const targetSummaries = selected.targets.map(
    (target): StudioCodegenTargetSummary => ({
      files: finalDiffs.filter((diff) => diff.target === target.id),
      label: target.label,
      target: target.id,
    }),
  );

  const diagnostics = [...planned.diagnostics, ...staleDiagnostics, ...hookDiagnostics];
  return {
    checked: options.check === true,
    configPath: resolved.config.configPath,
    diagnostics,
    hooks,
    ok: diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    targets: targetSummaries,
  };
};

const mechanicRecords = (
  id: string,
  label: string,
  params: Record<string, unknown> = {},
): StudioCatalogRecord[] => {
  const broken = params.broken === true;
  return [
    { id, kind: "tag", label: `${label} tag` },
    { id, kind: "modifier", label: `${label} modifier`, value: 1 },
    {
      hook: `${id}_runtime_hook`,
      id,
      kind: "execution",
      label: `${label} execution`,
    },
    {
      executionId: id,
      id,
      kind: "effect",
      label: `${label} effect`,
      modifierId: id,
      tagIds: [id],
    },
    {
      effectId: broken ? `${id}_missing_effect` : id,
      id,
      kind: "ability",
      label: `${label} ability`,
    },
    {
      id,
      kind: "mechanic",
      label,
      recordIds: [
        `tag:${id}`,
        `modifier:${id}`,
        `execution:${id}`,
        `effect:${id}`,
        `ability:${id}`,
      ],
    },
  ];
};

const kindForRecord = (record: StudioCatalogRecord): StudioRecordKind => {
  const kind = normalizeRecordKind(record.kind);
  if (!kind) {
    throw new Error(`Unsupported Studio catalog record kind ${record.kind}.`);
  }
  return kind;
};

export const planStudioMechanic = async (
  options: PlanStudioMechanicOptions,
): Promise<PlanStudioMechanicResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { diagnostics: resolved.diagnostics, ok: false, plannedFiles: [], records: [] };
  }

  if (options.archetype !== "mechanic") {
    return {
      diagnostics: [
        workflowError(
          "unknown-mechanic-archetype",
          `Unknown Studio mechanic archetype "${options.archetype}".`,
          undefined,
          'Use "mechanic" for the built-in synthetic archetype.',
        ),
      ],
      ok: false,
      plannedFiles: [],
      records: [],
    };
  }

  const records = mechanicRecords(options.id, options.name, options.params);
  const plannedFiles = records.map((record) =>
    join(resolved.config.paths.catalogRoot, kindForRecord(record), `${record.id}.json`),
  );
  const diagnostics =
    options.allowExisting === true
      ? []
      : plannedFiles
          .filter((path) => existsSync(path))
          .map((path) =>
            workflowError(
              "planned-file-exists",
              `Planned Studio catalog file already exists: ${displayPath(resolved.config.configDir, path)}`,
              displayPath(resolved.config.configDir, path),
            ),
          );

  return {
    diagnostics,
    ok: diagnostics.length === 0,
    plannedFiles: plannedFiles.map((path) => displayPath(resolved.config.configDir, path)),
    records,
  };
};

export const scaffoldStudioMechanic = async (
  options: PlanStudioMechanicOptions,
): Promise<ScaffoldStudioMechanicResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      diagnostics: resolved.diagnostics,
      ok: false,
      plannedFiles: [],
      records: [],
      rolledBack: false,
      writtenFiles: [],
    };
  }

  const planned = await planStudioMechanic({ ...options, config: resolved.config });
  if (!planned.ok) {
    return { ...planned, rolledBack: false, writtenFiles: [] };
  }

  const absolutePlannedFiles = planned.records.map((record) =>
    join(resolved.config.paths.catalogRoot, kindForRecord(record), `${record.id}.json`),
  );
  const snapshots = snapshotPaths(absolutePlannedFiles);
  const writtenFiles: string[] = [];

  try {
    for (const record of planned.records) {
      const writeResult = writeStudioCatalogRecord(resolved.config, kindForRecord(record), record);
      if (writeResult.diagnostics.length > 0 || !writeResult.path) {
        restoreSnapshots(snapshots);
        return {
          diagnostics: writeResult.diagnostics,
          ok: false,
          plannedFiles: planned.plannedFiles,
          records: planned.records,
          rolledBack: true,
          writtenFiles,
        };
      }
      const { path } = writeResult;
      writtenFiles.push(displayPath(resolved.config.configDir, path));
    }

    const validation = await validateStudioCatalog({ config: resolved.config });
    if (!validation.ok) {
      restoreSnapshots(snapshots);
      return {
        diagnostics: validation.diagnostics,
        ok: false,
        plannedFiles: planned.plannedFiles,
        records: planned.records,
        rolledBack: true,
        writtenFiles,
      };
    }

    const codegen = await codegenStudioProject({
      config: resolved.config,
      targets: ["executions"],
    });
    return {
      diagnostics: codegen.diagnostics,
      ok: codegen.ok,
      plannedFiles: planned.plannedFiles,
      records: planned.records,
      rolledBack: false,
      writtenFiles: [
        ...writtenFiles,
        ...codegen.hooks
          .filter((hook) => hook.status === "created")
          .map((hook) => displayPath(resolved.config.configDir, hook.path)),
      ],
    };
  } catch (error) {
    restoreSnapshots(snapshots);
    return {
      diagnostics: [
        workflowError(
          "scaffold-failed",
          error instanceof Error
            ? `Failed to scaffold Studio mechanic: ${error.message}`
            : "Failed to scaffold Studio mechanic.",
        ),
      ],
      ok: false,
      plannedFiles: planned.plannedFiles,
      records: planned.records,
      rolledBack: true,
      writtenFiles,
    };
  }
};

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
): StudioHostAppFileResult[] =>
  Object.entries(hostAppScaffoldFiles(config, root)).map(([relativePath, expected]) => {
    const path = join(root, relativePath);
    const current = readTextIfExists(path);
    if (current === undefined) {
      return {
        path,
        reason: "Required host app scaffold file is missing.",
        status: "manual-follow-up",
      };
    }
    if (isHostAppProjectOwnedFile(relativePath)) {
      return {
        path,
        reason: "Project-owned host app file preserved.",
        status: "project-owned",
      };
    }
    if (current !== expected) {
      return {
        path,
        reason: "Host app scaffold file differs from the current scaffold template.",
        status: "manual-follow-up",
      };
    }
    return { path, status: "unchanged" };
  });

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

const hasErrorDiagnostic = (diagnostics: readonly StudioDiagnostic[]) =>
  diagnostics.some((diagnostic) => diagnostic.severity === "error");

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
      const diagnostics = diagnosticsMatching(validation.diagnostics, [
        `source "${source.id}"`,
        `adapter "${source.adapterId}"`,
        source.id,
        source.adapterId,
      ]);
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
        const diagnostics = diagnosticsMatching(validation.diagnostics, [
          `content mapper "${mapper.id}"`,
          mapper.id,
        ]);
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

const migrationCheck = (
  input: Omit<
    StudioMigrationCheckResult,
    "applied" | "changedFiles" | "diagnostics" | "manualFollowUps" | "skipped" | "status"
  > & {
    applied?: readonly string[];
    changedFiles?: readonly string[];
    diagnostics?: readonly StudioDiagnostic[];
    manualFollowUps?: readonly string[];
    skipped?: readonly string[];
  },
): StudioMigrationCheckResult => {
  const applied = [...(input.applied ?? [])];
  const changedFilePaths = [...(input.changedFiles ?? [])];
  const diagnostics = [...(input.diagnostics ?? [])];
  const followUps = [...(input.manualFollowUps ?? [])];
  const skipped = [...(input.skipped ?? [])];
  let status: StudioMigrationCheckStatus = "skipped";
  if (hasErrorDiagnostic(diagnostics)) {
    status = "failed";
  } else if (applied.length > 0 || changedFilePaths.length > 0) {
    status = "applied";
  }

  return {
    ...input,
    applied,
    changedFiles: changedFilePaths,
    diagnostics,
    manualFollowUps: followUps,
    skipped,
    status,
  };
};

const runHostAppMigration = (
  config: ResolvedStudioProjectConfig,
  root: string,
): StudioMigrationCheckResult => {
  if (!existsSync(hostAppMetadataPath(root))) {
    return migrationCheck({
      name: "host-app-scaffold",
      skipped: ["No local host app scaffold metadata found."],
      targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    });
  }

  const currentVersion = readHostAppMetadataVersion(root) ?? 0;
  if (currentVersion > STUDIO_HOST_APP_SCAFFOLD_VERSION) {
    const followUp = `Unsupported local host app scaffold version ${currentVersion}; active Studio supports ${STUDIO_HOST_APP_SCAFFOLD_VERSION}.`;
    return migrationCheck({
      currentVersion,
      diagnostics: [
        workflowError(
          "unsupported-host-app-scaffold-version",
          followUp,
          hostAppMetadataPath(root),
          "Upgrade Flexweave Studio or recreate the local host app scaffold manually.",
        ),
      ],
      manualFollowUps: [followUp],
      name: "host-app-scaffold",
      targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    });
  }

  if (currentVersion === STUDIO_HOST_APP_SCAFFOLD_VERSION) {
    return migrationCheck({
      currentVersion,
      name: "host-app-scaffold",
      skipped: ["Local host app scaffold is current."],
      targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    });
  }

  const files = writeHostAppScaffold(config, root, {
    updateMetadata: true,
  });
  const followUps = manualFollowUps(files);

  return migrationCheck({
    applied: [`host app scaffold ${currentVersion} -> ${STUDIO_HOST_APP_SCAFFOLD_VERSION}`],
    changedFiles: changedFiles(files),
    currentVersion,
    diagnostics: followUps.map((followUp) =>
      workflowWarning("host-app-manual-follow-up", followUp),
    ),
    manualFollowUps: followUps,
    name: "host-app-scaffold",
    targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
  });
};

const runExtensionMigrations = async (
  config: ResolvedStudioProjectConfig,
  appRoot: string,
): Promise<StudioMigrationCheckResult[]> => {
  const checks: StudioMigrationCheckResult[] = [];
  const extensions = [...config.extensions].toSorted((left, right) =>
    left.id.localeCompare(right.id),
  );

  for (const extension of extensions) {
    const migrations = [...(extension.migrations ?? [])].toSorted((left, right) =>
      left.id.localeCompare(right.id),
    );
    for (const migration of migrations) {
      const name = `extension:${extension.id}:${migration.id}`;
      try {
        const result = await migration.migrate({ appRoot, config });
        checks.push(
          migrationCheck({
            applied: result.applied,
            changedFiles: result.changedFiles,
            currentVersion: migration.fromVersion,
            diagnostics: result.diagnostics,
            extensionId: extension.id,
            manualFollowUps: result.manualFollowUps,
            name,
            skipped: result.skipped,
            targetVersion: migration.toVersion,
          }),
        );
      } catch (error) {
        checks.push(
          migrationCheck({
            currentVersion: migration.fromVersion,
            diagnostics: [
              workflowError(
                "extension-migration-failed",
                error instanceof Error
                  ? `Studio extension "${extension.id}" migration "${migration.id}" failed: ${error.message}`
                  : `Studio extension "${extension.id}" migration "${migration.id}" failed.`,
              ),
            ],
            extensionId: extension.id,
            manualFollowUps: [
              `Review extension migration "${extension.id}:${migration.id}" before rerunning migrate.`,
            ],
            name,
            targetVersion: migration.toVersion,
          }),
        );
      }
    }
  }

  return checks;
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
      ok: false,
      recordCount: 0,
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

  const validation = await validateStudioCatalog({ config: resolved.config });
  const codegen = await codegenStudioProject({ check: true, config: resolved.config });
  const hostApp = await verifyStudioHostApp({
    appRoot: options.appRoot,
    config: resolved.config,
  });
  const commandConfigs = options.fast
    ? resolved.config.verify.commands.filter((command) => command.fast)
    : resolved.config.verify.commands;
  const commands: StudioVerifyCommandResult[] = [];

  for (const commandConfig of commandConfigs) {
    const proc = Bun.spawn(commandConfig.command, {
      cwd: resolved.config.configDir,
      stderr: "pipe",
      stdout: "pipe",
    });
    const [stdout, stderr, exitCode] = await Promise.all([
      new Response(proc.stdout).text(),
      new Response(proc.stderr).text(),
      proc.exited,
    ]);
    commands.push({
      command: commandConfig.command,
      exitCode,
      fast: commandConfig.fast,
      name: commandConfig.name,
      stderr,
      stdout,
    });
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

export const migrateStudioProject = async (
  options: ScaffoldStudioHostAppOptions = {},
): Promise<MigrateStudioProjectResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      applied: [],
      changedFiles: [],
      checks: [
        migrationCheck({
          diagnostics: resolved.diagnostics,
          name: "config",
        }),
      ],
      diagnostics: resolved.diagnostics,
      manualFollowUps: [],
      ok: false,
      skipped: [],
    };
  }

  const root = hostAppRoot(resolved.config, options.appRoot);
  const checks = [
    runHostAppMigration(resolved.config, root),
    ...(await runExtensionMigrations(resolved.config, root)),
  ];
  const diagnostics = checks.flatMap((check) => check.diagnostics);

  return {
    applied: checks.flatMap((check) => check.applied),
    changedFiles: checks.flatMap((check) => check.changedFiles),
    checks,
    diagnostics,
    manualFollowUps: checks.flatMap((check) => check.manualFollowUps),
    ok: diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    skipped: checks.flatMap((check) => check.skipped),
  };
};

export const studioWorkflowNames = [
  "validate",
  "describe",
  "list",
  "show",
  "plan",
  "scaffold",
  "codegen",
  "verify",
  "migrate",
] as const;
