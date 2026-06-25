# Update Studio Packages

Use this flow when a consumer project receives a newer Flexweave Studio CLI or
host app package set.

## Steps

1. Update the Studio CLI.

   ```bash
   npm update --global @flexweave/studio
   ```

   If the repo has a local Studio host app, also update its local app package
   dependencies with that repo's package manager.

2. Run migrations supplied by the installed CLI version.

   ```bash
   flexweave-studio migrate --config studio.config.json
   ```

   `migrate` reads local host app scaffold metadata when present and lists
   changed files plus manual follow-ups.

3. Verify catalog contracts, generated output freshness, and runtime hook
   wiring.

   ```bash
   flexweave-studio verify --config studio.config.json
   ```

   When `studio.config.json` declares `app.root`, `verify` also checks the local
   host app scaffold and runs its configured check or build command.

## Expected Ownership

The package update changes versioned Flexweave code. The consumer project keeps
ownership of Studio project config, catalog content, generated output
directories, runtime hooks, local host app entry point, adapter, branding, and
deployment.
