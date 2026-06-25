---
name: flexweave-author-mechanic
description: Author Flexweave Core-backed game mechanics and abilities in a consumer repo by reading the integration map, using Core primitives, and implementing runtime behavior. Use when a user asks to add, create, change, or debug a mechanic, ability, effect, modifier, tag, cooldown, attribute, tick, or event backed by Flexweave Core.
---

# Flexweave Author Mechanic

## Workflow

Use this skill after a repo has Flexweave setup. If the repo does not have
`FLEXWEAVE.md` or an installed Flexweave Core dependency, repair setup first by
using the `flexweave-setup` skill.

1. Load context:
   - Read `FLEXWEAVE.md`.
   - Inspect the Core adoption map, runtime state owner, manual mechanics
     systems, and gameplay tests.
2. Translate the user request into a mechanic brief:
   - Stable id and display name.
   - Intended player-facing behavior.
   - Runtime state and lifecycle the mechanic needs.
   - Core backing plan: which Flexweave primitives own identity, attributes,
     ability lifecycle, cooldowns, effects, tags, ticking, or events.
   - Existing manual systems to preserve or call out.
   - Runtime modules, structs, or systems to edit.
   - Gameplay tests or scenarios that prove the mechanic works.
3. Inspect current runtime state before writing:
   - Search for existing uses of Flexweave Core types and stores.
   - Read similar mechanics, activation paths, effect paths, tick loops, and
     event flow.
   - Identify the narrow runtime seam where the requested behavior belongs.
4. Implement runtime behavior:
   - Route state and lifecycle through the repo's Flexweave-backed seam.
   - Reuse existing runtime helpers and mechanics patterns.
   - Preserve existing manual systems unless the user requested migration. If a
     manual system remains in the path of the mechanic, record that partial
     adoption gap.
   - Keep game-specific semantics in the consumer runtime; keep Flexweave Core
     responsible for reusable lifecycle/state shape.
5. Verify:
   - Run existing compile/check commands that cover the runtime crate.
   - Run the narrow gameplay tests that exercise the new mechanic.
   - Run broader repo checks when shared runtime behavior changed.
6. Update `FLEXWEAVE.md` coherently if the change establishes a new Core-backed
   runtime path, adoption status, or verification command:
   - Add or update an authored mechanics section for the new mechanic ids,
     runtime entry points, Core primitives, and tests.
   - Remove or revise open decisions that the mechanic work resolved, such as
     "ability lifecycle is not adopted yet", so the map does not contradict
     itself.

## Failure Handling

- If Flexweave Core is missing or the adoption map is absent, repair setup before
  authoring.
- If the repo has no Flexweave-backed seam for the primitive the mechanic needs,
  either create the smallest runtime seam required for the mechanic or record
  the partial adoption gap when the user wants the existing manual path kept.
- Ask the user for a design decision only when runtime semantics are ambiguous,
  not when paths or commands can be discovered from `FLEXWEAVE.md`.

## References

- Read `references/mechanic-workflow.md` for the mechanic brief and runtime
  editing checklist.
- Read `references/flexweave-backed-mechanic.md` before implementing runtime
  behavior.
- Read `references/runtime-seam-checklist.md` before creating or changing a
  shared runtime mechanics path.
