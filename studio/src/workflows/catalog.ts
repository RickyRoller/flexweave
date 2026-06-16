import { loadStudioCatalog, normalizeRecordKind, studioRecordKinds } from "../internal/catalog";
import { resolveWorkflowConfig, workflowError } from "./shared";
import type {
  DescribeStudioCatalogResult,
  ListStudioCatalogRecordsResult,
  ShowStudioCatalogRecordResult,
  StudioRecordDescription,
  StudioWorkflowOptions,
  ValidateStudioCatalogResult,
} from "./types";

const schemaDescriptions: StudioRecordDescription[] = [
  {
    fields: ["kind", "id", "label", "effectId"],
    kind: "abilities",
    summary: "Ability records name callable mechanics and may reference effects.",
  },
  {
    fields: ["kind", "id", "label", "executionId", "modifierId", "tagIds"],
    kind: "effects",
    summary: "Effect records connect generated definitions to executions and tags.",
  },
  {
    fields: ["kind", "id", "label", "hook"],
    kind: "executions",
    summary: "Execution records name runtime hooks declared by the consumer runtime.",
  },
  {
    fields: ["kind", "id", "label", "recordIds"],
    kind: "mechanics",
    summary: "Mechanic manifests record files created by Studio scaffolding.",
  },
  {
    fields: ["kind", "id", "label", "value"],
    kind: "modifiers",
    summary: "Modifier records provide reusable generated definition data.",
  },
  {
    fields: ["kind", "id", "label"],
    kind: "tags",
    summary: "Tag records provide stable grouping tokens for generated definitions.",
  },
];

export const validateStudioCatalog = async (
  options: StudioWorkflowOptions = {},
): Promise<ValidateStudioCatalogResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      diagnostics: resolved.diagnostics,
      ok: false,
      recordCount: 0,
      sourceRecordCount: 0,
      sources: [],
    };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const sourceSnapshots = catalog.sourceSnapshots.filter((snapshot) => snapshot.records.length > 0);
  return {
    configPath: resolved.config.configPath,
    diagnostics: catalog.diagnostics,
    ok: catalog.diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    recordCount: catalog.records.length,
    sourceRecordCount: catalog.sourceSnapshots.reduce(
      (total, snapshot) => total + snapshot.records.length,
      0,
    ),
    sources: sourceSnapshots.map((snapshot) => ({
      adapterId: snapshot.adapterId,
      recordCount: snapshot.records.length,
      sourceId: snapshot.sourceId,
    })),
  };
};

export const describeStudioCatalog = async (
  kind: string | undefined,
  options: StudioWorkflowOptions = {},
): Promise<DescribeStudioCatalogResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { descriptions: [], diagnostics: resolved.diagnostics, ok: false };
  }

  if (!kind) {
    return { descriptions: schemaDescriptions, diagnostics: [], ok: true };
  }

  const normalized = normalizeRecordKind(kind);
  if (!normalized) {
    return {
      descriptions: [],
      diagnostics: [
        workflowError(
          "unknown-record-kind",
          `Unknown Studio catalog record kind "${kind}".`,
          undefined,
          `Expected one of: ${studioRecordKinds.join(", ")}.`,
        ),
      ],
      ok: false,
    };
  }

  return {
    descriptions: schemaDescriptions.filter((description) => description.kind === normalized),
    diagnostics: [],
    ok: true,
  };
};

export const listStudioCatalogRecords = async (
  kind: string,
  options: StudioWorkflowOptions & { filter?: string } = {},
): Promise<ListStudioCatalogRecordsResult> => {
  const normalized = normalizeRecordKind(kind);
  if (!normalized) {
    return {
      diagnostics: [
        workflowError(
          "unknown-record-kind",
          `Unknown Studio catalog record kind "${kind}".`,
          undefined,
          `Expected one of: ${studioRecordKinds.join(", ")}.`,
        ),
      ],
      kind: "abilities",
      ok: false,
      records: [],
    };
  }

  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { diagnostics: resolved.diagnostics, kind: normalized, ok: false, records: [] };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const filter = options.filter?.toLowerCase();
  const records = catalog.byKind[normalized]
    .filter(
      (record) =>
        !filter ||
        record.id.toLowerCase().includes(filter) ||
        record.label.toLowerCase().includes(filter),
    )
    .map((record) => ({
      id: record.id,
      label: record.label,
      path: record.path,
    }));

  return {
    diagnostics: catalog.diagnostics,
    kind: normalized,
    ok: catalog.diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    records,
  };
};

export const showStudioCatalogRecord = async (
  kind: string,
  id: string,
  options: StudioWorkflowOptions = {},
): Promise<ShowStudioCatalogRecordResult> => {
  const normalized = normalizeRecordKind(kind);
  if (!normalized) {
    return {
      diagnostics: [
        workflowError("unknown-record-kind", `Unknown Studio catalog record kind "${kind}".`),
      ],
      ok: false,
    };
  }

  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return { diagnostics: resolved.diagnostics, ok: false };
  }

  const catalog = await loadStudioCatalog(resolved.config);
  const record = catalog.byKind[normalized].find((candidate) => candidate.id === id);
  if (!record) {
    return {
      diagnostics: [
        workflowError(
          "missing-record",
          `No ${normalized} record with id "${id}" exists in the Studio catalog.`,
        ),
      ],
      ok: false,
    };
  }

  return { diagnostics: catalog.diagnostics, ok: true, record };
};
