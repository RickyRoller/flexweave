# Use Flexweave

Use Flexweave when a runtime needs deterministic mechanics primitives.

## Add the Crate

This phase reserves the crate path and package name:

```toml
[dependencies]
flexweave = "0.0.0"
```

## Verify Flexweave Locally

Run Flexweave commands from the Flexweave repository root:

```bash
cargo build -p flexweave
cargo test -p flexweave
```

## Choose Lifecycle Event Shape

Use executor event sinks to choose how lifecycle facts are delivered. Borrowed
event sinks are for hot streaming paths where listeners handle the fact
immediately. Owned event sinks are for retained, replayed, inspected, or routed
facts. Drop-only event channels can publish borrowed events; retained channels
require owned events because they store the emitted batch.

## Compose Runtime Definition Bundles

Treat ability and effect registries as runtime-scoped bundles, not as one global
game catalog. For example, a procedural server can select the enemy archetypes
for a zone session, compose only those definitions, validate the bundle, and
drop it when the zone unloads.

One possible consumer-owned shape:

```rust
use flexweave::{
    AbilityDefinition, AbilityDefinitions, EffectApply, EffectApplicationInput,
    EffectDefinition, EffectDefinitions, Grant, NoEffectExecutor,
};

struct EnemyArchetype<AbilitySchema, EffectSchema> {
    ability_definitions: Vec<AbilityDefinition<AbilitySchema>>,
    effect_definitions: Vec<EffectDefinition<EffectSchema>>,
}

struct ZoneDefinitionBundle<AbilitySchema, EffectSchema> {
    abilities: AbilityDefinitions<AbilitySchema>,
    effects: EffectDefinitions<EffectSchema>,
}

fn build_zone_definitions<AbilitySchema, EffectSchema>(
    selected_archetypes: &[EnemyArchetype<AbilitySchema, EffectSchema>],
) -> Result<
    ZoneDefinitionBundle<AbilitySchema, EffectSchema>,
    Box<dyn std::error::Error>,
>
where
    AbilitySchema: Clone,
    EffectSchema: Clone,
{
    let ability_definitions = selected_archetypes
        .iter()
        .flat_map(|archetype| archetype.ability_definitions.iter().cloned());
    let effect_definitions = selected_archetypes
        .iter()
        .flat_map(|archetype| archetype.effect_definitions.iter().cloned());

    Ok(ZoneDefinitionBundle {
        abilities: AbilityDefinitions::new(ability_definitions)?,
        effects: EffectDefinitions::new(effect_definitions)?,
    })
}
```

The zone runtime then passes that session-local bundle to registered grant
helpers and operation builders:

```rust
let ability_id = ability_store.grant_registered(
    &zone_definitions.abilities,
    "enemy/wasp/sting",
    Grant::new(enemy_id, ability_tags, ability_payload),
)?;

let mut effect_events =
    NoEffectExecutor::new().with_owned_events(|event| lifecycle_events.push(event));
let mut effect_context = ();
let active_poison = EffectApply::registered(
    &zone_definitions.effects,
    "enemy/wasp/poison",
    EffectApplicationInput::accept(
        Some(enemy_id),
        player_id,
        effect_tags,
        effect_payload,
    ),
)
.run_with_executor(&mut effect_pipeline, &mut effect_context, &mut effect_events)?;
```

`AbilityDefinitions::new` and `EffectDefinitions::new` reject duplicate keys
inside the bundle being constructed. If a zone selects many instances of the
same enemy archetype, dedupe the archetypes before building the bundle. If a
zone intentionally layers base, biome, encounter, or mod definitions, duplicate
key rejection happens at that zone composition point.

Flexweave does not load content packs, decide which enemies are in a zone, or
store authored catalog files. It validates the active in-memory bundle the caller
hands it and gives runtime helpers a stable key lookup surface.

## Model Complex Attributes

For attribute behavior that combines stored attributes, derived values, effects,
tags, and local runtime policy, keep the calculation in consumer code. See
[model complex attributes](./model-complex-attributes.md) for the recommended
shape.

## Keep the Boundary

Flexweave should remain independent of catalog files, generated output directories,
runtime bindings, application behavior, and consumer project source.
