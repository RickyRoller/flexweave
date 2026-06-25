---
name: flexweave-author-mechanic
description: Author Flexweave Studio-backed game mechanics and abilities in a consumer repo by reading the setup integration map, using flexweave-studio plan/scaffold/codegen/verify, and implementing consumer-owned runtime hooks. Use when a user asks to add, create, change, or debug a mechanic, ability, effect, modifier, tag, execution hook, catalog record, or generated mechanics definition backed by Flexweave.
---

# Flexweave Author Mechanic

## Workflow

Use this skill after a repo has Flexweave Studio configured. If the repo does
not have `FLEXWEAVE.md`, do a lightweight setup discovery first and create that
artifact before making mechanic changes.

1. Load context:
   - Read `FLEXWEAVE.md`.
   - Read the active Studio config, usually `studio.config.json`.
   - Read generated reference docs when the integration map names one.
   - Inspect existing hook implementations, runtime registration, and tests.
2. Translate the user request into a mechanic brief:
   - Stable id and display name.
   - Intended player-facing behavior.
   - Catalog records expected: tags, modifiers, executions, effects, abilities.
   - Runtime hook behavior and state it may read/write.
   - Tests or gameplay scenarios that prove it works.
3. Inspect current Studio state before writing:
   - `flexweave-studio validate --config <path> --json`.
   - `flexweave-studio describe --config <path>` or targeted `describe <kind>`.
   - `flexweave-studio list <kind> --config <path>` for likely collision kinds.
   - `flexweave-studio codegen --check --config <path> --json`.
4. Dry-run the scaffold:
   - Use `flexweave-studio plan --archetype <id> --id <id> --name <name> --params '<json>' --config <path>`.
   - The built-in Flexweave Studio archetype is currently `mechanic`; consumer
     extensions may expose richer archetypes or wrapper commands. Prefer the
     project-documented command in `FLEXWEAVE.md` when present.
   - Stop and resolve diagnostics, collisions, or wrong planned paths before
     writing.
5. Scaffold through the CLI:
   - Run the matching `flexweave-studio scaffold ...` command only after the
     plan is understood.
   - If the CLI creates hook stubs, treat them as consumer-owned files.
6. Refresh generated output:
   - Run `flexweave-studio codegen --config <path>`.
   - Do not edit generated files directly.
7. Implement runtime behavior:
   - Edit the generated or existing hook implementation, runtime binding,
     registration, and game tests named by `FLEXWEAVE.md`.
   - Reuse existing runtime helpers and hook patterns.
   - Keep game-specific semantics in the consumer runtime, not in generated
     definitions.
8. Verify:
   - `flexweave-studio validate --config <path>`.
   - `flexweave-studio codegen --check --config <path>`.
   - `flexweave-studio verify --fast --config <path>`.
   - Run the narrow runtime tests that exercise the new mechanic.
   - Run full Studio verify or broader repo checks when generated contracts,
     hook dispatch, or shared runtime behavior changed.
9. Update `FLEXWEAVE.md` coherently if the change establishes a new hook
   pattern, generated target, local source path, or verification command:
   - Add or update an authored mechanics section for the new mechanic ids,
     catalog records, hook files, runtime entry points, and tests.
   - Do not move authored mechanics into the setup "Starter Content" section.
     That section should continue to describe setup-created sample content.
   - Remove or revise open decisions that the mechanic work resolved, such as
     "hook dispatch not wired yet", so the map does not contradict itself.

## Failure Handling

- Prefer JSON output when diagnosing CLI failures; use diagnostic `code`,
  `path`, `field`, and source location fields instead of parsing prose.
- If scaffold fails, check whether Studio rolled back writes before retrying.
- If an archetype cannot express the requested mechanic, use the closest
  scaffold only for boilerplate and document the manual catalog/runtime work.
- Ask the user for a design decision only when runtime semantics are ambiguous,
  not when paths or commands can be discovered from `FLEXWEAVE.md`.

## References

- Read `references/mechanic-workflow.md` for command templates and artifact
  expectations.
- Read `references/runtime-hook-checklist.md` before implementing or modifying
  consumer-owned hook behavior.
