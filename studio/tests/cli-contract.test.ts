import { join, resolve } from "node:path";
import { expect, test } from "bun:test";

const studioRoot = resolve(import.meta.dirname, "..");
const repoRoot = resolve(studioRoot, "..");
const fixtureConfigPath = join(studioRoot, "tests/fixtures/minimal/studio.config.ts");
const workflowCommands = [
  "validate",
  "describe",
  "list",
  "show",
  "plan",
  "scaffold",
  "codegen",
  "verify",
  "migrate",
];

const runCli = async (args: string[], cwd = studioRoot) => {
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

const parseJsonOutput = (stdout: string) => JSON.parse(stdout) as Record<string, unknown>;

test("CLI help advertises the Phase 3 public Studio contract", async () => {
  const rootHelp = await runCli(["--help"]);
  expect(rootHelp.exitCode).toBe(0);
  expect(rootHelp.stderr).toBe("");

  for (const command of workflowCommands) {
    expect(rootHelp.stdout).toContain(`  ${command}`);
  }

  const subcommandHelps = await Promise.all(
    workflowCommands.map(async (command) => ({
      command,
      help: await runCli([command, "--help"]),
    })),
  );

  for (const { command, help } of subcommandHelps) {
    expect(help.exitCode).toBe(0);
    expect(help.stderr).toBe("");
    expect(help.stdout).toContain(`flexweave-studio ${command}`);
  }

  const helpCorpus = [rootHelp.stdout, ...subcommandHelps.map(({ help }) => help.stdout)].join(
    "\n",
  );
  for (const phrase of [
    "flexweave-studio",
    "@flexweave/studio",
    "defineStudioConfig",
    "Studio catalog",
    "Studio project config",
    "generated mechanics definitions",
    "runtime hooks",
    "consumer runtime",
    "consumer project",
  ]) {
    expect(helpCorpus).toContain(phrase);
  }
});

test("CLI human output covers config loading success and workflow failure", async () => {
  const valid = await runCli(["validate", "--config", fixtureConfigPath]);
  expect(valid.exitCode).toBe(0);
  expect(valid.stderr).toBe("");
  expect(valid.stdout).toContain("Studio catalog valid.");
  expect(valid.stdout).toContain("Records checked: 6.");
  expect(valid.stdout).toContain(`Config: ${fixtureConfigPath}.`);

  const unknownKind = await runCli(["list", "unknown", "--config", fixtureConfigPath]);
  expect(unknownKind.exitCode).toBe(1);
  expect(unknownKind.stdout).toBe("");
  expect(unknownKind.stderr).toContain('error: Unknown Studio catalog record kind "unknown".');
});

test("CLI JSON output covers success, config loading failure, and workflow failure", async () => {
  const valid = await runCli(["validate", "--json", "--config", fixtureConfigPath]);
  expect(valid.exitCode).toBe(0);
  expect(valid.stderr).toBe("");
  expect(parseJsonOutput(valid.stdout)).toMatchObject({
    command: "validate",
    ok: true,
    recordCount: 6,
  });

  const missingConfig = await runCli(["validate", "--json"], join(repoRoot, "docs"));
  expect(missingConfig.exitCode).toBe(1);
  expect(missingConfig.stderr).toBe("");
  expect(parseJsonOutput(missingConfig.stdout)).toMatchObject({
    command: "validate",
    diagnostics: [{ code: "missing-config", severity: "error" }],
    ok: false,
    recordCount: 0,
  });

  const badTarget = await runCli([
    "codegen",
    "--json",
    "--target",
    "unknown",
    "--config",
    fixtureConfigPath,
  ]);
  expect(badTarget.exitCode).toBe(1);
  expect(badTarget.stderr).toBe("");
  expect(parseJsonOutput(badTarget.stdout)).toMatchObject({
    command: "codegen",
    diagnostics: [{ code: "unknown-codegen-target", severity: "error" }],
    ok: false,
  });
});

test("CLI rejects invalid arguments before running workflows", async () => {
  const unsupportedFlag = await runCli([
    "validate",
    "--target",
    "abilities",
    "--json",
    "--config",
    fixtureConfigPath,
  ]);
  expect(unsupportedFlag.exitCode).toBe(1);
  expect(unsupportedFlag.stderr).toBe("");
  expect(parseJsonOutput(unsupportedFlag.stdout)).toMatchObject({
    command: "flexweave-studio",
    diagnostics: [
      {
        code: "invalid-arguments",
        message: "Option --target is not supported by flexweave-studio validate.",
        severity: "error",
      },
    ],
    ok: false,
  });

  const unknownFlag = await runCli(["validate", "--unknown", "--config", fixtureConfigPath]);
  expect(unknownFlag.exitCode).toBe(1);
  expect(unknownFlag.stdout).toBe("");
  expect(unknownFlag.stderr).toContain("error: Unknown option --unknown.");

  const unexpectedArgument = await runCli(["validate", "abilities", "--config", fixtureConfigPath]);
  expect(unexpectedArgument.exitCode).toBe(1);
  expect(unexpectedArgument.stdout).toBe("");
  expect(unexpectedArgument.stderr).toContain('error: Unexpected argument "abilities".');
});
