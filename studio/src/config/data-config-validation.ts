import type {
  StudioDataAdapter,
  StudioDataAdapterCapability,
  StudioSourceConfig,
} from "../extensions";
import { configError } from "./diagnostics";
import { isObject, readOptionalString, readString } from "./primitive-readers";
import type { StudioDiagnostic, StudioProjectConfig } from "./types";

const validAdapterCapabilities: readonly StudioDataAdapterCapability[] = [
  "diff",
  "read",
  "schema",
  "watch",
  "write",
];

const validateCapabilities = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioDataAdapterCapability[] => {
  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        field,
        `Studio data adapter field ${field} must be an array of capabilities.`,
        `Expected one or more of: ${validAdapterCapabilities.join(", ")}.`,
      ),
    );
    return [];
  }

  const capabilities: StudioDataAdapterCapability[] = [];
  for (const [index, item] of value.entries()) {
    const itemField = `${field}.${index}`;
    if (
      typeof item !== "string" ||
      !(validAdapterCapabilities as readonly string[]).includes(item)
    ) {
      diagnostics.push(
        configError(
          "invalid-data-adapter",
          itemField,
          `Studio data adapter capability ${itemField} is not supported.`,
          `Expected one of: ${validAdapterCapabilities.join(", ")}.`,
        ),
      );
      continue;
    }
    if (!capabilities.includes(item as StudioDataAdapterCapability)) {
      capabilities.push(item as StudioDataAdapterCapability);
    }
  }

  if (capabilities.length === 0) {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        field,
        `Studio data adapter field ${field} must include at least one capability.`,
      ),
    );
  }

  return capabilities;
};

const validateDataAdapter = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioDataAdapter | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        field,
        `Studio data adapter ${field} must be an object returned by defineStudioDataAdapter.`,
      ),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const label =
    value.label === undefined ? undefined : readString(value.label, `${field}.label`, diagnostics);
  const capabilities = validateCapabilities(
    value.capabilities,
    `${field}.capabilities`,
    diagnostics,
  );

  if (typeof value.load !== "function") {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        `${field}.load`,
        `Studio data adapter ${field} must provide a load function.`,
      ),
    );
  }

  if (value.write !== undefined && typeof value.write !== "function") {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        `${field}.write`,
        `Studio data adapter ${field}.write must be a function when provided.`,
      ),
    );
  }

  if (value.writeSnapshotPaths !== undefined && typeof value.writeSnapshotPaths !== "function") {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        `${field}.writeSnapshotPaths`,
        `Studio data adapter ${field}.writeSnapshotPaths must be a function when provided.`,
      ),
    );
  }

  if (capabilities.includes("write") && typeof value.write !== "function") {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        `${field}.write`,
        `Writable Studio data adapter ${field} must provide a write function.`,
      ),
    );
  }

  if (capabilities.includes("write") && typeof value.writeSnapshotPaths !== "function") {
    diagnostics.push(
      configError(
        "invalid-data-adapter",
        `${field}.writeSnapshotPaths`,
        `Writable Studio data adapter ${field} must provide writeSnapshotPaths for transactional scaffold rollback.`,
      ),
    );
  }

  if (!id || typeof value.load !== "function" || capabilities.length === 0) {
    return undefined;
  }

  return {
    ...(value as unknown as StudioDataAdapter),
    capabilities,
    id,
    label,
  };
};

export const validateDataAdapters = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioDataAdapter[] => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        field,
        `Studio project config field ${field} must be an array of data adapters.`,
      ),
    );
    return [];
  }

  const adapters: StudioDataAdapter[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    const adapter = validateDataAdapter(item, `${field}.${index}`, diagnostics);
    if (!adapter) {
      continue;
    }
    if (seen.has(adapter.id)) {
      diagnostics.push(
        configError(
          "duplicate-data-adapter",
          `${field}.${index}.id`,
          `Studio data adapter "${adapter.id}" is registered more than once.`,
        ),
      );
      continue;
    }
    seen.add(adapter.id);
    adapters.push(adapter);
  }

  return adapters;
};

const validateSourceConfig = (
  value: unknown,
  field: string,
  diagnostics: StudioDiagnostic[],
): StudioSourceConfig | undefined => {
  if (!isObject(value)) {
    diagnostics.push(
      configError("invalid-source-config", field, `Studio source ${field} must be an object.`),
    );
    return undefined;
  }

  const id = readString(value.id, `${field}.id`, diagnostics);
  const adapterId = readString(value.adapterId, `${field}.adapterId`, diagnostics);
  const label =
    value.label === undefined ? undefined : readString(value.label, `${field}.label`, diagnostics);
  if (value.options !== undefined && !isObject(value.options)) {
    diagnostics.push(
      configError(
        "invalid-source-config",
        `${field}.options`,
        `Studio source field ${field}.options must be an object when provided.`,
      ),
    );
  }

  if (!id || !adapterId) {
    return undefined;
  }

  return {
    adapterId,
    id,
    label,
    options: isObject(value.options) ? value.options : undefined,
  };
};

export const validateDataConfig = (
  value: StudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): { adapters: StudioDataAdapter[]; sources: StudioSourceConfig[]; writeSourceId?: string } => {
  if (value.data === undefined) {
    return {
      adapters: [],
      sources: [],
      writeSourceId: undefined,
    };
  }

  if (!isObject(value.data)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        "data",
        "Studio project config field data must be an object when provided.",
      ),
    );
    return {
      adapters: [],
      sources: [],
      writeSourceId: undefined,
    };
  }

  const adapters = validateDataAdapters(value.data.adapters, "data.adapters", diagnostics);
  const writeSourceId = readOptionalString(
    value.data.writeSourceId,
    "data.writeSourceId",
    diagnostics,
  );
  const sourcesValue = value.data.sources;
  const sources: StudioSourceConfig[] = [];
  if (sourcesValue !== undefined && !Array.isArray(sourcesValue)) {
    diagnostics.push(
      configError(
        "invalid-config-field",
        "data.sources",
        "Studio project config field data.sources must be an array of source declarations.",
      ),
    );
  } else {
    for (const [index, item] of (sourcesValue ?? []).entries()) {
      const source = validateSourceConfig(item, `data.sources.${index}`, diagnostics);
      if (source) {
        sources.push(source);
      }
    }
  }

  return { adapters, sources, writeSourceId };
};

export const validateStudioWriteSourceReference = (
  writeSourceId: string | undefined,
  sources: readonly StudioSourceConfig[],
  diagnostics: StudioDiagnostic[],
) => {
  if (writeSourceId === undefined) {
    return;
  }

  if (!sources.some((source) => source.id === writeSourceId)) {
    diagnostics.push(
      configError(
        "missing-write-source",
        "data.writeSourceId",
        `Studio catalog write source "${writeSourceId}" is not declared in data.sources.`,
        "Omit data.writeSourceId to use the built-in JSON catalog writer, or set it to a declared source id.",
      ),
    );
  }
};
