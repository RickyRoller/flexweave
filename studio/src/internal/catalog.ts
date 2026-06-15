import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import {
  defineStudioContentMapper,
  defineStudioDataAdapter,
  loadStudioSourceSnapshots,
  studioSourceLocationLabel,
} from "../extensions";
import type {
  StudioContentMapper,
  StudioMappedContentRecord,
  StudioSourceLocation,
  StudioSourceRecord,
  StudioSourceSnapshot,
} from "../extensions";

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
): StudioDiagnostic => ({
  code,
  field,
  message,
  path,
  severity: "error",
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

const catalogWritesUseBuiltInJsonAdapter = (config: ResolvedStudioProjectConfig) =>
  config.data.sources.length === 0;

export const writeStudioCatalogRecord = (
  config: ResolvedStudioProjectConfig,
  kind: StudioRecordKind,
  record: StudioCatalogRecord,
) => {
  if (!catalogWritesUseBuiltInJsonAdapter(config)) {
    return {
      diagnostics: [
        diagnostic(
          "source-write-unsupported",
          "Studio scaffold writes require a writable content adapter for the active source configuration.",
          config.configPath,
        ),
      ],
      path: undefined,
    };
  }

  const snapshot = builtInJsonCatalogAdapter.write?.({
    config,
    records: [
      {
        id: record.id,
        kind,
        value: record,
      },
    ],
    source: {
      adapterId: builtInJsonCatalogAdapter.id,
      id: "studio-json-catalog",
    },
  });

  const written = snapshot?.records[0];
  return {
    diagnostics: [],
    path: written?.location?.path
      ? resolve(config.paths.catalogRoot, written.location.path)
      : resolve(config.paths.catalogRoot, kind, `${record.id}.json`),
  };
};

export const removePathIfExists = (path: string) => {
  if (existsSync(path)) {
    rmSync(path, { force: true, recursive: true });
  }
};
