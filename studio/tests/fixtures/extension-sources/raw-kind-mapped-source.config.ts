import { defineStudioConfig } from "@flexweave/studio/config";
import {
  defineStudioContentMapper,
  defineStudioExtension,
  studioSourceLocationLabel,
} from "@flexweave/studio/extensions";

import { syntheticFileAdapter } from "./synthetic-extension";

const rawKindContentMapper = defineStudioContentMapper({
  id: "raw-kind-content-mapper",
  label: "Raw kind content mapper",
  map: ({ snapshots }) => ({
    records: snapshots.flatMap((snapshot) =>
      snapshot.records
        .filter((record) => record.kind === "tags")
        .map((record) => ({
          expectedKind: record.kind,
          location: record.location,
          path: studioSourceLocationLabel(record.location),
          sourceRecord: record,
          value: record.value,
        })),
    ),
  }),
});

const rawKindSourceExtension = defineStudioExtension({
  contentMappers: [rawKindContentMapper],
  dataAdapters: [syntheticFileAdapter],
  id: "raw-kind-source-extension",
  label: "Raw kind source extension",
});

export default defineStudioConfig({
  catalogRoot: "catalog",
  data: {
    sources: [
      {
        adapterId: "synthetic-file",
        id: "raw-kind-file",
        options: {
          path: "sources/raw-kind-file-record.json",
          recordKind: "tags",
        },
      },
    ],
  },
  extensions: [rawKindSourceExtension],
  mode: "validate-only",
});
