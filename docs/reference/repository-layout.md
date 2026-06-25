# Repository Layout

```text
flexweave/
├── Cargo.toml
├── package.json
├── README.md
├── core/
├── docs/
├── scripts/
└── studio/
```

## Root

The root owns workspace membership, toolchain versions, shared scripts, and
repository documentation.

## Flexweave Rust Crate

`core` is the Rust workspace member for the package named `flexweave`.

## Studio Package

`studio` is the package workspace member for `@flexweave/studio`.

## Studio App

`studio/app` is the package workspace member for `@flexweave/studio-app`.
