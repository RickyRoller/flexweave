# Update Studio Packages

Use this flow when a consumer project receives a newer Flexweave Studio package
set.

## Steps

1. Update the Studio packages.

   ```bash
   bun update @flexweave/studio @flexweave/studio-app
   ```

2. Run migrations supplied by the installed package version.

   ```bash
   bun run flexweave-studio migrate
   ```

   `migrate` reads local host app scaffold metadata when present and lists
   changed files plus manual follow-ups.

3. Verify catalog contracts, generated output freshness, and runtime hook
   wiring.

   ```bash
   bun run flexweave-studio verify
   ```

   When `studio.config.ts` declares `app.root`, `verify` also checks the local
   host app scaffold and runs its configured check or build command.

## Expected Ownership

The package update changes versioned Flexweave code. The consumer project keeps
ownership of Studio project config, catalog content, generated output
directories, runtime hooks, local host app entry point, adapter, branding, and
deployment.
