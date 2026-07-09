# Repository Layout

```text
flexweave/
├── Cargo.toml
├── package.json
├── README.md
├── core/
├── docs/
└── project-docs/
```

## Root

The root owns workspace membership, toolchain versions, shared scripts, and
shared verification.

## Flexweave Rust Crate

`core` is the Rust workspace member for the package named `flexweave`.

## Documentation Site

`docs` is a TanStack Start app using Fumadocs, Fumadocs MDX, and Tailwind CSS.

## Project Documentation

`project-docs` contains source-first ADRs and implementation models.
