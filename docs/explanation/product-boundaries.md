# Product Boundaries

Flexweave provides a Rust mechanics primitive crate and optional Studio
authoring surfaces.

The Rust crate stays domain-agnostic. It provides object identity, attached
data, attributes, derived attributes, tags, queries, abilities, effects,
signals, registries, caller-defined clock units, and primitive errors. Caller
code gives those primitives application meaning.

Core registries validate in-memory ability and effect definitions and provide
key-aware runtime helpers; they do not own catalog files, generated output, or
runtime bindings.

Studio is the authoring and build-time layer. It owns Studio catalog contracts,
Studio project config loading, validation, migrations, generated output checks,
workflow APIs, runtime hook contract docs, and reusable app shell behavior.

Consumer projects own the concrete integration. They provide catalog content,
runtime bindings, generated output directories, runtime hooks, local host app
entry point, adapter, branding, and deployment.
