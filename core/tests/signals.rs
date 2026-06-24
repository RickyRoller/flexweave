mod common;

use common::TestAtom;
use flexweave::{
    ActiveEffectId, EffectApplication, EffectExecution, EffectLifecycleEvent, EventChannel,
    EventChannelDefinition, EventRetention, LifecycleEventKind, ObjectId, SignalDefinition,
    SignalDefinitionError, SignalDefinitions, SignalExportPolicy, SignalKind, SignalProjection,
    SignalRemovalReason, SignalRetentionPolicy, SignalTagMatch, Tag, TagSet, TagSetQuery,
};

#[test]
fn signal_projection_matches_tags_and_preserves_definition_order() {
    let category_variant = Tag::new([TestAtom::Category, TestAtom::Variant]);
    let definitions = SignalDefinitions::new([
        signal_definition(
            "first",
            SignalKind::ActiveStart,
            vec![LifecycleEventKind::EffectActiveCreated],
            SignalTagMatch::Query(TagSetQuery {
                all: vec![category_variant.clone()],
                any: Vec::new(),
                none: Vec::new(),
            }),
        ),
        signal_definition(
            "second",
            SignalKind::ActiveStart,
            vec![LifecycleEventKind::EffectActiveCreated],
            SignalTagMatch::Any,
        ),
    ])
    .unwrap();
    let projection = SignalProjection::new(definitions);

    let facts = projection.project_effect_event(&EffectLifecycleEvent::ActiveCreated(
        effect_instance_with(
            ActiveEffectId::new(7),
            TagSet::new([category_variant]),
            SourcePayload::Buff,
        ),
    ));

    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].key, "first");
    assert_eq!(facts[1].key, "second");
    assert_eq!(facts[0].signal_kind, SignalKind::ActiveStart);
    assert_eq!(facts[0].target_id, ObjectId::new(20));
    assert_eq!(facts[0].source_payload, Some(SourcePayload::Buff));
}

#[test]
fn signal_definitions_validate_authoring_data() {
    assert_eq!(
        signal_definition(
            "",
            SignalKind::Executed,
            vec![LifecycleEventKind::EffectExecuted],
            SignalTagMatch::Any,
        )
        .validate()
        .unwrap_err(),
        SignalDefinitionError::EmptyKey
    );

    let missing_lifecycle = SignalDefinition {
        lifecycle_event_kinds: Vec::new(),
        ..signal_definition(
            "missing_lifecycle",
            SignalKind::Executed,
            vec![LifecycleEventKind::EffectExecuted],
            SignalTagMatch::Any,
        )
    };
    assert_eq!(
        missing_lifecycle.validate().unwrap_err(),
        SignalDefinitionError::MissingLifecycleEventKinds {
            key: "missing_lifecycle".to_owned(),
        }
    );

    assert_eq!(
        signal_definition(
            "invalid_query",
            SignalKind::Executed,
            vec![LifecycleEventKind::EffectExecuted],
            SignalTagMatch::Query(TagSetQuery {
                all: Vec::new(),
                any: Vec::new(),
                none: Vec::new(),
            }),
        )
        .validate()
        .unwrap_err(),
        SignalDefinitionError::InvalidTagQuery {
            key: "invalid_query".to_owned(),
        }
    );

    let duplicate = signal_definition(
        "duplicate",
        SignalKind::Executed,
        vec![LifecycleEventKind::EffectExecuted],
        SignalTagMatch::Any,
    );
    assert_eq!(
        SignalDefinitions::new([duplicate.clone(), duplicate]).unwrap_err(),
        SignalDefinitionError::DuplicateKey {
            key: "duplicate".to_owned(),
        }
    );

    let definitions = SignalDefinitions::new([signal_definition(
        "channel_check",
        SignalKind::Executed,
        vec![LifecycleEventKind::EffectExecuted],
        SignalTagMatch::Any,
    )])
    .unwrap();
    assert_eq!(
        definitions.validate_channels(&["other"]).unwrap_err(),
        SignalDefinitionError::UnknownChannelKey {
            key: "channel_check".to_owned(),
            channel_key: "signals/effects".to_owned(),
        }
    );
}

#[test]
fn signal_projection_maps_effect_lifecycle_kinds() {
    let definitions = SignalDefinitions::new([
        signal_definition(
            "active",
            SignalKind::ActiveStart,
            vec![LifecycleEventKind::EffectApplicationAccepted],
            SignalTagMatch::Any,
        ),
        signal_definition(
            "executed",
            SignalKind::Executed,
            vec![LifecycleEventKind::EffectExecuted],
            SignalTagMatch::Any,
        ),
        signal_definition(
            "recurring",
            SignalKind::Recurring,
            vec![LifecycleEventKind::EffectPeriodicExecuted],
            SignalTagMatch::Any,
        ),
        signal_definition(
            "removed",
            SignalKind::Removed,
            vec![
                LifecycleEventKind::EffectRemoved,
                LifecycleEventKind::EffectExpired,
            ],
            SignalTagMatch::Any,
        ),
    ])
    .unwrap();
    let projection = SignalProjection::new(definitions);
    let accepted = EffectLifecycleEvent::ApplicationAccepted(EffectApplication {
        source_id: Some(ObjectId::new(10)),
        target_id: ObjectId::new(20),
        tags: TagSet::new([Tag::new([TestAtom::Category])]),
        payload: SourcePayload::Buff,
    });
    let executed = effect_execution(SourcePayload::Hit);

    assert_eq!(projection.project_effect_event(&accepted)[0].key, "active");
    assert_eq!(
        projection.project_effect_event(&EffectLifecycleEvent::Executed(executed.clone()))[0].key,
        "executed"
    );
    assert_eq!(
        projection.project_effect_event(&EffectLifecycleEvent::PeriodicExecuted(executed))[0].key,
        "recurring"
    );

    let removed_effect = effect_instance(SourcePayload::Buff);
    let removed =
        projection.project_effect_event(&EffectLifecycleEvent::Removed(removed_effect.clone()));
    let expired = projection.project_effect_event(&EffectLifecycleEvent::Expired(removed_effect));
    assert_eq!(removed[0].key, "removed");
    assert_eq!(
        removed[0].removal_reason,
        Some(SignalRemovalReason::Removed)
    );
    assert_eq!(
        expired[0].removal_reason,
        Some(SignalRemovalReason::Expired)
    );
}

#[test]
fn reinvoking_active_signals_emits_while_active_without_execution_duplicates() {
    let definitions = SignalDefinitions::new([
        signal_definition(
            "loop",
            SignalKind::WhileActive,
            vec![LifecycleEventKind::EffectActiveCreated],
            SignalTagMatch::Any,
        ),
        signal_definition(
            "impact",
            SignalKind::Executed,
            vec![LifecycleEventKind::EffectExecuted],
            SignalTagMatch::Any,
        ),
    ])
    .unwrap();
    let projection = SignalProjection::new(definitions);
    let effect = effect_instance_with(
        ActiveEffectId::new(1),
        TagSet::new([Tag::new([TestAtom::Category])]),
        SourcePayload::Buff,
    );

    let facts = projection.reinvoke_effect_instances([&effect]);

    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].key, "loop");
    assert_eq!(facts[0].signal_kind, SignalKind::WhileActive);
    assert_eq!(
        facts[0].source_lifecycle_event_kind,
        LifecycleEventKind::SignalReinvoked
    );
}

#[test]
fn signal_facts_route_through_named_event_channels() {
    let definitions = SignalDefinitions::new([signal_definition(
        "impact",
        SignalKind::Executed,
        vec![LifecycleEventKind::EffectExecuted],
        SignalTagMatch::Any,
    )])
    .unwrap();
    let projection = SignalProjection::new(definitions);
    let facts = projection.project_effect_event(&EffectLifecycleEvent::Executed(effect_execution(
        SourcePayload::Hit,
    )));
    let channel_definition =
        EventChannelDefinition::new("signals/effects", [LifecycleEventKind::EffectExecuted])
            .unwrap();
    let mut channel = EventChannel::with_retention(channel_definition, EventRetention::Retain);

    for fact in facts {
        channel.publish(fact).unwrap();
    }

    let retained = channel.drain_retained();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].key, "impact");
    assert_eq!(retained[0].channel_key, "signals/effects");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourcePayload {
    Buff,
    Hit,
}

fn signal_definition(
    key: &str,
    signal_kind: SignalKind,
    lifecycle_event_kinds: Vec<LifecycleEventKind>,
    tag_match: SignalTagMatch<TestAtom>,
) -> SignalDefinition<TestAtom, &'static str> {
    SignalDefinition {
        key: key.to_owned(),
        signal_kind,
        lifecycle_event_kinds,
        tag_match,
        payload_schema: "test/payload".to_owned(),
        signal_payload: "signal-payload",
        channel_key: "signals/effects".to_owned(),
        category: "presentation".to_owned(),
        retention: SignalRetentionPolicy::Retain,
        export: SignalExportPolicy::Internal,
        debug_label: key.to_owned(),
        description: format!("{key} signal"),
    }
}

fn effect_instance_with(
    id: ActiveEffectId,
    tags: TagSet<TestAtom>,
    payload: SourcePayload,
) -> flexweave::EffectInstance<TagSet<TestAtom>, SourcePayload> {
    flexweave::EffectInstance {
        id,
        source_id: Some(ObjectId::new(10)),
        target_id: ObjectId::new(20),
        remaining_units: Some(100),
        period: None,
        period_elapsed_units: 0,
        tags,
        payload,
    }
}

fn effect_instance(
    payload: SourcePayload,
) -> flexweave::EffectInstance<TagSet<TestAtom>, SourcePayload> {
    effect_instance_with(
        ActiveEffectId::new(1),
        TagSet::new([Tag::new([TestAtom::Category])]),
        payload,
    )
}

fn effect_execution(payload: SourcePayload) -> EffectExecution<TagSet<TestAtom>, SourcePayload> {
    EffectExecution {
        active_effect_id: None,
        source_id: Some(ObjectId::new(10)),
        target_id: ObjectId::new(20),
        tags: TagSet::new([Tag::new([TestAtom::Category])]),
        payload,
        elapsed_units: None,
    }
}
