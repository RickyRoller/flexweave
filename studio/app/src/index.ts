export type StudioAppRouteKind = "authoring-editor" | "generated-output" | "overview";

export type StudioAppWorkflowCommandName =
  | "codegen"
  | "describe"
  | "list"
  | "migrate"
  | "plan"
  | "scaffold"
  | "show"
  | "validate"
  | "verify";

export type StudioAppActionVariant = "primary" | "secondary";

export interface StudioAppLabels {
  productName: string;
  projectName: string;
  shellSubtitle?: string;
  workspaceTitle: string;
  workflowTrail: readonly string[];
}

export interface StudioAppNavigationLink {
  href: string;
  icon?: string;
  id: string;
  label: string;
}

export interface StudioAppNavigationSection {
  id: string;
  label: string;
  links: readonly StudioAppNavigationLink[];
}

export interface StudioAppAuthoringAreaDefinition {
  description?: string;
  editorId?: string;
  icon?: string;
  id: string;
  label: string;
}

export interface StudioAppAuthoringEditorDefinition {
  areaId: string;
  commandName?: StudioAppWorkflowCommandName;
  description?: string;
  id: string;
  label: string;
  recordKind?: string;
}

export interface StudioAppWorkflowActionDefinition {
  commandName: StudioAppWorkflowCommandName;
  id: string;
  label: string;
  variant: StudioAppActionVariant;
}

export interface StudioAppCodegenTargetDefinition {
  description?: string;
  label: string;
  outputLabel?: string;
  target: string;
}

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
  id: string;
  labels: StudioAppLabels;
  navigation: readonly StudioAppNavigationSection[];
  serverFunctions: StudioAppServerFunctionBindings;
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
  labels: StudioAppLabels;
  navigation: readonly StudioAppNavigationSection[];
  routes: readonly StudioAppRouteDefinition[];
  workflowActions: readonly StudioAppWorkflowActionDefinition[];
}

export interface StudioAppPanelModel {
  codegenTargets: readonly StudioAppCodegenTargetDefinition[];
  editors: readonly StudioAppAuthoringEditorDefinition[];
  title: string;
  workflowActions: readonly StudioAppWorkflowActionDefinition[];
}

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
];

export const createStudioApp = (adapter: StudioAppAdapter): StudioAppShellModel => ({
  adapterId: adapter.id,
  codegenTargets: adapter.codegenTargets,
  labels: adapter.labels,
  navigation: adapter.navigation,
  routes: createStudioAppRoutes(adapter),
  workflowActions: adapter.workflowActions,
});

export const createStudioOverviewPanel = (adapter: StudioAppAdapter): StudioAppPanelModel => ({
  codegenTargets: adapter.codegenTargets,
  editors: adapter.authoring.editors,
  title: adapter.labels.workspaceTitle,
  workflowActions: adapter.workflowActions,
});
