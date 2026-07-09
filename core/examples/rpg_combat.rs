#[path = "rpg_combat/abilities.rs"]
mod abilities;

use abilities::{AbilityPayload, PlayerAbilities};
use flexweave::{
    AbilityActivation, AbilityActivationDecision, AbilityActivationGate, AbilityCommit,
    AbilityCommitAction, AbilityCommitActionExecutor, AbilityDefinitions, AbilityEnd,
    AbilityGateExecutor, AbilityLifecycleEvent, AbilityStore, ActiveAbilityView, Attribute,
    AttributeChange, AttributeSet, AttributeValue, DataStore, DerivedAttribute,
    DerivedAttributeRefresh, DerivedChange, EffectActionExecutor, EffectApplicationInput,
    EffectApply, EffectDefinition, EffectDefinitions, EffectExecutionView, EffectLifecycleEvent,
    EffectPipeline, EffectSourcePolicy, EffectTick, EventChannel, EventChannelDefinition,
    EventRetention, LifecycleEventKind, ObjectDestroy, ObjectDestructionDriver, ObjectId,
    ObjectStore, SignalDefinition, SignalDefinitions, SignalExportPolicy, SignalKind,
    SignalProjection, SignalRetentionPolicy, SignalTagMatch, Tag, TagSet,
};
use std::cell::RefCell;
use std::rc::Rc;

type CombatTags = TagSet<CombatTag>;
type CombatEffects = EffectPipeline<CombatTags, EffectPayload>;
type CombatEffectEvent = EffectLifecycleEvent<CombatTags, EffectPayload>;
type CombatAbilityEvent = AbilityLifecycleEvent<CombatTags, AbilityPayload>;
const TEN_SECONDS: u64 = 10_000;

fn main() {
    let report = run_demo();

    assert_eq!(report.player_id, ObjectId::new(1));
    assert_eq!(report.enemy_id, ObjectId::new(2));
    assert_eq!(report.enemy_health_after_slash, 68.0);
    assert_eq!(report.attack_speed_after_quickened, 1.5);
    assert_eq!(report.attack_speed_after_expiration, 1.0);
    assert_eq!(report.max_health_after_fortify, 125.0);
    assert_eq!(report.enemy_health_after_bleed, 62.0);
    assert_eq!(report.retained_effect_events, 19);
    assert!(report.projected_signal_keys.contains(&"damage".to_owned()));
    assert!(
        report
            .projected_signal_keys
            .contains(&"buff-start".to_owned())
    );
    assert!(report.destroyed_enemy);
}

#[derive(Clone, Debug, PartialEq)]
struct DemoReport {
    player_id: ObjectId,
    enemy_id: ObjectId,
    enemy_health_after_slash: AttributeValue,
    attack_speed_after_quickened: AttributeValue,
    attack_speed_after_expiration: AttributeValue,
    max_health_after_fortify: AttributeValue,
    enemy_health_after_bleed: AttributeValue,
    retained_effect_events: usize,
    projected_signal_keys: Vec<String>,
    destroyed_enemy: bool,
}

fn run_demo() -> DemoReport {
    let mut runtime = CombatRuntime::new();
    let player = runtime.create_combatant("Aria", Faction::Player);
    let enemy = runtime.create_combatant("Training Dummy", Faction::Enemy);

    runtime.seed_attributes(player, 100.0, 100.0, 1.0, 30.0);
    runtime.seed_attributes(enemy, 80.0, 80.0, 0.8, 0.0);
    runtime.state.sync_derived(player);
    runtime.state.sync_derived(enemy);

    let target = runtime
        .state
        .enemy_targets_for(player)
        .into_iter()
        .next()
        .expect("the demo creates one enemy");

    let ability_ids = runtime.grant_player_abilities(player, target);

    runtime.activate_and_commit(player, ability_ids.slash);
    let enemy_health_after_slash = runtime.state.current_health(enemy);

    runtime.activate_and_commit(player, ability_ids.quickened_strikes);
    runtime.state.refresh_derived(player);
    let attack_speed_after_quickened = runtime.state.attack_speed(player);

    runtime.activate_and_commit(player, ability_ids.fortify);
    runtime.state.refresh_derived(player);
    let max_health_after_fortify = runtime.state.max_health(player);

    runtime.apply_bleed(player, enemy);
    runtime.state.tick_effects(5_000);
    let enemy_health_after_bleed = runtime.state.current_health(enemy);

    runtime.state.tick_effects(5_000);
    runtime.state.refresh_derived(player);
    let attack_speed_after_expiration = runtime.state.attack_speed(player);

    let retained_effect_events = runtime.state.publish_retained_effect_events();
    let projected_signal_keys = runtime.state.project_signal_keys();
    let destroyed_enemy = runtime.state.destroy_enemy(enemy);

    DemoReport {
        player_id: player,
        enemy_id: enemy,
        enemy_health_after_slash,
        attack_speed_after_quickened,
        attack_speed_after_expiration,
        max_health_after_fortify,
        enemy_health_after_bleed,
        retained_effect_events,
        projected_signal_keys,
        destroyed_enemy,
    }
}

struct CombatRuntime {
    state: CombatState,
    abilities: AbilityStore<CombatTags, AbilityPayload>,
    ability_definitions: AbilityDefinitions<&'static str>,
}

impl CombatRuntime {
    fn new() -> Self {
        Self {
            state: CombatState::new(),
            abilities: AbilityStore::new(),
            ability_definitions: abilities::definitions(),
        }
    }

    fn create_combatant(&mut self, name: &'static str, faction: Faction) -> ObjectId {
        self.state.create_combatant(name, faction)
    }

    fn seed_attributes(
        &mut self,
        id: ObjectId,
        current_health: AttributeValue,
        base_vitality: AttributeValue,
        base_attack_speed: AttributeValue,
        mana: AttributeValue,
    ) {
        self.state
            .seed_attributes(id, current_health, base_vitality, base_attack_speed, mana);
    }

    fn grant_player_abilities(&mut self, player: ObjectId, target: ObjectId) -> PlayerAbilities {
        let slash = abilities::slash::grant(
            &self.ability_definitions,
            &self.state.objects,
            &mut self.abilities,
            player,
            target,
        );
        let quickened_strikes = abilities::quickened_strikes::grant(
            &self.ability_definitions,
            &self.state.objects,
            &mut self.abilities,
            player,
        );
        let fortify = abilities::fortify::grant(
            &self.ability_definitions,
            &self.state.objects,
            &mut self.abilities,
            player,
        );

        PlayerAbilities {
            slash,
            quickened_strikes,
            fortify,
        }
    }

    fn activate_and_commit(&mut self, owner: ObjectId, ability_id: flexweave::AbilityId) {
        let ability_events = Rc::clone(&self.state.ability_events);
        let mut gate = CombatGate;
        let mut gate_executor =
            AbilityGateExecutor::new(&mut gate).with_owned_events(move |event| {
                ability_events.borrow_mut().push(event);
            });
        let activation_id = AbilityActivation::registered(&self.ability_definitions, ability_id)
            .for_owner(owner)
            .run_with_executor(&mut self.abilities, &self.state, &mut gate_executor)
            .expect("demo activation should be allowed");

        let ability_events = Rc::clone(&self.state.ability_events);
        let mut commit = CombatCommit;
        let mut commit_executor =
            AbilityCommitActionExecutor::new(&mut commit).with_owned_events(move |event| {
                ability_events.borrow_mut().push(event);
            });
        AbilityCommit::new(activation_id)
            .run_with_executor(&mut self.abilities, &mut self.state, &mut commit_executor)
            .expect("demo commit should succeed");
        AbilityEnd::new(activation_id)
            .run(&mut self.abilities)
            .expect("demo ability should end");
    }

    fn apply_bleed(&mut self, source: ObjectId, target: ObjectId) {
        self.state.apply_effect(
            &EffectDefinition::periodic("effect/bleed", 10_000, 5_000, "BleedPayload"),
            EffectApplicationInput::accept(
                Some(source),
                target,
                tag_set([effect_damage_tag(), effect_bleed_tag()]),
                EffectPayload::Bleed {
                    damage_per_tick: 6.0,
                },
            ),
        );
    }
}

struct CombatState {
    objects: ObjectStore,
    profiles: DataStore<CombatProfile>,
    factions: DataStore<Faction>,
    current_health: Rc<RefCell<Attribute>>,
    base_vitality: Rc<RefCell<Attribute>>,
    base_attack_speed: Rc<RefCell<Attribute>>,
    mana: Rc<RefCell<Attribute>>,
    effects: Rc<RefCell<CombatEffects>>,
    cooldowns: CombatEffects,
    max_health: DerivedAttribute,
    attack_speed: DerivedAttribute,
    effect_definitions: EffectDefinitions<&'static str>,
    signal_definitions: SignalDefinitions<CombatTag, &'static str>,
    ability_events: Rc<RefCell<Vec<CombatAbilityEvent>>>,
    effect_events: Rc<RefCell<Vec<CombatEffectEvent>>>,
    attribute_events: Rc<RefCell<Vec<AttributeChange>>>,
    derived_events: Rc<RefCell<Vec<DerivedChange>>>,
}

impl CombatState {
    fn new() -> Self {
        let current_health = Rc::new(RefCell::new(Attribute::new()));
        let base_vitality = Rc::new(RefCell::new(Attribute::new()));
        let base_attack_speed = Rc::new(RefCell::new(Attribute::new()));
        let mana = Rc::new(RefCell::new(Attribute::new()));
        let effects = Rc::new(RefCell::new(EffectPipeline::new()));

        let max_health = {
            let base_vitality = Rc::clone(&base_vitality);
            let effects = Rc::clone(&effects);
            DerivedAttribute::new(move |id| {
                let base = base_vitality.borrow().get(id)?;
                let mut bonus = 0.0;
                effects.borrow().visit_target(id, |effect| {
                    if let EffectPayload::MaxHealthBonus { amount } = effect.payload {
                        bonus += amount;
                    }
                });
                Some(base + bonus)
            })
        };

        let attack_speed = {
            let base_attack_speed = Rc::clone(&base_attack_speed);
            let effects = Rc::clone(&effects);
            DerivedAttribute::new(move |id| {
                let base = base_attack_speed.borrow().get(id)?;
                let mut bonus = 0.0;
                effects.borrow().visit_target(id, |effect| {
                    if let EffectPayload::AttackSpeedBonus { amount } = effect.payload {
                        bonus += amount;
                    }
                });
                Some(base + bonus)
            })
        };

        Self {
            objects: ObjectStore::new(),
            profiles: DataStore::new(),
            factions: DataStore::new(),
            current_health,
            base_vitality,
            base_attack_speed,
            mana,
            effects,
            cooldowns: EffectPipeline::new(),
            max_health,
            attack_speed,
            effect_definitions: EffectDefinitions::new([
                EffectDefinition::instant("effect/slash-damage", "DamagePayload"),
                EffectDefinition::duration(
                    "effect/quickened-strikes",
                    TEN_SECONDS,
                    "AttackSpeedBuffPayload",
                ),
                EffectDefinition::duration("effect/fortify", TEN_SECONDS, "MaxHealthBuffPayload"),
                EffectDefinition::duration("effect/cooldown", TEN_SECONDS, "CooldownPayload"),
                EffectDefinition::periodic("effect/bleed", 10_000, 5_000, "BleedPayload"),
            ])
            .expect("demo effect definitions are valid"),
            signal_definitions: SignalDefinitions::new([
                signal_definition(
                    "damage",
                    SignalKind::Executed,
                    vec![
                        LifecycleEventKind::EffectExecuted,
                        LifecycleEventKind::EffectPeriodicExecuted,
                    ],
                    SignalTagMatch::Query(flexweave::TagSetQuery {
                        all: vec![effect_damage_tag()],
                        any: Vec::new(),
                        none: Vec::new(),
                    }),
                    "DamageSignal",
                ),
                signal_definition(
                    "buff-start",
                    SignalKind::ActiveStart,
                    vec![LifecycleEventKind::EffectActiveCreated],
                    SignalTagMatch::Query(flexweave::TagSetQuery {
                        all: vec![effect_buff_tag()],
                        any: Vec::new(),
                        none: Vec::new(),
                    }),
                    "BuffStartedSignal",
                ),
                signal_definition(
                    "buff-ended",
                    SignalKind::Removed,
                    vec![LifecycleEventKind::EffectExpired],
                    SignalTagMatch::Query(flexweave::TagSetQuery {
                        all: vec![effect_buff_tag()],
                        any: Vec::new(),
                        none: Vec::new(),
                    }),
                    "BuffEndedSignal",
                ),
            ])
            .expect("demo signal definitions are valid"),
            ability_events: Rc::new(RefCell::new(Vec::new())),
            effect_events: Rc::new(RefCell::new(Vec::new())),
            attribute_events: Rc::new(RefCell::new(Vec::new())),
            derived_events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn create_combatant(&mut self, name: &'static str, faction: Faction) -> ObjectId {
        let id = self.objects.create();
        self.profiles.attach(id, CombatProfile { name });
        self.factions.attach(id, faction);
        id
    }

    fn seed_attributes(
        &mut self,
        id: ObjectId,
        current_health: AttributeValue,
        base_vitality: AttributeValue,
        base_attack_speed: AttributeValue,
        mana: AttributeValue,
    ) {
        self.current_health.borrow_mut().attach(id, current_health);
        self.base_vitality.borrow_mut().attach(id, base_vitality);
        self.base_attack_speed
            .borrow_mut()
            .attach(id, base_attack_speed);
        self.mana.borrow_mut().attach(id, mana);
    }

    fn enemy_targets_for(&self, source: ObjectId) -> Vec<ObjectId> {
        let source_faction = self
            .factions
            .get(source)
            .copied()
            .expect("source combatant should have a faction");
        flexweave::query::collect_where(&self.objects, |candidate| {
            candidate != source
                && self
                    .factions
                    .get(candidate)
                    .is_some_and(|faction| *faction != source_faction)
                && self.current_health(candidate) > 0.0
        })
    }

    fn current_health(&self, id: ObjectId) -> AttributeValue {
        self.current_health.borrow().get(id).unwrap_or(0.0)
    }

    fn mana(&self, id: ObjectId) -> AttributeValue {
        self.mana.borrow().get(id).unwrap_or(0.0)
    }

    fn max_health(&self, id: ObjectId) -> AttributeValue {
        self.max_health.get(id).unwrap_or(0.0)
    }

    fn attack_speed(&self, id: ObjectId) -> AttributeValue {
        self.attack_speed.get(id).unwrap_or(0.0)
    }

    fn sync_derived(&mut self, id: ObjectId) {
        self.max_health.sync(id);
        self.attack_speed.sync(id);
    }

    fn refresh_derived(&mut self, id: ObjectId) {
        let derived_events = Rc::clone(&self.derived_events);
        DerivedAttributeRefresh::new(id).run_streaming(&mut self.max_health, |change| {
            derived_events.borrow_mut().push(change);
        });
        let derived_events = Rc::clone(&self.derived_events);
        DerivedAttributeRefresh::new(id).run_streaming(&mut self.attack_speed, |change| {
            derived_events.borrow_mut().push(change);
        });
    }

    fn spend_mana(&mut self, owner: ObjectId, amount: AttributeValue) {
        let current = self.mana(owner);
        let attribute_events = Rc::clone(&self.attribute_events);
        AttributeSet::new(owner, current - amount).run_streaming(
            &mut self.mana.borrow_mut(),
            |change| {
                attribute_events.borrow_mut().push(change);
            },
        );
    }

    fn apply_effect(
        &mut self,
        definition: &EffectDefinition<&'static str>,
        input: EffectApplicationInput<CombatTags, EffectPayload>,
    ) {
        let effect_events = Rc::clone(&self.effect_events);
        let mut action_context = EffectActionContext {
            current_health: Rc::clone(&self.current_health),
            attribute_events: Rc::clone(&self.attribute_events),
        };
        let mut action = apply_effect_payload;
        let mut executor = EffectActionExecutor::new(&mut action).with_owned_events(move |event| {
            effect_events.borrow_mut().push(event);
        });
        EffectApply::definition(definition, input)
            .checked(&self.objects, EffectSourcePolicy::RequireLiveSource)
            .run_with_executor(
                &mut self.effects.borrow_mut(),
                &mut action_context,
                &mut executor,
            )
            .expect("demo effect application should succeed");
    }

    fn apply_cooldown(&mut self, source: ObjectId, owner: ObjectId, tag: Tag<CombatTag>) {
        EffectApply::registered(
            &self.effect_definitions,
            "effect/cooldown",
            EffectApplicationInput::accept(
                Some(source),
                owner,
                tag_set([effect_cooldown_tag(), tag]),
                EffectPayload::Cooldown,
            ),
        )
        .checked(&self.objects, EffectSourcePolicy::RequireLiveSource)
        .run(&mut self.cooldowns)
        .expect("demo cooldown application should succeed");
    }

    fn tick_effects(&mut self, elapsed_units: u64) {
        let effect_events = Rc::clone(&self.effect_events);
        let mut action_context = EffectActionContext {
            current_health: Rc::clone(&self.current_health),
            attribute_events: Rc::clone(&self.attribute_events),
        };
        let mut action = apply_effect_payload;
        let mut executor = EffectActionExecutor::new(&mut action).with_owned_events(move |event| {
            effect_events.borrow_mut().push(event);
        });
        EffectTick::new(elapsed_units)
            .run_with_executor(
                &mut self.effects.borrow_mut(),
                &mut action_context,
                &mut executor,
            )
            .expect("demo tick should succeed");
        EffectTick::new(elapsed_units).run(&mut self.cooldowns);
    }

    fn publish_retained_effect_events(&self) -> usize {
        let definition = EventChannelDefinition::new(
            "combat/effects",
            [
                LifecycleEventKind::EffectApplicationAccepted,
                LifecycleEventKind::EffectActiveCreated,
                LifecycleEventKind::EffectExecuted,
                LifecycleEventKind::EffectPeriodicExecuted,
                LifecycleEventKind::EffectAdvanced,
                LifecycleEventKind::EffectExpired,
            ],
        )
        .expect("demo channel definition should be valid");
        let mut channel = EventChannel::with_retention(definition, EventRetention::Retain);

        for event in self.effect_events.borrow().iter().cloned() {
            channel
                .publish(event)
                .expect("demo channel accepts all effect facts");
        }

        channel.drain_retained().len()
    }

    fn project_signal_keys(&self) -> Vec<String> {
        let projection = SignalProjection::new(self.signal_definitions.clone());
        let mut keys = Vec::new();
        for event in self.effect_events.borrow().iter() {
            for fact in projection.project_effect_event(event) {
                keys.push(fact.key);
            }
        }
        keys
    }

    fn destroy_enemy(&mut self, enemy: ObjectId) -> bool {
        let effect_events = Rc::clone(&self.effect_events);
        {
            let mut current_health = self.current_health.borrow_mut();
            let mut base_vitality = self.base_vitality.borrow_mut();
            let mut base_attack_speed = self.base_attack_speed.borrow_mut();
            let mut mana = self.mana.borrow_mut();
            let mut effects = self.effects.borrow_mut();

            ObjectDestroy::new(enemy)
                .run_streaming(
                    ObjectDestructionDriver::<CombatEffectEvent>::new(&mut self.objects)
                        .with_store(&mut self.profiles)
                        .with_store(&mut self.factions)
                        .with_store(&mut *current_health)
                        .with_store(&mut *base_vitality)
                        .with_store(&mut *base_attack_speed)
                        .with_store(&mut *mana)
                        .with_store(&mut *effects)
                        .with_store(&mut self.cooldowns),
                    |event| effect_events.borrow_mut().push(event),
                )
                .expect("demo enemy destroy should succeed");
        }

        !self.objects.exists(enemy)
            && !self.profiles.has(enemy)
            && !self.factions.has(enemy)
            && !self.current_health.borrow().has(enemy)
    }
}

struct CombatGate;

impl AbilityActivationGate<CombatState, CombatTags, AbilityPayload> for CombatGate {
    type Error = CombatError;
    type BlockReason = CombatBlockReason;

    fn can_activate(
        &mut self,
        context: &CombatState,
        attempt: flexweave::AbilityActivationAttemptView<'_, CombatTags, AbilityPayload>,
    ) -> Result<AbilityActivationDecision<Self::BlockReason>, Self::Error> {
        if let Some(cooldown_tag) = attempt.payload.cooldown_tag()
            && context.cooldowns.has_tag(attempt.owner_id, &cooldown_tag)
        {
            return Ok(AbilityActivationDecision::Block(
                CombatBlockReason::OnCooldown,
            ));
        }

        let mana_cost = attempt.payload.mana_cost();
        if context.mana(attempt.owner_id) < mana_cost {
            return Ok(AbilityActivationDecision::Block(
                CombatBlockReason::NotEnoughMana,
            ));
        }

        if let Some(target) = attempt.payload.target()
            && !context.objects.exists(target)
        {
            return Ok(AbilityActivationDecision::Block(
                CombatBlockReason::InvalidTarget,
            ));
        }

        Ok(AbilityActivationDecision::Allow)
    }
}

struct CombatCommit;

impl AbilityCommitAction<CombatState, CombatTags, AbilityPayload> for CombatCommit {
    type Error = CombatError;

    fn apply_commit(
        &mut self,
        context: &mut CombatState,
        active: ActiveAbilityView<'_, CombatTags, AbilityPayload>,
    ) -> Result<(), Self::Error> {
        active
            .payload
            .commit(context, active.source_id(), active.owner_id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CombatBlockReason {
    OnCooldown,
    NotEnoughMana,
    InvalidTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CombatError {
    Runtime,
}

struct EffectActionContext {
    current_health: Rc<RefCell<Attribute>>,
    attribute_events: Rc<RefCell<Vec<AttributeChange>>>,
}

fn apply_effect_payload(
    context: &mut EffectActionContext,
    execution: EffectExecutionView<'_, CombatTags, EffectPayload>,
) -> Result<(), CombatError> {
    let damage = match *execution.payload {
        EffectPayload::Damage { amount } => amount,
        EffectPayload::Bleed { damage_per_tick } => damage_per_tick,
        EffectPayload::AttackSpeedBonus { .. }
        | EffectPayload::MaxHealthBonus { .. }
        | EffectPayload::Cooldown => return Ok(()),
    };
    let current = context
        .current_health
        .borrow()
        .get(execution.target_id)
        .ok_or(CombatError::Runtime)?;
    let attribute_events = Rc::clone(&context.attribute_events);
    AttributeSet::new(execution.target_id, (current - damage).max(0.0)).run_streaming(
        &mut context.current_health.borrow_mut(),
        |change| {
            attribute_events.borrow_mut().push(change);
        },
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CombatProfile {
    name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CombatTag {
    Ability,
    Slash,
    QuickenedStrikes,
    Fortify,
    Effect,
    Damage,
    Buff,
    Cooldown,
    AttackSpeed,
    MaxHealth,
    Bleed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum EffectPayload {
    Damage { amount: AttributeValue },
    AttackSpeedBonus { amount: AttributeValue },
    MaxHealthBonus { amount: AttributeValue },
    Bleed { damage_per_tick: AttributeValue },
    Cooldown,
}

fn tag_set<const N: usize>(tags: [Tag<CombatTag>; N]) -> CombatTags {
    TagSet::new(tags)
}

fn effect_damage_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Damage])
}

fn effect_buff_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Buff])
}

fn effect_cooldown_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Cooldown])
}

fn effect_attack_speed_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Buff, CombatTag::AttackSpeed])
}

fn effect_max_health_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Buff, CombatTag::MaxHealth])
}

fn effect_bleed_tag() -> Tag<CombatTag> {
    Tag::new([CombatTag::Effect, CombatTag::Damage, CombatTag::Bleed])
}

fn signal_definition(
    key: &str,
    signal_kind: SignalKind,
    lifecycle_event_kinds: Vec<LifecycleEventKind>,
    tag_match: SignalTagMatch<CombatTag>,
    signal_payload: &'static str,
) -> SignalDefinition<CombatTag, &'static str> {
    SignalDefinition {
        key: key.to_owned(),
        signal_kind,
        lifecycle_event_kinds,
        tag_match,
        payload_schema: "combat/signal.v1".to_owned(),
        signal_payload,
        channel_key: "combat/signals".to_owned(),
        category: "combat".to_owned(),
        retention: SignalRetentionPolicy::Retain,
        export: SignalExportPolicy::Export,
        debug_label: key.to_owned(),
        description: format!("{key} combat signal"),
    }
}
