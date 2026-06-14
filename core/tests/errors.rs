use flexweave::{
    AbilityActivationError, AbilityDefinitionError, AbilityError, AttributeDefinitionError,
    CoreError, EffectDefinitionError, EventChannelDefinitionError, EventChannelError,
    LifecycleEventKind, SignalDefinitionError,
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
    assert_error::<AbilityError>();
    assert_error::<AbilityActivationError<HookError>>();
    assert_error::<AttributeDefinitionError>();
    assert_error::<EffectDefinitionError>();
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
        AttributeDefinitionError::ConflictingClampAndReject {
            key: "speed".to_owned(),
        }
        .to_string(),
        "attribute policy definition `speed` has conflicting clamp and reject domains"
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
}

#[test]
fn runtime_errors_have_contextual_display_messages_and_sources() {
    let ability = AbilityActivationError::<HookError>::Ability(AbilityError::AbilityOnCooldown);
    assert_eq!(
        ability.to_string(),
        "ability activation failed: ability is on cooldown"
    );
    assert_eq!(
        std::error::Error::source(&ability)
            .expect("ability error should be exposed as source")
            .to_string(),
        "ability is on cooldown"
    );

    let hook = AbilityActivationError::Hook(HookError);
    assert_eq!(
        hook.to_string(),
        "ability activation hook failed: hook denied activation"
    );
    assert_eq!(
        std::error::Error::source(&hook)
            .expect("hook error should be exposed as source")
            .to_string(),
        "hook denied activation"
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
}
