# Run Studio Workflows

Validate a catalog:

```bash
flexweave-studio validate --config studio.config.json
```

Inspect records:

```bash
flexweave-studio describe abilities --config studio.config.json
flexweave-studio list abilities --config studio.config.json
flexweave-studio show abilities minimal_ability --config studio.config.json
```

Plan and scaffold a mechanic:

```bash
flexweave-studio plan --archetype mechanic --id minimal_mechanic --name "Minimal mechanic" --config studio.config.json
flexweave-studio scaffold --archetype mechanic --id minimal_mechanic --name "Minimal mechanic" --config studio.config.json
```

Scaffold a local host app:

```bash
flexweave-studio scaffold host-app --config studio.config.json
```

Refresh and check generated mechanics definitions:

```bash
flexweave-studio codegen --config studio.config.json
flexweave-studio codegen --check --config studio.config.json
```

Run verification:

```bash
flexweave-studio verify --config studio.config.json
```

Run migrations after package updates:

```bash
flexweave-studio migrate --config studio.config.json
```

`migrate` reads host app scaffold metadata when present. It updates supported
scaffold metadata and reports any host app files that need manual follow-up.
