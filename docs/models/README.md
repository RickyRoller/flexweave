# System Models

This directory holds source-first implementation models for Flexweave core
systems. Models are organized by durable core system so they can evolve beside
the implementation and documentation.

## Organization

```text
docs/models/
|-- README.md
`-- <core-system>/
    |-- README.md
    |-- lifecycle.d2
    |-- data-model.d2
    `-- interactions.d2
```

Use D2 source files as the primary artifacts and keep peer `.svg` renders next
to each diagram so reviewers can inspect the current model without running D2.
Flexweave models use D2's built-in Dark Mauve theme (`theme-id: 200`) instead
of custom color palettes.

Each core-system README should name the model scope, source paths, model files,
implementation notes, and open questions.

Render a model after editing it:

```bash
d2 docs/models/<core-system>/<diagram>.d2 docs/models/<core-system>/<diagram>.svg
```
