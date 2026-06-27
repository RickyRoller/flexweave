# Ability System Models

These models describe the current Flexweave ability primitive as an
implementation resource. D2 sources are the editable artifacts, and peer SVGs
are tracked beside them for review. The diagrams use D2's built-in Dark Mauve
theme (`theme-id: 200`) for consistent contrast.

## Scope

The ability system covers authorable ability definitions, runtime grants,
activation attempts, commit timing, cooldowns, active execution state,
cancellation, ending, revocation, and lifecycle facts.

Flexweave owns deterministic primitive state and lifecycle facts. Caller code
owns hook behavior, resource semantics, event-channel publication, runtime
authority, task execution, and effect application derived from an ability.

## Source Paths

- `core/src/ability.rs`
- `core/src/ability/definition.rs`
- `core/src/ability/store.rs`
- `core/src/ability/events.rs`
- `core/src/ability/hooks.rs`
- `core/src/ability/ids.rs`
- `core/src/lifecycle/kind.rs`
- `core/tests/abilities.rs`
- `core/README.md`
- `docs/how-to/use-flexweave.md`

## Model Files

- `data-model.d2` shows the static definition, grant, store, hook, and event
  relationships.
- `lifecycle.d2` shows active and instant activation paths, commit timing,
  rejection, cancellation, ending, revocation, and cooldown advancement.
- `event-publication.d2` shows borrowed versus owned lifecycle facts and the
  caller-owned event-channel publication boundary.

Each D2 file has a same-name `.svg` render beside it.

## Implementation Notes

- `AbilityDefinition` validates authoring metadata, but `AbilityStore` does not
  automatically enforce `activation_mode`, `cancel_policy`,
  `tag_requirement_keys`, or `activation_tag_keys` at runtime.
- `begin_registered_activation_*` uses a granted definition key to choose
  `commit_timing`; other definition fields remain metadata or validation hints.
- Ability lifecycle facts are returned through callbacks. Channel keys on
  definitions do not auto-route events into `EventChannel`.
- `AbilityHooks` are the boundary where caller code decides resource checks,
  cooldown override, commit side effects, and end side effects.
- `ActiveAbility::source_id` exposes the owner id for caller-owned effect
  application derived from an activation; abilities do not apply effects
  automatically.

## Open Questions

- Should registered activation eventually enforce `activation_mode` and
  `cancel_policy`, or should those remain authoring metadata only?
- Should tag requirement and activation tag metadata get a first-class runtime
  helper, or should hooks remain the only enforcement point?
- Should ability-derived effect application get a documented adapter pattern
  that starts from `ActiveAbility::source_id`?
- Should a future task model caller-owned authority and prediction separately
  from the domain-neutral Flexweave primitive?
