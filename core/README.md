# Flexweave Core

Flexweave Core is the Rust crate for deterministic mechanics primitives. It is
designed to be used on its own, without the Studio package or Studio app.

The phase-one crate contains a compileable placeholder so the workspace can
verify Rust tooling before the full Core source is moved into this path.

## Commands

```bash
cargo build -p flexweave
cargo test -p flexweave
cargo clippy -p flexweave --all-targets -- -D warnings
cargo doc -p flexweave --no-deps
```

## Boundary

Core owns object identity, attached data, attributes, derived attributes, tags,
queries, abilities, effects, registries, signals, caller-defined clock units,
and primitive errors.

Core does not own consumer runtime source, catalog file formats, generated
output paths, authoring UI behavior, or runtime hook implementations.
