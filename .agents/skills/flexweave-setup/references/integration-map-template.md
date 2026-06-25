# FLEXWEAVE.md Template

Create this file at the consumer repo root. Keep it short and operational; it
is context for future agents, not user-facing product documentation. Link to it
from the repo's agent startup file, such as `AGENTS.md`, when one exists.

```md
# Flexweave Integration Map

## Purpose

This repo uses Flexweave for <core primitives | Studio authoring/codegen |
local Studio host app>. Flexweave-owned generated files are refreshed through
Studio commands; game semantics live in the consumer runtime.

## Integration Mode

- Core: <enabled/disabled>, imported by <crate/package/module>.
- Studio codegen: <enabled/disabled>, config path: `<path, usually studio.config.json>`.
- Studio host app: <enabled/disabled>, app root: `<path or none>`.

## Dependencies

- Rust crate: `flexweave` from <registry/path/version>.
- Studio CLI: `flexweave-studio` from <install source/version>.
- Studio app package: `@flexweave/studio-app` from <registry/path/version or none>.
- Studio command prefix: `<flexweave-studio | pnpm exec flexweave-studio | npx flexweave-studio | direct bin>`.

## Command Map

- Validate catalog: `<command>`.
- Refresh generated output: `<command>`.
- Check generated freshness: `<command>`.
- Migrate after package updates: `<command>`.
- Fast Studio verify: `<command>`.
- Full Studio verify: `<command>`.
- Runtime tests for mechanics: `<command>`.
- Local Studio host app: `<command or none>`.

## Catalog And Sources

- Catalog root: `<path>`.
- Source adapters: `<built-in JSON | project adapter ids>`.
- Writable scaffold source: `<catalogRoot | source id>`.
- Generated reference doc: `<path or none>`.

## Generated Output Ownership

Do not hand-edit these directories:

- Abilities: `<path>`.
- Effects: `<path>`.
- Executions: `<path>`.
- Modifiers: `<path>`.
- Tags: `<path>`.
- Reference: `<path>`.
- Extension targets: `<target id -> path>`.

## Runtime Hooks

- Hook root: `<path>`.
- Hook test stub root: `<path or none>`.
- Hook dispatch/registration entry point: `<path>`.
- Existing hook examples to copy: `<paths>`.
- Runtime state/API helpers available to hooks: `<paths>`.

## Rust Bindings

- Flexweave module path: `<rust path>`.
- Project-specific Rust bindings: `<extension namespace -> summary>`.

## Mechanic Authoring Protocol

1. Read this file and the active Studio config.
2. Run validate and generated freshness checks before writing.
3. Use `flexweave-studio plan` before `scaffold`.
4. Run `flexweave-studio codegen`; never edit generated files directly.
5. Implement consumer-owned hook behavior and runtime tests.
6. Run validate, generated freshness check, fast verify, and relevant runtime tests.

## Open Decisions

- <decision, owner, date or trigger>

## Starter Content

- Setup-created starter mechanics: none.
- If sample content is desired, use the Flexweave author mechanic skill after
  setup and record the generated hook/test pattern here.

## Last Verified

- <date>: `<command>` passed.
```
