# Flexweave

Flexweave covers reusable, domain-agnostic mechanics primitives.

## Language

**Object**:
A domain-neutral handle that can receive attached data, attributes, tags,
abilities, and active effects.

**Object id**:
A stable numeric handle assigned to an object in deterministic creation order.

**Object store**:
The primitive collection that creates object ids and preserves deterministic
iteration order.

**Attached data**:
A typed value associated with an object id to give that object caller-defined
meaning.

**Data store**:
A typed object-keyed collection of attached data.

**Attribute**:
A signed numeric value attached to an object and exposed as a mechanics
channel.

**Derived attribute**:
An attribute value calculated from caller-owned runtime state rather than
stored as source data.

**Attribute change**:
A reported transition from a previous attribute value to a current value.

**Tag**:
A caller-defined label attached to an object for deterministic grouping and
selection.

**Ability lifecycle**:
The domain-neutral sequence of ability grant, activation attempt, activation
decision, start, commit, cancellation, revocation, rollback, and completion.

**Ability cancellation**:
A lifecycle termination where caller-owned cancel behavior runs before active
ability state is removed.

**Ability revocation**:
A lifecycle cleanup where active ability state is removed because its owner is
revoked or destroyed.

**Ability rollback**:
A lifecycle cleanup where active ability state is removed after activation
startup or helper execution fails before normal completion.

**Clock unit**:
A caller-defined mechanics time unit used to advance effect lifetimes, periodic
effects, effect-backed cooldowns, and other lifecycle primitives.

**Effect lifecycle**:
The domain-neutral sequence of effect application, execution, active lifetime,
advancement, removal, and expiration facts.

**Active effect instance**:
Runtime effect state attached to an object for a finite or indefinite lifetime.

**Signal**:
A lifecycle fact exported through a caller-selected retention and projection
policy.

**Signal reinvocation**:
A signal-specific lifecycle fact used to project while-active signals from
active effect instances.

**Query**:
A deterministic selection over live object ids using caller-owned predicates and
required data checks.

**Primitive error**:
An explicit Flexweave failure condition such as invalid object id or missing
required data.

**Determinism**:
The guarantee that identical primitive inputs produce repeatable object ids,
iteration order, query order, lifecycle facts, and primitive results.

## Relationships

- Flexweave provides primitive mechanics building blocks while caller code assigns
  application meaning.
- Object ids identify Flexweave objects only.
- A data store attaches one typed value to an object id.
- Clock units are opaque to Flexweave.
- Query results preserve deterministic object iteration order.
- Abilities, effects, and signals define lifecycle shape while caller code owns
  product meaning at the boundary.
