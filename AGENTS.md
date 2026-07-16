# Agent Instructions

## Startup

Read `core/CONTEXT.md` before changing the Rust mechanics crate.

## Boundaries

- Root files own workspace orchestration and shared verification.
- `core` owns the Rust mechanics primitive crate.
- Consumer projects own their runtime bindings, authored content, and
  deployment.

## Verification

Run `bun fix` after substantive TypeScript, JSON, or markdown edits. Run
`bun run verify` before handing off broad skeleton changes.
