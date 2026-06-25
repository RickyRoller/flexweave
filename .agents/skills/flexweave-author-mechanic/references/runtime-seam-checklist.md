# Runtime Seam Checklist

Use this when a mechanic needs a new shared path through runtime state.

## Before Editing

- Read `FLEXWEAVE.md` for adopted primitives, manual systems to preserve, and
  verification commands.
- Search for existing Flexweave stores, imports, wrappers, and tests.
- Read the closest existing mechanic before adding a new module or state owner.

## Implementation

- Keep game semantics in the consumer runtime.
- Let Flexweave own the reusable lifecycle or state shape named in the
  mechanic brief.
- Put domain payloads, balance values, target rules, and UI labels in the
  consumer runtime's existing structures.
- Wire ticking, activation, application, mutation, and event emission through
  one runtime path per responsibility.
- Record partial adoption when a manual path stays in the mechanic.

## Tests

- Cover the player-visible behavior through the repo's existing mechanics test
  style.
- Include a boundary case when the mechanic has branching, limits, timing, or
  failure modes.
- Run the narrowest command that covers the changed runtime path, then broader
  checks when shared state changed.

## Common Failure Modes

- A mechanic imports Flexweave but keeps lifecycle state in unrelated local
  fields.
- An ability uses a no-op activation executor and performs the actual damage,
  effect application, or resource mutation only by inspecting lifecycle events
  after activation.
- Cooldowns, ticking, attributes, or effects are manually duplicated beside an
  adopted Flexweave store.
- UI or status reactions poll Flexweave-backed stores every frame even
  though the relevant attribute/effect/ability event is already emitted.
- A new runtime path bypasses existing event flow or query helpers.
- `FLEXWEAVE.md` claims a primitive is adopted without naming the path that owns
  it.
