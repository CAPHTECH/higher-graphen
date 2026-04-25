# HigherGraphen

HigherGraphen is an AI-native higher-structure framework for representing,
transforming, checking, completing, and projecting complex target worlds as
structured spaces.

It generalizes ordinary graphs into spaces of cells, complexes, contexts,
morphisms, invariants, obstructions, completions, projections, and
interpretation packages.

## Status

This repository is in the concept and specification phase. The current official
documentation is derived from the original proposal in
[`docs/highergraphen_proposal.md`](docs/highergraphen_proposal.md).

## Documentation

- [`docs/index.md`](docs/index.md) - Documentation index and reading order
- [`docs/overview.md`](docs/overview.md) - Product overview and positioning
- [`docs/concepts/core-concepts.md`](docs/concepts/core-concepts.md) - Core vocabulary
- [`docs/concepts/higher-structure-model.md`](docs/concepts/higher-structure-model.md) - Structural model
- [`docs/concepts/theoretical-foundations.md`](docs/concepts/theoretical-foundations.md) - Theoretical foundations used as engineering primitives
- [`docs/specs/package-boundaries.md`](docs/specs/package-boundaries.md) - Package and repository boundaries
- [`docs/specs/intermediate-tools-map.md`](docs/specs/intermediate-tools-map.md) - Core packages and intermediate `*graphen` tools
- [`docs/specs/rust-core-model.md`](docs/specs/rust-core-model.md) - Rust core data model specification
- [`docs/specs/engine-traits.md`](docs/specs/engine-traits.md) - Engine interface specification
- [`docs/product-packages/architecture-product.md`](docs/product-packages/architecture-product.md) - Reference Architecture Product
- [`docs/product-packages/domain-products.md`](docs/product-packages/domain-products.md) - Additional domain products
- [`docs/mvp-roadmap.md`](docs/mvp-roadmap.md) - MVP scope, roadmap, and success criteria
- [`docs/source-trace.md`](docs/source-trace.md) - Trace from proposal sections to official documents
- [`docs/adr/0001-rust-first-polyglot-friendly.md`](docs/adr/0001-rust-first-polyglot-friendly.md) - Architecture decision record

## Design Principle

HigherGraphen treats each concrete product as an interpretation package over a
shared higher-structure core:

```text
Product = Interpretation Package over Higher Structure
```

The goal is to avoid rebuilding a reasoning foundation for every product. The
core supplies structural primitives and engines. Domain packages supply
vocabulary, mappings, invariants, completion rules, and projections.
