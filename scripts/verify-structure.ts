import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = new URL("..", import.meta.url).pathname;

const requiredPaths = [
  "Cargo.toml",
  "package.json",
  "core/Cargo.toml",
  "core/CONTEXT.md",
  "core/README.md",
  "core/src/lib.rs",
  "core/tests",
  "studio/package.json",
  "studio/app/package.json",
];

const failures: string[] = [];

for (const requiredPath of requiredPaths) {
  if (!existsSync(join(root, requiredPath))) {
    failures.push(`Missing required path: ${requiredPath}`);
  }
}

const cargoToml = readFileSync(join(root, "Cargo.toml"), "utf-8");
if (!/members\s*=\s*\[\s*"core"\s*\]/.test(cargoToml)) {
  failures.push('Root Cargo workspace must include only the "core" member.');
}

const rootPackage = JSON.parse(readFileSync(join(root, "package.json"), "utf-8"));
const { workspaces } = rootPackage;
if (
  !Array.isArray(workspaces) ||
  !workspaces.includes("studio") ||
  !workspaces.includes("studio/app")
) {
  failures.push('Root package workspaces must include "studio" and "studio/app".');
}

const forbiddenDirectoryNames = new Set(["examples"]);
const ignoredDirectoryNames = new Set([".git", "node_modules", "target", "dist"]);
const guardedRoots = ["core", "studio"];

const visitDirectory = (directory: string) => {
  for (const entry of readdirSync(directory)) {
    if (ignoredDirectoryNames.has(entry)) {
      continue;
    }

    const path = join(directory, entry);
    const stats = statSync(path);
    if (!stats.isDirectory()) {
      continue;
    }

    if (forbiddenDirectoryNames.has(entry)) {
      failures.push(`Forbidden directory: ${relative(root, path)}`);
      continue;
    }

    visitDirectory(path);
  }
};

for (const guardedRoot of guardedRoots) {
  visitDirectory(join(root, guardedRoot));
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Structure guard passed.");
