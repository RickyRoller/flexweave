# Model Complex Attributes

Use consumer-owned runtime models when attribute behavior needs to combine
stored attributes, derived values, effects, tags, scalable values, and local
policy.

Flexweave attributes are primitive signed numeric channels on objects. Effects
and abilities can carry intent, configuration, tags, lifecycle facts, and caller
payloads, but Flexweave does not make effects directly mutate a named attribute
set or prescribe how damage, healing, mitigation, caps, resource costs, or
cooldowns are calculated.

## Define a Runtime Model

Group the primitive stores that matter to the consumer runtime, then expose
domain methods that ability and effect hooks can call.

```rust
use flexweave::{Attribute, AttributeValue, DerivedAttribute, ObjectId};

struct CombatAttributes {
    health: Attribute,
    shield: Attribute,
}

struct CombatModel {
    attributes: CombatAttributes,
    mitigation: DerivedAttribute,
}

impl CombatModel {
    fn apply_damage(&mut self, target: ObjectId, amount: AttributeValue) -> AttributeValue {
        let mitigation = self.mitigation.get(target).unwrap_or(0.0);
        let mut remaining = amount * (1.0 - mitigation);
        let shield = self.attributes.shield.get(target).unwrap_or(0.0);
        let absorbed = remaining.min(shield);

        if absorbed > 0.0 {
            self.attributes.shield.set(target, shield - absorbed);
            remaining -= absorbed;
        }

        let health = self.attributes.health.get(target).unwrap_or(0.0);
        self.attributes.health.set(target, (health - remaining).max(0.0));
        remaining
    }
}
```

## Call the Model From Hooks

Keep authored effects and ability payloads from becoming hardcoded attribute
operations. Let them describe intent or configured values, then let the consumer
runtime decide how that intent interacts with attributes.

```rust
use flexweave::{
    AbilityHooks, ActiveAbilityView, AttributeValue, ObjectId, TagSet,
};

struct Runtime {
    combat: CombatModel,
}

struct DamagePayload {
    target_id: ObjectId,
    amount: AttributeValue,
}

struct Hooks;

impl AbilityHooks<Runtime, TagSet<String>, DamagePayload> for Hooks {
    type Error = ();
    type BlockReason = ();

    async fn on_commit(
        &mut self,
        context: &mut Runtime,
        active: ActiveAbilityView<'_, TagSet<String>, DamagePayload>,
    ) -> Result<(), Self::Error> {
        context
            .combat
            .apply_damage(active.payload.target_id, active.payload.amount);
        Ok(())
    }
}
```

The same pattern works from an effect execution adapter, an event-channel
listener, or any consumer-owned runtime service. Flexweave provides the
deterministic primitives and lifecycle hooks; the consumer owns the calculation.

## Keep the Boundary

Do not add a generic attribute-model wrapper unless it owns a real runtime
invariant. A consumer-owned model can use ordinary Rust fields, methods, traits,
or services that match the runtime's domain. That keeps complex mechanics
explicit instead of hiding them behind default integrations that later need to
be replaced by custom calculations.
