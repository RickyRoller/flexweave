# Flexweave Core

Flexweave Core covers reusable, domain-agnostic mechanics primitives.

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

**Clock unit**:
A caller-defined mechanics time unit used to advance cooldowns, effect
lifetimes, periodic effects, and other lifecycle primitives.

**Effect lifecycle**:
The domain-neutral sequence of effect application, execution, active lifetime,
advancement, removal, and expiration facts.

**Active effect instance**:
Runtime effect state attached to an object for a finite or indefinite lifetime.

**Query**:
A deterministic selection over live object ids using caller-owned predicates and
required data checks.

**Primitive error**:
An explicit Flexweave failure condition such as invalid object id or missing
required data.

## Relationships

- Core provides primitive mechanics building blocks while caller code assigns
  application meaning.
- Object ids identify Flexweave objects only.
- A data store attaches one typed value to an object id.
- Clock units are opaque to Core.
- Query results preserve deterministic object iteration order.
