# Run Studio Workflows

Validate a catalog:

```bash
flexweave-studio validate --config studio.config.ts
```

Inspect records:

```bash
flexweave-studio describe abilities --config studio.config.ts
flexweave-studio list abilities --config studio.config.ts
flexweave-studio show abilities minimal_ability --config studio.config.ts
```

Plan and scaffold a mechanic:

```bash
flexweave-studio plan --archetype mechanic --id minimal_mechanic --name "Minimal mechanic" --config studio.config.ts
flexweave-studio scaffold --archetype mechanic --id minimal_mechanic --name "Minimal mechanic" --config studio.config.ts
```

Scaffold a local host app:

```bash
flexweave-studio scaffold host-app --config studio.config.ts
```

Refresh and check generated mechanics definitions:

```bash
flexweave-studio codegen --config studio.config.ts
flexweave-studio codegen --check --config studio.config.ts
```

Run verification:

```bash
flexweave-studio verify --config studio.config.ts
```

Run migrations after package updates:

```bash
flexweave-studio migrate --config studio.config.ts
```

`migrate` reads host app scaffold metadata when present. It updates supported
scaffold metadata and reports any host app files that need manual follow-up.
