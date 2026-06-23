# Runtime Hook Checklist

Runtime hooks are consumer-owned after Studio creates missing stubs.

## Before Editing

- Read `FLEXWEAVE.md` for hook root, dispatch entry point, and existing examples.
- Inspect generated execution definitions to confirm the hook id.
- Search for existing hooks with similar damage, targeting, status, or timing
  behavior.
- Read the runtime tests that cover hooks or ability execution.

## Implementation

- Keep game semantics in the consumer runtime.
- Reuse runtime helpers for target lookup, damage application, modifiers,
  state mutation, logging, and deterministic randomness.
- Keep hook inputs and outputs aligned with generated definitions.
- Add runtime vocabulary to `studio.config.ts` only when the generated code or
  validation needs that vocabulary.
- Update hook dispatch or registration if the runtime requires explicit wiring.

## Tests

- Add a focused runtime test for the new behavior.
- Include at least one negative or boundary case when the hook has targeting,
  cooldown, resource, resistance, duration, or stacking logic.
- Run generated freshness checks before runtime tests so failures point at the
  right layer.

## Common Failure Modes

- Hook stub exists but is not registered in the runtime dispatch table.
- Catalog references a hook id that does not match the implementation file.
- Generated output was manually edited and is overwritten by codegen.
- Runtime vocabulary omits a new damage type or ailment.
- The new mechanic passes validation but has no gameplay assertion in tests.
