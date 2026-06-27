# Use Flexweave

Use Flexweave when a runtime needs deterministic mechanics primitives without Studio
authoring workflows.

## Add the Crate

This phase reserves the crate path and package name:

```toml
[dependencies]
flexweave = "0.0.0"
```

## Verify Flexweave Locally

Run Flexweave commands from the Flexweave repository root:

```bash
cargo build -p flexweave
cargo test -p flexweave
```

## Choose Lifecycle Event Shape

Use borrowed lifecycle event methods for hot streaming paths where listeners
handle the fact immediately. Use owned lifecycle event methods when the caller
needs to retain, replay, inspect, or route materialized facts. Drop-only event
channels can publish borrowed events; retained channels require owned events
because they store the emitted batch.

## Keep the Boundary

Flexweave should remain independent of catalog files, generated output directories,
runtime hooks, Studio UI behavior, and consumer project source.
