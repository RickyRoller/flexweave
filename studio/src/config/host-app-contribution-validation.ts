import type { StudioHostAppContribution } from "../extensions";
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
] as const;

const validHostAppActionVariants = ["primary", "secondary"] as const;

const validateHostAppWorkflowCommand = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): (typeof validHostAppWorkflowCommands)[number] | undefined => {
  const command = readString(value, field, diagnostics);
  if (!command) {
    return undefined;
  }
  if (!(validHostAppWorkflowCommands as readonly string[]).includes(command)) {
    diagnostics.push(
      configError(
        "invalid-host-app-contribution",
        field,
        `Studio host app workflow command ${field} is not supported.`,
        `Expected one of: ${validHostAppWorkflowCommands.join(", ")}.`,
      ),
    );
    return undefined;
  }
  return command as (typeof validHostAppWorkflowCommands)[number];
};

const validateHostAppActionVariant = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): (typeof validHostAppActionVariants)[number] | undefined => {
  const variant = readString(value, field, diagnostics);
  if (!variant) {
    return undefined;
  }
  if (!(validHostAppActionVariants as readonly string[]).includes(variant)) {
    diagnostics.push(
      configError(
        "invalid-host-app-contribution",
        field,
        `Studio host app workflow action variant ${field} is not supported.`,
        `Expected one of: ${validHostAppActionVariants.join(", ")}.`,
      ),
    );
    return undefined;
  }
  return variant as (typeof validHostAppActionVariants)[number];
};

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

const validateHostAppNavigationLinks = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app navigation link ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    const href = readString(item.href, `${field}.${index}.href`, diagnostics);
    if (!id || !label || !href) {
      return [];
    }
    return [
      {
        href,
        icon: readOptionalString(item.icon, `${field}.${index}.icon`, diagnostics),
        id,
        label,
      },
    ];
  });

const validateHostAppNavigation = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app navigation section ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    const links = validateHostAppNavigationLinks(
      item.links,
      `${field}.${index}.links`,
      diagnostics,
    );
    if (!id || !label) {
      return [];
    }
    return [{ id, label, links }];
  });

const validateHostAppAuthoringAreas = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app authoring area ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    if (!id || !label) {
      return [];
    }
    return [
      {
        description: readOptionalString(
          item.description,
          `${field}.${index}.description`,
          diagnostics,
        ),
        editorId: readOptionalString(item.editorId, `${field}.${index}.editorId`, diagnostics),
        icon: readOptionalString(item.icon, `${field}.${index}.icon`, diagnostics),
        id,
        label,
      },
    ];
  });

const validateHostAppAuthoringEditors = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app authoring editor ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    const areaId = readString(item.areaId, `${field}.${index}.areaId`, diagnostics);
    const commandName =
      item.commandName === undefined
        ? undefined
        : validateHostAppWorkflowCommand(
            item.commandName,
            `${field}.${index}.commandName`,
            diagnostics,
          );
    if (!id || !label || !areaId) {
      return [];
    }
    return [
      {
        areaId,
        commandName,
        description: readOptionalString(
          item.description,
          `${field}.${index}.description`,
          diagnostics,
        ),
        id,
        label,
        recordKind: readOptionalString(
          item.recordKind,
          `${field}.${index}.recordKind`,
          diagnostics,
        ),
      },
    ];
  });

const validateHostAppWorkflowActions = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app workflow action ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    const commandName = validateHostAppWorkflowCommand(
      item.commandName,
      `${field}.${index}.commandName`,
      diagnostics,
    );
    const variant = validateHostAppActionVariant(
      item.variant,
      `${field}.${index}.variant`,
      diagnostics,
    );
    if (!id || !label || !commandName || !variant) {
      return [];
    }
    return [{ commandName, id, label, variant }];
  });

const validateHostAppCodegenTargets = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app generated target panel ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const target = readString(item.target, `${field}.${index}.target`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    if (!target || !label) {
      return [];
    }
    return [
      {
        description: readOptionalString(
          item.description,
          `${field}.${index}.description`,
          diagnostics,
        ),
        label,
        outputLabel: readOptionalString(
          item.outputLabel,
          `${field}.${index}.outputLabel`,
          diagnostics,
        ),
        target,
      },
    ];
  });

const validateHostAppGeneratedOutputPanels = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app generated output panel ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    if (!id || !label) {
      return [];
    }
    return [
      {
        description: readOptionalString(
          item.description,
          `${field}.${index}.description`,
          diagnostics,
        ),
        id,
        label,
        target: readOptionalString(item.target, `${field}.${index}.target`, diagnostics),
      },
    ];
  });

const validateHostAppDiagnosticsPanels = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app diagnostics panel ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    const commandName =
      item.commandName === undefined
        ? undefined
        : validateHostAppWorkflowCommand(
            item.commandName,
            `${field}.${index}.commandName`,
            diagnostics,
          );
    if (!id || !label) {
      return [];
    }
    return [
      {
        commandName,
        description: readOptionalString(
          item.description,
          `${field}.${index}.description`,
          diagnostics,
        ),
        id,
        label,
      },
    ];
  });

const validateHostAppSourceViews = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
) =>
  readHostAppContributionArray(value, field, diagnostics).flatMap((item, index) => {
    if (!isObject(item)) {
      diagnostics.push(
        configError(
          "invalid-host-app-contribution",
          `${field}.${index}`,
          `Studio host app source view ${field}.${index} must be an object.`,
        ),
      );
      return [];
    }
    const id = readString(item.id, `${field}.${index}.id`, diagnostics);
    const label = readString(item.label, `${field}.${index}.label`, diagnostics);
    if (!id || !label) {
      return [];
    }
    return [
      {
        adapterId: readOptionalString(item.adapterId, `${field}.${index}.adapterId`, diagnostics),
        description: readOptionalString(
          item.description,
          `${field}.${index}.description`,
          diagnostics,
        ),
        id,
        label,
        sourceId: readOptionalString(item.sourceId, `${field}.${index}.sourceId`, diagnostics),
      },
    ];
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
