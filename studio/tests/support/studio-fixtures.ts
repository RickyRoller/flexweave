import { cpSync, existsSync, mkdirSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { isAbsolute, join, relative, resolve } from "node:path";

export const studioRoot = resolve(import.meta.dirname, "../..");
export const repoRoot = resolve(studioRoot, "..");
export const fixtureRoot = join(studioRoot, "tests/fixtures/minimal");
export const fixtureConfigPath = join(fixtureRoot, "studio.config.ts");
export const generatedTargetFixtureConfigPath = join(fixtureRoot, "generated-target.config.ts");
export const extensionFixtureRoot = join(studioRoot, "tests/fixtures/extension-sources");
export const extensionFixtureConfigPath = join(extensionFixtureRoot, "studio.config.ts");

export const pathContains = (parent: string, child: string) => {
  const childRelativeToParent = relative(parent, child);
  return (
    childRelativeToParent === "" ||
    (!childRelativeToParent.startsWith("..") && !isAbsolute(childRelativeToParent))
  );
};

export const linkWorkspacePackage = (root: string) => {
  const scopeRoot = join(root, "node_modules/@flexweave");
  mkdirSync(scopeRoot, { recursive: true });
  const linkPath = join(scopeRoot, "studio");
  if (!existsSync(linkPath)) {
    symlinkSync(studioRoot, linkPath, "dir");
  }
};

export const linkHostAppPackages = (root: string) => {
  const scopeRoot = join(root, "node_modules/@flexweave");
  mkdirSync(scopeRoot, { recursive: true });

  const packageLinks = [
    ["studio", studioRoot],
    ["studio-app", join(studioRoot, "app")],
  ];
  for (const [name, target] of packageLinks) {
    const linkPath = join(scopeRoot, name);
    if (!existsSync(linkPath)) {
      symlinkSync(target, linkPath, "dir");
    }
  }

  const bunTypesLink = join(root, "node_modules/bun-types");
  if (!existsSync(bunTypesLink)) {
    symlinkSync(join(studioRoot, "node_modules/bun-types"), bunTypesLink, "dir");
  }

  const binRoot = join(root, "node_modules/.bin");
  mkdirSync(binRoot, { recursive: true });
  const tscLink = join(binRoot, "tsc");
  if (!existsSync(tscLink)) {
    symlinkSync(join(repoRoot, "node_modules/typescript/bin/tsc"), tscLink);
  }
};

export const copyMinimalFixture = () => {
  const root = join(tmpdir(), `studio-fixture-${crypto.randomUUID()}`);
  mkdirSync(root, { recursive: true });
  cpSync(fixtureRoot, root, { recursive: true });
  linkWorkspacePackage(root);
  return root;
};

export const copyExtensionFixture = () => {
  const root = join(tmpdir(), `studio-extension-fixture-${crypto.randomUUID()}`);
  mkdirSync(root, { recursive: true });
  cpSync(extensionFixtureRoot, root, { recursive: true });
  linkWorkspacePackage(root);
  return root;
};

export const copyFixtureTree = () => {
  const root = join(tmpdir(), `studio-fixture-tree-${crypto.randomUUID()}`);
  mkdirSync(root, { recursive: true });
  cpSync(fixtureRoot, join(root, "minimal"), { recursive: true });
  cpSync(extensionFixtureRoot, join(root, "extension-sources"), { recursive: true });
  linkWorkspacePackage(root);
  return root;
};

export const runStudioCli = async (args: string[], cwd = studioRoot) => {
  const proc = Bun.spawn(["bun", join(studioRoot, "src/cli/main.ts"), ...args], {
    cwd,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return { exitCode, stderr, stdout };
};
