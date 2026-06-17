import type {
  StudioHostAppActionVariant,
  StudioHostAppAuthoringAreaDefinition,
  StudioHostAppAuthoringEditorDefinition,
  StudioHostAppCodegenTargetDefinition,
  StudioHostAppContribution,
  StudioHostAppDiagnosticsPanelDefinition,
  StudioHostAppGeneratedOutputPanelDefinition,
  StudioHostAppNavigationLink,
  StudioHostAppNavigationSection,
  StudioHostAppSourceViewDefinition,
  StudioHostAppWorkflowActionDefinition,
  StudioHostAppWorkflowCommandName,
} from "../extensions";
import { configError } from "./diagnostics";
import { isObject, readOptionalString, readString } from "./primitive-readers";
import type { StudioDiagnostic } from "./types";

const validHostAppWorkflowCommands = [
  "codegen",
  "describe",
  "list",
  "migrate",
  "plan",
  "scaffold",
  "show",
  "validate",
  "verify",
] as const satisfies readonly StudioHostAppWorkflowCommandName[];

const validHostAppActionVariants = [
  "primary",
  "secondary",
] as const satisfies readonly StudioHostAppActionVariant[];

export interface StudioHostAppContributionModelEntry<Value> {
  field: string;
  value: Value;
}

export interface StudioHostAppContributionModel {
  authoring: {
    areas: readonly StudioHostAppContributionModelEntry<StudioHostAppAuthoringAreaDefinition>[];
    editors: readonly StudioHostAppContributionModelEntry<StudioHostAppAuthoringEditorDefinition>[];
  };
  codegenTargets: readonly StudioHostAppContributionModelEntry<StudioHostAppCodegenTargetDefinition>[];
  diagnosticsPanels: readonly StudioHostAppContributionModelEntry<StudioHostAppDiagnosticsPanelDefinition>[];
  generatedOutputPanels: readonly StudioHostAppContributionModelEntry<StudioHostAppGeneratedOutputPanelDefinition>[];
  navigation: readonly StudioHostAppContributionModelEntry<StudioHostAppNavigationSection>[];
  sourceViews: readonly StudioHostAppContributionModelEntry<StudioHostAppSourceViewDefinition>[];
  workflowActions: readonly StudioHostAppContributionModelEntry<StudioHostAppWorkflowActionDefinition>[];
}

export interface StudioHostAppContributionModelInput {
  authoring?: StudioHostAppContribution["authoring"];
  codegenTargets?: readonly StudioHostAppCodegenTargetDefinition[];
  diagnosticsPanels?: readonly StudioHostAppDiagnosticsPanelDefinition[];
  generatedOutputPanels?: readonly StudioHostAppGeneratedOutputPanelDefinition[];
  navigation?: readonly StudioHostAppNavigationSection[];
  sourceViews?: readonly StudioHostAppSourceViewDefinition[];
  workflowActions?: readonly StudioHostAppWorkflowActionDefinition[];
}

export interface StudioHostAppContributionModelValues {
  authoring: {
    areas: StudioHostAppAuthoringAreaDefinition[];
    editors: StudioHostAppAuthoringEditorDefinition[];
  };
  codegenTargets: StudioHostAppCodegenTargetDefinition[];
  diagnosticsPanels: StudioHostAppDiagnosticsPanelDefinition[];
  generatedOutputPanels: StudioHostAppGeneratedOutputPanelDefinition[];
  navigation: StudioHostAppNavigationSection[];
  sourceViews: StudioHostAppSourceViewDefinition[];
  workflowActions: StudioHostAppWorkflowActionDefinition[];
}

const joinField = (...parts: (string | undefined)[]) =>
  parts.filter((part): part is string => part !== undefined && part.length > 0).join(".");

const entriesFor = <Value>(
  values: readonly Value[] | undefined,
  field: string,
): StudioHostAppContributionModelEntry<Value>[] =>
  (values ?? []).map((value, index) => ({
    field: `${field}.${index}`,
    value,
  }));

export const normalizeHostAppContributionModel = (
  contribution: StudioHostAppContributionModelInput,
  field?: string,
): StudioHostAppContributionModel => ({
  authoring: {
    areas: entriesFor(contribution.authoring?.areas, joinField(field, "authoring.areas")),
    editors: entriesFor(contribution.authoring?.editors, joinField(field, "authoring.editors")),
  },
  codegenTargets: entriesFor(contribution.codegenTargets, joinField(field, "codegenTargets")),
  diagnosticsPanels: entriesFor(
    contribution.diagnosticsPanels,
    joinField(field, "diagnosticsPanels"),
  ),
  generatedOutputPanels: entriesFor(
    contribution.generatedOutputPanels,
    joinField(field, "generatedOutputPanels"),
  ),
  navigation: entriesFor(contribution.navigation, joinField(field, "navigation")),
  sourceViews: entriesFor(contribution.sourceViews, joinField(field, "sourceViews")),
  workflowActions: entriesFor(contribution.workflowActions, joinField(field, "workflowActions")),
});

export const mergeHostAppContributionModels = (
  models: readonly StudioHostAppContributionModel[],
): StudioHostAppContributionModel => ({
  authoring: {
    areas: models.flatMap((model) => model.authoring.areas),
    editors: models.flatMap((model) => model.authoring.editors),
  },
  codegenTargets: models.flatMap((model) => model.codegenTargets),
  diagnosticsPanels: models.flatMap((model) => model.diagnosticsPanels),
  generatedOutputPanels: models.flatMap((model) => model.generatedOutputPanels),
  navigation: models.flatMap((model) => model.navigation),
  sourceViews: models.flatMap((model) => model.sourceViews),
  workflowActions: models.flatMap((model) => model.workflowActions),
});

export const normalizeHostAppContributions = (
  contributions: readonly StudioHostAppContribution[],
  field: string,
): StudioHostAppContributionModel =>
  mergeHostAppContributionModels(
    contributions.map((contribution, index) =>
      normalizeHostAppContributionModel(contribution, `${field}.${index}`),
    ),
  );

export const composeHostAppContributionModel = (
  base: StudioHostAppContributionModelInput,
  contributions: readonly StudioHostAppContributionModelInput[],
): StudioHostAppContributionModel =>
  normalizeHostAppContributionModel({
    authoring: {
      areas: [
        ...(base.authoring?.areas ?? []),
        ...contributions.flatMap((contribution) => contribution.authoring?.areas ?? []),
      ],
      editors: [
        ...(base.authoring?.editors ?? []),
        ...contributions.flatMap((contribution) => contribution.authoring?.editors ?? []),
      ],
    },
    codegenTargets: [
      ...(base.codegenTargets ?? []),
      ...contributions.flatMap((contribution) => contribution.codegenTargets ?? []),
    ],
    diagnosticsPanels: [
      ...(base.diagnosticsPanels ?? []),
      ...contributions.flatMap((contribution) => contribution.diagnosticsPanels ?? []),
    ],
    generatedOutputPanels: [
      ...(base.generatedOutputPanels ?? []),
      ...contributions.flatMap((contribution) => contribution.generatedOutputPanels ?? []),
    ],
    navigation: [
      ...(base.navigation ?? []),
      ...contributions.flatMap((contribution) => contribution.navigation ?? []),
    ],
    sourceViews: [
      ...(base.sourceViews ?? []),
      ...contributions.flatMap((contribution) => contribution.sourceViews ?? []),
    ],
    workflowActions: [
      ...(base.workflowActions ?? []),
      ...contributions.flatMap((contribution) => contribution.workflowActions ?? []),
    ],
  });

export const hostAppContributionModelValues = (
  model: StudioHostAppContributionModel,
): StudioHostAppContributionModelValues => ({
  authoring: {
    areas: model.authoring.areas.map((entry) => entry.value),
    editors: model.authoring.editors.map((entry) => entry.value),
  },
  codegenTargets: model.codegenTargets.map((entry) => entry.value),
  diagnosticsPanels: model.diagnosticsPanels.map((entry) => entry.value),
  generatedOutputPanels: model.generatedOutputPanels.map((entry) => entry.value),
  navigation: model.navigation.map((entry) => entry.value),
  sourceViews: model.sourceViews.map((entry) => entry.value),
  workflowActions: model.workflowActions.map((entry) => entry.value),
});

const duplicateHostAppContributionDiagnostic = (field: string, key: string): StudioDiagnostic =>
  configError(
    "duplicate-host-app-contribution",
    field,
    `Studio app contribution id "${key}" is registered more than once.`,
    "Use stable, unique ids for host app contributions and contributed app surfaces.",
  );

const validateUniqueModelEntries = <Value>(
  entries: readonly StudioHostAppContributionModelEntry<Value>[],
  keyForValue: (value: Value) => string,
  diagnostics: StudioDiagnostic[],
) => {
  const seen = new Set<string>();
  for (const entry of entries) {
    const key = keyForValue(entry.value);
    if (seen.has(key)) {
      diagnostics.push(duplicateHostAppContributionDiagnostic(entry.field, key));
    }
    seen.add(key);
  }
};

export const validateHostAppContributionModel = (
  model: StudioHostAppContributionModel,
): StudioDiagnostic[] => {
  const diagnostics: StudioDiagnostic[] = [];
  validateUniqueModelEntries(model.navigation, (section) => section.id, diagnostics);
  validateUniqueModelEntries(model.authoring.areas, (area) => area.id, diagnostics);
  validateUniqueModelEntries(model.authoring.editors, (editor) => editor.id, diagnostics);
  validateUniqueModelEntries(model.codegenTargets, (target) => target.target, diagnostics);
  validateUniqueModelEntries(model.diagnosticsPanels, (panel) => panel.id, diagnostics);
  validateUniqueModelEntries(model.generatedOutputPanels, (panel) => panel.id, diagnostics);
  validateUniqueModelEntries(model.sourceViews, (view) => view.id, diagnostics);
  validateUniqueModelEntries(model.workflowActions, (action) => action.id, diagnostics);
  return diagnostics;
};

const readStringLiteral = <Value extends string>(
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
  validValues: readonly Value[],
  label: string,
): Value | undefined => {
  const stringValue = readString(value, field, diagnostics);
  if (!stringValue) {
    return undefined;
  }
  if (!(validValues as readonly string[]).includes(stringValue)) {
    diagnostics.push(
      configError(
        "invalid-host-app-contribution",
        field,
        `Studio host app ${label} ${field} is not supported.`,
        `Expected one of: ${validValues.join(", ")}.`,
      ),
    );
    return undefined;
  }
  return stringValue as Value;
};

const validateHostAppWorkflowCommand = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppWorkflowCommandName | undefined =>
  readStringLiteral(value, field, diagnostics, validHostAppWorkflowCommands, "workflow command");

const validateHostAppActionVariant = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppActionVariant | undefined =>
  readStringLiteral(
    value,
    field,
    diagnostics,
    validHostAppActionVariants,
    "workflow action variant",
  );

const readHostAppContributionArray = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): unknown[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-host-app-contribution",
        field,
        `Studio host app contribution field ${field} must be an array.`,
      ),
    );
    return [];
  }

  return value;
};

const validateHostAppItems = <Value>(
  value: unknown,
  field: string,
  itemLabel: string,
  diagnostics: StudioDiagnostic[],
  readItem: (
    item: Record<string, unknown>,
    itemField: string,
    diagnostics: StudioDiagnostic[],
  ) => Value | undefined,
): Value[] =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    const itemField = `${field}.${index}`;
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          itemField,
          `Studio host app ${itemLabel} ${itemField} must be an object.`,
        ),
      );
      return [];
    }

    const parsed = readItem(item, itemField, diagnostics);
    return parsed ? [parsed] : [];
  });

const readIdLabel = (
  item: Record<string, unknown>,
  field: string,
  diagnostics: StudioDiagnostic[],
) => {
  const id = readString(item.id, `${field}.id`, diagnostics);
  const label = readString(item.label, `${field}.label`, diagnostics);
  return id && label ? { id, label } : undefined;
};

const validateHostAppNavigationLinks = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppNavigationLink[] =>
  validateHostAppItems(value, field, "navigation link", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    const href = readString(item.href, `${itemField}.href`, diagnostics);
    if (!base || !href) {
      return;
    }
    return {
      href,
      icon: readOptionalString(item.icon, `${itemField}.icon`, diagnostics),
      ...base,
    };
  });

const validateHostAppNavigation = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppNavigationSection[] =>
  validateHostAppItems(value, field, "navigation section", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    const links = validateHostAppNavigationLinks(item.links, `${itemField}.links`, diagnostics);
    return base ? { ...base, links } : undefined;
  });

const validateHostAppAuthoringAreas = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppAuthoringAreaDefinition[] =>
  validateHostAppItems(value, field, "authoring area", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    if (!base) {
      return;
    }
    return {
      description: readOptionalString(item.description, `${itemField}.description`, diagnostics),
      editorId: readOptionalString(item.editorId, `${itemField}.editorId`, diagnostics),
      icon: readOptionalString(item.icon, `${itemField}.icon`, diagnostics),
      ...base,
    };
  });

const validateHostAppAuthoringEditors = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppAuthoringEditorDefinition[] =>
  validateHostAppItems(value, field, "authoring editor", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    const areaId = readString(item.areaId, `${itemField}.areaId`, diagnostics);
    const commandName =
      item.commandName === undefined
        ? undefined
        : validateHostAppWorkflowCommand(item.commandName, `${itemField}.commandName`, diagnostics);
    if (!base || !areaId) {
      return;
    }
    return {
      areaId,
      commandName,
      description: readOptionalString(item.description, `${itemField}.description`, diagnostics),
      recordKind: readOptionalString(item.recordKind, `${itemField}.recordKind`, diagnostics),
      ...base,
    };
  });

const validateHostAppWorkflowActions = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppWorkflowActionDefinition[] =>
  validateHostAppItems(value, field, "workflow action", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    const commandName = validateHostAppWorkflowCommand(
      item.commandName,
      `${itemField}.commandName`,
      diagnostics,
    );
    const variant = validateHostAppActionVariant(item.variant, `${itemField}.variant`, diagnostics);
    return base && commandName && variant ? { commandName, variant, ...base } : undefined;
  });

const validateHostAppCodegenTargets = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppCodegenTargetDefinition[] =>
  validateHostAppItems(value, field, "generated target panel", diagnostics, (item, itemField) => {
    const target = readString(item.target, `${itemField}.target`, diagnostics);
    const label = readString(item.label, `${itemField}.label`, diagnostics);
    if (!target || !label) {
      return;
    }
    return {
      description: readOptionalString(item.description, `${itemField}.description`, diagnostics),
      label,
      outputLabel: readOptionalString(item.outputLabel, `${itemField}.outputLabel`, diagnostics),
      target,
    };
  });

const validateHostAppGeneratedOutputPanels = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppGeneratedOutputPanelDefinition[] =>
  validateHostAppItems(value, field, "generated output panel", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    if (!base) {
      return;
    }
    return {
      description: readOptionalString(item.description, `${itemField}.description`, diagnostics),
      target: readOptionalString(item.target, `${itemField}.target`, diagnostics),
      ...base,
    };
  });

const validateHostAppDiagnosticsPanels = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppDiagnosticsPanelDefinition[] =>
  validateHostAppItems(value, field, "diagnostics panel", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    const commandName =
      item.commandName === undefined
        ? undefined
        : validateHostAppWorkflowCommand(item.commandName, `${itemField}.commandName`, diagnostics);
    if (!base) {
      return;
    }
    return {
      commandName,
      description: readOptionalString(item.description, `${itemField}.description`, diagnostics),
      ...base,
    };
  });

const validateHostAppSourceViews = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppSourceViewDefinition[] =>
  validateHostAppItems(value, field, "source view", diagnostics, (item, itemField) => {
    const base = readIdLabel(item, itemField, diagnostics);
    if (!base) {
      return;
    }
    return {
      adapterId: readOptionalString(item.adapterId, `${itemField}.adapterId`, diagnostics),
      description: readOptionalString(item.description, `${itemField}.description`, diagnostics),
      sourceId: readOptionalString(item.sourceId, `${itemField}.sourceId`, diagnostics),
      ...base,
    };
  });

export const validateHostAppContributions = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioHostAppContribution[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio extension field ${field} must be an array of host app contributions.`,
      ),
    );
    return [];
  }

  return value.flatMap((item, index) => {
    const contributionField = `${field}.${index}`;
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          contributionField,
          `Studio host app contribution ${contributionField} must be an object.`,
        ),
      );
      return [];
    }

    const id = readString(item.id, `${contributionField}.id`, diagnostics);
    if (!id) {
      return [];
    }

    let authoring: StudioHostAppContribution["authoring"];
    if (item.authoring !== undefined) {
      if (isObject(item.authoring)) {
        authoring = {
          areas: validateHostAppAuthoringAreas(
            item.authoring.areas,
            `${contributionField}.authoring.areas`,
            diagnostics,
          ),
          editors: validateHostAppAuthoringEditors(
            item.authoring.editors,
            `${contributionField}.authoring.editors`,
            diagnostics,
          ),
        };
      } else {
        diagnostics.push(
          configError(
            "invalid-host-app-contribution",
            `${contributionField}.authoring`,
            `Studio host app contribution field ${contributionField}.authoring must be an object.`,
          ),
        );
      }
    }

    return [
      {
        authoring,
        codegenTargets: validateHostAppCodegenTargets(
          item.codegenTargets,
          `${contributionField}.codegenTargets`,
          diagnostics,
        ),
        diagnosticsPanels: validateHostAppDiagnosticsPanels(
          item.diagnosticsPanels,
          `${contributionField}.diagnosticsPanels`,
          diagnostics,
        ),
        generatedOutputPanels: validateHostAppGeneratedOutputPanels(
          item.generatedOutputPanels,
          `${contributionField}.generatedOutputPanels`,
          diagnostics,
        ),
        id,
        label: readOptionalString(item.label, `${contributionField}.label`, diagnostics),
        navigation: validateHostAppNavigation(
          item.navigation,
          `${contributionField}.navigation`,
          diagnostics,
        ),
        sourceViews: validateHostAppSourceViews(
          item.sourceViews,
          `${contributionField}.sourceViews`,
          diagnostics,
        ),
        workflowActions: validateHostAppWorkflowActions(
          item.workflowActions,
          `${contributionField}.workflowActions`,
          diagnostics,
        ),
      },
    ];
  });
};
