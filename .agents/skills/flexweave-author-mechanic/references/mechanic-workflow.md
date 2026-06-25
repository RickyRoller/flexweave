# Mechanic Workflow Reference

## Command Templates

Replace `<bin>` with the command prefix recorded in `FLEXWEAVE.md`, usually
`flexweave-studio`. Use package-manager executors only when the consumer repo's
integration map records one.

```bash
<bin> validate --config <config> --json
<bin> describe --config <config>
<bin> list abilities --config <config>
<bin> list effects --config <config>
<bin> list executions --config <config>
<bin> list modifiers --config <config>
<bin> list tags --config <config>
<bin> codegen --check --config <config> --json
```

Plan before writing:

```bash
<bin> plan --archetype mechanic --id <id> --name "<Name>" --params '{}' --config <config> --json
```

Write after the plan is correct:

```bash
<bin> scaffold --archetype mechanic --id <id> --name "<Name>" --params '{}' --config <config> --json
<bin> codegen --config <config>
```

Verify:

```bash
<bin> validate --config <config>
<bin> codegen --check --config <config>
<bin> verify --fast --config <config>
```

Run the runtime test command from `FLEXWEAVE.md` after implementing hook
behavior.

## Mechanic Brief Template

Use this internally before editing files:

```md
## Mechanic Brief

- User request:
- Stable id:
- Display name:
- Archetype:
- Params:
- Catalog records expected:
- Runtime hook id:
- Runtime behavior:
- Existing hooks/tests to mirror:
- Verification commands:
- FLEXWEAVE.md updates:
```

## Generated Files

Generated files are owned by Studio and should be changed only by:

- Editing catalog source records.
- Updating the active Studio config or extension codegen config.
- Running `flexweave-studio codegen`.

If generated output is stale, fix the source of truth and rerun codegen.

## Archetypes

The reusable Flexweave Studio package currently guarantees the built-in
`mechanic` archetype. Consumer projects may add richer archetypes or wrapper
commands. Prefer project-specific authoring docs from `FLEXWEAVE.md` when they
exist; otherwise use the built-in archetype for skeleton records and complete
runtime behavior in consumer-owned hooks.

## Integration Map Updates

When the mechanic changes runtime wiring, update `FLEXWEAVE.md` as an
operational map:

- Record authored mechanic ids, catalog record paths, hook files, runtime
  entry points, and focused test commands under an "Authored Mechanics" or
  equivalent mechanics section.
- Keep setup provenance separate. Do not list authored mechanics as
  setup-created starter content.
- Revisit `Open Decisions` after wiring. Delete or rewrite entries that are no
  longer true.
- Keep dependency status precise. If generated hooks are imported but
  Flexweave Core is not installed, say that hook dispatch is wired while Core
  remains disabled.
