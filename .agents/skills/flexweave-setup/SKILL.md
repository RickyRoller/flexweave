---
name: flexweave-setup
description: Set up Flexweave Core and Flexweave Studio in a consumer game repository, including dependency wiring, studio.config.ts, codegen outputs, runtime hooks, repo scripts, local Studio host app scaffolding, and a FLEXWEAVE.md integration map. Use when a user wants to install, adopt, integrate, configure, scaffold, or onboard Flexweave in a repo, especially from a blank canvas or early game prototype.
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
   - Studio codegen: repo needs catalog validation, generated mechanics definitions, and runtime hooks.
   - Studio host app: repo also needs a local authoring UI shell.
3. Ask only for decisions that cannot be inferred safely:
   - Target runtime language/crate and package manager.
   - Where catalog source, generated Rust, runtime hooks, and hook tests should live.
   - Whether to create a local Studio host app now.
   - Initial runtime vocabulary such as damage types and ailments.
4. Install or wire dependencies using the repo's package manager:
   - Rust runtime: add `flexweave` to the owning crate.
   - Studio workflows: add `@flexweave/studio`.
   - Host app: add `@flexweave/studio-app` only when creating a host app.
5. Create or update `studio.config.ts` when Studio is in scope. Include:
   - `catalogRoot`.
   - `codegen.outputDirs` for `abilities`, `effects`, `executions`, `modifiers`, `reference`, and `tags`.
   - `hooks.dir` and optional `hooks.testStubsDir`.
   - `rust.flexweaveModule` and `rust.runtimeVocab`.
   - `verify.commands` for the repo's existing checks.
6. Add repo scripts for recurring workflows:
   - Validate catalog.
   - Refresh codegen.
   - Check generated freshness.
   - Migrate after package updates.
   - Fast and full Studio verify.
   - Local host app dev/build scripts when host app exists.
7. Run the narrowest checks as each layer lands:
   - `flexweave-studio validate --config studio.config.ts`.
   - `flexweave-studio codegen --config studio.config.ts`.
   - `flexweave-studio codegen --check --config studio.config.ts`.
   - `flexweave-studio scaffold host-app --config studio.config.ts` if requested.
   - `flexweave-studio verify --fast --config studio.config.ts`, then full verify if practical.
8. Create or update the repo-root `FLEXWEAVE.md` integration map. Use
   `references/integration-map-template.md`. This artifact is required context
   for the mechanic authoring skill.

## Boundaries

- Do not hand-edit generated mechanics definitions. Change catalog sources or
  generated target config, then rerun codegen.
- Treat runtime hooks as consumer-owned after Studio creates missing stubs.
- Keep game-specific concepts out of Flexweave Core. Put Player, Tower, Map,
  Inventory, and other game vocabulary in the consumer runtime.
- Prefer the repo's existing build/test scripts in `verify.commands`; do not
  invent a second verification stack.

## References

- Read `references/setup-checklist.md` when planning the integration.
- Read `references/integration-map-template.md` before writing `FLEXWEAVE.md`.
