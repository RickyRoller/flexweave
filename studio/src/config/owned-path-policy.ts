import { isAbsolute, relative } from "node:path";

import { configError } from "./diagnostics";
import type { StudioDiagnostic } from "./types";

const validateDuplicateOwnedPaths = (
  paths: Record<string, string | undefined>,
  diagnostics: StudioDiagnostic[],
) => {
  const byPath: Record<string, string | undefined> = {};
  for (const [field, value] of Object.entries(paths)) {
    if (!value) {
      continue;
    }

    const existing = byPath[value];
    if (existing) {
      diagnostics.push(
        configError(
          "duplicate-owned-path",
          field,
          `Studio project config fields ${existing} and ${field} resolve to the same owned path.`,
          "Use distinct directories for generated targets and runtime hook roots.",
        ),
      );
      continue;
    }

    byPath[value] = field;
  }
};

const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

const validateAmbiguousOwnedPaths = (
  paths: Record<string, string | undefined>,
  diagnostics: StudioDiagnostic[],
) => {
  const entries = Object.entries(paths).filter(
    (entry): entry is [string, string] => typeof entry[1] === "string" && entry[1].length > 0,
  );

  for (let leftIndex = 0; leftIndex < entries.length; leftIndex += 1) {
    for (let rightIndex = leftIndex + 1; rightIndex < entries.length; rightIndex += 1) {
      const [leftField, leftPath] = entries[leftIndex];
      const [rightField, rightPath] = entries[rightIndex];
      if (leftPath === rightPath) {
        continue;
      }

      if (pathContains(leftPath, rightPath) || pathContains(rightPath, leftPath)) {
        diagnostics.push(
          configError(
            "ambiguous-owned-path",
            rightField,
            `Studio project config fields ${leftField} and ${rightField} overlap owned paths.`,
            "Use sibling directories instead of nesting generated targets or runtime hook roots.",
          ),
        );
      }
    }
  }
};

export const validateOwnedPathPolicy = (
  paths: Record<string, string | undefined>,
  diagnostics: StudioDiagnostic[],
  options: { allowOverlappingOutputDirs: boolean },
) => {
  validateDuplicateOwnedPaths(paths, diagnostics);
  if (!options.allowOverlappingOutputDirs) {
    validateAmbiguousOwnedPaths(paths, diagnostics);
  }
};
