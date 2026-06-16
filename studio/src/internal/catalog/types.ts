import type { StudioDiagnostic } from "../../config/schema";
import type {
  StudioMapperDiagnosticAttribution,
  StudioSourceDiagnosticAttribution,
  StudioSourceLocation,
  StudioSourceSnapshot,
} from "../../extensions";
import type { StudioRecordKind, StudioRecordSingular } from "./kinds";

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
  mapperDiagnostics: StudioMapperDiagnosticAttribution[];
  records: StudioCatalogRecordWithPath[];
  sourceDiagnostics: StudioSourceDiagnosticAttribution[];
  sourceSnapshots: StudioSourceSnapshot[];
}
