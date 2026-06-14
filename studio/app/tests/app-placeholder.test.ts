import { expect, test } from "bun:test";

import { createStudioAppPlaceholder } from "../src";

test("placeholder app reserves adapter-backed surface", () => {
  expect(createStudioAppPlaceholder()).toEqual({
    requiresProjectAdapter: true,
    surface: "Flexweave Studio app",
  });
});
