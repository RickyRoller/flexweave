# Use Flexweave Studio

Use Studio when a consumer project needs package-driven authoring workflows,
validation, migrations, generated output checks, and a local Studio host app.

## Install Packages

A consumer project depends on the versioned Studio package and app package:

```json
{
  "dependencies": {
    "@flexweave/studio": "0.0.0",
    "@flexweave/studio-app": "0.0.0"
  }
}
```

## Provide Project-Owned Files

The consumer project owns:

- `studio.config.ts`
- Studio catalog content
- Generated output directories
- Runtime hook implementations
- Local Studio host app entry point
- Project adapter
- Branding and deployment settings

## Run the Local Host App

The local host app imports `@flexweave/studio-app` for reusable UI behavior and
imports `@flexweave/studio` for shared workflows. The host app passes its
project adapter and config to the shared app shell.
