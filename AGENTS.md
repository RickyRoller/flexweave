# Agent Instructions

## Startup

Read `CONTEXT-MAP.md` first, then read the context file for the surface you are
changing.

## Boundaries

- Root files own workspace orchestration and shared verification.
- `core` owns the Rust mechanics primitive crate.
- `studio` owns the reusable Studio package.
- `studio/app` owns the reusable Studio application shell.
- Consumer projects own their runtime bindings, catalog content, generated
  output directories, runtime hooks, local host app entry point, and deployment.

## Agent Skills

Installable package:

```bash
npx skills@latest add RickyRoller/flexweave --skill flexweave-setup --skill flexweave-author-mechanic
```

- Use `.agents/skills/flexweave-setup/SKILL.md` when integrating Flexweave into
  a consumer repo, creating `studio.config.ts`, wiring codegen scripts, or
  creating the `FLEXWEAVE.md` integration map.
- Use `.agents/skills/flexweave-author-mechanic/SKILL.md` when adding or
  changing a consumer mechanic, ability, effect, modifier, execution hook, tag,
  or generated mechanics definition.

## Verification

Run `bun fix` after substantive TypeScript, JSON, or markdown edits. Run
`bun run verify` before handing off broad skeleton changes.
