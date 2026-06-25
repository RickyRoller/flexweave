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
- Event consumers:
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
- For bounded values such as health, stamina, shields, or charges, model both
  current and maximum values through the Flexweave-backed attribute/data seam
  unless the repo has an established alternate resource shape.
- Consume Core-emitted facts for observable reactions. UI projections,
  death/despawn, status changes, and follow-on mechanics should be updated from
  attribute changes or lifecycle events, not by polling the backing stores every
  frame.
- Put an ability's primary gameplay command in the Flexweave ability executor or
  hook path. For example, damage, effect application, resource spend, or target
  mutation should happen during `AbilityStore` activation execution, while
  lifecycle events can be retained for projections and diagnostics.
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
- When the mechanic has UI, death/despawn, or status behavior, test that those
  reactions are driven by emitted attribute/lifecycle events.

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
