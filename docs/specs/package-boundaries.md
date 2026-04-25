# Package Boundaries

HigherGraphen is intended to be a single repository with multiple packages.
The repository should be organized as a Rust workspace with optional bindings,
apps, examples, and documentation.

## Proposed Repository Layout

```text
higher-graphen/
  README.md
  Cargo.toml

  crates/
    higher-graphen-core/
    higher-graphen-space/
    higher-graphen-morphism/
    higher-graphen-context/
    higher-graphen-obstruction/
    higher-graphen-completion/
    higher-graphen-invariant/
    higher-graphen-evidence/
    higher-graphen-projection/
    higher-graphen-correspondence/
    higher-graphen-evolution/
    higher-graphen-interpretation/
    higher-graphen-runtime/

  bindings/
    python/
    wasm/
    node/

  apps/
    studio/
    playground/
    docs-site/

  examples/
    architecture/
    contract/
    project/
    evidence/

  docs/
    concepts/
    specs/
    product-packages/
```

## Crate Responsibilities

| Crate | Responsibility |
| --- | --- |
| `higher-graphen-core` | Shared IDs, labels, source references, provenance, confidence, severity, review status, errors, and serialization primitives. |
| `higher-graphen-space` | Space, cell, incidence, complex, boundary, and storage abstractions. |
| `higher-graphen-morphism` | Structure mappings, composition, preservation checks, lost structure, and distortion. |
| `higher-graphen-context` | Contexts, sections, restrictions, covers, and gluing checks. |
| `higher-graphen-invariant` | Invariants, constraints, invariant checks, and constraint check results. |
| `higher-graphen-obstruction` | Obstructions, counterexamples, obstruction engines, and explanations. |
| `higher-graphen-completion` | Completion candidates, completion rules, completion engine, accept and reject workflow. |
| `higher-graphen-evidence` | Claims, evidence, support relations, contradiction relations, and evidence graphs. |
| `higher-graphen-projection` | Projection definitions, selectors, projection results, and renderers. |
| `higher-graphen-correspondence` | Structural correspondence and analogy support. |
| `higher-graphen-evolution` | Time evolution, versions, migrations, and change tracking. |
| `higher-graphen-interpretation` | Domain interpretation packages, type mappings, invariant templates, projection templates, and lift adapters. |
| `higher-graphen-runtime` | Runtime APIs for AI agents and humans to query, transform, project, and review structures. |

## Naming Rules

The public product name is `HigherGraphen`.

Repository and package names should use `higher-graphen` as the stable prefix.
The shorter name `Graphen` should not be used as the product name because it can
be confused with graphene, the carbon material.

Recommended external naming:

```text
HigherGraphen Core
HigherGraphen Space
HigherGraphen Morphism
HigherGraphen Context
HigherGraphen Obstruction
HigherGraphen Completion
HigherGraphen Projection
HigherGraphen Interpretation
```

## Dependency Direction

The intended dependency direction is:

```text
core
  -> space
  -> context
  -> morphism
  -> invariant
  -> obstruction
  -> completion
  -> projection
  -> interpretation
  -> runtime
```

This is a conceptual direction, not a strict linear dependency chain. The core
rule is that low-level model crates must not depend on product packages,
bindings, apps, or UI code.

## MVP Package Set

The MVP should implement:

- `higher-graphen-core`
- `higher-graphen-space`
- `higher-graphen-morphism`
- `higher-graphen-invariant`
- `higher-graphen-obstruction`
- `higher-graphen-completion`
- `higher-graphen-projection`
- `higher-graphen-interpretation`

Context, evidence, correspondence, evolution, and runtime can be introduced
incrementally if they are not required for the first Architecture Product
scenario.
