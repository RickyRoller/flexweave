import type { StudioDataAdapter, StudioExtension, StudioSourceConfig } from "../extensions";
import { configError } from "./diagnostics";
import type { StudioDiagnostic } from "./types";

export interface StudioDataAdapterRegistry {
  readonly adapters: readonly StudioDataAdapter[];
  readonly byId: Readonly<Record<string, StudioDataAdapter | undefined>>;
}

interface StudioDataAdapterRegistryEntry {
  adapter: StudioDataAdapter;
  field: string;
}

const projectAdapterEntries = (
  adapters: readonly StudioDataAdapter[],
): StudioDataAdapterRegistryEntry[] =>
  adapters.map((adapter, index) => ({
    adapter,
    field: `data.adapters.${index}.id`,
  }));

const extensionAdapterEntries = (
  extensions: readonly StudioExtension[],
): StudioDataAdapterRegistryEntry[] =>
  extensions.flatMap((extension, extensionIndex) =>
    (extension.dataAdapters ?? []).map((adapter, adapterIndex) => ({
      adapter,
      field: `extensions.${extensionIndex}.dataAdapters.${adapterIndex}.id`,
    })),
  );

export const createStudioDataAdapterRegistry = (
  projectAdapters: readonly StudioDataAdapter[],
  extensions: readonly StudioExtension[],
  diagnostics: StudioDiagnostic[],
): StudioDataAdapterRegistry => {
  const adapters: StudioDataAdapter[] = [];
  const byId = Object.create(null) as Record<string, StudioDataAdapter | undefined>;

  for (const { adapter, field } of [
    ...projectAdapterEntries(projectAdapters),
    ...extensionAdapterEntries(extensions),
  ]) {
    if (byId[adapter.id]) {
      diagnostics.push(
        configError(
          "duplicate-data-adapter",
          field,
          `Studio data adapter "${adapter.id}" is registered more than once across project data.adapters and active Studio extensions.`,
          "Use a globally unique data adapter id for each project or extension Adapter.",
        ),
      );
      continue;
    }

    adapters.push(adapter);
    byId[adapter.id] = adapter;
  }

  return {
    adapters,
    byId,
  };
};

export const resolveStudioDataAdapter = (
  registry: StudioDataAdapterRegistry,
  adapterId: string,
): StudioDataAdapter | undefined => registry.byId[adapterId];

export const validateStudioSourceAdapterReferences = (
  sources: readonly StudioSourceConfig[],
  registry: StudioDataAdapterRegistry,
  diagnostics: StudioDiagnostic[],
) => {
  for (const [index, source] of sources.entries()) {
    if (!resolveStudioDataAdapter(registry, source.adapterId)) {
      diagnostics.push(
        configError(
          "missing-data-adapter",
          `data.sources.${index}.adapterId`,
          `Studio source "${source.id}" references missing data adapter "${source.adapterId}".`,
          "Register the adapter in data.adapters or through an active Studio extension.",
        ),
      );
    }
  }
};
