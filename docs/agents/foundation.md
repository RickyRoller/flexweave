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

| Change type                  | Start here                                                            |
| ---------------------------- | --------------------------------------------------------------------- |
| Rust primitive crate         | `core/src`, `core/tests`, `core/README.md`, `core/CONTEXT.md`         |
| Studio package contracts     | `studio/src`, `studio/tests`, `studio/README.md`, `studio/CONTEXT.md` |
| Studio runtime contract docs | `studio/docs/runtime-contract.md`                                     |
| Studio app shell             | `studio/app/src`, `studio/app/tests`                                  |
| Repository verification      | `package.json`, `scripts`, `Cargo.toml`                               |
| Product documentation        | `README.md`, `docs`                                                   |

## Verification

Prefer the narrowest meaningful command first, then broaden:

| Area              | Command                                       |
| ----------------- | --------------------------------------------- |
| Core              | `cargo test -p flexweave`                     |
| Studio package    | `bun run --filter @flexweave/studio test`     |
| Studio app        | `bun run --filter @flexweave/studio-app test` |
| Structure guard   | `bun run check:structure`                     |
| Retired-term scan | `bun run check:terms`                         |
| Full gate         | `bun run verify`                              |
