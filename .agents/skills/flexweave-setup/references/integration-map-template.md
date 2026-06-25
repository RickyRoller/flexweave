# FLEXWEAVE.md Template

Create this file at the consumer repo root. Keep it short and operational; it
is context for future agents, not user-facing product documentation. Link to it
from the repo's agent startup file, such as `AGENTS.md`, when one exists.

```md
# Flexweave Integration Map

## Purpose

This repo uses Flexweave for runtime mechanics primitives. Game-specific
semantics live in the consumer runtime; Flexweave supplies reusable lifecycle,
state, query, and event building blocks.

## Integration Mode

- Flexweave: enabled, installed in `<runtime crate/package/module>`.

## Dependencies

- Rust crate: `flexweave` from <registry/path/version>.

## Command Map

- Runtime compile/check: `<command>`.
- Runtime tests for mechanics: `<command or none established yet>`.
- Full repo verification: `<command or none established yet>`.

## Flexweave Adoption Map

- Object identity: `<Flexweave-backed | manual | not adopted yet>` via `<paths or notes>`.
- Attributes: `<Flexweave-backed | manual | not adopted yet>` via `<paths or notes>`.
- Abilities/cooldowns: `<Flexweave-backed | manual | not adopted yet>` via `<paths or notes>`.
- Effects/lifecycle: `<Flexweave-backed | manual | not adopted yet>` via `<paths or notes>`.
- Tags/queries: `<Flexweave-backed | manual | not adopted yet>` via `<paths or notes>`.
- Mechanics ticking/events: `<Flexweave-backed | manual | not adopted yet>` via `<paths or notes>`.
- Manual systems to preserve: `<paths or none>`.

## Runtime Map

- Runtime state owner: `<path>`.
- Object creation/destruction path: `<path or not established yet>`.
- Attribute/state mutation path: `<path or not established yet>`.
- Ability activation path: `<path or not established yet>`.
- Effect application/ticking path: `<path or not established yet>`.
- Event publication/subscription path: `<path or not established yet>`.
- Existing mechanics examples to mirror: `<paths or none>`.

## Mechanic Authoring Protocol

1. Read this file and inspect the runtime paths it names.
2. Classify the requested mechanic by Flexweave primitive: identity, attributes,
   abilities, effects, tags, ticking, events, signals, or registries.
3. Reuse the repo's existing Flexweave-backed seam for adopted primitives.
4. Preserve documented manual systems unless migration is requested.
5. Run existing compile/check commands and relevant gameplay tests.
6. Update this file when the mechanic changes adoption status or establishes a
   new reusable runtime path.

## Open Decisions

- <decision, owner, date or trigger>

## Last Verified

- <date>: `<command>` passed.
```
