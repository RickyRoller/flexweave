# Runtime Hook Checklist

Runtime hooks are consumer-owned after Studio creates missing stubs.

## Before Editing

- Read `FLEXWEAVE.md` for hook root, dispatch entry point, and existing examples.
- Inspect generated execution definitions to confirm the hook id.
- Search for existing hooks with similar targeting, state mutation, tagging, or
  timing behavior.
- Read the runtime tests that cover hooks or ability execution.

## Implementation

- Keep game semantics in the consumer runtime.
- Reuse runtime helpers for target lookup, modifier resolution, state mutation,
  event emission, logging, and deterministic randomness.
- Keep hook inputs and outputs aligned with generated definitions.
- Update hook dispatch or registration if the runtime requires explicit wiring.
- Replace generated placeholder hook bodies with the consumer-owned behavior or
  data contract the runtime actually imports. Do not leave a no-op hook as the
  apparent implementation for an authored mechanic.

## Tests

- Add a focused runtime test for the new behavior.
- Include at least one negative or boundary case when the hook has branching,
  limits, lifecycle behavior, or failure modes.
- If generated hook test stubs are kept, they may remain declaration smoke
  tests, but the authored mechanic still needs meaningful runtime assertions
  elsewhere.
- Run generated freshness checks before runtime tests so failures point at the
  right layer.

## Common Failure Modes

- Hook stub exists but is not registered in the runtime dispatch table.
- Catalog references a hook id that does not match the implementation file.
- Generated output was manually edited and is overwritten by codegen.
- The new mechanic passes validation but has no gameplay assertion in tests.
