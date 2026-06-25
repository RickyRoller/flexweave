# Use Flexweave Studio

Use Studio when a consumer project needs CLI-driven authoring workflows,
validation, migrations, generated output checks, or a local Studio host app.

## Install The CLI

Install the versioned `flexweave-studio` CLI directly. Do not add a JavaScript
package manifest to non-JavaScript game repos just to run Studio workflows.

```bash
npm install --global @flexweave/studio
```

## Provide Project-Owned Files

The consumer project owns:

- `studio.config.json`
- Studio catalog content
- Generated output directories
- Runtime hook implementations
- Local Studio host app entry point
- Project adapter
- Branding and deployment settings

Create the initial local host app scaffold:

```bash
flexweave-studio scaffold host-app --config studio.config.json
```

## Run the Local Host App

The local host app may add project-local JavaScript packages because it is a
JavaScript app. It imports `@flexweave/studio-app` for reusable UI behavior and
uses the project config plus adapter to compose the shared app shell.
