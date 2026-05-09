# HigherGraphen Documentation

This directory contains the official English documentation for HigherGraphen.
The original proposal remains available as source material:

- [`highergraphen_proposal.md`](highergraphen_proposal.md)

## Recommended Reading Order

1. [`overview.md`](overview.md)
2. [`concepts/ai-operator-paradigm.md`](concepts/ai-operator-paradigm.md)
3. [`concepts/core-concepts.md`](concepts/core-concepts.md)
4. [`guides/product-integration-for-ai-agents.md`](guides/product-integration-for-ai-agents.md)
5. [`concepts/higher-structure-model.md`](concepts/higher-structure-model.md)
6. [`concepts/theoretical-foundations.md`](concepts/theoretical-foundations.md)
7. [`specs/package-boundaries.md`](specs/package-boundaries.md)
8. [`specs/intermediate-tools-map.md`](specs/intermediate-tools-map.md)
9. [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md)
10. [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md)
11. [`specs/core-contracts.md`](specs/core-contracts.md)
12. [`specs/graph-traversal-api.md`](specs/graph-traversal-api.md)
13. [`specs/math-extension-kernels.md`](specs/math-extension-kernels.md)
14. [`specs/math-kernel-api-examples.md`](specs/math-kernel-api-examples.md)
15. [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md)
16. [`specs/runtime-cli-scope.md`](specs/runtime-cli-scope.md)
17. [`specs/runtime-workflow-contract.md`](specs/runtime-workflow-contract.md)
18. [`specs/agent-tooling-handoff.md`](specs/agent-tooling-handoff.md)
19. [`specs/pr-review-target-report-contract.md`](specs/pr-review-target-report-contract.md)
20. [`specs/ddd-review-cli-contract.md`](specs/ddd-review-cli-contract.md)
21. [`specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`](specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md)
22. [`specs/intermediate-tools/casegraphen-current-surface-inventory.md`](specs/intermediate-tools/casegraphen-current-surface-inventory.md)
23. [`specs/intermediate-tools/casegraphen-workflow-contracts.md`](specs/intermediate-tools/casegraphen-workflow-contracts.md)
24. [`specs/intermediate-tools/casegraphen-feature-completion-contract.md`](specs/intermediate-tools/casegraphen-feature-completion-contract.md)
25. [`specs/intermediate-tools/casegraphen-native-case-management.md`](specs/intermediate-tools/casegraphen-native-case-management.md)
26. [`../examples/casegraphen/ddd/domain-model-design/README.md`](../examples/casegraphen/ddd/domain-model-design/README.md)
27. [`cli/highergraphen.md`](cli/highergraphen.md)
28. [`../skills/highergraphen/SKILL.md`](../skills/highergraphen/SKILL.md)
29. [`../skills/highergraphen-ddd/SKILL.md`](../skills/highergraphen-ddd/SKILL.md)
30. [`../skills/release-runner/SKILL.md`](../skills/release-runner/SKILL.md)
31. [`specs/rust-core-model.md`](specs/rust-core-model.md)
32. [`specs/engine-traits.md`](specs/engine-traits.md)
33. [`product-packages/architecture-product.md`](product-packages/architecture-product.md)
34. [`product-packages/feed-product.md`](product-packages/feed-product.md)
35. [`mvp-roadmap.md`](mvp-roadmap.md)

## Document Set

| Document | Purpose |
| --- | --- |
| [`overview.md`](overview.md) | Defines the product, the problem space, and the intended positioning. |
| [`concepts/ai-operator-paradigm.md`](concepts/ai-operator-paradigm.md) | Explains the shift from human-operated products to AI-operated products that expose higher-order structure directly. |
| [`concepts/core-concepts.md`](concepts/core-concepts.md) | Establishes the official vocabulary used by all later documents. |
| [`guides/product-integration-for-ai-agents.md`](guides/product-integration-for-ai-agents.md) | Provides the AI-agent procedure for lifting a bounded product snapshot into HigherGraphen spaces, cells, contexts, morphisms, invariants, obstructions, completions, evidence, and projections. |
| [`concepts/higher-structure-model.md`](concepts/higher-structure-model.md) | Describes how cells, complexes, contexts, morphisms, invariants, obstructions, completions, and projections fit together. |
| [`concepts/theoretical-foundations.md`](concepts/theoretical-foundations.md) | Records the mathematical and computer science concepts used as engineering primitives. |
| [`specs/package-boundaries.md`](specs/package-boundaries.md) | Defines crate boundaries, repository layout, and dependency direction. |
| [`specs/intermediate-tools-map.md`](specs/intermediate-tools-map.md) | Maps core packages to intermediate `*graphen` tools and their theoretical foundations. |
| [`specs/intermediate-tools/casegraphen.md`](specs/intermediate-tools/casegraphen.md) | Defines the redesigned `casegraphen` contract as a higher-order structure operation tool for lift, space replay, morphisms, obstructions, completions, projections, equivalence, and invariants. |
| [`specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`](specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md) | Defines the transitional `casegraphen` workflow reasoning engine contract inside the `higher-graphen` workspace. |
| [`specs/intermediate-tools/casegraphen-current-surface-inventory.md`](specs/intermediate-tools/casegraphen-current-surface-inventory.md) | Inventories the current `casegraphen` CLI, schema, test, example, and skill surface, including the destructive redesign mapping from legacy commands to higher-order operations. |
| [`specs/intermediate-tools/casegraphen-workflow-contracts.md`](specs/intermediate-tools/casegraphen-workflow-contracts.md) | Defines implementable workflow model and report contracts for readiness, obstructions, completions, evidence, transitions, projections, correspondence, and evolution, plus their migration into the higher-order command model. |
| [`specs/intermediate-tools/casegraphen-feature-completion-contract.md`](specs/intermediate-tools/casegraphen-feature-completion-contract.md) | Summarizes the completed higher-order CaseGraphen CLI and skill operator surface, bridge boundaries, review workflows, destructive command cleanup, and verification gates. |
| [`specs/intermediate-tools/casegraphen-native-case-management.md`](specs/intermediate-tools/casegraphen-native-case-management.md) | Defines the native CaseGraphen design around CaseSpace, CaseCell taxonomy, MorphismLog replay, derived readiness, review semantics, close invariants, store layout, higher-order CLI/API targets, and workflow migration. |
| [`../examples/casegraphen/native/README.md`](../examples/casegraphen/native/README.md) | Provides native CaseGraphen operator examples for the canonical higher-order `lift`, `space`, `obstruction`, `completion`, `projection`, `equivalence`, `invariant`, and `morphism` namespaces, including expected report pointers and residual limitations. |
| [`../examples/casegraphen/ddd/domain-model-design/README.md`](../examples/casegraphen/ddd/domain-model-design/README.md) | Provides the legacy Sales/Billing Customer DDD fixture used to motivate the HigherGraphen DDD review workflow. |
| [`specs/ai-agent-integration.md`](specs/ai-agent-integration.md) | Defines how AI agents should use HigherGraphen through skills, plugins, MCP, schemas, and marketplace bundles. |
| [`specs/static-analysis-policy.md`](specs/static-analysis-policy.md) | Defines formatting, linting, complexity, dependency, and package verification gates for implementation tasks. |
| [`specs/core-contracts.md`](specs/core-contracts.md) | Defines the implementation contract for the shared `higher-graphen-core` primitives. |
| [`specs/graph-traversal-api.md`](specs/graph-traversal-api.md) | Defines reusable reachability, path walking, and layer-pattern matching over `higher-graphen-structure::space`. |
| [`specs/math-extension-kernels.md`](specs/math-extension-kernels.md) | Designs additional mathematical kernels for uncertainty, optimization, information loss, order reasoning, abstract interpretation, graph analytics, model checking, and categorical construction at HigherGraphen's current abstraction level. |
| [`specs/math-kernel-api-examples.md`](specs/math-kernel-api-examples.md) | Provides short Rust examples for the implemented mathematical extension kernels. |
| [`specs/non-core-package-workplans.md`](specs/non-core-package-workplans.md) | Defines package-level implementation plans for non-core MVP crates. |
| [`specs/runtime-cli-scope.md`](specs/runtime-cli-scope.md) | Locks the immediate `higher-graphen-runtime` and `highergraphen` CLI scope, first command, and JSON report contract. |
| [`specs/runtime-workflow-contract.md`](specs/runtime-workflow-contract.md) | Defines the reusable runtime workflow contract for the Architecture Product direct database access smoke report. |
| [`specs/agent-tooling-handoff.md`](specs/agent-tooling-handoff.md) | Defines the handoff contract for provider-specific agent tooling that consumes the first runtime CLI report. |
| [`specs/pr-review-target-report-contract.md`](specs/pr-review-target-report-contract.md) | Defines the bounded PR review target input and report contract, including unreviewed AI target semantics and projection records. |
| [`specs/ddd-review-cli-contract.md`](specs/ddd-review-cli-contract.md) | Defines the bounded DDD review CLI contract, input and report schemas, evidence boundaries, projection loss, review gaps, and closeability semantics. |
| [`specs/test-gap-detector.md`](specs/test-gap-detector.md) | Defines the pre-implementation contract for a bounded missing unit test detector, including structural lift, invariants, obstructions, completion candidates, and projections. |
| [`cli/highergraphen.md`](cli/highergraphen.md) | Provides the user-facing CLI reference for `highergraphen` runtime workflows, including PR review target recommendation. |
| [`../skills/highergraphen/SKILL.md`](../skills/highergraphen/SKILL.md) | Provides the repository-owned CLI skill for agents using HigherGraphen report contracts, including PR review target reports. |
| [`../skills/highergraphen-ddd/SKILL.md`](../skills/highergraphen-ddd/SKILL.md) | Provides the repository-owned skill for agents using `highergraphen ddd` review contracts and reports. |
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
