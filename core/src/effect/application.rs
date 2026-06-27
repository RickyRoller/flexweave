use crate::ability::ActiveAbility;
use crate::identity::ObjectId;
use crate::tag::TagCollection;

/// One effect application attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplication<Tags, Payload>
where
    Tags: TagCollection,
{
    pub definition_key: Option<String>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
}

/// Borrowed view of one effect application attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectApplicationView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub definition_key: Option<&'event str>,
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: &'event Tags,
    pub payload: &'event Payload,
}

impl<'event, Tags, Payload> EffectApplicationView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_application(&self) -> EffectApplication<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectApplication {
            definition_key: self.definition_key.map(str::to_owned),
            source_id: self.source_id,
            target_id: self.target_id,
            tags: self.tags.clone(),
            payload: self.payload.clone(),
        }
    }
}

/// Runtime application policy selected by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectApplicationDecision {
    Accept,
    Reject { reason: String },
}

/// Policy for applications whose `source_id` is absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectSourcePolicy {
    /// Allow `source_id: None` for environmental or system effects.
    AllowSystemSource,
    /// Require `source_id: Some(_)` and validate that source against the object store.
    RequireLiveSource,
}

/// Application input for the effect pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplicationInput<Tags, Payload>
where
    Tags: TagCollection,
{
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
    pub decision: EffectApplicationDecision,
}

impl<Tags, Payload> EffectApplicationInput<Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn accept(
        source_id: impl Into<Option<ObjectId>>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id,
            tags,
            payload,
            decision: EffectApplicationDecision::Accept,
        }
    }

    #[must_use]
    pub fn reject(
        source_id: impl Into<Option<ObjectId>>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id,
            tags,
            payload,
            decision: EffectApplicationDecision::Reject {
                reason: reason.into(),
            },
        }
    }

    #[must_use]
    pub fn accept_from_active_ability<AbilityTags, Cost, AbilityPayload>(
        active: &ActiveAbility<AbilityTags, Cost, AbilityPayload>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
    ) -> Self
    where
        AbilityTags: TagCollection,
    {
        Self::accept(active.source_id(), target_id, tags, payload)
    }

    #[must_use]
    pub fn reject_from_active_ability<AbilityTags, Cost, AbilityPayload>(
        active: &ActiveAbility<AbilityTags, Cost, AbilityPayload>,
        target_id: ObjectId,
        tags: Tags,
        payload: Payload,
        reason: impl Into<String>,
    ) -> Self
    where
        AbilityTags: TagCollection,
    {
        Self::reject(active.source_id(), target_id, tags, payload, reason)
    }
}

/// Rejected effect application fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplicationRejection<Tags, Payload>
where
    Tags: TagCollection,
{
    pub application: EffectApplication<Tags, Payload>,
    pub reason: String,
}

/// Borrowed rejected effect application fact for streaming emission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectApplicationRejectionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    pub application: EffectApplicationView<'event, Tags, Payload>,
    pub reason: &'event str,
}

impl<'event, Tags, Payload> EffectApplicationRejectionView<'event, Tags, Payload>
where
    Tags: TagCollection,
{
    #[must_use]
    pub fn to_owned_rejection(&self) -> EffectApplicationRejection<Tags, Payload>
    where
        Payload: Clone,
    {
        EffectApplicationRejection {
            application: self.application.to_owned_application(),
            reason: self.reason.to_owned(),
        }
    }
}
