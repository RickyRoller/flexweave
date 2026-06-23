# Foundation Guide

This document is the fresh-context operating map for Flexweave.

## Surface Overview

| Surface                  | Path         | Owns                                                                                                                                 | Does not own                                                                                                  |
| ------------------------ | ------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------- |
| Flexweave Core           | `core`       | Rust mechanics primitives, deterministic stores, primitive errors, and Core docs.                                                    | Catalog files, code generation, authoring UI, consumer runtime source, and runtime hooks.                     |
| Flexweave Studio package | `studio`     | Studio project config, catalog contracts, validation, migrations, generated output checks, workflow APIs, and runtime contract docs. | Consumer catalog content, consumer runtime semantics, generated output directories, and hook implementations. |
| Flexweave Studio app     | `studio/app` | Reusable app shell and adapter-neutral UI contracts.                                                                                 | Consumer-owned app entry point, branding, deployment, and project adapter.                                    |
| Root workspace           | `.`          | Toolchain versions, workspace membership, cross-surface scripts, term scans, and repository docs.                                    | Surface-specific implementation details.                                                                      |

## Edit Guide

| Change type              | Start here                                                            |
| ------------------------ | --------------------------------------------------------------------- |
| Rust primitive crate     | `core/src`, `core/tests`, `core/README.md`, `core/CONTEXT.md`         |
| Studio package contracts | `studio/src`, `studio/tests`, `studio/README.md`, `studio/CONTEXT.md` |
| Studio runtime hook docs | `studio/docs/reference/runtime-hooks.md`                              |
| Studio app shell         | `studio/app/src`, `studio/app/tests`                                  |
| Repository verification  | `package.json`, `scripts`, `Cargo.toml`                               |
| Product documentation    | `README.md`, `docs`                                                   |

## Agent Skills

Flexweave skills are installable through the `skills` CLI:

```bash
npx skills@latest add RickyRoller/flexweave --skill flexweave-setup --skill flexweave-author-mechanic
```

| Skill                     | Path                                                | Use for                                                                                           |
| ------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Flexweave setup           | `.agents/skills/flexweave-setup/SKILL.md`           | Consumer repo onboarding, dependency wiring, Studio config, repo scripts, host app scaffold, map. |
| Flexweave author mechanic | `.agents/skills/flexweave-author-mechanic/SKILL.md` | Consumer mechanics, abilities, generated definitions, runtime hooks, and post-codegen tests.      |

The setup skill creates a repo-root `FLEXWEAVE.md` integration map. Mechanic
authoring agents should read that artifact before using Studio workflows.

## Verification

Prefer the narrowest meaningful command first, then broaden:

| Area              | Command                                       |
| ----------------- | --------------------------------------------- |
| Core              | `cargo test -p flexweave`                     |
| Studio package    | `bun run --filter @flexweave/studio test`     |
| Studio app        | `bun run --filter @flexweave/studio-app test` |
| Studio Phase 5    | `bun run verify:studio`                       |
| Structure guard   | `bun run check:structure`                     |
| Retired-term scan | `bun run check:terms`                         |
| Full gate         | `bun run verify`                              |
