# Ability System Models

These models describe the current Flexweave ability primitive as an
implementation resource. D2 sources are the editable artifacts, and peer SVGs
are tracked beside them for review. The diagrams use D2's built-in Dark Mauve
theme (`theme-id: 200`) for consistent contrast.

## Scope

The ability system covers authorable ability definitions, runtime grants,
activation attempts, explicit commitment, active execution state, cancellation,
revocation, rollback, ending, and lifecycle facts. Cooldowns are modeled through
caller-owned effects, tags, activation gates, and commit actions rather than
ability-owned state.

Flexweave owns deterministic primitive state and lifecycle facts. Caller code
owns activation gate behavior, commit action behavior, resource semantics,
event-channel publication, runtime authority, task execution, and effect
application derived from an ability.

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

- `data-model.d2` shows the static definition, grant, store, gate, action, and event
  relationships.
- `lifecycle.d2` shows begin, explicit commit, end, cancel, rollback,
  revocation, and rejection paths.
- `event-publication.d2` shows borrowed versus owned lifecycle facts and the
  caller-owned event-channel publication boundary.

Each D2 file has a same-name `.svg` render beside it.

## Implementation Notes

- `AbilityDefinition` validates authoring metadata for definition identity and
  lifecycle routing. Payload schema is carried as caller-owned metadata.
- `begin_registered_activation_*` validates that the granted definition key is
  registered; definition fields remain metadata or validation hints.
- Ability lifecycle facts are returned through callbacks. Channel keys on
  definitions do not auto-route events into `EventChannel`.
- `AbilityActivationGate::can_activate` is the begin-time synchronous caller
  hook where caller code decides activation blocking, including required tags,
  resource checks, cooldown override, authority, and targeting.
- `AbilityCommitAction::apply_commit` owns point-of-no-return side effects.
  End, cancel, revocation, and rollback commands do not accept caller
  participants.
- `ActiveAbility::source_id` exposes the owner id for caller-owned effect
  application derived from an activation; abilities do not apply effects
  automatically.

## Open Questions

- Should ability-derived effect application get a documented adapter pattern
  that starts from `ActiveAbility::source_id`?
- Should a future task model caller-owned authority and prediction separately
  from the domain-neutral Flexweave primitive?
