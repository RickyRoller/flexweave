# ADR 0001: Flexweave Core Library Scope

## Status

Accepted

## Context

Flexweave needs one stable reusable surface for deterministic mechanics
primitives. Consumer projects still need freedom to own domain content, runtime
binding choices, application behavior, and deployment.

## Decision

The repository uses one stable top-level implementation surface:

- `core` for the Rust crate named `flexweave`.

Consumer projects own authored content, runtime bindings, application-specific
semantics, and deployment.

## Consequences

- Rust crate verification can run with Rust commands only.
- The root workspace only orchestrates shared documentation and Rust crate
  verification.
- Documentation can describe Flexweave as a standalone library.
