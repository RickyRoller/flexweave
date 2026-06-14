# Flexweave Studio

Flexweave Studio is the optional authoring surface for Flexweave. It is split
into a reusable package and a reusable app shell:

- `@flexweave/studio` provides project config contracts, validation,
  migrations, generated output checks, and workflow APIs.
- `@flexweave/studio-app` provides the adapter-neutral application shell.

This phase contains placeholders so package workspace discovery, typechecking,
tests, and builds can run before full Studio source lands here.

## Package Commands

```bash
bun run --filter @flexweave/studio typecheck
bun run --filter @flexweave/studio test
bun run --filter @flexweave/studio build
```

## App Commands

```bash
bun run --filter @flexweave/studio-app typecheck
bun run --filter @flexweave/studio-app test
bun run --filter @flexweave/studio-app build
```

## Runtime Contract

Read [Runtime Contract](./docs/runtime-contract.md) for the boundary between
Studio packages and consumer-owned runtime hooks.
