import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";

export interface FileSnapshot {
  existed: boolean;
  path: string;
  value?: string;
}

export const readTextIfExists = (path: string): string | undefined =>
  existsSync(path) ? readFileSync(path, "utf-8") : undefined;

export const writeTextFile = (path: string, value: string) => {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, value);
};

export const snapshotPaths = (paths: string[]): FileSnapshot[] =>
  paths.map((path) => ({
    existed: existsSync(path),
    path,
    value: readTextIfExists(path),
  }));

export const restoreSnapshots = (snapshots: FileSnapshot[]) => {
  for (const snapshot of snapshots) {
    if (snapshot.existed) {
      writeTextFile(snapshot.path, snapshot.value ?? "");
    } else if (existsSync(snapshot.path)) {
      rmSync(snapshot.path, { force: true, recursive: true });
    }
  }
};

export const listFilesRecursive = (root: string): string[] => {
  if (!existsSync(root)) {
    return [];
  }

  const files: string[] = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursive(path));
    } else if (entry.isFile()) {
      files.push(path);
    }
  }
  return files.toSorted();
};

export const displayPath = (base: string, path: string) => relative(base, path) || ".";
