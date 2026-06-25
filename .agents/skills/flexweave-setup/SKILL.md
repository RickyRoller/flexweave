---
name: flexweave-setup
description: Set up Flexweave Core and Flexweave Studio in a consumer game repository by verifying the direct flexweave-studio CLI, adding minimal config/ownership directories, wiring the Core runtime dependency when requested, and writing the required FLEXWEAVE.md integration map. Use when a user wants to install, adopt, integrate, configure, or onboard Flexweave in a repo. Do not use to author starter mechanics, sample catalog records, or gameplay content unless the user explicitly asks for that separately.
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
   - Core only: runtime needs deterministic mechanics primitives but no Studio authoring flow.
   - Studio codegen: repo needs catalog validation, generated mechanics definitions, and runtime hook locations.
   - Studio host app: repo also needs a local authoring UI shell.
3. Ask only for decisions that cannot be inferred safely:
   - Target runtime language/crate and package manager.
   - Where catalog source, generated Rust, runtime hooks, and hook tests should live.
   - Whether to create a local Studio host app now.
4. Verify or wire dependencies without changing repo tooling unnecessarily:
   - Rust runtime: add `flexweave` to the owning crate.
   - Studio workflows: prefer the direct `flexweave-studio` CLI. Do not create
     `package.json`, `bun.lock`, `node_modules`, or repo scripts only to run
     Studio from a non-JavaScript repo.
   - Host app: add local JavaScript package dependencies only when creating a
     host app or when the repo already uses JavaScript tooling.
5. Create or update `studio.config.json` when Studio is in scope. Include:
   - `catalogRoot`.
   - `codegen.outputDirs` for `abilities`, `effects`, `executions`, `modifiers`, `reference`, and `tags`.
   - `hooks.dir` and optional `hooks.testStubsDir`.
   - `rust.flexweaveModule`.
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
- Keep game-specific concepts out of reusable Flexweave contracts. Put
  consumer-specific meaning in the consumer runtime, catalog, extensions, and
  local docs.
- Prefer the repo's existing build/test commands in `verify.commands`; do not
  invent a package-manager script layer just for Studio commands.

## References

- Read `references/setup-checklist.md` when planning the integration.
- Read `references/integration-map-template.md` before writing `FLEXWEAVE.md`.
