import { existsSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";

import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../../config/schema";
import {
  defineStudioContentMapper,
  defineStudioDataAdapter,
  studioSourceLocationLabel,
} from "../../extensions";
import type { StudioMappedContentRecord, StudioSourceRecord } from "../../extensions";
import { listFilesRecursive, writeTextFile } from "../files";
import { catalogDiagnostic } from "./diagnostics";
import { normalizeRecordKind, studioRecordKinds } from "./kinds";
import type { StudioRecordKind } from "./kinds";
import { isObject } from "./record-value";
import type { StudioCatalogRecord } from "./types";

const collectJsonFiles = (dir: string): string[] =>
  listFilesRecursive(dir).filter((path) => path.endsWith(".json"));

export const writeJsonRecord = (
  catalogRoot: string,
  kind: StudioRecordKind,
  record: StudioCatalogRecord,
) => {
  const path = resolve(catalogRoot, kind, `${record.id}.json`);
  writeTextFile(path, `${JSON.stringify(record, null, 2)}\n`);
  return path;
};

export const builtInJsonCatalogAdapter = defineStudioDataAdapter({
  capabilities: ["read", "write"],
  id: "studio-json-catalog",
  label: "Studio JSON catalog",
  load: ({ config }) => {
    const diagnostics: StudioDiagnostic[] = [];
    const records: StudioSourceRecord[] = [];

    if (!existsSync(config.paths.catalogRoot)) {
      diagnostics.push(
        catalogDiagnostic(
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
            catalogDiagnostic(
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

export const builtInJsonCatalogMapper = defineStudioContentMapper({
  id: "studio-json-catalog-mapper",
  label: "Studio JSON catalog mapper",
  map: ({ snapshots }) => ({
    records: snapshots
      .filter(
        (snapshot) =>
          snapshot.adapterId === builtInJsonCatalogAdapter.id &&
          snapshot.sourceId === "studio-json-catalog",
      )
      .flatMap((snapshot) =>
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

export const loadBuiltInJsonSnapshot = async (config: ResolvedStudioProjectConfig) => {
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
