import {
  composeHostAppContributionModel,
  hostAppContributionModelValues,
  validateHostAppContributionModel,
} from "@flexweave/studio/config";
import type {
  ResolvedStudioProjectConfig,
  StudioDiagnostic,
  StudioHostAppContributionModelValidationContext,
} from "@flexweave/studio/config";
import { loadStudioConfig } from "@flexweave/studio/config/load";
import type {
  StudioExtension,
  StudioHostAppActionVariant,
  StudioHostAppAuthoringAreaDefinition,
  StudioHostAppAuthoringContribution,
  StudioHostAppAuthoringEditorDefinition,
  StudioHostAppCodegenTargetDefinition,
  StudioHostAppContribution,
  StudioHostAppDiagnosticsPanelDefinition,
  StudioHostAppGeneratedOutputPanelDefinition,
  StudioHostAppNavigationLink,
  StudioHostAppNavigationSection,
  StudioHostAppRouteKind,
  StudioHostAppSourceViewDefinition,
  StudioHostAppWorkflowActionDefinition,
  StudioHostAppWorkflowCommandName,
} from "@flexweave/studio/extensions";
import {
  codegenStudioProject,
  describeStudioCatalog,
  listStudioCatalogRecords,
  listStudioGeneratedTargetMetadata,
  migrateStudioProject,
  planStudioMechanic,
  scaffoldStudioMechanic,
  showStudioCatalogRecord,
  validateStudioCatalog,
  verifyStudioProject,
} from "@flexweave/studio/workflows";

export type StudioAppRouteKind = StudioHostAppRouteKind;

export type StudioAppWorkflowCommandName = StudioHostAppWorkflowCommandName;

export type StudioAppActionVariant = StudioHostAppActionVariant;

export interface StudioAppLabels {
  productName: string;
  projectName: string;
  shellSubtitle?: string;
  workspaceTitle: string;
  workflowTrail: readonly string[];
}

export type StudioAppNavigationLink = StudioHostAppNavigationLink;

export type StudioAppNavigationSection = StudioHostAppNavigationSection;

export type StudioAppAuthoringAreaDefinition = StudioHostAppAuthoringAreaDefinition;

export type StudioAppAuthoringEditorDefinition = StudioHostAppAuthoringEditorDefinition;

export type StudioAppWorkflowActionDefinition = StudioHostAppWorkflowActionDefinition;

export type StudioAppCodegenTargetDefinition = StudioHostAppCodegenTargetDefinition;

export type StudioAppGeneratedOutputPanelDefinition = StudioHostAppGeneratedOutputPanelDefinition;

export type StudioAppDiagnosticsPanelDefinition = StudioHostAppDiagnosticsPanelDefinition;

export type StudioAppSourceViewDefinition = StudioHostAppSourceViewDefinition;

export type StudioAppAuthoringContribution = StudioHostAppAuthoringContribution;

export type StudioAppContribution = StudioHostAppContribution;

export type StudioAppServerFunction<Input = unknown, Output = unknown> = (
  input: Input,
) => Output | Promise<Output>;

export interface StudioAppServerFunctionBindings {
  codegen?: StudioAppServerFunction<{ check?: boolean; targets?: readonly string[] }>;
  describe?: StudioAppServerFunction<{ kind?: string }>;
  list?: StudioAppServerFunction<{ filter?: string; kind: string }>;
  migrate?: StudioAppServerFunction;
  plan?: StudioAppServerFunction<Record<string, unknown>>;
  scaffold?: StudioAppServerFunction<Record<string, unknown>>;
  show?: StudioAppServerFunction<{ id: string; kind: string }>;
  validate?: StudioAppServerFunction;
  verify?: StudioAppServerFunction<{ fast?: boolean }>;
}

export interface StudioAppAuthoringConfig {
  areas: readonly StudioAppAuthoringAreaDefinition[];
  editors: readonly StudioAppAuthoringEditorDefinition[];
}

export interface StudioAppAdapter {
  authoring: StudioAppAuthoringConfig;
  codegenTargets: readonly StudioAppCodegenTargetDefinition[];
  diagnosticsPanels?: readonly StudioAppDiagnosticsPanelDefinition[];
  generatedOutputPanels?: readonly StudioAppGeneratedOutputPanelDefinition[];
  id: string;
  labels: StudioAppLabels;
  navigation: readonly StudioAppNavigationSection[];
  serverFunctions: StudioAppServerFunctionBindings;
  sourceViews?: readonly StudioAppSourceViewDefinition[];
  workflowActions: readonly StudioAppWorkflowActionDefinition[];
}

export interface StudioAppRouteDefinition {
  editorId?: string;
  href: string;
  id: string;
  kind: StudioAppRouteKind;
  label: string;
}

export interface StudioAppShellModel {
  adapterId: string;
  codegenTargets: readonly StudioAppCodegenTargetDefinition[];
  diagnostics: readonly StudioDiagnostic[];
  diagnosticsPanels: readonly StudioAppDiagnosticsPanelDefinition[];
  generatedOutputPanels: readonly StudioAppGeneratedOutputPanelDefinition[];
  labels: StudioAppLabels;
  navigation: readonly StudioAppNavigationSection[];
  ok: boolean;
  routes: readonly StudioAppRouteDefinition[];
  sourceViews: readonly StudioAppSourceViewDefinition[];
  workflowActions: readonly StudioAppWorkflowActionDefinition[];
}

export interface StudioAppPanelModel {
  codegenTargets: readonly StudioAppCodegenTargetDefinition[];
  diagnosticsPanels: readonly StudioAppDiagnosticsPanelDefinition[];
  editors: readonly StudioAppAuthoringEditorDefinition[];
  generatedOutputPanels: readonly StudioAppGeneratedOutputPanelDefinition[];
  sourceViews: readonly StudioAppSourceViewDefinition[];
  title: string;
  workflowActions: readonly StudioAppWorkflowActionDefinition[];
}

export interface StudioAppCompositionSuccessResult {
  adapter: StudioAppAdapter;
  diagnostics: StudioDiagnostic[];
  ok: true;
}

export interface StudioAppCompositionFailureResult {
  adapter?: StudioAppAdapter;
  diagnostics: StudioDiagnostic[];
  ok: false;
}

export type StudioAppCompositionResult =
  | StudioAppCompositionFailureResult
  | StudioAppCompositionSuccessResult;

export type StudioAppContributionValidationContext = Omit<
  StudioHostAppContributionModelValidationContext,
  "serverFunctionNames"
>;

export type StudioAppInput = StudioAppAdapter | StudioAppCompositionResult;

export const defineStudioAppAdapter = <const Adapter extends StudioAppAdapter>(
  adapter: Adapter,
): Adapter => adapter;

const editorRoute = (editor: StudioAppAuthoringEditorDefinition): StudioAppRouteDefinition => ({
  editorId: editor.id,
  href: `/authoring/${editor.id}`,
  id: `authoring.${editor.id}`,
  kind: "authoring-editor",
  label: editor.label,
});

const generatedOutputRoute = (
  panel: StudioAppGeneratedOutputPanelDefinition,
): StudioAppRouteDefinition => ({
  href: `/generated/${panel.id}`,
  id: `generated.${panel.id}`,
  kind: "generated-output",
  label: panel.label,
});

const diagnosticsRoute = (
  panel: StudioAppDiagnosticsPanelDefinition,
): StudioAppRouteDefinition => ({
  href: `/diagnostics/${panel.id}`,
  id: `diagnostics.${panel.id}`,
  kind: "diagnostics-panel",
  label: panel.label,
});

const sourceViewRoute = (view: StudioAppSourceViewDefinition): StudioAppRouteDefinition => ({
  href: `/sources/${view.id}`,
  id: `source.${view.id}`,
  kind: "source-view",
  label: view.label,
});

export const collectStudioAppContributions = (
  extensions: readonly StudioExtension[],
): StudioAppContribution[] => extensions.flatMap((extension) => extension.appContributions ?? []);

const serverFunctionNames = (adapter: StudioAppAdapter): string[] =>
  Object.entries(adapter.serverFunctions).flatMap(([name, binding]) =>
    typeof binding === "function" ? [name] : [],
  );

const appContributionValidationContext = (
  adapter: StudioAppAdapter,
  context: StudioAppContributionValidationContext = {},
): StudioHostAppContributionModelValidationContext => ({
  ...context,
  serverFunctionNames: serverFunctionNames(adapter),
});

export const validateStudioAppAdapter = (
  adapter: StudioAppAdapter,
  context: StudioAppContributionValidationContext = {},
): StudioDiagnostic[] =>
  validateHostAppContributionModel(
    composeHostAppContributionModel(adapter, []),
    appContributionValidationContext(adapter, context),
  );

export const composeStudioAppContributions = (
  adapter: StudioAppAdapter,
  contributions: readonly StudioAppContribution[],
  context: StudioAppContributionValidationContext = {},
): StudioAppCompositionResult => {
  const contributionModel = composeHostAppContributionModel(adapter, contributions);
  const composedContributions = hostAppContributionModelValues(contributionModel);
  const composed: StudioAppAdapter = {
    ...adapter,
    authoring: composedContributions.authoring,
    codegenTargets: composedContributions.codegenTargets,
    diagnosticsPanels: composedContributions.diagnosticsPanels,
    generatedOutputPanels: composedContributions.generatedOutputPanels,
    navigation: composedContributions.navigation,
    sourceViews: composedContributions.sourceViews,
    workflowActions: composedContributions.workflowActions,
  };
  const diagnostics = validateHostAppContributionModel(
    contributionModel,
    appContributionValidationContext(composed, context),
  );

  if (diagnostics.length > 0) {
    return {
      adapter: composed,
      diagnostics,
      ok: false,
    };
  }

  return {
    adapter: composed,
    diagnostics,
    ok: true,
  };
};

export interface CreateDefaultStudioProjectAdapterOptions {
  configPath: string;
  id?: string;
  labels?: Partial<StudioAppLabels>;
}

const defaultStudioAppLabels: StudioAppLabels = {
  productName: "Flexweave Studio",
  projectName: "Consumer project",
  shellSubtitle: "Catalog authoring",
  workflowTrail: ["Studio catalog", "Generated mechanics definitions", "Consumer runtime"],
  workspaceTitle: "Authoring workspace",
};

const asMechanicInput = (input: Record<string, unknown>) => ({
  archetype: typeof input.archetype === "string" ? input.archetype : "mechanic",
  id: typeof input.id === "string" ? input.id : "",
  name: typeof input.name === "string" ? input.name : "",
  params:
    typeof input.params === "object" && input.params !== null && !Array.isArray(input.params)
      ? (input.params as Record<string, unknown>)
      : undefined,
});

const extensionContributedCodegenTargetIds = (config: ResolvedStudioProjectConfig) =>
  new Set(
    config.extensions.flatMap((extension) =>
      (extension.appContributions ?? []).flatMap((contribution) =>
        (contribution.codegenTargets ?? []).map((target) => target.target),
      ),
    ),
  );

const activeGeneratedTargetIds = (config: ResolvedStudioProjectConfig) => [
  ...config.codegen.builtInTargets,
  ...config.extensions.flatMap((extension) =>
    (extension.generatedTargets ?? []).map((target) => target.id),
  ),
];

const defaultCodegenTargets = (config?: ResolvedStudioProjectConfig) => {
  if (!config) {
    return [];
  }

  const extensionCodegenTargetIds = extensionContributedCodegenTargetIds(config);
  return listStudioGeneratedTargetMetadata(config, { configuredOnly: true }).filter(
    (target) => !extensionCodegenTargetIds.has(target.target),
  );
};

const defaultServerFunctions = (configPath: string): StudioAppServerFunctionBindings => ({
  codegen: (input = {}) => codegenStudioProject({ configPath, ...input }),
  describe: (input = {}) => describeStudioCatalog(input.kind, { configPath }),
  list: (input) => listStudioCatalogRecords(input.kind, { configPath, filter: input.filter }),
  migrate: () => migrateStudioProject({ configPath }),
  plan: (input) => planStudioMechanic({ configPath, ...asMechanicInput(input) }),
  scaffold: (input) => scaffoldStudioMechanic({ configPath, ...asMechanicInput(input) }),
  show: (input) => showStudioCatalogRecord(input.kind, input.id, { configPath }),
  validate: () => validateStudioCatalog({ configPath }),
  verify: (input = {}) => verifyStudioProject({ configPath, ...input }),
});

export const createDefaultStudioProjectAdapter = async (
  options: CreateDefaultStudioProjectAdapterOptions,
): Promise<StudioAppCompositionResult> => {
  const loadedConfig = await loadStudioConfig({ configPath: options.configPath });

  if (!loadedConfig.ok || !loadedConfig.config) {
    return {
      diagnostics: loadedConfig.diagnostics,
      ok: false,
    };
  }

  const config = loadedConfig.config;
  const extensionContributions = collectStudioAppContributions(config.extensions);
  const baseProjectAdapter = defineStudioAppAdapter({
    authoring: {
      areas: [
        { editorId: "tags", id: "tags", label: "Tags" },
        { editorId: "abilities", id: "abilities", label: "Abilities" },
        { editorId: "effects", id: "effects", label: "Effects" },
      ],
      editors: [
        { areaId: "tags", commandName: "list", id: "tags", label: "Tags", recordKind: "tags" },
        {
          areaId: "abilities",
          commandName: "list",
          id: "abilities",
          label: "Abilities",
          recordKind: "abilities",
        },
        {
          areaId: "effects",
          commandName: "list",
          id: "effects",
          label: "Effects",
          recordKind: "effects",
        },
      ],
    },
    codegenTargets: defaultCodegenTargets(config),
    id: options.id ?? "local-studio-host",
    labels: {
      ...defaultStudioAppLabels,
      ...options.labels,
    },
    navigation: [
      {
        id: "workspace",
        label: "Workspace",
        links: [{ href: "/", id: "overview", label: "Overview" }],
      },
      {
        id: "generated",
        label: "Generated",
        links: [{ href: "/#generated-output", id: "generated-output", label: "Generated output" }],
      },
    ],
    serverFunctions: defaultServerFunctions(options.configPath),
    workflowActions: [
      { commandName: "validate", id: "validate", label: "Validate", variant: "secondary" },
      { commandName: "codegen", id: "codegen", label: "Generate", variant: "secondary" },
      { commandName: "verify", id: "verify", label: "Verify", variant: "primary" },
    ],
  });
  const composedProjectAdapter = composeStudioAppContributions(
    baseProjectAdapter,
    extensionContributions,
    {
      dataAdapterIds: config.data.adapterRegistry.adapters.map((dataAdapter) => dataAdapter.id),
      generatedTargetIds: activeGeneratedTargetIds(config),
      sourceReferences: config.data.sources.map((source) => ({
        adapterId: source.adapterId,
        sourceId: source.id,
      })),
    },
  );
  const diagnostics = [...loadedConfig.diagnostics, ...composedProjectAdapter.diagnostics];

  if (!composedProjectAdapter.ok) {
    return {
      adapter: composedProjectAdapter.adapter,
      diagnostics,
      ok: false,
    };
  }

  return {
    adapter: composedProjectAdapter.adapter,
    diagnostics,
    ok: true,
  };
};

export const createStudioAppRoutes = (adapter: StudioAppAdapter): StudioAppRouteDefinition[] => [
  {
    href: "/",
    id: "overview",
    kind: "overview",
    label: adapter.labels.workspaceTitle,
  },
  {
    href: "/#generated-output",
    id: "generated-output",
    kind: "generated-output",
    label: "Generated output",
  },
  ...adapter.authoring.editors.map(editorRoute),
  ...(adapter.generatedOutputPanels ?? []).map(generatedOutputRoute),
  ...(adapter.diagnosticsPanels ?? []).map(diagnosticsRoute),
  ...(adapter.sourceViews ?? []).map(sourceViewRoute),
];

const isStudioAppCompositionResult = (input: StudioAppInput): input is StudioAppCompositionResult =>
  "diagnostics" in input && "ok" in input;

const createDiagnosticStudioApp = (
  diagnostics: readonly StudioDiagnostic[],
): StudioAppShellModel => ({
  adapterId: "diagnostic-studio-host",
  codegenTargets: [],
  diagnostics,
  diagnosticsPanels: [],
  generatedOutputPanels: [],
  labels: {
    ...defaultStudioAppLabels,
    shellSubtitle: "Configuration diagnostics",
    workspaceTitle: "Studio diagnostics",
  },
  navigation: [
    {
      id: "diagnostics",
      label: "Diagnostics",
      links: [{ href: "/", id: "diagnostics", label: "Diagnostics" }],
    },
  ],
  ok: false,
  routes: [
    {
      href: "/",
      id: "diagnostics",
      kind: "overview",
      label: "Studio diagnostics",
    },
  ],
  sourceViews: [],
  workflowActions: [],
});

const createStudioAppShell = (
  adapter: StudioAppAdapter,
  diagnostics: readonly StudioDiagnostic[],
  ok: boolean,
): StudioAppShellModel => ({
  adapterId: adapter.id,
  codegenTargets: adapter.codegenTargets,
  diagnostics,
  diagnosticsPanels: adapter.diagnosticsPanels ?? [],
  generatedOutputPanels: adapter.generatedOutputPanels ?? [],
  labels: adapter.labels,
  navigation: adapter.navigation,
  ok,
  routes: createStudioAppRoutes(adapter),
  sourceViews: adapter.sourceViews ?? [],
  workflowActions: adapter.workflowActions,
});

export const createStudioApp = (input: StudioAppInput): StudioAppShellModel => {
  if (isStudioAppCompositionResult(input)) {
    if (!input.adapter) {
      return createDiagnosticStudioApp(input.diagnostics);
    }
    return createStudioAppShell(input.adapter, input.diagnostics, input.ok);
  }

  const diagnostics = validateStudioAppAdapter(input);
  return createStudioAppShell(input, diagnostics, diagnostics.length === 0);
};

export const createStudioOverviewPanel = (adapter: StudioAppAdapter): StudioAppPanelModel => ({
  codegenTargets: adapter.codegenTargets,
  diagnosticsPanels: adapter.diagnosticsPanels ?? [],
  editors: adapter.authoring.editors,
  generatedOutputPanels: adapter.generatedOutputPanels ?? [],
  sourceViews: adapter.sourceViews ?? [],
  title: adapter.labels.workspaceTitle,
  workflowActions: adapter.workflowActions,
});
