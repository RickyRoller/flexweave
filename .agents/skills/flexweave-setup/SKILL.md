---
name: flexweave-setup
description: Set up Flexweave Studio in a consumer game repository by verifying the direct flexweave-studio CLI, adding minimal JSON config/ownership directories, and writing the required FLEXWEAVE.md integration map. Optionally wire Flexweave Core or runtime modules only when the user explicitly asks for that. Use when a user wants to install, adopt, integrate, configure, or onboard Flexweave in a repo. Do not use to author starter mechanics, sample catalog records, gameplay content, or runtime wiring unless requested separately.
---

# Flexweave Setup

## Workflow

Use this skill to turn an existing or blank game repo into a repo that future
agents can reliably operate with Flexweave.

1. Read existing context before changing files:
   - Repo root docs: `README*`, `AGENTS.md`, package manifests, workspace files.
   - Rust manifests: `Cargo.toml`, crate layout, runtime modules.
   - Existing data/codegen/docs directories.
   - If this is the Flexweave repo, read `CONTEXT-MAP.md`, `docs/how-to/use-core.md`, `docs/how-to/use-studio.md`, and `studio/docs/reference/cli.md`.
2. Identify the integration mode:
   - Studio codegen: default setup mode; repo needs catalog validation,
     generated mechanics definitions, runtime hook locations, and an
     integration map for later authoring.
   - Core runtime: opt-in only; runtime needs deterministic mechanics
     primitives imported into the game now.
   - Studio host app: opt-in only; repo also needs a local authoring UI shell.
3. Ask only for decisions that cannot be inferred safely:
   - Target runtime language/crate and package manager.
   - Where catalog source, generated Rust, runtime hooks, and hook tests should live.
   - Whether to create a local Studio host app now.
4. Verify or wire dependencies without changing repo tooling unnecessarily:
   - Studio workflows: prefer the direct `flexweave-studio` CLI. Do not create
     `package.json`, `bun.lock`, `node_modules`, or repo scripts only to run
     Studio from a non-JavaScript repo.
   - Rust runtime: add the `flexweave` crate only when the user asked for Core
     runtime integration or existing runtime code already imports it.
   - Host app: add local JavaScript package dependencies only when creating a
     host app or when the repo already uses JavaScript tooling.
5. Create or update `studio.config.json` when Studio is in scope. Include:
   - `catalogRoot`.
   - `codegen.outputDirs` for `abilities`, `effects`, `executions`, `modifiers`, `reference`, and `tags`.
   - `hooks.dir` and optional `hooks.testStubsDir`.
   - `rust.flexweaveModule` as the intended import path for later generated
     Rust/runtime integration. This config field does not require installing
     the Rust crate during setup.
   - `verify.commands` for the repo's existing checks.
6. Record recurring workflows in `FLEXWEAVE.md`:
   - Validate catalog.
   - Refresh codegen.
   - Check generated freshness.
   - Migrate after package updates.
   - Fast and full Studio verify.
   - Local host app dev/build scripts when host app exists.
7. Run the narrowest checks as each layer lands:
   - `flexweave-studio validate --config studio.config.json`.
   - `flexweave-studio codegen --config studio.config.json`.
   - `flexweave-studio codegen --check --config studio.config.json`.
   - `flexweave-studio scaffold host-app --config studio.config.json` if requested.
   - `flexweave-studio verify --fast --config studio.config.json`, then full verify if practical.
8. Create or update the repo-root `FLEXWEAVE.md` integration map. Use
   `references/integration-map-template.md`. This artifact is required context
   for the mechanic authoring skill.
9. If the repo has an agent startup file such as `AGENTS.md`, add a durable
   pointer there to read `FLEXWEAVE.md` for Flexweave and Studio work. Do not
   require future agents to use this setup skill during normal authoring.

## Boundaries

- Do not hand-edit generated mechanics definitions. Change catalog sources or
  generated target config, then rerun codegen.
- Treat runtime hooks as consumer-owned after Studio creates missing stubs.
- Do not create sample mechanics, starter catalog records, or gameplay-specific
  hook stubs during setup. If the user wants starter content, hand that off to
  the mechanic authoring workflow after setup is complete.
- Do not edit game runtime entry points, add module imports, create runtime
  availability tests, or add the Flexweave Core crate during setup unless the
  user specifically asked for Core/runtime wiring.
- It is acceptable for `flexweave-studio codegen` to create empty generated
  definition files from an empty catalog. Do not add extra `mod.rs`, `lib.rs`,
  dispatch, or hook implementation files just to make those files part of the
  runtime during setup.
- Keep game-specific concepts out of reusable Flexweave contracts. Put
  consumer-specific meaning in the consumer runtime, catalog, extensions, and
  local docs.
- Prefer the repo's existing build/test commands in `verify.commands`; do not
  invent a package-manager script layer just for Studio commands.

## References

- Read `references/setup-checklist.md` when planning the integration.
- Read `references/integration-map-template.md` before writing `FLEXWEAVE.md`.
