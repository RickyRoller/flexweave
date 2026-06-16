import { existsSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "bun:test";

import { planStudioMechanic, scaffoldStudioMechanic } from "@flexweave/studio/workflows";

import { copyMinimalFixture } from "./support/studio-fixtures";

test("mechanic planning and scaffolding are transactional", async () => {
  const root = copyMinimalFixture();
  const configPath = join(root, "studio.config.ts");

  const planned = await planStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "planned_mechanic",
    name: "Planned mechanic",
  });
  expect(planned.ok).toBe(true);
  expect(planned.plannedFiles).toHaveLength(6);

  const scaffolded = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "scaffolded_mechanic",
    name: "Scaffolded mechanic",
  });
  expect(scaffolded.ok).toBe(true);
  expect(existsSync(join(root, "catalog/abilities/scaffolded_mechanic.json"))).toBe(true);
  expect(existsSync(join(root, "runtime-hooks/scaffolded_mechanic_runtime_hook.rs"))).toBe(true);

  const failed = await scaffoldStudioMechanic({
    archetype: "mechanic",
    configPath,
    id: "broken_mechanic",
    name: "Broken mechanic",
    params: { broken: true },
  });
  expect(failed.ok).toBe(false);
  expect(failed.rolledBack).toBe(true);
  expect(existsSync(join(root, "catalog/abilities/broken_mechanic.json"))).toBe(false);
});
