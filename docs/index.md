# HigherGraphen Documentation

This directory contains the official English documentation for HigherGraphen.
The original proposal remains available as source material:

- [`highergraphen_proposal.md`](highergraphen_proposal.md)

## Recommended Reading Order

1. [`overview.md`](overview.md)
2. [`concepts/ai-operator-paradigm.md`](concepts/ai-operator-paradigm.md)
3. [`concepts/core-concepts.md`](concepts/core-concepts.md)
4. [`concepts/higher-structure-model.md`](concepts/higher-structure-model.md)
5. [`concepts/theoretical-foundations.md`](concepts/theoretical-foundations.md)
6. [`specs/package-boundaries.md`](specs/package-boundaries.md)
7. [`specs/intermediate-tools-map.md`](specs/intermediate-tools-map.md)
8. [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md)
9. [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md)
10. [`specs/core-contracts.md`](specs/core-contracts.md)
11. [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md)
12. [`specs/runtime-cli-scope.md`](specs/runtime-cli-scope.md)
13. [`specs/runtime-workflow-contract.md`](specs/runtime-workflow-contract.md)
14. [`specs/agent-tooling-handoff.md`](specs/agent-tooling-handoff.md)
15. [`specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`](specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md)
16. [`specs/intermediate-tools/casegraphen-current-surface-inventory.md`](specs/intermediate-tools/casegraphen-current-surface-inventory.md)
17. [`specs/intermediate-tools/casegraphen-workflow-contracts.md`](specs/intermediate-tools/casegraphen-workflow-contracts.md)
18. [`cli/highergraphen.md`](cli/highergraphen.md)
19. [`../skills/highergraphen/SKILL.md`](../skills/highergraphen/SKILL.md)
20. [`specs/rust-core-model.md`](specs/rust-core-model.md)
21. [`specs/engine-traits.md`](specs/engine-traits.md)
22. [`product-packages/architecture-product.md`](product-packages/architecture-product.md)
23. [`product-packages/feed-product.md`](product-packages/feed-product.md)
24. [`mvp-roadmap.md`](mvp-roadmap.md)

## Document Set

| Document | Purpose |
| --- | --- |
| [`overview.md`](overview.md) | Defines the product, the problem space, and the intended positioning. |
| [`concepts/ai-operator-paradigm.md`](concepts/ai-operator-paradigm.md) | Explains the shift from human-operated products to AI-operated products that expose higher-order structure directly. |
| [`concepts/core-concepts.md`](concepts/core-concepts.md) | Establishes the official vocabulary used by all later documents. |
| [`concepts/higher-structure-model.md`](concepts/higher-structure-model.md) | Describes how cells, complexes, contexts, morphisms, invariants, obstructions, completions, and projections fit together. |
| [`concepts/theoretical-foundations.md`](concepts/theoretical-foundations.md) | Records the mathematical and computer science concepts used as engineering primitives. |
| [`specs/package-boundaries.md`](specs/package-boundaries.md) | Defines crate boundaries, repository layout, and dependency direction. |
| [`specs/intermediate-tools-map.md`](specs/intermediate-tools-map.md) | Maps core packages to intermediate `*graphen` tools and their theoretical foundations. |
| [`specs/intermediate-tools/casegraphen.md`](specs/intermediate-tools/casegraphen.md) | Defines the baseline `casegraphen` intermediate tool contract for structured cases, coverage, missing cases, conflicts, projections, and comparison. |
| [`specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`](specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md) | Defines the next-stage `casegraphen` workflow reasoning engine contract inside the `higher-graphen` workspace. |
| [`specs/intermediate-tools/casegraphen-current-surface-inventory.md`](specs/intermediate-tools/casegraphen-current-surface-inventory.md) | Inventories the current `casegraphen` CLI, schema, test, example, and skill surface against the workflow reasoning engine target. |
| [`specs/intermediate-tools/casegraphen-workflow-contracts.md`](specs/intermediate-tools/casegraphen-workflow-contracts.md) | Defines implementable workflow model and report contracts for readiness, obstructions, completions, evidence, transitions, projections, correspondence, and evolution. |
| [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md) | Defines how AI agents should use HigherGraphen through skills, plugins, MCP, schemas, and marketplace bundles. |
| [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md) | Defines formatting, linting, complexity, dependency, and package verification gates for implementation tasks. |
| [`specs/core-contracts.md`](specs/core-contracts.md) | Defines the implementation contract for the shared `higher-graphen-core` primitives. |
| [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md) | Defines package-level implementation plans for non-core MVP crates. |
| [`specs/runtime-cli-scope.md`](specs/runtime-cli-scope.md) | Locks the immediate `higher-graphen-runtime` and `highergraphen` CLI scope, first command, and JSON report contract. |
| [`specs/runtime-workflow-contract.md`](specs/runtime-workflow-contract.md) | Defines the reusable runtime workflow contract for the Architecture Product direct database access smoke report. |
| [`specs/agent-tooling-handoff.md`](specs/agent-tooling-handoff.md) | Defines the handoff contract for provider-specific agent tooling that consumes the first runtime CLI report. |
| [`cli/highergraphen.md`](cli/highergraphen.md) | Provides the user-facing CLI reference for the first `highergraphen` command. |
| [`../skills/highergraphen/SKILL.md`](../skills/highergraphen/SKILL.md) | Provides the repository-owned CLI skill for agents using the first HigherGraphen report contract. |
| [`specs/rust-core-model.md`](specs/rust-core-model.md) | Specifies the core Rust data model at a stable contract level. |
| [`specs/engine-traits.md`](specs/engine-traits.md) | Specifies the engine interfaces that operate on the model. |
| [`product-packages/architecture-product.md`](product-packages/architecture-product.md) | Defines the first reference product and MVP scenario. |
| [`product-packages/feed-product.md`](product-packages/feed-product.md) | Defines the bounded Feed/RSS Reader example contract. |
| [`product-packages/domain-products.md`](product-packages/domain-products.md) | Captures additional product-package directions. |
| [`mvp-roadmap.md`](mvp-roadmap.md) | Defines MVP scope, phases, success criteria, and recommended stack. |
| [`source-trace.md`](source-trace.md) | Maps proposal sections to official documents. |
| [`adr/0001-rust-first-polyglot-friendly.md`](adr/0001-rust-first-polyglot-friendly.md) | Records the Rust-first, polyglot-friendly technical decision. |

## Documentation Status

These documents are formalized drafts. They are suitable for implementation
planning, issue creation, and design review, but they should be updated as soon
as the first Rust workspace and reference product are created.
