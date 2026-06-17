import type {
  StudioContentMapper,
  StudioExtension,
  StudioExtensionMigration,
  StudioRustBindingConfigValidator,
} from "../extensions";
import { validateDataAdapters } from "./data-config-validation";
import { configError } from "./diagnostics";
import { validateGeneratedTargets } from "./generated-target-validation";
import {
  mergeHostAppContributionModels,
  normalizeHostAppContributions,
  validateHostAppContributionModel,
  validateHostAppContributions,
} from "./host-app-contribution-validation";
import type { StudioHostAppContributionModel } from "./host-app-contribution-validation";
import {
  isObject,
  readNonNegativeInteger,
  readOptionalString,
  readString,
} from "./primitive-readers";
import type { StudioDiagnostic } from "./types";

const validateContentMapper = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioContentMapper | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-content-mapper",
        field,
        `Studio content mapper ${field} must be an object.`,
      ),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const label =
    value.label === undefined ? undefined : readString(value.label, `${field}.label`, diagnostics);

  if (typeof value.map !== "function") {
    diagnostics.push(
      configError(
        "invalid-content-mapper",
        `${field}.map`,
        `Studio content mapper ${field} must provide a map function.`,
      ),
    );
  }

  if (!id || typeof value.map !== "function") {
    return undefined;
  }

  return {
    ...(value as unknown as StudioContentMapper),
    id,
    label,
  };
};

const validateContentMappers = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioContentMapper[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio extension field ${field} must be an array of content mappers.`,
      ),
    );
    return [];
  }

  const mappers: StudioContentMapper[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const mapper = validateContentMapper(item, `${field}.${index}`, diagnostics);
    if (!mapper) {
      continue;
    }
    if (seen.has(mapper.id)) {
      diagnostics.push(
        configError(
          "duplicate-content-mapper",
          `${field}.${index}.id`,
          `Studio content mapper "${mapper.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(mapper.id);
    mappers.push(mapper);
  }

  return mappers;
};

const validateExtensionMigration = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioExtensionMigration | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-extension-migration",
        field,
        `Studio extension migration ${field} must be an object.`,
      ),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const label = readOptionalString(value.label, `${field}.label`, diagnostics);
  const fromVersion = readNonNegativeInteger(
    value.fromVersion,
    `${field}.fromVersion`,
    diagnostics,
  );
  const toVersion = readNonNegativeInteger(value.toVersion, `${field}.toVersion`, diagnostics);
  if (typeof value.migrate !== "function") {
    diagnostics.push(
      configError(
        "invalid-extension-migration",
        `${field}.migrate`,
        `Studio extension migration ${field} must provide a migrate function.`,
      ),
    );
  }

  if (
    !id ||
    fromVersion === undefined ||
    toVersion === undefined ||
    typeof value.migrate !== "function"
  ) {
    return undefined;
  }

  if (fromVersion >= toVersion) {
    diagnostics.push(
      configError(
        "invalid-extension-migration",
        `${field}.toVersion`,
        `Studio extension migration ${field} must move from a lower version to a higher version.`,
      ),
    );
    return undefined;
  }

  return {
    ...(value as unknown as StudioExtensionMigration),
    fromVersion,
    id,
    label,
    toVersion,
  };
};

const validateExtensionMigrations = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioExtensionMigration[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio extension field ${field} must be an array of migrations.`,
      ),
    );
    return [];
  }

  const migrations: StudioExtensionMigration[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const migration = validateExtensionMigration(item, `${field}.${index}`, diagnostics);
    if (!migration) {
      continue;
    }
    if (seen.has(migration.id)) {
      diagnostics.push(
        configError(
          "duplicate-extension-migration",
          `${field}.${index}.id`,
          `Studio extension migration "${migration.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(migration.id);
    migrations.push(migration);
  }

  return migrations;
};

const validateRustBindingConfigValidator = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioRustBindingConfigValidator | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-rust-binding-config",
        field,
        `Studio Rust binding config ${field} must be an object.`,
      ),
    );
    return undefined;
  }

  const namespace = readString(value.namespace, `${field}.namespace`, diagnostics);
  if (typeof value.validate !== "function") {
    diagnostics.push(
      configError(
        "invalid-rust-binding-config",
        `${field}.validate`,
        `Studio Rust binding config ${field} must provide a validate function.`,
      ),
    );
  }

  if (!namespace || typeof value.validate !== "function") {
    return undefined;
  }

  return {
    ...(value as unknown as StudioRustBindingConfigValidator),
    namespace,
  };
};

const validateRustBindingConfigValidators = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioRustBindingConfigValidator[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio extension field ${field} must be an array of Rust binding config validators.`,
      ),
    );
    return [];
  }

  const validators: StudioRustBindingConfigValidator[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const validator = validateRustBindingConfigValidator(item, `${field}.${index}`, diagnostics);
    if (!validator) {
      continue;
    }
    if (seen.has(validator.namespace)) {
      diagnostics.push(
        configError(
          "duplicate-rust-binding-config",
          `${field}.${index}.namespace`,
          `Studio Rust binding config namespace "${validator.namespace}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(validator.namespace);
    validators.push(validator);
  }

  return validators;
};

const validateStudioExtension = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioExtension | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError(
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
  const contentMappers = validateContentMappers(
    value.contentMappers,
    `${field}.contentMappers`,
    diagnostics,
  );
  const generatedTargets = validateGeneratedTargets(
    value.generatedTargets,
    `${field}.generatedTargets`,
    diagnostics,
  );
  const appContributions = validateHostAppContributions(
    value.appContributions,
    `${field}.appContributions`,
    diagnostics,
  );
  const migrations = validateExtensionMigrations(
    value.migrations,
    `${field}.migrations`,
    diagnostics,
  );
  const rustBindingConfigs = validateRustBindingConfigValidators(
    value.rustBindingConfigs,
    `${field}.rustBindingConfigs`,
    diagnostics,
  );

  if (value.validateSources !== undefined && typeof value.validateSources !== "function") {
    diagnostics.push(
      configError(
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
    appContributions,
    contentMappers,
    dataAdapters,
    generatedTargets,
    id,
    label,
    migrations,
    rustBindingConfigs,
  };
};

export const validateStudioExtensions = (
  value: unknown,
  diagnostics: StudioDiagnostic[],
): StudioExtension[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        "extensions",
        "Studio project config field extensions must be an array of Studio extensions.",
      ),
    );
    return [];
  }

  const extensions: StudioExtension[] = [];
  const hostAppContributionModels: StudioHostAppContributionModel[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const field = `extensions.${index}`;
    const extension = validateStudioExtension(item, field, diagnostics);
    if (!extension) {
      continue;
    }
    if (seen.has(extension.id)) {
      diagnostics.push(
        configError(
          "duplicate-studio-extension",
          `extensions.${index}.id`,
          `Studio extension "${extension.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(extension.id);
    extensions.push(extension);
    hostAppContributionModels.push(
      normalizeHostAppContributions(extension.appContributions ?? [], `${field}.appContributions`),
    );
  }
  diagnostics.push(
    ...validateHostAppContributionModel(mergeHostAppContributionModels(hostAppContributionModels)),
  );

  return extensions;
};
