# Runtime Contract

Flexweave Studio is package-driven. Consumer projects provide the runtime side
of the integration through explicit config, generated output paths, and runtime
hooks.

## Studio Owns

- Studio project config schema and loading.
- Studio catalog contract validation.
- Migration workflows.
- Generated output freshness checks.
- Runtime hook contract documentation.
- Workflow APIs used by the Studio app.
- Adapter-neutral app shell behavior.

## Consumer Projects Own

- Catalog content.
- Generated output directories.
- Runtime hook implementations.
- Runtime binding modules.
- Local host app entry point.
- Project adapter, branding, and deployment.

## Update Flow

```bash
bun update @flexweave/studio @flexweave/studio-app
bun run flexweave-studio migrate
bun run flexweave-studio verify
```

The migration step updates files the package is allowed to manage. The verify
step checks catalog contracts, generated output freshness, and runtime hook
wiring declared by the consumer project.
