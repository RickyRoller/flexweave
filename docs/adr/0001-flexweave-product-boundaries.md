# ADR 0001: Flexweave Product Boundaries

## Status

Accepted

## Context

Flexweave contains a Rust mechanics primitive crate and optional Studio
authoring surfaces. These surfaces need independent ownership so Core can be
used without Studio, while Studio can provide package-driven authoring,
validation, migrations, generated output checks, and a reusable app shell.

## Decision

The repository uses three stable top-level surfaces:

- `core` for the Rust crate named `flexweave`.
- `studio` for the TypeScript package named `@flexweave/studio`.
- `studio/app` for the application package named `@flexweave/studio-app`.

Consumer projects own their Studio project config, catalog content, generated
output directories, runtime hook implementations, local host app entry point,
adapter, branding, and deployment.

## Consequences

- Core verification can run with Rust commands only.
- Studio verification can run without consumer project source.
- Package updates follow update, migrate, and verify commands.
- Documentation can describe Flexweave as a standalone product.
