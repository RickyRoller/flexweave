import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../../config/schema";
import { loadStudioSourceSnapshots } from "../../extensions";
import type {
  StudioSourceDiagnosticAttribution,
  StudioSourceLocation,
  StudioSourceSnapshot,
} from "../../extensions";
import { loadBuiltInJsonSnapshot } from "./json-source";
import { emptyCatalogByKind, normalizeRecordKind, studioRecordKinds } from "./kinds";
import { mapContentRecords } from "./normalize";
import type { StudioCatalog } from "./types";
import { validateDuplicateRecords, validateRecordReferences } from "./validation";

type SourceDiagnosticOwner = Omit<StudioSourceDiagnosticAttribution, "diagnostics">;
type SourceDiagnosticGroup = SourceDiagnosticOwner & { diagnostics: Set<StudioDiagnostic> };

const sourceLocationFields = [
  "cell",
  "column",
  "display",
  "field",
  "jsonPointer",
  "line",
  "path",
  "row",
  "sheet",
  "uri",
] as const satisfies readonly (keyof StudioSourceLocation)[];

const sourceLocationKey = (location: StudioSourceLocation | undefined): string | undefined => {
  if (!location || sourceLocationFields.every((field) => location[field] === undefined)) {
    return undefined;
  }

  return JSON.stringify(sourceLocationFields.map((field) => [field, location[field] ?? null]));
};

const sourceOwnerKey = (owner: SourceDiagnosticOwner) =>
  `${owner.sourceId ?? ""}\0${owner.adapterId ?? ""}`;

const buildSourceDiagnostics = (
  snapshots: readonly StudioSourceSnapshot[],
  seededGroups: readonly StudioSourceDiagnosticAttribution[],
  diagnostics: readonly StudioDiagnostic[],
): StudioSourceDiagnosticAttribution[] => {
  const groups: Record<string, SourceDiagnosticGroup | undefined> = Object.create(null);
  const ownersByLocation: Record<
    string,
    Record<string, SourceDiagnosticOwner | undefined> | undefined
  > = Object.create(null);

  const ensureGroup = (owner: SourceDiagnosticOwner) => {
    const key = sourceOwnerKey(owner);
    const existing = groups[key];
    if (existing) {
      return existing;
    }

    const group = {
      adapterId: owner.adapterId,
      diagnostics: new Set<StudioDiagnostic>(),
      sourceId: owner.sourceId,
    };
    groups[key] = group;
    return group;
  };

  for (const seeded of seededGroups) {
    const group = ensureGroup(seeded);
    for (const diagnostic of seeded.diagnostics) {
      group.diagnostics.add(diagnostic);
    }
  }

  for (const snapshot of snapshots) {
    const owner = {
      adapterId: snapshot.adapterId,
      sourceId: snapshot.sourceId,
    };
    const ownerKey = sourceOwnerKey(owner);

    for (const record of snapshot.records) {
      const locationKey = sourceLocationKey(record.location);
      if (!locationKey) {
        continue;
      }

      const owners = ownersByLocation[locationKey] ?? Object.create(null);
      owners[ownerKey] = owner;
      ownersByLocation[locationKey] = owners;
    }
  }

  for (const diagnostic of diagnostics) {
    const locationKey = sourceLocationKey(diagnostic.source);
    if (!locationKey) {
      continue;
    }

    const owners = ownersByLocation[locationKey];
    if (!owners) {
      continue;
    }

    for (const owner of Object.values(owners)) {
      if (!owner) {
        continue;
      }
      ensureGroup(owner).diagnostics.add(diagnostic);
    }
  }

  return Object.values(groups)
    .filter((group): group is SourceDiagnosticGroup => group !== undefined)
    .map((group) => ({
      adapterId: group.adapterId,
      diagnostics: [...group.diagnostics],
      sourceId: group.sourceId,
    }));
};

export const loadStudioCatalog = async (
  config: ResolvedStudioProjectConfig,
): Promise<StudioCatalog> => {
  const diagnostics: StudioDiagnostic[] = [];
  const byKind = emptyCatalogByKind();
  const builtInJsonSnapshot = await loadBuiltInJsonSnapshot(config);
  const builtInJsonDiagnostics = [...(builtInJsonSnapshot.diagnostics ?? [])];
  const projectSources = await loadStudioSourceSnapshots(config);
  const sourceSnapshots = [builtInJsonSnapshot, ...projectSources.snapshots];
  diagnostics.push(...builtInJsonDiagnostics, ...projectSources.diagnostics);

  const mappedContent = await mapContentRecords(config, sourceSnapshots, diagnostics);
  const { records } = mappedContent;
  for (const kind of studioRecordKinds) {
    byKind[kind] = records.filter((record) => normalizeRecordKind(record.kind) === kind);
  }

  validateDuplicateRecords(records, diagnostics);
  validateRecordReferences(byKind, diagnostics);

  return {
    byKind,
    diagnostics,
    mapperDiagnostics: mappedContent.mapperDiagnostics,
    records,
    sourceDiagnostics: buildSourceDiagnostics(
      sourceSnapshots,
      [
        ...(builtInJsonDiagnostics.length > 0
          ? [
              {
                adapterId: builtInJsonSnapshot.adapterId,
                diagnostics: builtInJsonDiagnostics,
                sourceId: builtInJsonSnapshot.sourceId,
              },
            ]
          : []),
        ...projectSources.sourceDiagnostics,
      ],
      diagnostics,
    ),
    sourceSnapshots,
  };
};
