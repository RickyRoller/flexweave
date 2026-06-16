import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";

import { resolveStudioDataAdapter } from "../config/data-adapter-registry";
import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import {
  defineStudioContentMapper,
  defineStudioDataAdapter,
  loadStudioSourceSnapshots,
  studioDataAdapterCanWrite,
  studioSourceLocationLabel,
} from "../extensions";
import type {
  StudioContentMapper,
  StudioDataAdapter,
  StudioMappedContentRecord,
  StudioSourceConfig,
  StudioSourceLocation,
  StudioSourceRecord,
  StudioSourceSnapshot,
} from "../extensions";
import { restoreSnapshots, snapshotPaths } from "./files";

export const studioRecordKinds = [
  "abilities",
  "effects",
  "executions",
  "mechanics",
  "modifiers",
  "tags",
] as const;

export type StudioRecordKind = (typeof studioRecordKinds)[number];
export type StudioRecordSingular =
  | "ability"
  | "effect"
  | "execution"
  | "mechanic"
  | "modifier"
  | "tag";

export interface StudioCatalogRecord {
  description?: string;
  effectId?: string;
  executionId?: string;
  hook?: string;
  id: string;
  kind: StudioRecordSingular;
  label: string;
  modifierId?: string;
  recordIds?: string[];
  tagIds?: string[];
  value?: number;
}

export interface StudioCatalogRecordWithPath extends StudioCatalogRecord {
  path: string;
  source?: StudioSourceLocation;
}

export interface StudioCatalog {
  byKind: Record<StudioRecordKind, StudioCatalogRecordWithPath[]>;
  diagnostics: StudioDiagnostic[];
  records: StudioCatalogRecordWithPath[];
  sourceSnapshots: StudioSourceSnapshot[];
}

const singularByKind: Record<StudioRecordKind, StudioRecordSingular> = {
  abilities: "ability",
  effects: "effect",
  executions: "execution",
  mechanics: "mechanic",
  modifiers: "modifier",
  tags: "tag",
};

export const kindFromSingular = (value: string): StudioRecordKind | undefined => {
  const entry = Object.entries(singularByKind).find(([, singular]) => singular === value);
  return entry?.[0] as StudioRecordKind | undefined;
};

export const normalizeRecordKind = (value: string): StudioRecordKind | undefined => {
  if ((studioRecordKinds as readonly string[]).includes(value)) {
    return value as StudioRecordKind;
  }
  return kindFromSingular(value);
};

const diagnostic = (
  code: string,
  message: string,
  path?: string,
  field?: string,
  source?: StudioSourceLocation,
  severity: StudioDiagnostic["severity"] = "error",
): StudioDiagnostic => ({
  code,
  field,
  message,
  path,
  severity,
  source,
});

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const emptyCatalogByKind = () =>
  Object.fromEntries(studioRecordKinds.map((kind) => [kind, []])) as unknown as Record<
    StudioRecordKind,
    StudioCatalogRecordWithPath[]
  >;

const collectJsonFiles = (dir: string): string[] => {
  if (!existsSync(dir)) {
    return [];
  }

  const files: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectJsonFiles(path));
    } else if (entry.isFile() && entry.name.endsWith(".json")) {
      files.push(path);
    }
  }
  return files.toSorted();
};

export const writeJsonRecord = (
  catalogRoot: string,
  kind: StudioRecordKind,
  record: StudioCatalogRecord,
) => {
  const path = resolve(catalogRoot, kind, `${record.id}.json`);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(record, null, 2)}\n`);
  return path;
};

const builtInJsonCatalogAdapter = defineStudioDataAdapter({
  capabilities: ["read", "write"],
  id: "studio-json-catalog",
  label: "Studio JSON catalog",
  load: ({ config }) => {
    const diagnostics: StudioDiagnostic[] = [];
    const records: StudioSourceRecord[] = [];

    if (!existsSync(config.paths.catalogRoot)) {
      diagnostics.push(
        diagnostic(
          "missing-catalog-root",
          "Configured Studio catalog root does not exist.",
          config.raw.catalogRoot,
        ),
      );
      return { diagnostics, records };
    }

    for (const kind of studioRecordKinds) {
      const kindDir = join(config.paths.catalogRoot, kind);
      for (const filePath of collectJsonFiles(kindDir)) {
        const relativePath = relative(config.paths.catalogRoot, filePath);
        try {
          const value = JSON.parse(readFileSync(filePath, "utf-8"));
          records.push({
            id: isObject(value) && typeof value.id === "string" ? value.id : relativePath,
            kind,
            location: {
              jsonPointer: "/",
              path: relativePath,
            },
            value,
          });
        } catch (error) {
          diagnostics.push(
            diagnostic(
              "invalid-json",
              error instanceof Error
                ? `Could not parse Studio catalog record: ${error.message}`
                : "Could not parse Studio catalog record.",
              relativePath,
            ),
          );
        }
      }
    }

    return { diagnostics, records };
  },
  write: ({ config, records }) => {
    const written = records.map((record) => {
      if (!isObject(record.value)) {
        throw new Error(`Studio JSON catalog write expected object record ${record.id}.`);
      }

      const kind = normalizeRecordKind(
        typeof record.kind === "string" ? record.kind : String(record.value.kind),
      );
      if (!kind) {
        throw new Error(`Studio JSON catalog write received unknown record kind ${record.kind}.`);
      }

      const path = writeJsonRecord(
        config.paths.catalogRoot,
        kind,
        record.value as unknown as StudioCatalogRecord,
      );
      return {
        ...record,
        location: {
          jsonPointer: "/",
          path: relative(config.paths.catalogRoot, path),
        },
      };
    });

    return { records: written };
  },
});

const builtInJsonCatalogMapper = defineStudioContentMapper({
  id: "studio-json-catalog-mapper",
  label: "Studio JSON catalog mapper",
  map: ({ snapshots }) => ({
    records: snapshots.flatMap((snapshot) =>
      snapshot.records
        .filter((record) => (studioRecordKinds as readonly string[]).includes(record.kind))
        .map(
          (record): StudioMappedContentRecord => ({
            expectedKind: record.kind,
            location: record.location,
            path: studioSourceLocationLabel(record.location),
            sourceRecord: record,
            value: record.value,
          }),
        ),
    ),
  }),
});

const validateRecordFields = (
  value: Record<string, unknown>,
  expectedKind: StudioRecordKind,
  path: string,
  source?: StudioSourceLocation,
): StudioDiagnostic[] => {
  const diagnostics: StudioDiagnostic[] = [];

  if (value.kind !== singularByKind[expectedKind]) {
    diagnostics.push(
      diagnostic(
        "invalid-record-kind",
        `Expected a ${singularByKind[expectedKind]} record in ${expectedKind}.`,
        path,
        "kind",
        source,
      ),
    );
  }

  for (const field of ["id", "label"] as const) {
    if (typeof value[field] !== "string" || value[field].trim().length === 0) {
      diagnostics.push(
        diagnostic(
          "invalid-record-field",
          `Studio catalog record field ${field} must be a non-empty string.`,
          path,
          field,
          source,
        ),
      );
    }
  }

  if (value.description !== undefined && typeof value.description !== "string") {
    diagnostics.push(
      diagnostic(
        "invalid-record-field",
        "Studio catalog record field description must be a string when provided.",
        path,
        "description",
        source,
      ),
    );
  }

  if (value.value !== undefined && typeof value.value !== "number") {
    diagnostics.push(
      diagnostic(
        "invalid-record-field",
        "Studio catalog record field value must be a number when provided.",
        path,
        "value",
        source,
      ),
    );
  }

  for (const field of ["effectId", "executionId", "hook", "modifierId"] as const) {
    if (value[field] !== undefined && typeof value[field] !== "string") {
      diagnostics.push(
        diagnostic(
          "invalid-record-field",
          `Studio catalog record field ${field} must be a string when provided.`,
          path,
          field,
          source,
        ),
      );
    }
  }

  for (const field of ["recordIds", "tagIds"] as const) {
    if (
      value[field] !== undefined &&
      (!Array.isArray(value[field]) ||
        value[field].some((item) => typeof item !== "string" || item.length === 0))
    ) {
      diagnostics.push(
        diagnostic(
          "invalid-record-field",
          `Studio catalog record field ${field} must be an array of strings when provided.`,
          path,
          field,
          source,
        ),
      );
    }
  }

  return diagnostics;
};

const normalizeMappedRecord = (
  mapped: StudioMappedContentRecord,
  diagnostics: StudioDiagnostic[],
): StudioCatalogRecordWithPath | undefined => {
  const source = mapped.location ?? mapped.sourceRecord?.location;
  const path = mapped.path ?? studioSourceLocationLabel(source) ?? "unknown source";
  if (!isObject(mapped.value)) {
    diagnostics.push(
      diagnostic(
        "invalid-record",
        "Studio catalog record must be an object.",
        path,
        undefined,
        source,
      ),
    );
    return undefined;
  }

  let expectedKind: StudioRecordKind | undefined;
  if (typeof mapped.expectedKind === "string") {
    expectedKind = normalizeRecordKind(mapped.expectedKind);
  } else if (typeof mapped.value.kind === "string") {
    expectedKind = normalizeRecordKind(mapped.value.kind);
  }

  if (!expectedKind) {
    diagnostics.push(
      diagnostic(
        "unknown-record-kind",
        "Mapped Studio content record did not declare a supported record kind.",
        path,
        "kind",
        source,
      ),
    );
    return undefined;
  }

  const recordDiagnostics = validateRecordFields(mapped.value, expectedKind, path, source);

  diagnostics.push(...recordDiagnostics);
  if (recordDiagnostics.length > 0) {
    return undefined;
  }

  return {
    ...(mapped.value as unknown as StudioCatalogRecord),
    path,
    source,
  };
};

const mapContentRecords = async (
  config: ResolvedStudioProjectConfig,
  snapshots: StudioSourceSnapshot[],
  diagnostics: StudioDiagnostic[],
): Promise<StudioCatalogRecordWithPath[]> => {
  const mappedRecords: StudioMappedContentRecord[] = [];
  const contentMappers: StudioContentMapper[] = [
    builtInJsonCatalogMapper,
    ...config.extensions.flatMap((extension) => extension.contentMappers ?? []),
  ];

  for (const mapper of contentMappers) {
    try {
      const result = await mapper.map({ config, snapshots });
      diagnostics.push(...(result.diagnostics ?? []));
      mappedRecords.push(...result.records);
    } catch (error) {
      diagnostics.push(
        diagnostic(
          "content-mapper-failed",
          error instanceof Error
            ? `Studio content mapper "${mapper.id}" failed: ${error.message}`
            : `Studio content mapper "${mapper.id}" failed.`,
          config.configPath,
        ),
      );
    }
  }

  return mappedRecords
    .map((record) => normalizeMappedRecord(record, diagnostics))
    .filter((record): record is StudioCatalogRecordWithPath => record !== undefined);
};

const validateDuplicateRecords = (
  records: StudioCatalogRecordWithPath[],
  diagnostics: StudioDiagnostic[],
) => {
  const seen: Record<string, StudioCatalogRecordWithPath | undefined> = {};
  for (const record of records) {
    const key = `${record.kind}:${record.id}`;
    const existing = seen[key];
    if (existing) {
      diagnostics.push(
        diagnostic(
          "duplicate-record",
          `Studio catalog record ${record.kind}:${record.id} is declared more than once.`,
          record.path,
          "id",
          record.source,
        ),
      );
      diagnostics.push(
        diagnostic(
          "duplicate-record",
          `Studio catalog record ${record.kind}:${record.id} is declared more than once.`,
          existing.path,
          "id",
          existing.source,
        ),
      );
      continue;
    }
    seen[key] = record;
  }
};

const validateRecordReferences = (
  byKind: Record<StudioRecordKind, StudioCatalogRecordWithPath[]>,
  diagnostics: StudioDiagnostic[],
) => {
  const ids = {
    effects: new Set(byKind.effects.map((record) => record.id)),
    executions: new Set(byKind.executions.map((record) => record.id)),
    modifiers: new Set(byKind.modifiers.map((record) => record.id)),
    tags: new Set(byKind.tags.map((record) => record.id)),
  };

  for (const record of byKind.abilities) {
    if (record.effectId && !ids.effects.has(record.effectId)) {
      diagnostics.push(
        diagnostic(
          "missing-record-reference",
          `Ability record ${record.id} references missing effect ${record.effectId}.`,
          record.path,
          "effectId",
          record.source,
        ),
      );
    }
  }

  for (const record of byKind.effects) {
    if (record.executionId && !ids.executions.has(record.executionId)) {
      diagnostics.push(
        diagnostic(
          "missing-record-reference",
          `Effect record ${record.id} references missing execution ${record.executionId}.`,
          record.path,
          "executionId",
          record.source,
        ),
      );
    }
    if (record.modifierId && !ids.modifiers.has(record.modifierId)) {
      diagnostics.push(
        diagnostic(
          "missing-record-reference",
          `Effect record ${record.id} references missing modifier ${record.modifierId}.`,
          record.path,
          "modifierId",
          record.source,
        ),
      );
    }
    for (const tagId of record.tagIds ?? []) {
      if (!ids.tags.has(tagId)) {
        diagnostics.push(
          diagnostic(
            "missing-record-reference",
            `Effect record ${record.id} references missing tag ${tagId}.`,
            record.path,
            "tagIds",
            record.source,
          ),
        );
      }
    }
  }

  for (const record of byKind.executions) {
    if (!record.hook || record.hook.trim().length === 0) {
      diagnostics.push(
        diagnostic(
          "missing-runtime-hook",
          `Execution record ${record.id} must declare a runtime hook.`,
          record.path,
          "hook",
          record.source,
        ),
      );
    }
  }
};

const loadBuiltInJsonSnapshot = async (config: ResolvedStudioProjectConfig) => {
  const snapshot = await builtInJsonCatalogAdapter.load({
    config,
    source: {
      adapterId: builtInJsonCatalogAdapter.id,
      id: "studio-json-catalog",
    },
  });

  return {
    ...snapshot,
    adapterId: builtInJsonCatalogAdapter.id,
    sourceId: "studio-json-catalog",
  };
};

export const loadStudioCatalog = async (
  config: ResolvedStudioProjectConfig,
): Promise<StudioCatalog> => {
  const diagnostics: StudioDiagnostic[] = [];
  const byKind = emptyCatalogByKind();
  const builtInJsonSnapshot = await loadBuiltInJsonSnapshot(config);
  const projectSources = await loadStudioSourceSnapshots(config);
  const sourceSnapshots = [builtInJsonSnapshot, ...projectSources.snapshots];
  diagnostics.push(...(builtInJsonSnapshot.diagnostics ?? []), ...projectSources.diagnostics);

  const records = await mapContentRecords(config, sourceSnapshots, diagnostics);
  for (const kind of studioRecordKinds) {
    byKind[kind] = records.filter((record) => normalizeRecordKind(record.kind) === kind);
  }

  validateDuplicateRecords(records, diagnostics);
  validateRecordReferences(byKind, diagnostics);

  return { byKind, diagnostics, records, sourceSnapshots };
};

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

const kindForCatalogRecord = (record: StudioCatalogRecord): StudioRecordKind => {
  const kind = normalizeRecordKind(record.kind);
  if (!kind) {
    throw new Error(`Unsupported Studio catalog record kind ${record.kind}.`);
  }
  return kind;
};

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
          diagnostic(
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
                diagnostic(
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
            diagnostic(
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
              diagnostic(
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
                diagnostic(
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
  const writeDiagnostic = () => diagnostic(diagnosticCode, message, config.configPath);
  const plan = (records: readonly StudioCatalogRecord[]) => ({
    diagnostics: [],
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
