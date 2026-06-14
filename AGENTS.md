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

## Verification

Run `bun fix` after substantive TypeScript, JSON, or markdown edits. Run
`bun run verify` before handing off broad skeleton changes.

Do not add any directory named `examples` in this phase.
