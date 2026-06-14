import { expect, test } from "bun:test";

import config from "./fixtures/minimal/studio.config";
import {
  listReservedStudioWorkflows,
  minimalStudioProjectConfig,
  studioPackageStatus,
} from "../src";

test("placeholder package exposes reserved workflows", () => {
  expect(listReservedStudioWorkflows()).toEqual(["validate", "migrate", "verify"]);
});

test("minimal fixture matches the reserved config shape", () => {
  expect(config).toEqual(minimalStudioProjectConfig);
});

test("package status names the reserved surface", () => {
  expect(studioPackageStatus().surface).toBe("Flexweave Studio package");
});
