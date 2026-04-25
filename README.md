# HigherGraphen

HigherGraphen is an AI-native higher-structure framework for products and tools
whose primary operator may be an AI agent rather than a human using screens and
forms.

Most software products have been shaped around human cognitive limits: UI
screens, CRUD flows, dashboards, reports, tickets, and workflows. Those are
still useful human projections, but they are not the only useful product model
when AI agents can operate structured concepts directly.

HigherGraphen generalizes ordinary graphs into spaces of cells, complexes,
contexts, morphisms, invariants, obstructions, completions, projections, and
interpretation packages. These higher-order concepts become first-class
operational objects for AI agents.

## Why HigherGraphen Exists

AI agents can read documents, logs, code, tickets, API responses, and tabular
data. Reading is not enough when the target world contains:

- Problems that emerge only from three or more entities at once.
- Local structures that are individually valid but globally inconsistent.
- Meaning or constraints that are lost during transformation.
- Invariants that must remain true after change.
- Unverified or undefined regions of a state space.
- Concepts whose meaning changes by context.
- Structural analogies rather than surface-level similarity.
- Mixed observations, human claims, AI-generated inferences, and accepted
  conclusions.

HigherGraphen exists because AI-operated products can expose these structures
directly. A human-facing report becomes a projection from the structure, not the
whole system model.

## Operator Paradigm

The central product shift is:

```text
Human-operated product:
  Product model is constrained by what humans can manually inspect and operate.

AI-operated product:
  Product model can expose higher-order structure directly, then project it
  into human views when needed.
```

This is why HigherGraphen treats concepts such as `Invariant`, `Obstruction`,
`CompletionCandidate`, `Morphism`, `Context`, and `Projection` as product-level
objects rather than hidden implementation details.

## Status

This repository is in the concept and specification phase. The current official
documentation is derived from the original proposal in
[`docs/highergraphen_proposal.md`](docs/highergraphen_proposal.md).

The repository already contains the Rust workspace shape, core package
boundaries, initial runtime and CLI contracts, and reference product packages
for architecture analysis and feed/RSS analysis.

## Public Development Case Graph

This repository keeps a public CaseGraphen workspace under
[`.casegraphen/`](.casegraphen/README.md). It records goals, tasks, decisions,
evidence, blockers, completion candidates, and verification events for
HigherGraphen itself.

The workspace is intentional public material. It is meant to show how
HigherGraphen is decomposed and verified while keeping local runtime artifacts,
private cases, customer data, and commercial-only strategy out of the
repository.

## License And Commercial Boundary

HigherGraphen's public core is licensed under the
[Apache License 2.0](LICENSE).

The public repository is intended to contain the shared higher-structure core,
baseline intermediate tools, schemas, documentation, public examples, skills,
and public CaseGraphen development traces. Production interpretation packages,
hosted execution, customer-specific assets, private evaluation datasets,
commercial strategy, and private operations material belong outside this public
repository unless they are explicitly open-sourced later.

See [`COMMERCIAL_BOUNDARY.md`](COMMERCIAL_BOUNDARY.md) for the publication
boundary.

## Documentation

- [`docs/index.md`](docs/index.md) - Documentation index and reading order
- [`docs/overview.md`](docs/overview.md) - Product overview and positioning
- [`docs/concepts/ai-operator-paradigm.md`](docs/concepts/ai-operator-paradigm.md) - Why AI-operated products can use higher-order structure directly
- [`docs/concepts/core-concepts.md`](docs/concepts/core-concepts.md) - Core vocabulary
- [`docs/concepts/higher-structure-model.md`](docs/concepts/higher-structure-model.md) - Structural model
- [`docs/concepts/theoretical-foundations.md`](docs/concepts/theoretical-foundations.md) - Theoretical foundations used as engineering primitives
- [`docs/specs/package-boundaries.md`](docs/specs/package-boundaries.md) - Package and repository boundaries
- [`docs/specs/intermediate-tools-map.md`](docs/specs/intermediate-tools-map.md) - Core packages and intermediate `*graphen` tools
- [`docs/specs/ai-agent-integration.md`](docs/specs/ai-agent-integration.md) - Skills, plugins, MCP, and marketplace integration strategy
- [`skills/highergraphen/SKILL.md`](skills/highergraphen/SKILL.md) - Repository-owned CLI skill for the first HigherGraphen report contract
- [`docs/specs/rust-core-model.md`](docs/specs/rust-core-model.md) - Rust core data model specification
- [`docs/specs/engine-traits.md`](docs/specs/engine-traits.md) - Engine interface specification
- [`docs/product-packages/architecture-product.md`](docs/product-packages/architecture-product.md) - Reference Architecture Product
- [`docs/product-packages/domain-products.md`](docs/product-packages/domain-products.md) - Additional domain products
- [`docs/mvp-roadmap.md`](docs/mvp-roadmap.md) - MVP scope, roadmap, and success criteria
- [`docs/source-trace.md`](docs/source-trace.md) - Trace from proposal sections to official documents
- [`docs/adr/0001-rust-first-polyglot-friendly.md`](docs/adr/0001-rust-first-polyglot-friendly.md) - Architecture decision record
- [`.casegraphen/README.md`](.casegraphen/README.md) - Public development case graph and publication rules
- [`COMMERCIAL_BOUNDARY.md`](COMMERCIAL_BOUNDARY.md) - Public/commercial repository boundary

## Design Principle

HigherGraphen treats each concrete product as an interpretation package over a
shared higher-structure core:

```text
Product = Interpretation Package over Higher Structure
```

The goal is to avoid rebuilding a reasoning foundation for every product. The
core supplies structural primitives and engines. Domain packages supply
vocabulary, mappings, invariants, completion rules, and projections.

## Layer Model

HigherGraphen is organized into three layers:

```text
Level 0: Higher Structure Core
  Space, Cell, Complex, Context, Morphism, Invariant, Obstruction,
  Completion, Projection, and related primitives.

Level 1: Intermediate Abstract Tools
  Case, morphism, context, obstruction, completion, invariant, evidence,
  projection, correspondence, evolution, and interpretation tooling.

Level 2: Domain Products
  Architecture, contract, project, incident, research, governance, feed
  analysis, and other concrete products.
```

Human-facing UI, reports, dashboards, and summaries should be treated as
projections over this structure. Agent-facing CLI, schemas, skills, and future
MCP surfaces should preserve the underlying structure so agents can inspect
provenance, candidates, obstructions, invariants, and information loss directly.
