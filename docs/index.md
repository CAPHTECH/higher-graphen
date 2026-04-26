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
11. [`specs/graph-traversal-api.md`](specs/graph-traversal-api.md)
12. [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md)
13. [`specs/runtime-cli-scope.md`](specs/runtime-cli-scope.md)
14. [`specs/runtime-workflow-contract.md`](specs/runtime-workflow-contract.md)
15. [`specs/agent-tooling-handoff.md`](specs/agent-tooling-handoff.md)
16. [`specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`](specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md)
17. [`specs/intermediate-tools/casegraphen-current-surface-inventory.md`](specs/intermediate-tools/casegraphen-current-surface-inventory.md)
18. [`specs/intermediate-tools/casegraphen-workflow-contracts.md`](specs/intermediate-tools/casegraphen-workflow-contracts.md)
19. [`specs/intermediate-tools/casegraphen-feature-completion-contract.md`](specs/intermediate-tools/casegraphen-feature-completion-contract.md)
20. [`specs/intermediate-tools/casegraphen-native-case-management.md`](specs/intermediate-tools/casegraphen-native-case-management.md)
21. [`../examples/casegraphen/ddd/domain-model-design/README.md`](../examples/casegraphen/ddd/domain-model-design/README.md)
22. [`../skills/casegraphen-ddd-diagnostics/SKILL.md`](../skills/casegraphen-ddd-diagnostics/SKILL.md)
23. [`cli/highergraphen.md`](cli/highergraphen.md)
24. [`../skills/highergraphen/SKILL.md`](../skills/highergraphen/SKILL.md)
25. [`../skills/release-runner/SKILL.md`](../skills/release-runner/SKILL.md)
26. [`specs/rust-core-model.md`](specs/rust-core-model.md)
27. [`specs/engine-traits.md`](specs/engine-traits.md)
28. [`product-packages/architecture-product.md`](product-packages/architecture-product.md)
29. [`product-packages/feed-product.md`](product-packages/feed-product.md)
30. [`mvp-roadmap.md`](mvp-roadmap.md)

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
| [`specs/intermediate-tools/casegraphen-feature-completion-contract.md`](specs/intermediate-tools/casegraphen-feature-completion-contract.md) | Summarizes the completed CaseGraphen CLI and skill operator surface, bridge boundaries, review workflows, and verification gates. |
| [`specs/intermediate-tools/casegraphen-native-case-management.md`](specs/intermediate-tools/casegraphen-native-case-management.md) | Defines the native CaseGraphen case management design around CaseSpace, CaseCell taxonomy, MorphismLog replay, derived readiness, review semantics, close invariants, store layout, CLI/API targets, and workflow migration. |
| [`../examples/casegraphen/native/README.md`](../examples/casegraphen/native/README.md) | Provides native CaseGraphen operator examples for `casegraphen case ...` and `casegraphen morphism ...`, including expected report pointers and residual limitations. |
| [`../examples/casegraphen/ddd/domain-model-design/README.md`](../examples/casegraphen/ddd/domain-model-design/README.md) | Provides a DDD domain model diagnostic example using native CaseGraphen reports for Sales/Billing Customer boundary review. |
| [`../skills/casegraphen-ddd-diagnostics/SKILL.md`](../skills/casegraphen-ddd-diagnostics/SKILL.md) | Provides the repository-owned skill for DDD and bounded context diagnostics over CaseGraphen data. |
| [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md) | Defines how AI agents should use HigherGraphen through skills, plugins, MCP, schemas, and marketplace bundles. |
| [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md) | Defines formatting, linting, complexity, dependency, and package verification gates for implementation tasks. |
| [`specs/core-contracts.md`](specs/core-contracts.md) | Defines the implementation contract for the shared `higher-graphen-core` primitives. |
| [`specs/graph-traversal-api.md`](specs/graph-traversal-api.md) | Defines reusable reachability, path walking, and layer-pattern matching over `higher-graphen-space`. |
| [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md) | Defines package-level implementation plans for non-core MVP crates. |
| [`specs/runtime-cli-scope.md`](specs/runtime-cli-scope.md) | Locks the immediate `higher-graphen-runtime` and `highergraphen` CLI scope, first command, and JSON report contract. |
| [`specs/runtime-workflow-contract.md`](specs/runtime-workflow-contract.md) | Defines the reusable runtime workflow contract for the Architecture Product direct database access smoke report. |
| [`specs/agent-tooling-handoff.md`](specs/agent-tooling-handoff.md) | Defines the handoff contract for provider-specific agent tooling that consumes the first runtime CLI report. |
| [`cli/highergraphen.md`](cli/highergraphen.md) | Provides the user-facing CLI reference for the first `highergraphen` command. |
| [`../skills/highergraphen/SKILL.md`](../skills/highergraphen/SKILL.md) | Provides the repository-owned CLI skill for agents using the first HigherGraphen report contract. |
| [`../skills/release-runner/SKILL.md`](../skills/release-runner/SKILL.md) | Provides the repository-owned release preparation, verification, packaging, and publication workflow. |
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
