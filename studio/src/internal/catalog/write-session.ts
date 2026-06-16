import { existsSync, rmSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";

import { resolveStudioDataAdapter } from "../../config/data-adapter-registry";
import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../../config/schema";
import { studioDataAdapterCanWrite, studioSourceLocationLabel } from "../../extensions";
import type {
  StudioDataAdapter,
  StudioSourceConfig,
  StudioSourceRecord,
  StudioSourceSnapshot,
} from "../../extensions";
import { restoreSnapshots, snapshotPaths } from "../files";
import { catalogDiagnostic } from "./diagnostics";
import { builtInJsonCatalogAdapter } from "./json-source";
import { kindForCatalogRecord } from "./kinds";
import type { StudioCatalogRecord } from "./types";

export interface StudioCatalogWritePlanOptions {
  allowExisting?: boolean;
}

export interface StudioCatalogWritePlan {
  diagnostics: StudioDiagnostic[];
  plannedPaths: string[];
}

export interface StudioCatalogWriteResult {
  diagnostics: StudioDiagnostic[];
  writtenPaths: string[];
}

export interface StudioCatalogRollbackResult {
  diagnostics: StudioDiagnostic[];
  rolledBack: boolean;
}

export interface PreparedStudioCatalogWrite extends StudioCatalogWritePlan {
  rollback: () => StudioCatalogRollbackResult;
  write: () => Promise<StudioCatalogWriteResult>;
}

interface StudioCatalogWriterAdapter {
  plan: (
    records: readonly StudioCatalogRecord[],
    options?: StudioCatalogWritePlanOptions,
  ) => StudioCatalogWritePlan;
  prepare: (
    records: readonly StudioCatalogRecord[],
    options?: StudioCatalogWritePlanOptions,
  ) => PreparedStudioCatalogWrite;
}

const catalogWriteRecord = (record: StudioCatalogRecord): StudioSourceRecord => ({
  id: record.id,
  kind: kindForCatalogRecord(record),
  value: record,
});

const plannedJsonWritePaths = (
  config: ResolvedStudioProjectConfig,
  records: readonly StudioCatalogRecord[],
) =>
  records.map((record) =>
    resolve(config.paths.catalogRoot, kindForCatalogRecord(record), `${record.id}.json`),
  );

const displayCatalogWritePath = (config: ResolvedStudioProjectConfig, path: string) =>
  relative(config.configDir, path) || ".";

const plannedJsonWriteDiagnostics = (
  config: ResolvedStudioProjectConfig,
  paths: readonly string[],
  options?: StudioCatalogWritePlanOptions,
) =>
  options?.allowExisting === true
    ? []
    : paths
        .filter((path) => existsSync(path))
        .map((path) =>
          catalogDiagnostic(
            "planned-file-exists",
            `Planned Studio catalog file already exists: ${displayCatalogWritePath(config, path)}`,
            displayCatalogWritePath(config, path),
          ),
        );

const builtInJsonCatalogWriter = (
  config: ResolvedStudioProjectConfig,
): StudioCatalogWriterAdapter => {
  const plan = (
    records: readonly StudioCatalogRecord[],
    options?: StudioCatalogWritePlanOptions,
  ) => {
    const paths = plannedJsonWritePaths(config, records);
    return {
      diagnostics: plannedJsonWriteDiagnostics(config, paths, options),
      plannedPaths: paths.map((path) => displayCatalogWritePath(config, path)),
    };
  };

  return {
    plan,
    prepare: (records, options) => {
      const paths = plannedJsonWritePaths(config, records);
      const snapshots = snapshotPaths(paths);
      return {
        ...plan(records, options),
        rollback: () => {
          restoreSnapshots(snapshots);
          return { diagnostics: [], rolledBack: true };
        },
        write: async () => {
          try {
            const snapshot = (await builtInJsonCatalogAdapter.write?.({
              config,
              records: records.map(catalogWriteRecord),
              source: {
                adapterId: builtInJsonCatalogAdapter.id,
                id: "studio-json-catalog",
              },
            })) as StudioSourceSnapshot | undefined;

            return {
              diagnostics: [...(snapshot?.diagnostics ?? [])],
              writtenPaths:
                snapshot?.records.map((record, index) => {
                  const fallbackPath = paths[index] ?? paths[0] ?? config.paths.catalogRoot;
                  return record.location?.path
                    ? displayCatalogWritePath(
                        config,
                        resolve(config.paths.catalogRoot, record.location.path),
                      )
                    : displayCatalogWritePath(config, fallbackPath);
                }) ?? paths.map((path) => displayCatalogWritePath(config, path)),
            };
          } catch (error) {
            return {
              diagnostics: [
                catalogDiagnostic(
                  "catalog-write-failed",
                  error instanceof Error
                    ? `Failed to write Studio JSON catalog records: ${error.message}`
                    : "Failed to write Studio JSON catalog records.",
                  config.configPath,
                ),
              ],
              writtenPaths: [],
            };
          }
        },
      };
    },
  };
};

const sourceCanWrite = (
  adapter: StudioDataAdapter | undefined,
): adapter is StudioDataAdapter & { write: NonNullable<StudioDataAdapter["write"]> } =>
  adapter !== undefined && studioDataAdapterCanWrite(adapter);

const sourceWriteLabel = (source: StudioSourceConfig, record: StudioCatalogRecord) => {
  const label = source.label ?? source.id;
  return `${label}:${record.kind}:${record.id}`;
};

const sourceRecordWritePath = (
  source: StudioSourceConfig,
  record: StudioSourceRecord,
  fallback: StudioCatalogRecord | undefined,
) =>
  studioSourceLocationLabel(record.location) ??
  (fallback
    ? sourceWriteLabel(source, fallback)
    : `${source.label ?? source.id}:${record.kind}:${record.id}`);

const sourceBackedCatalogWriter = (
  config: ResolvedStudioProjectConfig,
  source: StudioSourceConfig,
  adapter: StudioDataAdapter & { write: NonNullable<StudioDataAdapter["write"]> },
): StudioCatalogWriterAdapter => {
  const plan = (records: readonly StudioCatalogRecord[]) => ({
    diagnostics: [],
    plannedPaths: records.map((record) => sourceWriteLabel(source, record)),
  });

  return {
    plan,
    prepare: (records) => {
      const writeRecords = records.map(catalogWriteRecord);
      const sourceWriteContext = {
        config,
        records: writeRecords,
        source,
      };
      const snapshotDiagnostics: StudioDiagnostic[] = [];
      const snapshots = (() => {
        try {
          const paths = adapter.writeSnapshotPaths?.(sourceWriteContext) ?? [];
          return snapshotPaths(
            paths.map((path) => (isAbsolute(path) ? path : resolve(config.configDir, path))),
          );
        } catch (error) {
          snapshotDiagnostics.push(
            catalogDiagnostic(
              "source-write-snapshot-failed",
              error instanceof Error
                ? `Studio data adapter "${adapter.id}" could not prepare write snapshots for source "${source.id}": ${error.message}`
                : `Studio data adapter "${adapter.id}" could not prepare write snapshots for source "${source.id}".`,
              config.configPath,
              undefined,
              undefined,
              "warning",
            ),
          );
          return [];
        }
      })();
      let writeAttempted = false;
      return {
        ...plan(records),
        diagnostics: snapshotDiagnostics,
        rollback: () => {
          if (!writeAttempted) {
            return { diagnostics: [], rolledBack: true };
          }

          if (snapshots.length > 0) {
            restoreSnapshots(snapshots);
            return { diagnostics: [], rolledBack: true };
          }

          return {
            diagnostics: [
              catalogDiagnostic(
                "source-write-rollback-unsupported",
                `Studio source adapter "${adapter.id}" did not provide filesystem snapshots; source writes could not be rolled back automatically.`,
                config.configPath,
                undefined,
                undefined,
                "warning",
              ),
            ],
            rolledBack: false,
          };
        },
        write: async () => {
          writeAttempted = true;
          try {
            const snapshot = await adapter.write({
              ...sourceWriteContext,
            });

            return {
              diagnostics: [...(snapshot.diagnostics ?? [])],
              writtenPaths: snapshot.records.map((record, index) =>
                sourceRecordWritePath(source, record, records[index]),
              ),
            };
          } catch (error) {
            return {
              diagnostics: [
                catalogDiagnostic(
                  "source-write-failed",
                  error instanceof Error
                    ? `Studio data adapter "${adapter.id}" failed to write source "${source.id}": ${error.message}`
                    : `Studio data adapter "${adapter.id}" failed to write source "${source.id}".`,
                  config.configPath,
                ),
              ],
              writtenPaths: [],
            };
          }
        },
      };
    },
  };
};

const unsupportedCatalogWriter = (
  config: ResolvedStudioProjectConfig,
  diagnosticCode: "source-write-ambiguous" | "source-write-unsupported",
  message: string,
): StudioCatalogWriterAdapter => {
  const writeDiagnostic = () => catalogDiagnostic(diagnosticCode, message, config.configPath);
  const plan = (records: readonly StudioCatalogRecord[]) => ({
    diagnostics: [writeDiagnostic()],
    plannedPaths: records.map((record) =>
      sourceWriteLabel(
        config.data.sources[0] ?? {
          adapterId: "unknown",
          id: "source",
        },
        record,
      ),
    ),
  });

  return {
    plan,
    prepare: (records) => ({
      ...plan(records),
      rollback: () => ({ diagnostics: [], rolledBack: true }),
      write: () => Promise.resolve({ diagnostics: [writeDiagnostic()], writtenPaths: [] }),
    }),
  };
};

const resolveStudioCatalogWriter = (
  config: ResolvedStudioProjectConfig,
): StudioCatalogWriterAdapter => {
  if (config.data.sources.length === 0) {
    return builtInJsonCatalogWriter(config);
  }

  const writableSources = config.data.sources
    .map((source) => ({
      adapter: resolveStudioDataAdapter(config.data.adapterRegistry, source.adapterId),
      source,
    }))
    .filter(
      (
        entry,
      ): entry is {
        adapter: StudioDataAdapter & { write: NonNullable<StudioDataAdapter["write"]> };
        source: StudioSourceConfig;
      } => sourceCanWrite(entry.adapter),
    );

  if (writableSources.length === 1) {
    const [{ adapter, source }] = writableSources;
    return sourceBackedCatalogWriter(config, source, adapter);
  }

  if (writableSources.length > 1) {
    return unsupportedCatalogWriter(
      config,
      "source-write-ambiguous",
      "Studio scaffold writes require exactly one writable content adapter for the active source configuration.",
    );
  }

  return unsupportedCatalogWriter(
    config,
    "source-write-unsupported",
    "Studio scaffold writes require a writable content adapter for the active source configuration.",
  );
};

export const planStudioCatalogWrite = (
  config: ResolvedStudioProjectConfig,
  records: readonly StudioCatalogRecord[],
  options?: StudioCatalogWritePlanOptions,
) => resolveStudioCatalogWriter(config).plan(records, options);

export const prepareStudioCatalogWrite = (
  config: ResolvedStudioProjectConfig,
  records: readonly StudioCatalogRecord[],
  options?: StudioCatalogWritePlanOptions,
) => resolveStudioCatalogWriter(config).prepare(records, options);

export const removePathIfExists = (path: string) => {
  if (existsSync(path)) {
    rmSync(path, { force: true, recursive: true });
  }
};
