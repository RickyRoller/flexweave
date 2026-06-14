#!/usr/bin/env bun

import {
  codegenStudioProject,
  describeStudioCatalog,
  listStudioCatalogRecords,
  migrateStudioProject,
  planStudioMechanic,
  scaffoldStudioMechanic,
  showStudioCatalogRecord,
  studioWorkflowNames,
  validateStudioCatalog,
  verifyStudioProject,
} from "../workflows";
import type {
  CodegenStudioResult,
  DescribeStudioCatalogResult,
  ListStudioCatalogRecordsResult,
  MigrateStudioProjectResult,
  PlanStudioMechanicOptions,
  PlanStudioMechanicResult,
  ScaffoldStudioMechanicResult,
  ShowStudioCatalogRecordResult,
  StudioWorkflowResult,
  ValidateStudioCatalogResult,
  VerifyStudioProjectResult,
} from "../workflows";

interface ParsedArgs {
  command: string;
  flags: Record<string, string | boolean | undefined>;
  positionals: string[];
}

interface CommandExecution {
  lines: string[];
  result: StudioWorkflowResult;
}

type CommandHandler = (parsed: ParsedArgs) => Promise<CommandExecution>;

const commandHelp: Record<string, string[]> = {
  codegen: [
    "flexweave-studio codegen [--check] [--target name|a,b] [--config path] [--json] [--quiet]",
    "Refresh or check generated mechanics definitions.",
  ],
  describe: [
    "flexweave-studio describe [kind] [--config path] [--json] [--quiet]",
    "Describe Studio catalog record schemas.",
  ],
  list: [
    "flexweave-studio list <kind> [--filter text] [--config path] [--json] [--quiet]",
    "List Studio catalog records.",
  ],
  migrate: [
    "flexweave-studio migrate [--config path] [--json] [--quiet]",
    "Run Studio project migrations after package updates.",
  ],
  plan: [
    "flexweave-studio plan --archetype mechanic --id <id> --name <name> [--params json] [--config path] [--json] [--quiet]",
    "Preview Studio mechanic scaffolding writes.",
  ],
  scaffold: [
    "flexweave-studio scaffold --archetype mechanic --id <id> --name <name> [--params json] [--allow-existing] [--config path] [--json] [--quiet]",
    "Write Studio mechanic records and runtime hook stubs transactionally.",
  ],
  show: [
    "flexweave-studio show <kind> <id> [--config path] [--json] [--quiet]",
    "Show one Studio catalog record.",
  ],
  validate: [
    "flexweave-studio validate [--config path] [--json] [--quiet]",
    "Validate the configured Studio catalog.",
  ],
  verify: [
    "flexweave-studio verify [--fast] [--config path] [--json] [--quiet]",
    "Run validation, generated freshness checks, and configured verification commands.",
  ],
};

const rootHelp = () => [
  "flexweave-studio <command> [--config path] [--json] [--quiet]",
  "",
  "Commands:",
  ...studioWorkflowNames.map((command) => `  ${command}`),
  "",
  'Run "flexweave-studio <command> --help" for command details.',
];

const parseArgs = (argv: string[]): ParsedArgs => {
  const [command = "help", ...rest] = argv;
  const flags: ParsedArgs["flags"] = {};
  const positionals: string[] = [];

  for (let index = 0; index < rest.length; index += 1) {
    const arg = rest[index];
    if (!arg.startsWith("--")) {
      positionals.push(arg);
      continue;
    }

    const name = arg.slice(2);
    if (["allow-existing", "check", "fast", "help", "json", "quiet"].includes(name)) {
      flags[name] = true;
      continue;
    }

    const value = rest[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`Missing value for --${name}.`);
    }
    flags[name] = value;
    index += 1;
  }

  return { command, flags, positionals };
};

const flagString = (parsed: ParsedArgs, name: string): string | undefined => {
  const value = parsed.flags[name];
  return typeof value === "string" ? value : undefined;
};

const flagBoolean = (parsed: ParsedArgs, name: string): boolean => parsed.flags[name] === true;

const workflowOptions = (parsed: ParsedArgs) => ({
  configPath: flagString(parsed, "config"),
});

const parseParams = (value: string | undefined): Record<string, unknown> => {
  if (!value) {
    return {};
  }
  const parsed = JSON.parse(value);
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    throw new Error("--params must be a JSON object.");
  }
  return parsed as Record<string, unknown>;
};

const requireOption = (parsed: ParsedArgs, name: string) => {
  const value = flagString(parsed, name);
  if (!value) {
    throw new Error(`Missing required option --${name}.`);
  }
  return value;
};

const requirePositional = (parsed: ParsedArgs, index: number, label: string) => {
  const value = parsed.positionals[index];
  if (!value) {
    throw new Error(`Missing required ${label}.`);
  }
  return value;
};

const mechanicOptions = (parsed: ParsedArgs): PlanStudioMechanicOptions => ({
  ...workflowOptions(parsed),
  allowExisting: flagBoolean(parsed, "allow-existing"),
  archetype: requireOption(parsed, "archetype"),
  id: requireOption(parsed, "id"),
  name: requireOption(parsed, "name"),
  params: parseParams(flagString(parsed, "params")),
});

const printJson = (command: string, result: unknown) => {
  const body =
    typeof result === "object" && result !== null
      ? (result as Record<string, unknown>)
      : { value: result };
  console.log(JSON.stringify({ command, ...body }, null, 2));
};

const diagnosticLines = (result: StudioWorkflowResult) =>
  result.diagnostics.map((diagnostic) => {
    const location = diagnostic.path ? `${diagnostic.path}: ` : "";
    return `${diagnostic.severity}: ${location}${diagnostic.message}`;
  });

const printResult = (command: string, result: StudioWorkflowResult, lines: string[]) => {
  if (result.ok) {
    console.log(lines.join("\n"));
    return;
  }

  const diagnostics = diagnosticLines(result);
  console.error((diagnostics.length > 0 ? diagnostics : [`${command} failed.`]).join("\n"));
};

const formatValidate = (result: ValidateStudioCatalogResult) => [
  `Studio catalog ${result.ok ? "valid" : "invalid"}.`,
  `Records checked: ${result.recordCount}.`,
  ...(result.configPath ? [`Config: ${result.configPath}.`] : []),
];

const formatDescribe = (result: DescribeStudioCatalogResult) =>
  result.descriptions.flatMap((description) => [
    `${description.kind}: ${description.summary}`,
    `Fields: ${description.fields.join(", ")}`,
  ]);

const formatList = (result: ListStudioCatalogRecordsResult) => [
  `${result.kind}: ${result.records.length} record(s).`,
  ...result.records.map((record) => `${record.id} - ${record.label}`),
];

const formatShow = (result: ShowStudioCatalogRecordResult) =>
  result.record ? [JSON.stringify(result.record, null, 2)] : ["Record not found."];

const formatPlan = (result: PlanStudioMechanicResult) => [
  `Planned files: ${result.plannedFiles.length}.`,
  ...result.plannedFiles,
];

const formatScaffold = (result: ScaffoldStudioMechanicResult) => [
  result.rolledBack ? "Scaffold rolled back." : "Scaffold complete.",
  `Written files: ${result.writtenFiles.length}.`,
  ...result.writtenFiles,
];

const formatCodegen = (result: CodegenStudioResult) => [
  result.checked
    ? "Generated freshness check complete."
    : "Generated mechanics definitions refreshed.",
  ...result.targets.flatMap((target) => [
    `${target.label}: ${target.files.length} file(s).`,
    ...target.files.map((file) => `${file.status}: ${file.path}`),
  ]),
  ...result.hooks.map((hook) => `${hook.status}: ${hook.path}`),
];

const formatVerify = (result: VerifyStudioProjectResult) => [
  "Studio verify complete.",
  `Validation: ${result.validation.ok ? "ok" : "failed"}.`,
  `Generated freshness: ${result.codegen.ok ? "ok" : "failed"}.`,
  ...result.commands.map(
    (command) => `${command.exitCode === 0 ? "ok" : "failed"}: ${command.name}`,
  ),
];

const formatMigrate = (result: MigrateStudioProjectResult) => [
  "Studio project migrations complete.",
  `Applied: ${result.applied.length}.`,
  ...result.skipped,
];

const commandHandlers: Record<string, CommandHandler> = {
  codegen: async (parsed) => {
    const result = await codegenStudioProject({
      ...workflowOptions(parsed),
      check: flagBoolean(parsed, "check"),
      targets: flagString(parsed, "target")?.split(",").filter(Boolean),
    });
    return { lines: formatCodegen(result), result };
  },
  describe: async (parsed) => {
    const result = await describeStudioCatalog(parsed.positionals[0], workflowOptions(parsed));
    return { lines: formatDescribe(result), result };
  },
  list: async (parsed) => {
    const result = await listStudioCatalogRecords(requirePositional(parsed, 0, "record kind"), {
      ...workflowOptions(parsed),
      filter: flagString(parsed, "filter"),
    });
    return { lines: formatList(result), result };
  },
  migrate: async (parsed) => {
    const result = await migrateStudioProject(workflowOptions(parsed));
    return { lines: formatMigrate(result), result };
  },
  plan: async (parsed) => {
    const result = await planStudioMechanic(mechanicOptions(parsed));
    return { lines: formatPlan(result), result };
  },
  scaffold: async (parsed) => {
    const result = await scaffoldStudioMechanic(mechanicOptions(parsed));
    return { lines: formatScaffold(result), result };
  },
  show: async (parsed) => {
    const result = await showStudioCatalogRecord(
      requirePositional(parsed, 0, "record kind"),
      requirePositional(parsed, 1, "record id"),
      workflowOptions(parsed),
    );
    return { lines: formatShow(result), result };
  },
  validate: async (parsed) => {
    const result = await validateStudioCatalog(workflowOptions(parsed));
    return { lines: formatValidate(result), result };
  },
  verify: async (parsed) => {
    const result = await verifyStudioProject({
      ...workflowOptions(parsed),
      fast: flagBoolean(parsed, "fast"),
    });
    return { lines: formatVerify(result), result };
  },
};

const run = async (argv: string[]) => {
  const parsed = parseArgs(argv);
  const json = flagBoolean(parsed, "json");
  const quiet = flagBoolean(parsed, "quiet");

  if (parsed.command === "help" || parsed.command === "--help" || flagBoolean(parsed, "help")) {
    const lines =
      parsed.command in commandHelp && parsed.command !== "help"
        ? commandHelp[parsed.command]
        : rootHelp();
    console.log(lines.join("\n"));
    return 0;
  }

  const handler = commandHandlers[parsed.command];
  if (!handler) {
    throw new Error(`Unknown flexweave-studio command "${parsed.command}".`);
  }

  const { lines, result } = await handler(parsed);
  if (json) {
    printJson(parsed.command, result);
  } else if (!quiet) {
    printResult(parsed.command, result, lines);
  }

  return result.ok ? 0 : 1;
};

try {
  const exitCode = await run(Bun.argv.slice(2));
  process.exit(exitCode);
} catch (error) {
  const message = error instanceof Error ? error.message : "Command failed.";
  if (Bun.argv.includes("--json")) {
    printJson("flexweave-studio", {
      diagnostics: [
        {
          code: "invalid-arguments",
          message,
          severity: "error",
        },
      ],
      ok: false,
    });
  } else {
    console.error(`error: ${message}`);
  }
  process.exit(1);
}
