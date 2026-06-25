# Mechanic Workflow Reference

## Mechanic Brief Template

Use this internally before editing files:

```md
## Mechanic Brief

- User request:
- Stable id:
- Display name:
- Runtime owner:
- Core backing plan:
- Manual systems preserved or partial adoption gaps:
- Runtime behavior:
- Existing mechanics/tests to mirror:
- Verification commands:
- FLEXWEAVE.md updates:
```

## Runtime Editing Checklist

- Read the runtime state owner and all paths named in `FLEXWEAVE.md`.
- Search for existing `flexweave::` imports and aliases before adding new state.
- Match the repo's existing ownership pattern for world/state, systems, and
  tests.
- Use Core for the primitive named in the mechanic brief; keep domain-specific
  payloads and rules in the consumer runtime.
- Preserve manual systems recorded in `FLEXWEAVE.md` unless the user requested
  migration.
- Keep IDs, tags, event names, and test names stable and grep-friendly.

## Primitive Selection

- Identity: object creation, lookup, existence, or deterministic iteration.
- Attribute/data: numeric values, attached state, derived values, mutation
  policies, or mutation events.
- Ability: activation, readiness, cooldown, active ability lifecycle, or grants.
- Effect: application, duration, periodic ticking, removal, routing, or effect
  lifecycle events.
- Tags/query: grouping, targeting, filtering, or condition checks.
- Tick/clock: deterministic elapsed units, fixed-step advancement, or multiple
  ticking stores.
- Events/signals: reusable event channels, retained events, projections, or
  subscriptions.

## Verification

- Run the compile/check command named in `FLEXWEAVE.md`.
- Run the focused gameplay or runtime tests that exercise activation, cooldown,
  attribute mutation, effect application, targeting, ticking, or event emission.
- Add a narrow behavior test when the repo already has mechanics tests and the
  new mechanic changes observable runtime behavior.

## Integration Map Updates

When the mechanic changes runtime wiring, update `FLEXWEAVE.md` as an
operational map:

- Record authored mechanic ids, Core primitives, runtime entry points, and
  focused test commands under an "Authored Mechanics" or equivalent mechanics
  section.
- Revisit `Open Decisions` after wiring. Delete or rewrite entries that are no
  longer true.
- Keep dependency and adoption status precise. Flexweave Core should be
  installed; if a mechanic still depends on a manual attribute, cooldown,
  effect, tag, ticking, or event system, record that partial adoption gap instead
  of calling the path fully Flexweave-backed.
