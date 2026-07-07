use flexweave::{
    AbilityBeginError, AbilityCommitError, AbilityDefinitionError, AbilityDefinitionRegistryError,
    AbilityEndError, AbilityError, AbilityGrantError, AbilityId, AbilityRollbackError, CoreError,
    EffectApplicationError, EffectApplyError, EffectDefinitionError, EffectDefinitionRegistryError,
    EventChannelDefinitionError, EventChannelError, LifecycleEventKind,
    RegisteredAbilityActivationError, SignalDefinitionError,
};
use std::fmt;

fn assert_error<Error>()
where
    Error: std::error::Error,
{
}

#[derive(Debug)]
struct HookError;

impl fmt::Display for HookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("hook denied activation")
    }
}

impl std::error::Error for HookError {}

#[test]
fn public_flexweave_errors_implement_std_error() {
    assert_error::<CoreError>();
    assert_error::<AbilityDefinitionError>();
    assert_error::<AbilityDefinitionRegistryError>();
    assert_error::<AbilityError>();
    assert_error::<AbilityGrantError>();
    assert_error::<AbilityBeginError<HookError>>();
    assert_error::<AbilityCommitError<HookError>>();
    assert_error::<AbilityEndError>();
    assert_error::<AbilityRollbackError>();
    assert_error::<RegisteredAbilityActivationError<HookError>>();
    assert_error::<EffectApplyError<HookError, HookError>>();
    assert_error::<EffectApplicationError>();
    assert_error::<EffectDefinitionError>();
    assert_error::<EffectDefinitionRegistryError>();
    assert_error::<EventChannelDefinitionError>();
    assert_error::<EventChannelError>();
    assert_error::<SignalDefinitionError>();
}

#[test]
fn core_errors_have_clear_display_messages() {
    assert_eq!(CoreError::InvalidObjectId.to_string(), "invalid object id");
    assert_eq!(
        CoreError::MissingRequiredData.to_string(),
        "missing required data"
    );
}

#[test]
fn definition_errors_include_relevant_keys_in_display_messages() {
    assert_eq!(
        AbilityDefinitionError::UnknownEmittedChannelKey {
            key: "dash".to_owned(),
            channel_key: "ability-events".to_owned(),
        }
        .to_string(),
        "ability definition `dash` references unknown emitted channel `ability-events`"
    );
    assert_eq!(
        EffectDefinitionError::DurationRequired {
            key: "burn".to_owned(),
        }
        .to_string(),
        "effect definition `burn` requires a duration"
    );
    assert_eq!(
        SignalDefinitionError::UnknownChannelKey {
            key: "burn-start".to_owned(),
            channel_key: "missing-channel".to_owned(),
        }
        .to_string(),
        "signal definition `burn-start` references unknown channel `missing-channel`"
    );
    assert_eq!(
        AbilityDefinitionRegistryError::DuplicateKey {
            key: "dash".to_owned(),
        }
        .to_string(),
        "ability definition `dash` is defined more than once"
    );
    assert_eq!(
        EffectDefinitionRegistryError::MissingDefinition {
            key: "burn".to_owned(),
        }
        .to_string(),
        "effect definition `burn` is not registered"
    );

    assert_eq!(
        EffectApplyError::<HookError, HookError>::RegisteredDefinition(
            EffectDefinitionRegistryError::MissingDefinition {
                key: "burn".to_owned(),
            }
        )
        .to_string(),
        "registered effect application failed: effect definition `burn` is not registered"
    );
}

#[test]
fn runtime_errors_have_contextual_display_messages_and_sources() {
    let ability = AbilityBeginError::<HookError>::Ability(AbilityError::MissingActivation);
    assert_eq!(
        ability.to_string(),
        "ability activation failed: missing ability activation"
    );
    assert_eq!(
        std::error::Error::source(&ability)
            .expect("ability error should be exposed as source")
            .to_string(),
        "missing ability activation"
    );

    let blocked = AbilityBeginError::<HookError, &str>::Blocked("cooldown");
    assert_eq!(
        blocked.to_string(),
        "ability activation blocked: \"cooldown\""
    );
    assert!(std::error::Error::source(&blocked).is_none());

    let gate = AbilityBeginError::<HookError>::Gate(HookError);
    assert_eq!(
        gate.to_string(),
        "ability activation gate failed: hook denied activation"
    );
    assert_eq!(
        std::error::Error::source(&gate)
            .expect("gate error should be exposed as source")
            .to_string(),
        "hook denied activation"
    );

    let commit = AbilityCommitError::Action(HookError);
    assert_eq!(
        commit.to_string(),
        "ability commit action failed: hook denied activation"
    );
    assert_eq!(
        std::error::Error::source(&commit)
            .expect("commit action error should be exposed as source")
            .to_string(),
        "hook denied activation"
    );

    let effect_action = EffectApplyError::<HookError, HookError>::Execution(HookError);
    assert_eq!(
        effect_action.to_string(),
        "effect execution failed: hook denied activation"
    );
    assert_eq!(
        std::error::Error::source(&effect_action)
            .expect("effect action error should be exposed as source")
            .to_string(),
        "hook denied activation"
    );

    let initialized_action = EffectApplyError::<HookError, HookError>::Initialize(HookError);
    assert_eq!(
        initialized_action.to_string(),
        "effect initialization failed: hook denied activation"
    );
    assert_eq!(
        std::error::Error::source(&initialized_action)
            .expect("initialized effect action error should be exposed as source")
            .to_string(),
        "hook denied activation"
    );

    assert_eq!(
        AbilityEndError::UncommittedActivation.to_string(),
        "uncommitted ability activation"
    );
    assert_eq!(
        AbilityRollbackError::AlreadyCommitted.to_string(),
        "ability activation is already committed"
    );

    assert_eq!(
        EventChannelDefinitionError::DuplicatePayloadKind {
            channel_name: "combat".to_owned(),
            kind: LifecycleEventKind::EffectExecuted,
        }
        .to_string(),
        "event channel `combat` has duplicate payload kind EffectExecuted"
    );
    assert_eq!(
        EventChannelError::PayloadMismatch {
            channel_name: "combat".to_owned(),
            kind: LifecycleEventKind::AttributeChanged,
        }
        .to_string(),
        "event channel `combat` does not accept payload kind AttributeChanged"
    );
    assert_eq!(
        RegisteredAbilityActivationError::<HookError>::MissingGrantedDefinitionKey {
            ability_id: AbilityId::new(7),
        }
        .to_string(),
        "ability `7` was not granted from a registered definition"
    );
}
