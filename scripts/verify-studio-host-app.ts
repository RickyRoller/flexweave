import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  migrateStudioProject,
  scaffoldStudioHostApp,
  verifyStudioHostApp,
} from "../studio/src/workflows";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const studioRoot = join(root, "studio");
const fixtureRoot = join(studioRoot, "tests/fixtures/minimal");
const failures: string[] = [];

const fail = (message: string) => {
  failures.push(message);
};

const linkIfMissing = (target: string, path: string, type?: "dir") => {
  if (!existsSync(path)) {
    symlinkSync(target, path, type);
  }
};

const linkHostAppPackages = (appRoot: string) => {
  const scopeRoot = join(appRoot, "node_modules/@flexweave");
  mkdirSync(scopeRoot, { recursive: true });
  linkIfMissing(studioRoot, join(scopeRoot, "studio"), "dir");
  linkIfMissing(join(studioRoot, "app"), join(scopeRoot, "studio-app"), "dir");
  linkIfMissing(
    join(studioRoot, "node_modules/bun-types"),
    join(appRoot, "node_modules/bun-types"),
    "dir",
  );

  const binRoot = join(appRoot, "node_modules/.bin");
  mkdirSync(binRoot, { recursive: true });
  linkIfMissing(join(root, "node_modules/typescript/bin/tsc"), join(binRoot, "tsc"));
};

const linkFixtureConfigPackage = (rootDirectory: string) => {
  const scopeRoot = join(rootDirectory, "node_modules/@flexweave");
  mkdirSync(scopeRoot, { recursive: true });
  linkIfMissing(studioRoot, join(scopeRoot, "studio"), "dir");
};

const rootDirectory = join(tmpdir(), `studio-host-app-${crypto.randomUUID()}`);

try {
  mkdirSync(rootDirectory, { recursive: true });
  cpSync(fixtureRoot, rootDirectory, { recursive: true });
  linkFixtureConfigPackage(rootDirectory);
  const configPath = join(rootDirectory, "studio.config.ts");
  const appRoot = join(rootDirectory, "studio-host");

  const scaffolded = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  if (!scaffolded.ok || scaffolded.changedFiles.length !== 5) {
    fail("Initial local host app scaffold did not create the expected files.");
  }

  const secondScaffold = await scaffoldStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  if (!secondScaffold.ok || secondScaffold.changedFiles.length !== 0) {
    fail("Second local host app scaffold was not clean.");
  }

  const metadataPath = join(appRoot, ".flexweave-studio-app.json");
  const metadata = JSON.parse(readFileSync(metadataPath, "utf-8")) as Record<string, unknown>;
  writeFileSync(metadataPath, `${JSON.stringify({ ...metadata, version: 0 }, null, 2)}\n`);

  const migrated = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  if (!migrated.ok || migrated.applied.length !== 1) {
    fail("Local host app migration did not apply the supported metadata update.");
  }

  const secondMigration = await migrateStudioProject({
    appRoot: "studio-host",
    configPath,
  });
  if (!secondMigration.ok || secondMigration.changedFiles.length !== 0) {
    fail("Second local host app migration was not clean.");
  }

  linkHostAppPackages(appRoot);
  const verified = await verifyStudioHostApp({
    appRoot: "studio-host",
    configPath,
  });
  if (!verified.ok || verified.command?.exitCode !== 0) {
    fail(
      [
        "Local host app verification failed.",
        ...verified.diagnostics.map((diagnostic) => diagnostic.message),
        verified.command?.stderr,
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }
} finally {
  rmSync(rootDirectory, { force: true, recursive: true });
}

if (failures.length > 0) {
  console.error(
    failures.map((message) => `${relative(root, rootDirectory)}: ${message}`).join("\n"),
  );
  process.exit(1);
}

console.log("Studio host app verification passed.");
