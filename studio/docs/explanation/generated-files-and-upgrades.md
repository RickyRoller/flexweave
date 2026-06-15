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

`migrate` gives package changes a stable command contract. `verify` runs
catalog validation, generated freshness checks, and configured consumer
verification commands.

When a consumer project has a local host app scaffold, `migrate` updates
supported scaffold metadata and `verify` checks the scaffold plus its
configured typecheck or build command. Files that differ from the current
scaffold template are reported as manual follow-ups instead of being
overwritten.
