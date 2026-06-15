import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, isAbsolute, join, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { loadStudioConfig } from "../studio/src/config/load";
import { codegenStudioProject } from "../studio/src/workflows";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const studioRoot = join(root, "studio");
const studioAppRoot = join(studioRoot, "app");
const fixtureRoot = join(studioRoot, "tests/fixtures/minimal");
const fixtureConfigPath = join(fixtureRoot, "studio.config.ts");
const studioRetiredTermScanRoots = [
  "studio/package.json",
  "studio/README.md",
  "studio/CONTEXT.md",
  "studio/app",
  "studio/docs",
  "studio/src",
  "studio/tests",
];
const failures: string[] = [];

const fail = (message: string) => {
  failures.push(message);
};

const readJson = (path: string) => JSON.parse(readFileSync(path, "utf-8"));

const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

const listFilesRecursive = (directory: string): string[] => {
  if (!existsSync(directory)) {
    return [];
  }

  const files: string[] = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === "dist") {
      continue;
    }

    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursive(path));
    } else if (entry.isFile()) {
      files.push(path);
    }
  }
  return files.toSorted();
};

const listDirectoriesRecursive = (directory: string): string[] => {
  if (!existsSync(directory)) {
    return [];
  }

  const directories: string[] = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === "dist") {
      continue;
    }

    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      directories.push(path, ...listDirectoriesRecursive(path));
    }
  }
  return directories.toSorted();
};

const assertPackageMetadata = () => {
  const packageJson = readJson(join(studioRoot, "package.json"));
  if (packageJson.name !== "@flexweave/studio") {
    fail('studio/package.json name must remain "@flexweave/studio".');
  }

  const binNames = Object.keys(packageJson.bin ?? {});
  if (binNames.length !== 1 || binNames[0] !== "flexweave-studio") {
    fail("studio/package.json must expose only the flexweave-studio bin.");
  }

  const exportNames = Object.keys(packageJson.exports ?? {}).toSorted();
  const expectedExports = ["./codegen", "./config", "./config/load", "./workflows"];
  if (JSON.stringify(exportNames) !== JSON.stringify(expectedExports)) {
    fail(`studio/package.json exports must be ${expectedExports.join(", ")}.`);
  }
};

const assertPackageImportBoundary = () => {
  const forbiddenPatterns = [
    {
      label: "Studio app import",
      pattern: /(^|\n)\s*import\s+.*@flexweave\/studio-app|(^|\n)\s*import\s+.*\.\.\/app/,
    },
    { label: "consumer app source", pattern: /(?:^|["'])apps\// },
    { label: "old package source", pattern: /packages\/game-data|gamedata\.config/ },
    { label: "old package import", pattern: /@forge\// },
    { label: "old Studio app name", pattern: /atlas-design-studio/ },
  ];

  for (const scanRoot of [join(studioRoot, "src"), join(studioRoot, "tests")]) {
    for (const file of listFilesRecursive(scanRoot)) {
      if (![".json", ".md", ".rs", ".ts", ".tsx"].some((extension) => file.endsWith(extension))) {
        continue;
      }

      const content = readFileSync(file, "utf-8");
      for (const { label, pattern } of forbiddenPatterns) {
        if (pattern.test(content)) {
          fail(`${relative(root, file)} crosses Studio package boundary: ${label}.`);
        }
      }
    }
  }
};

const assertAppPackageBoundary = () => {
  const packageJson = readJson(join(studioAppRoot, "package.json"));
  if (packageJson.name !== "@flexweave/studio-app") {
    fail('studio/app/package.json name must remain "@flexweave/studio-app".');
  }
  if (packageJson.dependencies?.["@flexweave/studio"] !== "workspace:*") {
    fail("studio/app/package.json must depend on @flexweave/studio.");
  }

  const forbiddenPatterns = [
    { label: "consumer app source", pattern: /(?:^|["'])apps\// },
    { label: "old package source", pattern: /packages\/game-data|gamedata\.config/ },
    { label: "old package import", pattern: /@forge\// },
    { label: "old Studio app name", pattern: /atlas-design-studio/ },
  ];

  for (const scanRoot of [join(studioAppRoot, "src"), join(studioAppRoot, "tests")]) {
    for (const file of listFilesRecursive(scanRoot)) {
      if (![".json", ".md", ".ts", ".tsx"].some((extension) => file.endsWith(extension))) {
        continue;
      }

      const content = readFileSync(file, "utf-8");
      for (const { label, pattern } of forbiddenPatterns) {
        if (pattern.test(content)) {
          fail(`${relative(root, file)} crosses Studio app package boundary: ${label}.`);
        }
      }
    }
  }
};

const assertRetiredTermInventory = () => {
  const result = spawnSync(
    process.execPath,
    [join(root, "scripts/verify-retired-terms.ts"), ...studioRetiredTermScanRoots],
    {
      cwd: root,
      encoding: "utf-8",
    },
  );

  if (result.error) {
    fail(`Studio retired-term inventory scan failed to start: ${result.error.message}`);
    return;
  }

  if (result.status !== 0) {
    const report = [result.stdout.trim(), result.stderr.trim()].filter(Boolean).join("\n");
    fail(`Studio retired-term inventory scan failed:\n${report}`);
  }
};

const assertFixtureBoundary = async () => {
  if (existsSync(join(studioRoot, "examples"))) {
    fail("studio/examples must not exist.");
  }

  if (!existsSync(fixtureConfigPath)) {
    fail("Minimal Studio fixture must include studio.config.ts.");
    return;
  }

  const fixturesDir = join(studioRoot, "tests/fixtures");
  const fixtureNames = existsSync(fixturesDir)
    ? readdirSync(fixturesDir, { withFileTypes: true })
        .filter((entry) => entry.isDirectory())
        .map((entry) => entry.name)
        .toSorted()
    : [];
  if (JSON.stringify(fixtureNames) !== JSON.stringify(["minimal"])) {
    fail("studio/tests/fixtures/minimal must be the only Studio-owned fixture.");
  }

  const catalogDirs = listDirectoriesRecursive(join(studioRoot, "tests"))
    .filter((path) => basename(path) === "catalog")
    .map((path) => relative(studioRoot, path));
  if (JSON.stringify(catalogDirs) !== JSON.stringify(["tests/fixtures/minimal/catalog"])) {
    fail("Minimal fixture must be the only Studio-owned catalog fixture.");
  }

  const loaded = await loadStudioConfig({ configPath: fixtureConfigPath });
  if (!loaded.ok || !loaded.config) {
    fail(
      `Minimal fixture config failed to load: ${loaded.diagnostics
        .map((diagnostic) => diagnostic.message)
        .join("; ")}`,
    );
    return;
  }

  const outputDirs = Object.values(loaded.config.paths.codegen.outputDirs);
  const hookDirs = [loaded.config.paths.hooks.dir, loaded.config.paths.hooks.testStubsDir].filter(
    (path): path is string => typeof path === "string",
  );
  const ownedPaths = [...outputDirs, ...hookDirs];

  for (const ownedPath of ownedPaths) {
    if (!pathContains(fixtureRoot, ownedPath)) {
      fail(`Minimal fixture owned path escapes the fixture root: ${relative(root, ownedPath)}.`);
    }
  }

  for (let leftIndex = 0; leftIndex < ownedPaths.length; leftIndex += 1) {
    for (let rightIndex = leftIndex + 1; rightIndex < ownedPaths.length; rightIndex += 1) {
      const left = ownedPaths[leftIndex];
      const right = ownedPaths[rightIndex];
      if (left === right || pathContains(left, right) || pathContains(right, left)) {
        fail(
          `Minimal fixture owned paths must be distinct siblings: ${relative(
            fixtureRoot,
            left,
          )} and ${relative(fixtureRoot, right)}.`,
        );
      }
    }
  }
};

const linkWorkspacePackage = (rootDirectory: string) => {
  const scopeRoot = join(rootDirectory, "node_modules/@flexweave");
  mkdirSync(scopeRoot, { recursive: true });
  const linkPath = join(scopeRoot, "studio");
  if (!existsSync(linkPath)) {
    symlinkSync(studioRoot, linkPath, "dir");
  }
};

const copyFixture = () => {
  const rootDirectory = join(tmpdir(), `studio-boundary-${crypto.randomUUID()}`);
  mkdirSync(rootDirectory, { recursive: true });
  cpSync(fixtureRoot, rootDirectory, { recursive: true });
  linkWorkspacePackage(rootDirectory);
  return rootDirectory;
};

const snapshotOutsideOwnedPaths = (rootDirectory: string, ownedPaths: string[]) => {
  const snapshot = new Map<string, string>();
  for (const file of listFilesRecursive(rootDirectory)) {
    if (ownedPaths.some((ownedPath) => pathContains(ownedPath, file))) {
      continue;
    }

    if (statSync(file).isFile()) {
      snapshot.set(relative(rootDirectory, file), readFileSync(file, "utf-8"));
    }
  }
  return snapshot;
};

const assertSnapshotsMatch = (before: Map<string, string>, after: Map<string, string>) => {
  for (const [path, value] of before) {
    if (after.get(path) !== value) {
      fail(`Codegen changed an unconfigured fixture path: ${path}.`);
    }
  }

  for (const path of after.keys()) {
    if (!before.has(path)) {
      fail(`Codegen created an unconfigured fixture path: ${path}.`);
    }
  }
};

const assertGeneratedWritesStayConfigured = async () => {
  const rootDirectory = copyFixture();
  try {
    const configPath = join(rootDirectory, "studio.config.ts");
    const loaded = await loadStudioConfig({ configPath });
    if (!loaded.ok || !loaded.config) {
      fail("Copied minimal fixture config failed to load.");
      return;
    }

    const outputDirs = Object.values(loaded.config.paths.codegen.outputDirs);
    const hookDirs = [loaded.config.paths.hooks.dir, loaded.config.paths.hooks.testStubsDir].filter(
      (path): path is string => typeof path === "string",
    );
    const ownedPaths = [...outputDirs, ...hookDirs];
    const hookPath = join(rootDirectory, "runtime-hooks/minimal_execution.rs");
    const hookValue = "//! consumer-owned hook\n\npub fn minimal_execution() {}\n";

    rmSync(join(rootDirectory, "generated/abilities/generated.rs"), { force: true });
    rmSync(join(rootDirectory, "generated-hook-tests/minimal_execution.rs"), { force: true });
    writeFileSync(hookPath, hookValue);

    const outsideBefore = snapshotOutsideOwnedPaths(rootDirectory, ownedPaths);
    const result = await codegenStudioProject({ configPath });
    const outsideAfter = snapshotOutsideOwnedPaths(rootDirectory, ownedPaths);

    assertSnapshotsMatch(outsideBefore, outsideAfter);

    if (!result.ok) {
      fail(
        `Codegen boundary fixture failed: ${result.diagnostics
          .map((diagnostic) => diagnostic.message)
          .join("; ")}`,
      );
    }

    if (readFileSync(hookPath, "utf-8") !== hookValue) {
      fail("Codegen overwrote an existing runtime hook implementation.");
    }

    for (const target of result.targets) {
      for (const file of target.files) {
        if (!outputDirs.some((outputDir) => pathContains(outputDir, file.path))) {
          fail(`Generated file escaped configured output dirs: ${relative(root, file.path)}.`);
        }
      }
    }

    for (const hook of result.hooks) {
      if (!hookDirs.some((hookDir) => pathContains(hookDir, hook.path))) {
        fail(`Runtime hook file escaped configured hook dirs: ${relative(root, hook.path)}.`);
      }
    }
  } finally {
    rmSync(rootDirectory, { force: true, recursive: true });
  }
};

assertPackageMetadata();
assertPackageImportBoundary();
assertAppPackageBoundary();
assertRetiredTermInventory();
await assertFixtureBoundary();
await assertGeneratedWritesStayConfigured();

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Studio content-boundary guard passed.");
