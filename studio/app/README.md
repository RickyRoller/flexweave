# Flexweave Studio App

`@flexweave/studio-app` owns adapter-neutral shell contracts for local Studio
host apps. A consumer project provides a project adapter with labels,
navigation, authoring areas, workflow actions, generated output targets, and
server function bindings. Active Studio extensions may contribute additional
navigation, authoring editors, workflow actions, diagnostics panels,
generated-output panels, and source views.

The package exports:

- `collectStudioAppContributions`
- `composeStudioAppContributions`
- `defineStudioAppAdapter`
- `createStudioApp`
- `createStudioAppRoutes`
- `createStudioOverviewPanel`
- `validateStudioAppAdapter`

The package does not ship consumer content, branding, deployment config, or a
local host app entry point. Create those files with:

```bash
flexweave-studio scaffold host-app --config studio.config.ts
```
