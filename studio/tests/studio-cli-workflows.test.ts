import { join } from "node:path";
import { expect, test } from "bun:test";

import {
  copyMinimalFixture,
  extensionFixtureConfigPath,
  fixtureConfigPath,
  runStudioCli,
} from "./support/studio-fixtures";

test("CLI workflow JSON output covers extension sources and host app scaffold", async () => {
  const extensionValidate = await runStudioCli([
    "validate",
    "--json",
    "--config",
    extensionFixtureConfigPath,
  ]);
  expect(extensionValidate.exitCode).toBe(0);
  expect(JSON.parse(extensionValidate.stdout)).toMatchObject({
    ok: true,
    sourceRecordCount: 2,
    sources: [
      { adapterId: "synthetic-file", recordCount: 1, sourceId: "file-backed" },
      { adapterId: "synthetic-table", recordCount: 1, sourceId: "table-backed" },
    ],
  });

  const root = copyMinimalFixture();
  const scaffold = await runStudioCli([
    "scaffold",
    "host-app",
    "--json",
    "--app-root",
    "studio-host",
    "--config",
    join(root, "studio.config.ts"),
  ]);
  const scaffoldJson = JSON.parse(scaffold.stdout) as { changedFiles: string[]; ok: boolean };
  expect(scaffold.exitCode).toBe(0);
  expect(scaffoldJson.ok).toBe(true);
  expect(scaffoldJson.changedFiles).toHaveLength(5);

  const validate = await runStudioCli(["validate", "--json", "--config", fixtureConfigPath]);
  expect(validate.exitCode).toBe(0);
  expect(JSON.parse(validate.stdout).recordCount).toBe(6);
});
