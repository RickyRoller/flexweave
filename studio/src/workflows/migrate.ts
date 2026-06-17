import { existsSync } from "node:fs";

import type { ResolvedStudioProjectConfig, StudioDiagnostic } from "../config/schema";
import { STUDIO_HOST_APP_SCAFFOLD_VERSION } from "./constants";
import {
  changedFiles,
  hostAppMetadataForScaffold,
  hostAppMetadataPath,
  hostAppPackagePath,
  hostAppRoot,
  manualFollowUps,
  prepareHostAppScaffoldWrite,
  readHostAppMetadata,
  readHostAppMetadataVersion,
  readHostAppPackageDependencies,
} from "./host-app";
import {
  hasErrorDiagnostic,
  resolveWorkflowConfig,
  workflowError,
  workflowWarning,
} from "./shared";
import type {
  MigrateStudioProjectResult,
  ScaffoldStudioHostAppOptions,
  StudioMigrationCheckResult,
  StudioMigrationCheckStatus,
} from "./types";

const migrationCheck = (
  input: Omit<
    StudioMigrationCheckResult,
    "applied" | "changedFiles" | "diagnostics" | "manualFollowUps" | "skipped" | "status"
  > & {
    applied?: readonly string[];
    changedFiles?: readonly string[];
    diagnostics?: readonly StudioDiagnostic[];
    manualFollowUps?: readonly string[];
    skipped?: readonly string[];
  },
): StudioMigrationCheckResult => {
  const applied = [...(input.applied ?? [])];
  const changedFilePaths = [...(input.changedFiles ?? [])];
  const diagnostics = [...(input.diagnostics ?? [])];
  const followUps = [...(input.manualFollowUps ?? [])];
  const skipped = [...(input.skipped ?? [])];
  let status: StudioMigrationCheckStatus = "skipped";
  if (hasErrorDiagnostic(diagnostics)) {
    status = "failed";
  } else if (applied.length > 0 || changedFilePaths.length > 0) {
    status = "applied";
  }

  return {
    ...input,
    applied,
    changedFiles: changedFilePaths,
    diagnostics,
    manualFollowUps: followUps,
    skipped,
    status,
  };
};

const runHostAppMigration = (
  config: ResolvedStudioProjectConfig,
  root: string,
): StudioMigrationCheckResult => {
  if (!existsSync(hostAppMetadataPath(root))) {
    return migrationCheck({
      name: "host-app-scaffold",
      skipped: ["No local host app scaffold metadata found."],
      targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    });
  }

  const currentVersion = readHostAppMetadataVersion(root) ?? 0;
  if (currentVersion > STUDIO_HOST_APP_SCAFFOLD_VERSION) {
    const followUp = `Unsupported local host app scaffold version ${currentVersion}; active Studio supports ${STUDIO_HOST_APP_SCAFFOLD_VERSION}.`;
    return migrationCheck({
      currentVersion,
      diagnostics: [
        workflowError(
          "unsupported-host-app-scaffold-version",
          followUp,
          hostAppMetadataPath(root),
          "Upgrade Flexweave Studio or recreate the local host app scaffold manually.",
        ),
      ],
      manualFollowUps: [followUp],
      name: "host-app-scaffold",
      targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    });
  }

  if (currentVersion === STUDIO_HOST_APP_SCAFFOLD_VERSION) {
    return migrationCheck({
      currentVersion,
      name: "host-app-scaffold",
      skipped: ["Local host app scaffold is current."],
      targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
    });
  }

  const writeSession = prepareHostAppScaffoldWrite(config, root, {
    requireDefaultProjectAdapter: true,
    updateMetadata: true,
  });
  const files = writeSession.write();
  const followUps = manualFollowUps(files);

  return migrationCheck({
    applied: [`host app scaffold ${currentVersion} -> ${STUDIO_HOST_APP_SCAFFOLD_VERSION}`],
    changedFiles: changedFiles(files),
    currentVersion,
    diagnostics: followUps.map((followUp) =>
      workflowWarning("host-app-manual-follow-up", followUp),
    ),
    manualFollowUps: followUps,
    name: "host-app-scaffold",
    targetVersion: STUDIO_HOST_APP_SCAFFOLD_VERSION,
  });
};

const runHostAppPackageRefCheck = (root: string): StudioMigrationCheckResult => {
  if (!existsSync(hostAppMetadataPath(root))) {
    return migrationCheck({
      name: "host-app-package-refs",
      skipped: ["No local host app scaffold metadata found."],
    });
  }

  const metadata = hostAppMetadataForScaffold(readHostAppMetadata(root));
  const dependencies = readHostAppPackageDependencies(root);
  if (!dependencies) {
    const followUp = `Local host app package manifest is missing or malformed at ${hostAppPackagePath(root)}.`;
    return migrationCheck({
      diagnostics: [
        workflowError(
          "unsupported-host-app-package-ref",
          followUp,
          hostAppPackagePath(root),
          "Restore package.json or regenerate the local host app scaffold before rerunning migrate.",
        ),
      ],
      manualFollowUps: [followUp],
      name: "host-app-package-refs",
    });
  }

  const missingRefs = [
    ...new Set(
      Object.values(metadata.packageRefs).filter(
        (packageName) => dependencies[packageName] === undefined,
      ),
    ),
  ];

  if (missingRefs.length === 0) {
    return migrationCheck({
      name: "host-app-package-refs",
      skipped: ["Local host app package refs are supported."],
    });
  }

  const followUps = missingRefs.map(
    (packageName) =>
      `Local host app package metadata references "${packageName}", but package.json does not declare it as a dependency.`,
  );

  return migrationCheck({
    diagnostics: followUps.map((followUp) =>
      workflowError(
        "unsupported-host-app-package-ref",
        followUp,
        hostAppPackagePath(root),
        "Add the package dependency or update .flexweave-studio-app.json packageRefs before rerunning migrate.",
      ),
    ),
    manualFollowUps: followUps,
    name: "host-app-package-refs",
  });
};

const runExtensionMigrations = async (
  config: ResolvedStudioProjectConfig,
  appRoot: string,
): Promise<StudioMigrationCheckResult[]> => {
  const checks: StudioMigrationCheckResult[] = [];
  const extensions = [...config.extensions].toSorted((left, right) =>
    left.id.localeCompare(right.id),
  );

  for (const extension of extensions) {
    const migrations = [...(extension.migrations ?? [])].toSorted((left, right) =>
      left.id.localeCompare(right.id),
    );
    for (const migration of migrations) {
      const name = `extension:${extension.id}:${migration.id}`;
      try {
        const result = await migration.migrate({ appRoot, config });
        checks.push(
          migrationCheck({
            applied: result.applied,
            changedFiles: result.changedFiles,
            currentVersion: migration.fromVersion,
            diagnostics: result.diagnostics,
            extensionId: extension.id,
            manualFollowUps: result.manualFollowUps,
            name,
            skipped: result.skipped,
            targetVersion: migration.toVersion,
          }),
        );
      } catch (error) {
        checks.push(
          migrationCheck({
            currentVersion: migration.fromVersion,
            diagnostics: [
              workflowError(
                "extension-migration-failed",
                error instanceof Error
                  ? `Studio extension "${extension.id}" migration "${migration.id}" failed: ${error.message}`
                  : `Studio extension "${extension.id}" migration "${migration.id}" failed.`,
              ),
            ],
            extensionId: extension.id,
            manualFollowUps: [
              `Review extension migration "${extension.id}:${migration.id}" before rerunning migrate.`,
            ],
            name,
            targetVersion: migration.toVersion,
          }),
        );
      }
    }
  }

  return checks;
};

export const migrateStudioProject = async (
  options: ScaffoldStudioHostAppOptions = {},
): Promise<MigrateStudioProjectResult> => {
  const resolved = await resolveWorkflowConfig(options);
  if (!resolved.ok) {
    return {
      applied: [],
      changedFiles: [],
      checks: [
        migrationCheck({
          diagnostics: resolved.diagnostics,
          name: "config",
        }),
      ],
      diagnostics: resolved.diagnostics,
      manualFollowUps: [],
      ok: false,
      skipped: [],
    };
  }

  const root = hostAppRoot(resolved.config, options.appRoot);
  const checks = [
    runHostAppMigration(resolved.config, root),
    runHostAppPackageRefCheck(root),
    ...(await runExtensionMigrations(resolved.config, root)),
  ];
  const diagnostics = checks.flatMap((check) => check.diagnostics);

  return {
    applied: checks.flatMap((check) => check.applied),
    changedFiles: checks.flatMap((check) => check.changedFiles),
    checks,
    diagnostics,
    manualFollowUps: checks.flatMap((check) => check.manualFollowUps),
    ok: diagnostics.every((diagnostic) => diagnostic.severity !== "error"),
    skipped: checks.flatMap((check) => check.skipped),
  };
};
