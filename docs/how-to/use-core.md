# Use Flexweave Core

Use Core when a runtime needs deterministic mechanics primitives without Studio
authoring workflows.

## Add the Crate

This phase reserves the crate path and package name:

```toml
[dependencies]
flexweave = "0.0.0"
```

## Verify Core Locally

Run Core commands from the Flexweave repository root:

```bash
cargo build -p flexweave
cargo test -p flexweave
```

## Keep the Boundary

Core should remain independent of catalog files, generated output directories,
runtime hooks, Studio UI behavior, and consumer project source.
