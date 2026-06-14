import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";

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
}

export interface StudioCatalog {
  byKind: Record<StudioRecordKind, StudioCatalogRecordWithPath[]>;
  diagnostics: StudioDiagnostic[];
  records: StudioCatalogRecordWithPath[];
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
): StudioDiagnostic => ({
  code,
  field,
  message,
  path,
  severity: "error",
});

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

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

const validateRecordFields = (
  value: Record<string, unknown>,
  expectedKind: StudioRecordKind,
  relativePath: string,
): StudioDiagnostic[] => {
  const diagnostics: StudioDiagnostic[] = [];

  if (value.kind !== singularByKind[expectedKind]) {
    diagnostics.push(
      diagnostic(
        "invalid-record-kind",
        `Expected a ${singularByKind[expectedKind]} record in ${expectedKind}.`,
        relativePath,
        "kind",
      ),
    );
  }

  for (const field of ["id", "label"] as const) {
    if (typeof value[field] !== "string" || value[field].trim().length === 0) {
      diagnostics.push(
        diagnostic(
          "invalid-record-field",
          `Studio catalog record field ${field} must be a non-empty string.`,
          relativePath,
          field,
        ),
      );
    }
  }

  if (value.description !== undefined && typeof value.description !== "string") {
    diagnostics.push(
      diagnostic(
        "invalid-record-field",
        "Studio catalog record field description must be a string when provided.",
        relativePath,
        "description",
      ),
    );
  }

  if (value.value !== undefined && typeof value.value !== "number") {
    diagnostics.push(
      diagnostic(
        "invalid-record-field",
        "Studio catalog record field value must be a number when provided.",
        relativePath,
        "value",
      ),
    );
  }

  for (const field of ["effectId", "executionId", "hook", "modifierId"] as const) {
    if (value[field] !== undefined && typeof value[field] !== "string") {
      diagnostics.push(
        diagnostic(
          "invalid-record-field",
          `Studio catalog record field ${field} must be a string when provided.`,
          relativePath,
          field,
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
          relativePath,
          field,
        ),
      );
    }
  }

  return diagnostics;
};

const readRecord = (
  catalogRoot: string,
  filePath: string,
  expectedKind: StudioRecordKind,
  diagnostics: StudioDiagnostic[],
): StudioCatalogRecordWithPath | undefined => {
  const relativePath = relative(catalogRoot, filePath);
  let value: unknown;
  try {
    value = JSON.parse(readFileSync(filePath, "utf-8"));
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
    return undefined;
  }

  if (!isObject(value)) {
    diagnostics.push(
      diagnostic("invalid-record", "Studio catalog record must be a JSON object.", relativePath),
    );
    return undefined;
  }

  const recordDiagnostics = validateRecordFields(value, expectedKind, relativePath);

  diagnostics.push(...recordDiagnostics);
  if (recordDiagnostics.length > 0) {
    return undefined;
  }

  return {
    ...(value as unknown as StudioCatalogRecord),
    path: relativePath,
  };
};

const collectCatalogRecords = (
  config: ResolvedStudioProjectConfig,
  diagnostics: StudioDiagnostic[],
): Record<StudioRecordKind, StudioCatalogRecordWithPath[]> => {
  const byKind = Object.fromEntries(
    studioRecordKinds.map((kind) => [kind, []]),
  ) as unknown as Record<StudioRecordKind, StudioCatalogRecordWithPath[]>;

  for (const kind of studioRecordKinds) {
    const kindDir = join(config.paths.catalogRoot, kind);
    for (const filePath of collectJsonFiles(kindDir)) {
      const record = readRecord(config.paths.catalogRoot, filePath, kind, diagnostics);
      if (record) {
        byKind[kind].push(record);
      }
    }
  }

  return byKind;
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
        ),
      );
      diagnostics.push(
        diagnostic(
          "duplicate-record",
          `Studio catalog record ${record.kind}:${record.id} is declared more than once.`,
          existing.path,
          "id",
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
        ),
      );
    }
  }
};

export const readStudioCatalog = (config: ResolvedStudioProjectConfig): StudioCatalog => {
  const diagnostics: StudioDiagnostic[] = [];
  const emptyCatalog = Object.fromEntries(
    studioRecordKinds.map((kind) => [kind, []]),
  ) as unknown as Record<StudioRecordKind, StudioCatalogRecordWithPath[]>;

  if (!existsSync(config.paths.catalogRoot)) {
    diagnostics.push(
      diagnostic(
        "missing-catalog-root",
        "Configured Studio catalog root does not exist.",
        config.raw.catalogRoot,
      ),
    );
    return { byKind: emptyCatalog, diagnostics, records: [] };
  }

  const byKind = collectCatalogRecords(config, diagnostics);
  const records = studioRecordKinds.flatMap((kind) => byKind[kind]);
  validateDuplicateRecords(records, diagnostics);
  validateRecordReferences(byKind, diagnostics);

  return { byKind, diagnostics, records };
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

export const removePathIfExists = (path: string) => {
  if (existsSync(path)) {
    rmSync(path, { force: true, recursive: true });
  }
};
