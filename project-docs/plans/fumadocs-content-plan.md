# Fumadocs Content Plan

This plan expands the hosted Flexweave documentation site as a book-like guide,
with a separate top-level API Reference. Diataxis still informs the writing
intent of each page, but it is not the site navigation.

The organization should feel closer to the Rust Book than to a documentation
taxonomy: begin with orientation, build a cohesive example in chapters, deepen
the core concepts, cover runtime integration patterns, then keep exhaustive API
surface material in a separate reference section.

Sources:

- <https://doc.rust-lang.org/book/>
- <https://raw.githubusercontent.com/rust-lang/book/main/src/SUMMARY.md>
- <https://diataxis.fr/>
- <https://diataxis.fr/tutorials/>
- <https://diataxis.fr/how-to-guides/>
- <https://diataxis.fr/reference/>
- <https://diataxis.fr/explanation/>
- <https://diataxis.fr/complex-hierarchies/>

## Organization Model

The Rust Book does not group pages by writing type. It uses a progressive
learning path:

- Introduction and getting started.
- A concrete early project.
- Core concepts in increasing depth.
- Applied project chapters.
- Advanced topics.
- Appendices.

Flexweave should use the same kind of reader journey:

- **Start Here** introduces what Flexweave is and how to add it to a Rust
  runtime.
- **Building an RPG Combat Runtime** is the main cohesive example.
- **Core Concepts** explains the primitives through the same RPG nouns.
- **Runtime Patterns** gives focused integration recipes for consumer apps.
- **Design Notes** explains boundaries and tradeoffs.
- **API Reference** is a separate top-level section exclusively for public API
  surfaces.

## Writing Rules

- Do not expose "tutorials", "how-to guides", "reference", or "explanation" as
  top-level navigation groups.
- Keep a page's Diataxis intent visible in the issue plan so authors know how to
  write it.
- Use chapter-style ordering for the main docs. Later pages may depend on
  earlier pages.
- Keep API Reference separate. API pages should not be mixed into the learning
  path, runtime patterns, or design notes.
- Keep examples cohesive. Most pages should reuse the same RPG combat runtime
  unless they need a smaller API snippet.
- Keep Flexweave domain-agnostic in descriptions. RPG nouns belong to examples,
  not to the library model.

## Target Site Structure

```text
docs/
  index
  getting-started/
    what-is-flexweave
    install-and-verify
    mechanics-at-the-boundary
  rpg-combat/
    01-create-combatants
    02-attach-data-and-tags
    03-add-attributes
    04-derived-max-health-and-attack-speed
    05-grant-player-abilities
    06-target-and-activate-abilities
    07-commit-abilities-into-effects
    08-apply-damage-and-buffs
    09-advance-time-and-expire-effects
    10-publish-combat-events
    11-clean-up-combatants
    12-complete-runtime
  core-concepts/
    objects-and-attached-data
    attributes-and-derived-attributes
    tags-and-target-selection
    abilities-and-commitment
    effects-and-active-instances
    lifecycle-facts-channels-and-signals
    clocks-and-mechanics-ticks
    registries-and-definition-bundles
    object-lifetime-and-cleanup
  runtime-patterns/
    model-health-mana-and-resources
    clamp-and-reject-attribute-mutations
    calculate-derived-stats-from-effects
    gate-abilities-with-runtime-state
    turn-abilities-into-costs-cooldowns-and-effects
    apply-instant-damage-and-healing
    run-temporary-buffs-and-debuffs
    run-periodic-effects
    advance-turn-based-and-realtime-mechanics
    publish-lifecycle-facts
    project-signals-for-adapters
    organize-runtime-definition-bundles
  design-notes/
    deterministic-mechanics
    product-boundaries
    stored-vs-derived-state
    ability-effect-lifecycle-boundaries
    facts-channels-signals-and-app-events
    tags-as-structured-labels
    content-authoring-vs-runtime-definitions
  api-reference/
    index
    identity
    data-store
    query
    attribute
    derived-attribute
    tag
    ability
    effect
    lifecycle
    signal
    clock
    mechanics
    registry
    object-lifecycle
    errors-and-outcomes
```

## Canonical RPG Example

The main example is a small RPG combat runtime owned by a consumer app.
Flexweave provides primitive stores and lifecycle shape; the example owns game
nouns and behavior.

### Domain Nouns

- **Player character**: an object with profile data, faction tags, current
  health, base vitality, base attack speed, mana, granted abilities, and active
  effects.
- **Enemy**: an object with profile data, faction tags, current health, base
  vitality, base armor, and active effects.
- **Current health**: stored `Attribute`; it is mutable combat state.
- **Max health**: `DerivedAttribute`; it is calculated from base vitality and
  active max-health effects.
- **Effective attack speed**: `DerivedAttribute`; it is calculated from base
  attack speed and active attack-speed effects.
- **Mana**: stored `Attribute`; it is spent by ability commitment.
- **Slash**: player ability that targets an enemy and applies instant damage.
- **Quickened Strikes**: player self-buff ability that applies a 10 second
  attack speed effect and a cooldown effect.
- **Fortify**: player self-buff ability that applies a temporary max-health
  effect to demonstrate why max health is derived while current health is stored.
- **Bleed**: periodic enemy effect used to teach periodic effect execution.

### Runtime Shape

The chapters should converge on a runtime shaped roughly like this:

```rust
struct CombatRuntime {
    objects: ObjectStore,
    profiles: DataStore<CombatProfile>,
    factions: DataStore<Faction>,
    current_health: Attribute,
    base_vitality: Attribute,
    base_attack_speed: Attribute,
    mana: Attribute,
    abilities: AbilityStore<TagSet<CombatTag>, AbilityPayload>,
    effects: EffectPipeline<TagSet<CombatTag>, EffectPayload>,
    cooldowns: EffectPipeline<TagSet<CombatTag>, EffectPayload>,
    max_health: DerivedAttribute,
    attack_speed: DerivedAttribute,
}
```

This is only a teaching shape, not a framework abstraction. The docs should
state that consumers can use ordinary Rust structs, ECS resources, engine
services, or server session state instead.

## Issue Plan

### DOC-001: Rework Navigation into a Book-Like Structure

**Content intent**: information architecture.

**Files**: `docs/content/docs/meta.json`, landing pages, section `meta.json`
files.

**Scope**:

- Replace `tutorials`, `how-to`, `reference`, and `explanation` navigation with
  `getting-started`, `rpg-combat`, `core-concepts`, `runtime-patterns`,
  `design-notes`, and `api-reference`.
- Keep `api-reference` as a separate top-level section.
- Update the home page to describe the reader path.
- Preserve existing starter pages only where they fit the new structure.

**Acceptance criteria**:

- No top-level nav item is named Tutorials, How-to, Reference, or Explanation.
- API Reference is top-level and clearly separate from narrative docs.
- The site builds and search indexes all new sections.

### DOC-002: Build a Compile-Checked RPG Combat Example

**Content intent**: supporting artifact.

**Files**: `core/examples/rpg_combat.rs` or an equivalent compile-checked
example location.

**Scope**:

- Build the complete player-versus-enemy RPG example used throughout the docs.
- Include objects, data stores, tags, attributes, derived attributes, abilities,
  effects, clocks, lifecycle channels, signals, and cleanup.
- Keep code organized into excerptable functions so chapters can cite stable
  slices.

**Acceptance criteria**:

- The example demonstrates Slash, Quickened Strikes, Fortify, Bleed, and cleanup
  after combatant destruction.
- The example is compiled or tested in normal verification.
- All chapter code uses names from this fixture.

### DOC-003: Getting Started - What Flexweave Is

**Content intent**: conceptual orientation.

**Page**: `docs/content/docs/getting-started/what-is-flexweave.mdx`

**Scope**:

- Explain Flexweave as deterministic mechanics primitives for Rust runtimes.
- Clarify that caller code owns game/application meaning.
- Show the major primitive families: objects, data, attributes, tags, abilities,
  effects, clocks, lifecycle facts, signals.

**Writing style**:

- Use plain orientation prose.
- Keep examples tiny and defer implementation to the RPG chapters.

**Acceptance criteria**:

- New readers understand what Flexweave does and does not own.

### DOC-004: Getting Started - Install and Verify

**Content intent**: task guide.

**Page**: `docs/content/docs/getting-started/install-and-verify.mdx`

**Scope**:

- Show dependency setup.
- Show local workspace verification commands.
- Show the first successful import.

**Acceptance criteria**:

- A consumer can add Flexweave and confirm the crate is available.

### DOC-005: Getting Started - Mechanics at the Boundary

**Content intent**: conceptual orientation.

**Page**: `docs/content/docs/getting-started/mechanics-at-the-boundary.mdx`

**Scope**:

- Explain the boundary between Flexweave primitives and consumer runtime logic.
- Introduce the RPG example as caller-owned domain code.
- Preview the book path.

**Acceptance criteria**:

- Readers understand why later chapters define game nouns outside Flexweave.

### DOC-006: Chapter 1 - Create Combatants

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/01-create-combatants.mdx`

**Scope**:

- Create an `ObjectStore`.
- Create player and enemy object ids in deterministic order.
- Attach `CombatProfile` data.
- Assert ids and attached profile values.

**Acceptance criteria**:

- The reader has a player and enemy object with caller-owned profile data.

### DOC-007: Chapter 2 - Attach Data and Tags

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/02-attach-data-and-tags.mdx`

**Scope**:

- Add `Faction` data.
- Define `CombatTag` and attach tag sets to ability/effect payloads where used.
- Select enemy targets deterministically with `query::collect_where`.

**Acceptance criteria**:

- The reader can select enemies in deterministic object order.

### DOC-008: Chapter 3 - Add Attributes

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/03-add-attributes.mdx`

**Scope**:

- Add `current_health`, `base_vitality`, `base_attack_speed`, and `mana`.
- Use `AttributeSet` for current health and mana changes.
- Introduce mutation results without hooks yet.

**Acceptance criteria**:

- The player and enemy have stored numeric combat state.

### DOC-009: Chapter 4 - Derive Max Health and Attack Speed

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/04-derived-max-health-and-attack-speed.mdx`

**Scope**:

- Add `DerivedAttribute` for max health.
- Add `DerivedAttribute` for effective attack speed.
- Calculate from base attributes plus active effects.
- Demonstrate why current health remains stored.

**Acceptance criteria**:

- The page establishes the current health versus max health distinction.

### DOC-010: Chapter 5 - Grant Player Abilities

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/05-grant-player-abilities.mdx`

**Scope**:

- Define `AbilityPayload` for Slash, Quickened Strikes, and Fortify.
- Define ability tags.
- Grant abilities to the player with checked object references.
- Look up granted ability ids by owner and tag.

**Acceptance criteria**:

- The player owns three usable ability grants.

### DOC-011: Chapter 6 - Target and Activate Abilities

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/06-target-and-activate-abilities.mdx`

**Scope**:

- Build activation inputs from selected target data.
- Add an activation gate for cooldowns, mana, and target validity.
- Show allowed and blocked activation outcomes.

**Acceptance criteria**:

- Readers see activation attempt, start, and rejection lifecycle facts.

### DOC-012: Chapter 7 - Commit Abilities into Effects

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/07-commit-abilities-into-effects.mdx`

**Scope**:

- Implement `AbilityCommitAction`.
- Spend mana at commitment.
- Convert Slash, Quickened Strikes, and Fortify into effect applications.
- Explain commitment as the point of no return in the example.

**Acceptance criteria**:

- Ability commitment produces costs, cooldowns, and combat effects through
  caller-owned code.

### DOC-013: Chapter 8 - Apply Damage and Buffs

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/08-apply-damage-and-buffs.mdx`

**Scope**:

- Execute Slash as instant damage.
- Apply Quickened Strikes as a 10 second attack speed buff.
- Apply Fortify as a temporary max-health buff.
- Refresh derived attributes after effect application.

**Acceptance criteria**:

- Enemy health changes, player attack speed rises, and player max health rises.

### DOC-014: Chapter 9 - Advance Time and Expire Effects

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/09-advance-time-and-expire-effects.mdx`

**Scope**:

- Advance effect lifetimes with `MechanicsTick`.
- Expire Quickened Strikes after 10 seconds.
- Refresh attack speed after expiration.
- Run Bleed as a periodic effect.

**Acceptance criteria**:

- Readers see active effects advance, execute periodically, and expire
  deterministically.

### DOC-015: Chapter 10 - Publish Combat Events

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/10-publish-combat-events.mdx`

**Scope**:

- Publish attribute, ability, and effect facts to `EventChannel`.
- Project selected effect facts to signals.
- Drain retained facts into a simple combat log.

**Acceptance criteria**:

- The RPG flow produces inspectable runtime events without implying automatic
  publication.

### DOC-016: Chapter 11 - Clean Up Combatants

**Content intent**: tutorial.

**Page**: `docs/content/docs/rpg-combat/11-clean-up-combatants.mdx`

**Scope**:

- Destroy defeated or removed objects.
- Run `ObjectDestroy` with registered stores.
- Clean object-keyed data, abilities, active effects, and cooldowns.

**Acceptance criteria**:

- The example demonstrates cleanup and checked paths rejecting destroyed ids.

### DOC-017: Chapter 12 - Complete Runtime

**Content intent**: tutorial and recap.

**Page**: `docs/content/docs/rpg-combat/12-complete-runtime.mdx`

**Scope**:

- Present the complete RPG runtime assembled from previous chapters.
- Link to the compile-checked example.
- Summarize how data flows through ability activation, effect application,
  derived stats, ticking, and events.

**Acceptance criteria**:

- Readers can see the whole example as one coherent system.

### DOC-018: Core Concept - Objects and Attached Data

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/objects-and-attached-data.mdx`

**Scope**:

- Explain objects as domain-neutral handles.
- Explain object ids, deterministic order, and attached data.
- Use player/enemy examples without step-by-step tutorial pacing.

**Acceptance criteria**:

- Readers understand how app entities map to Flexweave objects.

### DOC-019: Core Concept - Attributes and Derived Attributes

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/attributes-and-derived-attributes.mdx`

**Scope**:

- Explain stored attributes, mutation results, listeners, and hooks.
- Explain derived attributes as caller-owned calculations.
- Use current health, max health, mana, and attack speed.

**Acceptance criteria**:

- Readers understand when to store state and when to derive it.

### DOC-020: Core Concept - Tags and Target Selection

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/tags-and-target-selection.mdx`

**Scope**:

- Explain tag paths, exact matches, prefix matches, atom matches, and queries.
- Explain deterministic target selection.
- Use faction, ability, cooldown, buff, and damage examples.

**Acceptance criteria**:

- Readers understand tags as structured labels rather than flat strings.

### DOC-021: Core Concept - Abilities and Commitment

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/abilities-and-commitment.mdx`

**Scope**:

- Explain grants, activation attempts, gates, active activation state, commit,
  end, cancel, rollback, and revocation.
- Explain that costs/cooldowns are caller-owned behavior at commit time.

**Acceptance criteria**:

- Readers understand the ability lifecycle without scanning API types.

### DOC-022: Core Concept - Effects and Active Instances

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/effects-and-active-instances.mdx`

**Scope**:

- Explain instant, duration, periodic, and indefinite effects.
- Explain effect application, execution, active instances, advancement, removal,
  and expiration.
- Use damage, attack speed buff, max health buff, cooldown, and Bleed.

**Acceptance criteria**:

- Readers understand how active effects become runtime state.

### DOC-023: Core Concept - Lifecycle Facts, Channels, and Signals

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/lifecycle-facts-channels-and-signals.mdx`

**Scope**:

- Explain lifecycle facts as primitive mechanics facts.
- Explain event channels as caller-wired transport and retention.
- Explain signals as projected/exportable facts.

**Acceptance criteria**:

- Readers understand the difference between Flexweave facts and app events.

### DOC-024: Core Concept - Clocks and Mechanics Ticks

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/clocks-and-mechanics-ticks.mdx`

**Scope**:

- Explain clock units.
- Explain `MechanicsTick` and `MechanicsDriver`.
- Compare turn-based and realtime runtime loops.

**Acceptance criteria**:

- Readers understand how time enters Flexweave without Flexweave owning a game
  loop.

### DOC-025: Core Concept - Registries and Definition Bundles

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/registries-and-definition-bundles.mdx`

**Scope**:

- Explain ability/effect/signal/channel definitions.
- Explain runtime-scoped bundles.
- Explain generic `Registry` helpers.

**Acceptance criteria**:

- Readers understand the difference between authored content and active runtime
  definitions.

### DOC-026: Core Concept - Object Lifetime and Cleanup

**Content intent**: conceptual guide.

**Page**: `docs/content/docs/core-concepts/object-lifetime-and-cleanup.mdx`

**Scope**:

- Explain object ids as Flexweave handles.
- Explain stale references and checked runtime paths.
- Explain cleanup drivers and store registration.

**Acceptance criteria**:

- Readers understand object destruction before using cleanup recipes.

### DOC-027: Runtime Pattern - Model Health, Mana, and Resources

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/model-health-mana-and-resources.mdx`

**Scope**:

- Store current health and mana.
- Derive max health.
- Decide when to clamp current health after max health decreases.

**Acceptance criteria**:

- Consumers have a concrete resource modeling pattern.

### DOC-028: Runtime Pattern - Clamp and Reject Attribute Mutations

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/clamp-and-reject-attribute-mutations.mdx`

**Scope**:

- Use `AttributeMutationHooks`.
- Clamp health into `[0, max_health]`.
- Reject impossible mana spends.
- Handle committed, rejected, and unchanged results.

**Acceptance criteria**:

- Consumers can enforce local runtime policy around attribute changes.

### DOC-029: Runtime Pattern - Calculate Derived Stats from Effects

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/calculate-derived-stats-from-effects.mdx`

**Scope**:

- Visit active effects by target.
- Calculate attack speed and max health from effect payloads.
- Refresh derived attributes after effect lifecycle changes.

**Acceptance criteria**:

- Consumers can implement effect-backed derived stats.

### DOC-030: Runtime Pattern - Gate Abilities with Runtime State

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/gate-abilities-with-runtime-state.mdx`

**Scope**:

- Implement `AbilityActivationGate`.
- Block by cooldown tags, mana, and target validity.
- Map block reasons to app-level command responses.

**Acceptance criteria**:

- Consumers can validate ability use before commitment.

### DOC-031: Runtime Pattern - Turn Abilities into Costs, Cooldowns, and Effects

**Content intent**: task guide.

**Page**:
`docs/content/docs/runtime-patterns/turn-abilities-into-costs-cooldowns-and-effects.mdx`

**Scope**:

- Implement `AbilityCommitAction`.
- Spend mana.
- Apply cooldown effects.
- Apply combat effects.
- Handle commit failures and rollback.

**Acceptance criteria**:

- Consumers can connect the ability lifecycle to runtime behavior.

### DOC-032: Runtime Pattern - Apply Instant Damage and Healing

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/apply-instant-damage-and-healing.mdx`

**Scope**:

- Use instant effect definitions.
- Use `EffectActionExecutor`.
- Mutate current health and publish attribute changes.

**Acceptance criteria**:

- Consumers can implement one-shot combat effects.

### DOC-033: Runtime Pattern - Run Temporary Buffs and Debuffs

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/run-temporary-buffs-and-debuffs.mdx`

**Scope**:

- Use duration effects.
- Apply attack speed and max health buffs.
- Remove or expire effects.
- Refresh derived attributes.

**Acceptance criteria**:

- Consumers can implement finite buffs that affect derived stats.

### DOC-034: Runtime Pattern - Run Periodic Effects

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/run-periodic-effects.mdx`

**Scope**:

- Use periodic effect definitions.
- Implement Bleed.
- Advance the effect pipeline and execute periodic actions.

**Acceptance criteria**:

- Consumers can run deterministic periodic effects.

### DOC-035: Runtime Pattern - Advance Turn-Based and Realtime Mechanics

**Content intent**: task guide.

**Page**:
`docs/content/docs/runtime-patterns/advance-turn-based-and-realtime-mechanics.mdx`

**Scope**:

- Use direct `MechanicsTick` for turn-based systems.
- Use `FixedStepClock`, `RealtimeClock`, and `RealtimeClockAccumulator` for
  realtime systems.
- Register multiple mechanics stores in a driver.

**Acceptance criteria**:

- Consumers can wire Flexweave ticking into a runtime loop.

### DOC-036: Runtime Pattern - Publish Lifecycle Facts

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/publish-lifecycle-facts.mdx`

**Scope**:

- Define event channels.
- Publish attribute, ability, and effect facts explicitly.
- Choose retained or drop retention.
- Drain retained facts into a combat log.

**Acceptance criteria**:

- Consumers can move Flexweave facts into app infrastructure.

### DOC-037: Runtime Pattern - Project Signals for Adapters

**Content intent**: task guide.

**Page**: `docs/content/docs/runtime-patterns/project-signals-for-adapters.mdx`

**Scope**:

- Define signal definitions.
- Project effect facts into signal facts.
- Reinvoke while-active signals.
- Hand signals to UI, audio, scripting, network, or analytics adapters.

**Acceptance criteria**:

- Consumers can derive adapter-facing facts from primitive lifecycle facts.

### DOC-038: Runtime Pattern - Organize Runtime Definition Bundles

**Content intent**: task guide.

**Page**:
`docs/content/docs/runtime-patterns/organize-runtime-definition-bundles.mdx`

**Scope**:

- Compose ability, effect, signal, and channel definitions per runtime session.
- Validate route/channel keys.
- Use `Registry` for caller-owned definition lookup.

**Acceptance criteria**:

- Consumers can avoid global catalog assumptions.

### DOC-039: Design Note - Deterministic Mechanics

**Content intent**: explanation.

**Page**: `docs/content/docs/design-notes/deterministic-mechanics.mdx`

**Scope**:

- Explain deterministic object ids, iteration order, query order, lifecycle
  facts, and store registration order.
- Explain what Flexweave does and does not guarantee across consumer systems.

**Acceptance criteria**:

- The page answers why deterministic order is part of the library contract.

### DOC-040: Design Note - Product Boundaries

**Content intent**: explanation.

**Page**: `docs/content/docs/design-notes/product-boundaries.mdx`

**Scope**:

- Expand the existing product-boundaries page.
- Clarify that Flexweave does not own ECS integration, rendering, networking,
  persistence, AI, animation, content loading, balance, or deployment.

**Acceptance criteria**:

- Consumers know which responsibilities stay in their app.

### DOC-041: Design Note - Stored vs Derived State

**Content intent**: explanation.

**Page**: `docs/content/docs/design-notes/stored-vs-derived-state.mdx`

**Scope**:

- Discuss current health versus max health.
- Discuss derived attributes as caller-owned calculations.
- Discuss refresh timing and clamping policy.

**Acceptance criteria**:

- The page answers why max health is derived but current health is stored.

### DOC-042: Design Note - Ability and Effect Lifecycle Boundaries

**Content intent**: explanation.

**Page**:
`docs/content/docs/design-notes/ability-effect-lifecycle-boundaries.mdx`

**Scope**:

- Explain where ability lifecycle ends and effect lifecycle begins.
- Explain commitment without equating it to costs, cooldowns, or resource spend.
- Compare instant, active, periodic, and cleanup paths.

**Acceptance criteria**:

- Consumers can decide where behavior belongs in their runtime.

### DOC-043: Design Note - Facts, Channels, Signals, and App Events

**Content intent**: explanation.

**Page**:
`docs/content/docs/design-notes/facts-channels-signals-and-app-events.mdx`

**Scope**:

- Compare raw lifecycle facts, event channels, projected signals, and
  application-level events.
- Explain explicit wiring.
- Explain retention and projection policy.

**Acceptance criteria**:

- The page answers why these layers are separate.

### DOC-044: Design Note - Tags as Structured Labels

**Content intent**: explanation.

**Page**: `docs/content/docs/design-notes/tags-as-structured-labels.mdx`

**Scope**:

- Explain tag paths versus flat flags.
- Explain exact, prefix, atom, all/any/none matching.
- Use RPG tags such as `ability/slash`, `status/buff/haste`, and
  `status/cooldown`.

**Acceptance criteria**:

- Consumers understand tag modeling tradeoffs.

### DOC-045: Design Note - Content Authoring vs Runtime Definitions

**Content intent**: explanation.

**Page**:
`docs/content/docs/design-notes/content-authoring-vs-runtime-definitions.mdx`

**Scope**:

- Explain authored catalogs versus active in-memory definitions.
- Explain why Flexweave validates definitions but does not load content packs.
- Discuss zone, session, and combat-instance definition scopes.

**Acceptance criteria**:

- Consumers understand where authored ability/effect content lives.

### DOC-046: API Reference - Overview

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/index.mdx`

**Scope**:

- Introduce API Reference as exhaustive public surface documentation.
- Link every API page.
- State that narrative pages are elsewhere.

**Acceptance criteria**:

- API Reference is clearly separate from the guide content.

### DOC-047: API Reference - Identity

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/identity.mdx`

**Scope**:

- Document `ObjectId`, `INVALID_OBJECT_ID`, and `ObjectStore`.
- Include constructors, methods, ordering guarantees, and error cases.

**Acceptance criteria**:

- Every public identity export is covered.

### DOC-048: API Reference - Data Store

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/data-store.mdx`

**Scope**:

- Document `DataStore<T>`.
- Include attach, detach, has, get, count, and emptiness behavior.

**Acceptance criteria**:

- Every public data-store export is covered.

### DOC-049: API Reference - Query

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/query.mdx`

**Scope**:

- Document `query::require_object`, `query::require_attached`, and
  `query::collect_where`.
- Include ordering and error behavior.

**Acceptance criteria**:

- Every public query helper is covered.

### DOC-050: API Reference - Attribute

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/attribute.mdx`

**Scope**:

- Document `Attribute`, `AttributeSet`, `AttributeValue`, mutation request,
  mutation hooks, mutation decisions, mutation results, mutation rejections, and
  `AttributeChange`.
- Include listener, hook, and streaming order.

**Acceptance criteria**:

- Every public attribute export is covered.

### DOC-051: API Reference - Derived Attribute

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/derived-attribute.mdx`

**Scope**:

- Document `DerivedAttribute`, `DerivedAttributeRefresh`, and `DerivedChange`.
- Include sync, refresh, listener, streaming, tracking, and untracking behavior.

**Acceptance criteria**:

- Every public derived-attribute export is covered.

### DOC-052: API Reference - Tag

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/tag.mdx`

**Scope**:

- Document `Tag<Atom>`, `TagSet<Atom>`, `TagSetQuery<Atom>`, and
  `TagCollection`.
- Include exact, prefix, atom, all/any/none matching.

**Acceptance criteria**:

- Every public tag export is covered.

### DOC-053: API Reference - Ability

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/ability.mdx`

**Scope**:

- Document ability ids, definitions, registries, grants, store, active
  activations, commands, gates, commit actions, executors, lifecycle events,
  outcomes, and errors.
- Include checked versus low-level command paths.

**Acceptance criteria**:

- Every public ability export from `core/src/lib.rs` is covered or explicitly
  grouped.

### DOC-054: API Reference - Effect

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/effect.mdx`

**Scope**:

- Document active effect ids, definitions, kinds, clock policy, routing,
  application inputs, initializers, executors, pipeline, tick/remove operations,
  lifecycle events, outcomes, and errors.
- Include instant, duration, periodic, and indefinite semantics.

**Acceptance criteria**:

- Every public effect export from `core/src/lib.rs` is covered or explicitly
  grouped.

### DOC-055: API Reference - Lifecycle

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/lifecycle.mdx`

**Scope**:

- Document `LifecycleEventKind`, `LifecycleEvent`, `LocalLifecycleEvent`,
  channel definitions, route definitions, channels, retention, connection
  handles, scoped connections, and channel errors.
- Include owned and borrowed publication constraints.

**Acceptance criteria**:

- Every public lifecycle export is covered.

### DOC-056: API Reference - Signal

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/signal.mdx`

**Scope**:

- Document signal definitions, definition registries, kinds, retention policy,
  export policy, tag matching, facts, removal reasons, and projection.
- Include effect-event projection and reinvocation.

**Acceptance criteria**:

- Every public signal export is covered.

### DOC-057: API Reference - Clock

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/clock.mdx`

**Scope**:

- Document `ClockUnits`, `Clock`, `FixedStepClock`, `RealtimeClock`, and
  `RealtimeClockAccumulator`.
- Include duration-to-unit conversion behavior.

**Acceptance criteria**:

- Every public clock export is covered.

### DOC-058: API Reference - Mechanics

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/mechanics.mdx`

**Scope**:

- Document `MechanicsStore`, `MechanicsDriver`, and `MechanicsTick`.
- Include store registration order, zero elapsed behavior, and streaming.

**Acceptance criteria**:

- Every public mechanics export is covered.

### DOC-059: API Reference - Registry

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/registry.mdx`

**Scope**:

- Document `RegistryEntry`, `DefinitionRegistryEntry`, and `Registry`.
- Include lookup and definition-building behavior.

**Acceptance criteria**:

- Every public registry export is covered.

### DOC-060: API Reference - Object Lifecycle

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/object-lifecycle.mdx`

**Scope**:

- Document `ObjectLifecycleStore`, `ObjectDestructionDriver`, and
  `ObjectDestroy`.
- Include registered-store cleanup behavior and emitted events.

**Acceptance criteria**:

- Every public object-lifecycle export is covered.

### DOC-061: API Reference - Errors and Outcomes

**Content intent**: API reference.

**Page**: `docs/content/docs/api-reference/errors-and-outcomes.mdx`

**Scope**:

- Document `CoreError`.
- Catalog public definition errors, runtime command errors, and outcome enums.
- State when to match command outcomes versus emitted lifecycle facts.

**Acceptance criteria**:

- Consumers have a complete error-handling map.

### DOC-062: Cross-Link and Coverage Pass

**Content intent**: quality pass.

**Files**: all published docs pages and all `meta.json` files.

**Scope**:

- Add previous/next links through the RPG chapters.
- Add concept links from RPG chapters.
- Add runtime-pattern links from concept pages.
- Add API links from narrative pages.
- Ensure API Reference remains separate from the book-like content.

**Acceptance criteria**:

- Every public module has API Reference coverage.
- Every major narrative page links to the relevant API page.
- No navigation group is named after a Diataxis mode.
- `bun fix`, `bun run check:docs`, and `bun run build:docs` pass.

## Coverage Matrix

| Core surface                          | RPG chapters  | Core concepts | Runtime patterns | API Reference |
| ------------------------------------- | ------------- | ------------- | ---------------- | ------------- |
| `identity`, `ObjectStore`, `ObjectId` | DOC-006       | DOC-018       |                  | DOC-047       |
| `data_store`                          | DOC-006/7     | DOC-018       |                  | DOC-048       |
| `query`                               | DOC-007       | DOC-020       | DOC-027          | DOC-049       |
| `attribute`                           | DOC-008/13    | DOC-019       | DOC-027/28/32    | DOC-050       |
| `derived_attribute`                   | DOC-009/13/14 | DOC-019       | DOC-027/29/33    | DOC-051       |
| `tag`                                 | DOC-007/10/13 | DOC-020       | DOC-030/33       | DOC-052       |
| `ability`                             | DOC-010/11/12 | DOC-021       | DOC-030/31       | DOC-053       |
| `effect`                              | DOC-012/13/14 | DOC-022       | DOC-031/34       | DOC-054       |
| `lifecycle`                           | DOC-015       | DOC-023       | DOC-036          | DOC-055       |
| `signal`                              | DOC-015       | DOC-023       | DOC-037          | DOC-056       |
| `clock`                               | DOC-014       | DOC-024       | DOC-035          | DOC-057       |
| `mechanics`                           | DOC-014       | DOC-024       | DOC-035          | DOC-058       |
| `registry`                            | DOC-010/12    | DOC-025       | DOC-038          | DOC-059       |
| `object_lifecycle`                    | DOC-016       | DOC-026       |                  | DOC-060       |
| `errors` and outcomes                 | DOC-011/12    | DOC-021/22    | DOC-028/30/31    | DOC-061       |

## Recommended Implementation Order

1. DOC-001 first, so navigation matches the intended book-like structure.
2. DOC-002 next, so all code samples have one compile-checked source.
3. DOC-006 through DOC-017 next, because the RPG chapters define the reader
   journey.
4. DOC-018 through DOC-026 after the main example exists, so concept pages can
   explain real code without becoming tutorials.
5. DOC-027 through DOC-038 next, using established runtime state from the RPG
   example.
6. DOC-039 through DOC-045 after concept and pattern pages exist, so design
   notes can link to concrete material.
7. DOC-046 through DOC-061 in parallel with the narrative work, but keep API
   Reference isolated in navigation and page tone.
8. DOC-062 last as the coverage and cross-link quality pass.
