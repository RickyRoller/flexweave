import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../../config/schema";
import { studioSourceLocationLabel } from "../../extensions";
import type {
  StudioContentMapper,
  StudioMappedContentRecord,
  StudioMapperDiagnosticAttribution,
  StudioSourceSnapshot,
} from "../../extensions";
import { catalogDiagnostic } from "./diagnostics";
import { builtInJsonCatalogMapper } from "./json-source";
import { normalizeRecordKind } from "./kinds";
import type { StudioRecordKind } from "./kinds";
import { isObject } from "./record-value";
import type { StudioCatalogRecord, StudioCatalogRecordWithPath } from "./types";
import { validateRecordFields } from "./validation";

interface RegisteredStudioContentMapper {
  extensionId?: string;
  mapper: StudioContentMapper;
}

interface OwnedMappedContentRecord {
  mapper: RegisteredStudioContentMapper;
  record: StudioMappedContentRecord;
}

export interface MapContentRecordsResult {
  mapperDiagnostics: StudioMapperDiagnosticAttribution[];
  records: StudioCatalogRecordWithPath[];
}

export const normalizeMappedRecord = (
  mapped: StudioMappedContentRecord,
  diagnostics: StudioDiagnostic[],
): StudioCatalogRecordWithPath | undefined => {
  const source = mapped.location ?? mapped.sourceRecord?.location;
  const path = mapped.path ?? studioSourceLocationLabel(source) ?? "unknown source";
  if (!isObject(mapped.value)) {
    diagnostics.push(
      catalogDiagnostic(
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
      catalogDiagnostic(
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

export const mapContentRecords = async (
  config: ResolvedStudioProjectConfig,
  snapshots: StudioSourceSnapshot[],
  diagnostics: StudioDiagnostic[],
): Promise<MapContentRecordsResult> => {
  const mappedRecords: OwnedMappedContentRecord[] = [];
  const mapperDiagnostics: Record<string, StudioMapperDiagnosticAttribution | undefined> =
    Object.create(null);
  const contentMappers: RegisteredStudioContentMapper[] = [
    { mapper: builtInJsonCatalogMapper },
    ...config.extensions.flatMap((extension) =>
      (extension.contentMappers ?? []).map((mapper) => ({
        extensionId: extension.id,
        mapper,
      })),
    ),
  ];

  const addMapperDiagnostics = (
    mapper: RegisteredStudioContentMapper,
    ownedDiagnostics: readonly StudioDiagnostic[],
  ) => {
    if (ownedDiagnostics.length === 0) {
      return;
    }

    const key = `${mapper.extensionId ?? ""}\0${mapper.mapper.id}`;
    const existing = mapperDiagnostics[key];
    if (existing) {
      mapperDiagnostics[key] = {
        ...existing,
        diagnostics: [...existing.diagnostics, ...ownedDiagnostics],
      };
      return;
    }

    mapperDiagnostics[key] = {
      diagnostics: [...ownedDiagnostics],
      extensionId: mapper.extensionId,
      mapperId: mapper.mapper.id,
    };
  };

  for (const mapper of contentMappers) {
    try {
      const result = await mapper.mapper.map({ config, snapshots });
      const ownedDiagnostics = [...(result.diagnostics ?? [])];
      diagnostics.push(...ownedDiagnostics);
      addMapperDiagnostics(mapper, ownedDiagnostics);
      mappedRecords.push(...result.records.map((record) => ({ mapper, record })));
    } catch (error) {
      const diagnostic = catalogDiagnostic(
        "content-mapper-failed",
        error instanceof Error
          ? `Studio content mapper "${mapper.mapper.id}" failed: ${error.message}`
          : `Studio content mapper "${mapper.mapper.id}" failed.`,
        config.configPath,
      );
      diagnostics.push(diagnostic);
      addMapperDiagnostics(mapper, [diagnostic]);
    }
  }

  const records = mappedRecords
    .map(({ mapper, record }) => {
      const normalizationDiagnostics: StudioDiagnostic[] = [];
      const normalized = normalizeMappedRecord(record, normalizationDiagnostics);
      diagnostics.push(...normalizationDiagnostics);
      addMapperDiagnostics(mapper, normalizationDiagnostics);
      return normalized;
    })
    .filter((record): record is StudioCatalogRecordWithPath => record !== undefined);

  return {
    mapperDiagnostics: Object.values(mapperDiagnostics).filter(
      (group): group is StudioMapperDiagnosticAttribution => group !== undefined,
    ),
    records,
  };
};
