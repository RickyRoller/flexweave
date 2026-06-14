import { readdirSync, readFileSync, statSync } from "node:fs";
import { extname, join, relative } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const requestedPaths = process.argv.slice(2);
const scanRoots =
  requestedPaths.length > 0
    ? requestedPaths
    : ["README.md", "CONTEXT-MAP.md", "docs", "core", "studio"];

const ignoredDirectoryNames = new Set([".git", "node_modules", "target", "dist"]);
const scannedExtensions = new Set([".md", ".mdx", ".rs", ".toml", ".ts", ".tsx", ".json"]);

const retiredTerms = [
  { label: "Forge", pattern: /\bForge\b/ },
  { label: "Atlas", pattern: /\bAtlas\b/ },
  { label: "Player", pattern: /\bPlayers?\b/ },
  { label: "Tower", pattern: /\bTowers?\b/ },
  { label: "Map", pattern: /\bMaps?\b/ },
  { label: "Zone", pattern: /\bZones?\b/ },
  { label: "Inventory", pattern: /\bInventory\b/ },
  { label: "Equipment", pattern: /\bEquipment\b/ },
  { label: "Placement", pattern: /\bPlacement\b/ },
  { label: "source crate path", pattern: /crates\/flexweave/ },
];

interface Finding {
  path: string;
  line: number;
  term: string;
  text: string;
}

const findings: Finding[] = [];

const scanFile = (path: string) => {
  if (!scannedExtensions.has(extname(path))) {
    return;
  }

  const content = readFileSync(path, "utf-8");
  const lines = content.split(/\r?\n/);
  for (const [index, lineText] of lines.entries()) {
    for (const retiredTerm of retiredTerms) {
      if (retiredTerm.pattern.test(lineText)) {
        findings.push({
          line: index + 1,
          path: relative(root, path),
          term: retiredTerm.label,
          text: lineText.trim(),
        });
      }
    }
  }
};

const scanPath = (path: string) => {
  const stats = statSync(path);
  if (stats.isFile()) {
    scanFile(path);
    return;
  }

  if (!stats.isDirectory()) {
    return;
  }

  for (const entry of readdirSync(path)) {
    if (ignoredDirectoryNames.has(entry)) {
      continue;
    }

    scanPath(join(path, entry));
  }
};

for (const scanRoot of scanRoots) {
  scanPath(join(root, scanRoot));
}

if (findings.length > 0) {
  for (const finding of findings) {
    console.error(
      `${finding.path}:${finding.line}: retired term "${finding.term}" in: ${finding.text}`,
    );
  }
  process.exit(1);
}

console.log("Retired-term scan passed.");
