use crate::identity::ObjectId;
use crate::tag::TagCollection;

/// One effect application attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectApplication<Tags, Payload>
where
    Tags: TagCollection,
{
    pub source_id: Option<ObjectId>,
    pub target_id: ObjectId,
    pub tags: Tags,
    pub payload: Payload,
}

/// Runtime application policy selected by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectApplicationDecision {
    Accept,
    Reject { reason: String },
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
