use crate::identity::ObjectId;

/// Caller-owned model of stored and derived attributes.
///
/// Flexweave keeps this generic so callers can expose domain methods such as
/// `apply_damage` without making effects couple directly to raw attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeModel<Attributes, Derived = ()> {
    attributes: Attributes,
    derived: Derived,
}

impl<Attributes> AttributeModel<Attributes, ()> {
    #[must_use]
    pub fn from_attributes(attributes: Attributes) -> Self {
        Self {
            attributes,
            derived: (),
        }
    }
}

impl<Attributes, Derived> AttributeModel<Attributes, Derived> {
    #[must_use]
    pub fn new(attributes: Attributes, derived: Derived) -> Self {
        Self {
            attributes,
            derived,
        }
    }

    #[must_use]
    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    #[must_use]
    pub fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }

    #[must_use]
    pub fn derived(&self) -> &Derived {
        &self.derived
    }

    #[must_use]
    pub fn derived_mut(&mut self) -> &mut Derived {
        &mut self.derived
    }

    #[must_use]
    pub fn into_parts(self) -> (Attributes, Derived) {
        (self.attributes, self.derived)
    }

    pub fn apply_operation<Context, Operation>(
        &mut self,
        object_id: ObjectId,
        context: &mut Context,
        mut operation: Operation,
    ) -> Result<Operation::Output, Operation::Error>
    where
        Operation: AttributeOperation<Context, Attributes, Derived>,
    {
        operation.apply(AttributeOperationContext {
            object_id,
            context,
            attributes: &mut self.attributes,
            derived: &mut self.derived,
        })
    }
}

/// Mutable operation context handed to caller-owned attribute operations.
pub struct AttributeOperationContext<'operation, Context, Attributes, Derived = ()> {
    pub object_id: ObjectId,
    pub context: &'operation mut Context,
    pub attributes: &'operation mut Attributes,
    pub derived: &'operation mut Derived,
}

/// Caller-owned operation over an [`AttributeModel`].
pub trait AttributeOperation<Context, Attributes, Derived = ()> {
    type Output;
    type Error;

    fn apply(
        &mut self,
        context: AttributeOperationContext<'_, Context, Attributes, Derived>,
    ) -> Result<Self::Output, Self::Error>;
}
