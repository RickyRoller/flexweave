import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "bun:test";

import { studioCodegenTargets } from "@flexweave/studio/codegen";

import { studioRoot } from "./support/studio-fixtures";

test("package metadata exposes only the Studio public contract", () => {
  const packageJson = JSON.parse(readFileSync(join(studioRoot, "package.json"), "utf-8"));

  expect(packageJson.name).toBe("@flexweave/studio");
  expect(Object.keys(packageJson.bin)).toEqual(["flexweave-studio"]);
  expect(Object.keys(packageJson.exports).toSorted()).toEqual([
    "./codegen",
    "./config",
    "./config/load",
    "./extensions",
    "./workflows",
  ]);
  expect(studioCodegenTargets).toEqual([
    "abilities",
    "effects",
    "executions",
    "modifiers",
    "reference",
    "tags",
  ]);
  expect(
    readdirSync(join(studioRoot, "tests/fixtures"), { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .toSorted(),
  ).toEqual(["extension-sources", "minimal"]);
});
