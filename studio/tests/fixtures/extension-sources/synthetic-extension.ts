import { readFileSync } from "node:fs";
import { join } from "node:path";

import {
  defineStudioDataAdapter,
  defineStudioExtension,
  studioSourceLocationLabel,
} from "@flexweave/studio/extensions";
import type { StudioDiagnostic, StudioSourceRecord } from "@flexweave/studio/extensions";

const optionString = (
  options: Record<string, unknown> | undefined,
  key: string,
): string | undefined => {
  const value = options?.[key];
  return typeof value === "string" && value.length > 0 ? value : undefined;
};

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const sourceRecordDiagnostic = (record: StudioSourceRecord): StudioDiagnostic => ({
  code: "synthetic-source-invalid",
  message: `Synthetic source record ${record.id} is invalid.`,
  path: studioSourceLocationLabel(record.location),
  severity: "error",
  source: record.location,
});

export const syntheticFileAdapter = defineStudioDataAdapter({
  capabilities: ["read"],
  id: "synthetic-file",
  label: "Synthetic file adapter",
  load: ({ config, source }) => {
    const path = optionString(source.options, "path");
    if (!path) {
      throw new Error("Expected source option path.");
    }

    const value = JSON.parse(readFileSync(join(config.configDir, path), "utf-8"));
    if (!isObject(value) || typeof value.id !== "string") {
      throw new Error("Expected file-backed source record with an id.");
    }

    return {
      records: [
        {
          id: value.id,
          kind: "synthetic.file",
          location: {
            jsonPointer: "/",
            path,
          },
          value,
        },
      ],
    };
  },
});

export const syntheticTableAdapter = defineStudioDataAdapter({
  capabilities: ["read", "schema", "write"],
  id: "synthetic-table",
  label: "Synthetic table adapter",
  load: ({ source }) => {
    const rows = Array.isArray(source.options?.rows) ? source.options.rows : [];

    return {
      records: rows.map((row, index) => {
        if (!isObject(row) || typeof row.id !== "string") {
          throw new Error(`Expected table row ${index + 2} to contain an id.`);
        }

        return {
          id: row.id,
          kind: "synthetic.table",
          location: {
            column: 1,
            field: "id",
            row: index + 2,
            sheet: "synthetic-table",
          },
          value: row,
        };
      }),
    };
  },
  write: ({ records }) => ({ records }),
});

export const syntheticSourceExtension = defineStudioExtension({
  dataAdapters: [syntheticFileAdapter, syntheticTableAdapter],
  id: "synthetic-source-extension",
  label: "Synthetic source extension",
  validateSources: ({ snapshots }) =>
    snapshots
      .flatMap((snapshot) => snapshot.records)
      .filter((record) => isObject(record.value) && record.value.valid === false)
      .map(sourceRecordDiagnostic),
});
