import type { StudioDiagnostic } from "../../config/schema";
import type { StudioSourceLocation } from "../../extensions";
import { catalogDiagnostic } from "./diagnostics";
import { singularByKind } from "./kinds";
import type { StudioRecordKind } from "./kinds";
import type { StudioCatalogRecordWithPath } from "./types";

export const validateRecordFields = (
  value: Record<string, unknown>,
  expectedKind: StudioRecordKind,
  path: string,
  source?: StudioSourceLocation,
): StudioDiagnostic[] => {
  const diagnostics: StudioDiagnostic[] = [];

  if (value.kind !== singularByKind[expectedKind]) {
    diagnostics.push(
      catalogDiagnostic(
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
        catalogDiagnostic(
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
      catalogDiagnostic(
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
      catalogDiagnostic(
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
        catalogDiagnostic(
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
        catalogDiagnostic(
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

export const validateDuplicateRecords = (
  records: StudioCatalogRecordWithPath[],
  diagnostics: StudioDiagnostic[],
) => {
  const seen: Record<string, StudioCatalogRecordWithPath | undefined> = {};
  for (const record of records) {
    const key = `${record.kind}:${record.id}`;
    const existing = seen[key];
    if (existing) {
      diagnostics.push(
        catalogDiagnostic(
          "duplicate-record",
          `Studio catalog record ${record.kind}:${record.id} is declared more than once.`,
          record.path,
          "id",
          record.source,
        ),
      );
      diagnostics.push(
        catalogDiagnostic(
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

export const validateRecordReferences = (
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
        catalogDiagnostic(
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
        catalogDiagnostic(
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
        catalogDiagnostic(
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
          catalogDiagnostic(
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
        catalogDiagnostic(
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
