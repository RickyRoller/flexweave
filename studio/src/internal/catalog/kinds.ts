import type { StudioCatalogRecord, StudioCatalogRecordWithPath } from "./types";

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

export const singularByKind: Record<StudioRecordKind, StudioRecordSingular> = {
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

export const emptyCatalogByKind = () =>
  Object.fromEntries(studioRecordKinds.map((kind) => [kind, []])) as unknown as Record<
    StudioRecordKind,
    StudioCatalogRecordWithPath[]
  >;

export const kindForCatalogRecord = (record: StudioCatalogRecord): StudioRecordKind => {
  const kind = normalizeRecordKind(record.kind);
  if (!kind) {
    throw new Error(`Unsupported Studio catalog record kind ${record.kind}.`);
  }
  return kind;
};
