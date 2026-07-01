# Product Boundaries

Flexweave provides a Rust mechanics primitive crate.

The Rust crate stays domain-agnostic. It provides object identity, attached
data, attributes, derived attributes, tags, queries, abilities, effects,
signals, registries, caller-defined clock units, and primitive errors. Caller
code gives those primitives application meaning.

Core registries validate in-memory ability and effect definitions and provide
key-aware runtime helpers; they do not own catalog files, generated output,
runtime bindings, or deployment.

Consumer projects own the concrete integration. They provide authored content,
runtime bindings, application behavior, and deployment.
