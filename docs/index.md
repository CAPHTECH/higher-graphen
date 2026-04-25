# HigherGraphen Documentation

This directory contains the official English documentation for HigherGraphen.
The original proposal remains available as source material:

- [`highergraphen_proposal.md`](highergraphen_proposal.md)

## Recommended Reading Order

1. [`overview.md`](overview.md)
2. [`concepts/core-concepts.md`](concepts/core-concepts.md)
3. [`concepts/higher-structure-model.md`](concepts/higher-structure-model.md)
4. [`concepts/theoretical-foundations.md`](concepts/theoretical-foundations.md)
5. [`specs/package-boundaries.md`](specs/package-boundaries.md)
6. [`specs/intermediate-tools-map.md`](specs/intermediate-tools-map.md)
7. [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md)
8. [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md)
9. [`specs/core-contracts.md`](specs/core-contracts.md)
10. [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md)
11. [`specs/rust-core-model.md`](specs/rust-core-model.md)
12. [`specs/engine-traits.md`](specs/engine-traits.md)
13. [`product-packages/architecture-product.md`](product-packages/architecture-product.md)
14. [`mvp-roadmap.md`](mvp-roadmap.md)

## Document Set

| Document | Purpose |
| --- | --- |
| [`overview.md`](overview.md) | Defines the product, the problem space, and the intended positioning. |
| [`concepts/core-concepts.md`](concepts/core-concepts.md) | Establishes the official vocabulary used by all later documents. |
| [`concepts/higher-structure-model.md`](concepts/higher-structure-model.md) | Describes how cells, complexes, contexts, morphisms, invariants, obstructions, completions, and projections fit together. |
| [`concepts/theoretical-foundations.md`](concepts/theoretical-foundations.md) | Records the mathematical and computer science concepts used as engineering primitives. |
| [`specs/package-boundaries.md`](specs/package-boundaries.md) | Defines crate boundaries, repository layout, and dependency direction. |
| [`specs/intermediate-tools-map.md`](specs/intermediate-tools-map.md) | Maps core packages to intermediate `*graphen` tools and their theoretical foundations. |
| [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md) | Defines how AI agents should use HigherGraphen through skills, plugins, MCP, schemas, and marketplace bundles. |
| [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md) | Defines formatting, linting, complexity, dependency, and package verification gates for implementation tasks. |
| [`specs/core-contracts.md`](specs/core-contracts.md) | Defines the implementation contract for the shared `higher-graphen-core` primitives. |
| [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md) | Defines package-level implementation plans for non-core MVP crates. |
| [`specs/rust-core-model.md`](specs/rust-core-model.md) | Specifies the core Rust data model at a stable contract level. |
| [`specs/engine-traits.md`](specs/engine-traits.md) | Specifies the engine interfaces that operate on the model. |
| [`product-packages/architecture-product.md`](product-packages/architecture-product.md) | Defines the first reference product and MVP scenario. |
| [`product-packages/domain-products.md`](product-packages/domain-products.md) | Captures additional product-package directions. |
| [`mvp-roadmap.md`](mvp-roadmap.md) | Defines MVP scope, phases, success criteria, and recommended stack. |
| [`source-trace.md`](source-trace.md) | Maps proposal sections to official documents. |
| [`adr/0001-rust-first-polyglot-friendly.md`](adr/0001-rust-first-polyglot-friendly.md) | Records the Rust-first, polyglot-friendly technical decision. |

## Documentation Status

These documents are formalized drafts. They are suitable for implementation
planning, issue creation, and design review, but they should be updated as soon
as the first Rust workspace and reference product are created.
