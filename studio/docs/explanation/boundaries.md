# Product Boundaries

Flexweave Core provides reusable mechanics primitives. Flexweave Studio is an
optional authoring and build-time layer on top of those primitives.

The Studio package owns contracts and workflows:

- Studio project config loading.
- Studio extension and data adapter contracts.
- Source snapshot and source location contracts.
- Normalized Studio content and mapper contracts.
- Generated target registry and managed-file checks.
- Studio catalog validation.
- Mechanic planning and scaffolding.
- Generated mechanics definition rendering.
- Generated freshness checks.
- Runtime hook diagnostics.
- Migration and verification workflows.
- Structured verify and migrate result contracts.
- Reusable local host app shell contracts.

Projects own content, source systems, and runtime meaning:

- Studio catalog records.
- Source adapters for project-specific file layouts, tables, services, or APIs.
- Mappers from adapter records into Studio content or project models.
- Generated output directories.
- Generated targets for project-specific outputs.
- Extension-owned migrations for project-specific source or schema state.
- Runtime hook implementations after stub creation.
- Runtime bindings.
- Local host app entry points.
- Project adapters and labels.
- Project adapters, labels, deployment, and verification commands.

This boundary keeps the package reusable. Studio can create and verify files
only through paths declared by the active Studio project config.
