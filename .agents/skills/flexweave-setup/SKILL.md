---
name: flexweave-setup
description: Set up Flexweave in a consumer game repository by installing the Rust library, mapping current runtime adoption, and writing FLEXWEAVE.md. Use when a user wants to install, adopt, integrate, configure, or onboard Flexweave in a repo.
---

# Flexweave Setup

## Workflow

Use this skill to make Flexweave available to the runtime and to leave future
agents a precise operating map.

1. Read existing context before changing files:
   - Repo root docs: `README*`, `AGENTS.md`, package manifests, workspace files.
   - Rust manifests: `Cargo.toml`, crate layout, runtime modules.
   - Existing mechanics, world/state, systems, schedules, and tests.
2. Identify the runtime adoption state:
   - Flexweave availability: required for Rust game runtimes. Locate the owning
     runtime crate before editing dependency files.
   - Flexweave adoption map: record whether object identity, attributes, abilities,
     effects, tags, ticking, and events are Flexweave-backed, manual, or not
     adopted yet.
3. Ask only for decisions that cannot be inferred safely:
   - Target runtime language/crate when more than one plausible runtime exists.
   - Which existing runtime seam should own Flexweave state when several
     candidates are equally plausible.
4. Wire dependencies without changing repo tooling unnecessarily:
   - Rust runtime: add the `flexweave` crate to the owning runtime crate and
     verify the dependency edit with existing check commands.
5. Create or update the repo-root `FLEXWEAVE.md` integration map. Use
   `references/integration-map-template.md`. This artifact is required context
   for the mechanic authoring skill.
6. If the repo has an agent startup file such as `AGENTS.md`, add a durable
   pointer there to read `FLEXWEAVE.md` for Flexweave work.

## Completion Criteria

- The owning runtime crate depends on `flexweave`.
- `FLEXWEAVE.md` names the runtime crate, Flexweave dependency source, adoption map,
  existing mechanics seams, verification commands, and open migration decisions.
- Future agents can tell which runtime responsibilities are already
  Flexweave-backed and which manual systems should be preserved.

## References

- Read `references/setup-checklist.md` when planning the integration.
- Read `references/integration-map-template.md` before writing `FLEXWEAVE.md`.
