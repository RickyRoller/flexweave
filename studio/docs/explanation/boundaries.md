# Product Boundaries

Flexweave Core provides reusable mechanics primitives. Flexweave Studio is an
optional authoring and build-time layer on top of those primitives.

The Studio package owns contracts and workflows:

- Studio project config loading.
- Studio catalog validation.
- Mechanic planning and scaffolding.
- Generated mechanics definition rendering.
- Generated freshness checks.
- Runtime hook diagnostics.
- Migration and verification workflows.

Consumer projects own content and runtime meaning:

- Studio catalog records.
- Generated output directories.
- Runtime hook implementations after stub creation.
- Runtime bindings.
- Local host app entry points.
- Project adapters, labels, deployment, and verification commands.

This boundary keeps the package reusable. Studio can create and verify files
only through paths declared by the active Studio project config.
