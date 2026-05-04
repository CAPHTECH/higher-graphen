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

## What This Is

HigherGraphen is a structural substrate for AI-operated software development.
It is not a UI framework, ticket tracker, knowledge base, or architecture
diagramming tool by itself. It is the layer underneath those products that
turns domain material into AI-operable structure:

- Cases and work graphs.
- Evidence, provenance, confidence, and review status.
- Invariants that must be preserved.
- Obstructions that explain why a change, interpretation, or workflow cannot
  safely proceed.
- Completion candidates that represent plausible missing structure without
  silently promoting them to accepted fact.
- Projections for humans, AI agents, audits, and other consumers.
- Interpretation packages that map a domain onto the shared higher-structure
  core.

The practical goal is to let AI agents operate on the structure directly while
humans receive reports, dashboards, and review surfaces as projections.

## Higher-Order Concepts

HigherGraphen uses the following product-level concepts as AI-operable objects:

- `Space`: the bounded structural world under analysis, such as a codebase,
  project, incident, contract, feed, or research corpus.
- `Cell`: a unit inside that world. A cell may represent an entity, relation,
  observation, constraint, or higher-order relation.
- `Complex`: an organized structure of cells and incidences. It represents
  relationships that are richer than ordinary node-edge graphs.
- `Context`: a local region where vocabulary, validity, or rules apply. This
  lets the system represent cases where local structures are valid but do not
  compose cleanly into a global structure.
- `Morphism`: a structure-preserving or structure-transforming map, such as a
  lift, projection, migration, interpretation, or comparison.
- `Invariant`: a property that should remain true across changes or
  interpretations.
- `Obstruction`: a structured reason that something cannot hold, proceed, or
  be accepted safely.
- `CompletionCandidate`: a plausible missing structure proposed by a system or
  AI agent, kept separate from accepted fact until review.
- `Projection`: a purpose-specific view for humans, AI agents, audits, CLI
  output, or other consumers, with declared information loss.
- `InterpretationPackage`: the domain-specific meaning layer that maps a
  product domain onto the shared higher-structure core.

These are not only internal implementation types. They are product concepts
that an AI agent can inspect, operate, validate, and pass through workflows.

## Mathematical Influences

HigherGraphen does not require users to know advanced mathematics. The project
borrows structural ideas from several fields and turns them into engineering
objects that AI agents can inspect and operate.

- **Graphs and hypergraphs** inform how ordinary relations and simultaneous
  multi-entity relations are represented.
- **Cell complexes and simplicial complexes** inform how points, edges, faces,
  higher-dimensional cells, boundaries, and holes can be modeled.
- **Category theory and morphisms** inform transformations, composition,
  preservation, distortion, and loss between structures.
- **Sheaf-inspired local-to-global modeling** informs how locally valid
  contexts can fail to glue into a consistent global picture.
- **Topology and obstruction theory** inform how missing regions, holes,
  impossibility, and blocked progress are represented structurally.
- **Type theory, contracts, and invariants** inform how invalid states are
  prevented or detected.
- **Provenance and evidence modeling** inform the distinction between
  observation, claim, AI inference, reviewable candidate, and accepted
  conclusion.

The detailed theory-to-engineering mapping lives in
[`docs/concepts/theoretical-foundations.md`](docs/concepts/theoretical-foundations.md).

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

## What You Can Build With HigherGraphen

HigherGraphen is not limited to one product category. It is a substrate for
turning domain material into AI-operable structures: cases, evidence,
invariants, obstructions, completions, projections, and interpretation
packages.

Examples of products that can be built on top of it:

- **Architecture review products** that lift design documents and code
  structure into components, boundaries, invariants, violations, obstructions,
  completion candidates, and review projections.
- **AI coding governance tools** that track AI-performed changes as cases with
  evidence, decisions, blockers, review state, and invariant checks instead of
  treating code diffs as the only durable record.
- **Incident analysis products** that connect logs, metrics, deploys,
  investigation notes, candidate causes, missing evidence, and prevention
  actions.
- **Research and knowledge synthesis tools** that separate observations,
  claims, AI inferences, contradictions, correspondence, and accepted
  conclusions.
- **Contract and policy review products** that represent obligations,
  exceptions, undefined terms, conflicts, risks, and reviewable completion
  candidates.
- **Project and roadmap reasoning tools** that expose goals, tasks, decisions,
  dependencies, blockers, and verification evidence as an agent-operable case
  graph.
- **Feed and signal intelligence products** that treat feeds, news, issues,
  notifications, and other source material as source-contexted observations
  with correspondences, gaps, obstructions, and projections.

In each case, the human-facing product may still look like a report, review
screen, dashboard, or CLI output. The difference is that those surfaces are
projections from a richer structure that an AI agent can inspect and operate.

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

## Release And Install

The latest release is
[`v0.4.0`](https://github.com/CAPHTECH/higher-graphen/releases/tag/v0.4.0).
It includes Darwin arm64 binaries for:

- `casegraphen`
- `highergraphen`

Cargo packages are configured for registry packaging. After the package
publication step, install the CLI surfaces with:

```sh
cargo install highergraphen-cli
cargo install casegraphen
```

Library consumers can depend on the workspace crates directly, for example:

```toml
[dependencies]
higher-graphen-core = "0.4.0"
higher-graphen-runtime = "0.4.0"
```

Publish reusable crates before dependent crates and CLI tools:

```sh
cargo publish -p higher-graphen-core
cargo publish -p higher-graphen-structure
cargo publish -p higher-graphen-projection
cargo publish -p higher-graphen-evidence
cargo publish -p higher-graphen-reasoning
cargo publish -p higher-graphen-interpretation
cargo publish -p higher-graphen-runtime
cargo publish -p casegraphen
cargo publish -p highergraphen-cli
```

The example workspace packages remain unpublished validation fixtures. To build
locally from source, use Cargo from the repository root:

```sh
cargo build --workspace --release --locked
```

## What You Can Run Today

The repository already includes a Rust workspace, core crates, schemas,
reference examples, and two CLI surfaces:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access --format json
```

This emits a deterministic Architecture Product report showing an invariant
violation, obstruction, completion candidate, provenance, and projections.

```sh
cargo run -q -p casegraphen -- \
  workflow reason \
  --input schemas/casegraphen/workflow.graph.example.json \
  --format json
```

This emits a CaseGraphen workflow reasoning report over a structured workflow
graph.

Native CaseGraphen case management is also available through the repo-owned
`casegraphen` CLI:

```sh
cargo run -q -p casegraphen -- \
  case import \
  --store /tmp/casegraphen-native-store \
  --input schemas/casegraphen/native.case.space.example.json \
  --revision-id revision:native-reference-imported \
  --format json
```

After import, derive native reasoning views from the replayed `CaseSpace` plus
`MorphismLog`:

```sh
cargo run -q -p casegraphen -- case reason --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case close-check --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --base-revision-id revision:native-reference-imported --validation-evidence-id evidence:native-schema-json-valid --format json
```

The DDD diagnostic example uses the same native report surface to review a
Sales/Billing `Customer` domain model decision:

```sh
cargo run -q -p casegraphen -- \
  case import \
  --store /tmp/casegraphen-ddd-store \
  --input examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json \
  --revision-id revision:ddd-sales-billing-imported \
  --format json
```

Then run:

```sh
cargo run -q -p casegraphen -- case reason --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
```

The expected result is a blocked domain model decision with boundary semantic
loss, missing accepted evidence, unreviewed completion candidates, and
projection loss represented as successful JSON report data.

## CaseGraphen As The First Real Case

CaseGraphen is the first concrete system built inside this repository from the
HigherGraphen thesis. It is not only an example file format or a side utility;
it is a real intermediate tool for representing complex, evidence-heavy,
decision-rich work as case graphs and workflow graphs.

AI-operated software development is the first reference domain, but the
abstraction is not limited to software development. CaseGraphen fits work where
goals, tasks, decisions, evidence, unresolved questions, blockers, completions,
reviews, and future projections need to remain inspectable over time.

CaseGraphen makes the product thesis inspectable:

- [`tools/casegraphen/`](tools/casegraphen/) provides a CLI that reasons over
  cases, evidence, tasks, blockers, completions, projections, and workflow
  state.
- [`schemas/casegraphen/`](schemas/casegraphen/) defines the structured
  contracts that make those concepts machine-checkable.
- [`skills/casegraphen/SKILL.md`](skills/casegraphen/SKILL.md) gives AI agents
  an operating protocol for reading and authoring CaseGraphen workspaces.
- [`examples/casegraphen/reference/`](examples/casegraphen/reference/) shows a
  runnable reference workflow graph and report.
- [`examples/casegraphen/native/`](examples/casegraphen/native/) shows the
  native `CaseSpace` plus `MorphismLog` case management flow.
- [`examples/casegraphen/ddd/domain-model-design/`](examples/casegraphen/ddd/domain-model-design/)
  remains the legacy fixture that motivates the product-facing DDD review
  workflow.
- [`skills/highergraphen-ddd/SKILL.md`](skills/highergraphen-ddd/SKILL.md)
  gives AI agents an operating protocol for `highergraphen ddd` review reports.

This matters because it demonstrates the intended direction of HigherGraphen:
complex work is not reduced to a human-facing issue list, document, dashboard,
or command history. Goals, decisions, evidence, obstructions, completions,
reviews, and future projections become first-class structures that an AI agent
can inspect and operate directly.

The current CaseGraphen surface is specified in
[`docs/specs/intermediate-tools/casegraphen.md`](docs/specs/intermediate-tools/casegraphen.md),
[`docs/specs/intermediate-tools/casegraphen-workflow-contracts.md`](docs/specs/intermediate-tools/casegraphen-workflow-contracts.md),
[`docs/specs/intermediate-tools/casegraphen-feature-completion-contract.md`](docs/specs/intermediate-tools/casegraphen-feature-completion-contract.md),
and
[`docs/specs/intermediate-tools/casegraphen-native-case-management.md`](docs/specs/intermediate-tools/casegraphen-native-case-management.md).

## Status

This repository is an early public implementation. It contains the core Rust
workspace, package boundaries, report schemas, CLI contracts, reference product
packages, public examples, native CaseGraphen case management, and CaseGraphen
CLI and skill surfaces.

The implementation is still evolving. The most stable entry point is the
reference Architecture Product smoke report. The broader goal is to make the
case, evidence, obstruction, completion, projection, and interpretation-package
surfaces robust enough for AI agents to use directly.

## How To Read This Repository

If you are new to HigherGraphen, start here:

1. Read this README for the product model and current runnable surfaces.
2. Read
   [`docs/concepts/ai-operator-paradigm.md`](docs/concepts/ai-operator-paradigm.md)
   for the reason HigherGraphen is shaped around AI operators.
3. Run the Architecture Product smoke command above and inspect the JSON report.
4. Run the CaseGraphen workflow reasoning command above and inspect
   [`examples/casegraphen/reference/`](examples/casegraphen/reference/).
5. Run the native CaseGraphen examples when you want to see `CaseSpace` and
   `MorphismLog` in action, or run the `highergraphen ddd` workflow when you
   want DDD evidence boundaries, completion candidates, projection loss, and
   closeability in a product CLI report.
6. Use [`docs/index.md`](docs/index.md) when you want the full specification
   reading order.

## License And Commercial Boundary

HigherGraphen's public core is licensed under the
[Apache License 2.0](LICENSE).

Copyright 2026 CAPH TECH Inc.

The public repository is intended to contain the shared higher-structure core,
baseline intermediate tools, schemas, documentation, public examples, skills,
and reference workflows. Production interpretation packages, hosted execution,
customer-specific assets, private evaluation datasets, commercial strategy, and
private operations material belong outside this public repository unless they
are explicitly open-sourced later.

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
- [`docs/specs/intermediate-tools/casegraphen.md`](docs/specs/intermediate-tools/casegraphen.md) - CaseGraphen intermediate tool specification
- [`docs/specs/intermediate-tools/casegraphen-workflow-contracts.md`](docs/specs/intermediate-tools/casegraphen-workflow-contracts.md) - CaseGraphen workflow contracts
- [`docs/specs/intermediate-tools/casegraphen-native-case-management.md`](docs/specs/intermediate-tools/casegraphen-native-case-management.md) - Native CaseGraphen CaseSpace and MorphismLog case management contract
- [`examples/casegraphen/reference/README.md`](examples/casegraphen/reference/README.md) - CaseGraphen reference workflow example
- [`examples/casegraphen/native/README.md`](examples/casegraphen/native/README.md) - Native CaseGraphen reference flow
- [`examples/casegraphen/ddd/domain-model-design/README.md`](examples/casegraphen/ddd/domain-model-design/README.md) - Legacy DDD domain model fixture that motivates the HigherGraphen DDD review workflow
- [`docs/specs/ddd-review-cli-contract.md`](docs/specs/ddd-review-cli-contract.md) - HigherGraphen DDD review CLI contract
- [`docs/specs/ai-agent-integration.md`](docs/specs/ai-agent-integration.md) - Skills, plugins, MCP, and marketplace integration strategy
- [`skills/highergraphen/SKILL.md`](skills/highergraphen/SKILL.md) - Repository-owned CLI skill for the first HigherGraphen report contract
- [`skills/highergraphen-ddd/SKILL.md`](skills/highergraphen-ddd/SKILL.md) - Repository-owned DDD review CLI skill
- [`skills/casegraphen/SKILL.md`](skills/casegraphen/SKILL.md) - Repository-owned CaseGraphen CLI skill
- [`skills/release-runner/SKILL.md`](skills/release-runner/SKILL.md) - Repository-owned release preparation and publication skill
- [`docs/specs/rust-core-model.md`](docs/specs/rust-core-model.md) - Rust core data model specification
- [`docs/specs/engine-traits.md`](docs/specs/engine-traits.md) - Engine interface specification
- [`docs/product-packages/architecture-product.md`](docs/product-packages/architecture-product.md) - Reference Architecture Product
- [`docs/product-packages/domain-products.md`](docs/product-packages/domain-products.md) - Additional domain products
- [`docs/mvp-roadmap.md`](docs/mvp-roadmap.md) - MVP scope, roadmap, and success criteria
- [`docs/source-trace.md`](docs/source-trace.md) - Trace from proposal sections to official documents
- [`docs/adr/0001-rust-first-polyglot-friendly.md`](docs/adr/0001-rust-first-polyglot-friendly.md) - Architecture decision record
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
