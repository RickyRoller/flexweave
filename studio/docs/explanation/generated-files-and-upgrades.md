# Generated Files And Upgrades

Generated mechanics definitions are managed files. Studio renders them from
the configured catalog and writes them only to configured output directories.
They should be refreshed with:

```bash
flexweave-studio codegen --config studio.config.ts
```

CI should use the no-write freshness check:

```bash
flexweave-studio codegen --check --config studio.config.ts
```

Runtime hook stubs are different from generated mechanics definitions. Studio
creates missing stubs only once and never overwrites existing hook files.

Package updates should use an explicit upgrade flow:

```bash
flexweave-studio migrate --config studio.config.ts
flexweave-studio verify --config studio.config.ts
```

`migrate` gives package and extension-owned changes a stable command contract.
`verify` runs config, extension, source, mapper, validation, generated target,
runtime hook, host app, and configured project command checks.

When a consumer project has a local host app scaffold, `migrate` updates
supported scaffold metadata and `verify` checks the scaffold plus its
configured typecheck or build command. Files that differ from the current
scaffold template are reported as manual follow-ups instead of being
overwritten. Legacy project adapters that still contain copied scaffold wiring
are reported as manual follow-ups so projects can move customizations onto the
package-owned default adapter module. Unsupported future scaffold versions fail
with manual follow-ups instead of guessing through an ambiguous migration.

Extensions may register migrations for their own schema or source metadata.
Those migrations run after host app scaffold detection, report changed files
and skipped state, and remain responsible for preserving project-owned source
content unless their own writable adapter explicitly performs a migration.
