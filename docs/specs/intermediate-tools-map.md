# Intermediate Tools Map

This document distinguishes HigherGraphen core packages from intermediate
`*graphen` tools.

## Layer Distinction

Core packages provide structural primitives and engines:

```text
higher-graphen-core
higher-graphen-structure
higher-graphen-projection
higher-graphen-evidence
higher-graphen-reasoning
higher-graphen-interpretation
higher-graphen-runtime
```

Intermediate tools are centered on one abstract object and provide vocabulary,
schemas, workflows, checks, completions, obstructions, and projections over the
core:

```text
casegraphen
morphographen
contextgraphen
invariantgraphen
obstructiongraphen
completiongraphen
evidencegraphen
projectiongraphen
```

The relationship is:

```text
higher-graphen-structure::morphism
  = Historical package name for the morphism module now exposed as
    higher-graphen-structure::morphism.

morphographen
  = Morphism-centered intermediate tool built on HigherGraphen.
```

## Intermediate Tool Definition

An intermediate tool should provide:

- A central abstract object.
- Domain-neutral vocabulary around that object.
- Schemas and structural templates.
- Invariants and constraints.
- Obstruction and counterexample patterns.
- Completion rules.
- Projection templates.
- Review workflows.
- Example scenarios.

Intermediate tools are not end-user business applications. They are reusable
toolkits that can be interpreted into domain products.

## Core Package to Tool Map

| Core package/module | Intermediate tool | Central object | Primary concepts | Main outputs |
| --- | --- | --- | --- | --- |
| `higher-graphen-structure::space` | `casegraphen` | Case, scenario, situation | Typed graph, cell complex, coverage, boundary | Case map, missing case, conflicting case, case projection |
| `higher-graphen-structure::morphism` | `morphographen` | Morphism, transformation, mapping | Category theory, composition, commutative diagram, preservation, information loss | Preservation report, lost structure report, transformation chain |
| `higher-graphen-structure::context` | `contextgraphen` | Context, local model, global consistency | Sheaf-inspired modeling, cover, section, restriction, gluing | Context map, context mismatch, gluing failure |
| `higher-graphen-reasoning::invariant` | `invariantgraphen` | Invariant, conserved property | Topological invariant, type invariant, equivalence class, design by contract, abstract interpretation | Invariant catalog, preservation check, violated invariant |
| `higher-graphen-reasoning::obstruction` | `obstructiongraphen` | Obstruction, contradiction, impossibility | Unsatisfiability, counterexample, non-commutative diagram, local/global inconsistency | Obstruction report, counterexample, required resolution |
| `higher-graphen-reasoning::completion` | `completiongraphen` | Missing structure, completion candidate | Graph completion, constrained completion, free construction, structural analogy, holes in complexes | Completion candidate, missing test, missing API, missing constraint |
| `higher-graphen-evidence` | `evidencegraphen` | Claim, evidence, counter-evidence | Argumentation graph, provenance graph, Bayesian update, defeasible reasoning, proof object | Evidence report, unsupported claim, contradiction report |
| `higher-graphen-projection` | `projectiongraphen` | Projection, view, audience | Projection map, quotient structure, lens, abstraction, information loss, observer model | Human report, AI view, audit view, loss declaration |

## Primary Initial Set

The first intermediate tools should be:

| Tool | Why it is foundational |
| --- | --- |
| `casegraphen` | Captures concrete situations and scenarios as structured cases. |
| `morphographen` | Handles transformations, which are central to AI-native work. |
| `contextgraphen` | Prevents context collapse and manages local/global consistency. |
| `invariantgraphen` | States what must survive change. |
| `obstructiongraphen` | Explains why a structure, plan, or transformation cannot hold. |
| `completiongraphen` | Turns AI completion into reviewable structural completion. |
| `evidencegraphen` | Separates claims, evidence, observations, and AI inference. |
| `projectiongraphen` | Makes higher structure usable for humans, AI agents, and systems. |

These tools map onto conceptual modules, not one package per tool. The module
boundaries should shape early examples, tests, and documentation without
forcing a matching crate boundary.

## Extended Candidate Tools

The following tools are plausible but should be introduced after the primary set
unless a reference product requires them earlier.

| Tool | Central object | Primary concepts | Typical questions |
| --- | --- | --- | --- |
| `stategraphen` | State, transition, trace | State machine, model checking, temporal logic, reachability, Petri net | Can this state be reached? Is there a path to a forbidden state? |
| `tracegraphen` | Trace, history, audit trail | Provenance graph, trace semantics, event sourcing, temporal graph, causality | Where did this structure come from? Which transformation changed it? |
| `constraintgraphen` | Constraint set, feasible region | SAT, SMT, constraint satisfaction, optimization, priority constraints | Can these constraints be satisfied together? Which constraint must relax? |
| `boundarygraphen` | Boundary, interface, crossing | Boundary map, information flow, context map, interface theory, gluing | What is preserved or lost across a boundary? |
| `sectiongraphen` | Section, local assignment | Sheaf-inspired section, restriction, gluing, local truth, partial assignment | Can local observations be glued into a global section? |
| `liftgraphen` | Lift, raw input, structured representation | Semantic parsing, typed extraction, provenance, confidence modeling | How should text, logs, or documents become higher structure? |
| `quotientgraphen` | Quotient, equivalence class, abstraction | Quotient structure, equivalence relation, coarse graining, Galois connection, information loss | Which objects may be identified? What does abstraction hide? |
| `intentiongraphen` | Intent, goal, desired state | Goal graph, objective function, morphism, completion, constraint | What is the operation trying to achieve? Is there a better morphism? |
| `decisiongraphen` | Decision, option, criterion | Decision theory, Pareto frontier, multi-objective optimization, regret minimization | Why was this option chosen? Which alternatives are dominated? |
| `interfacegraphen` | Interface, contract, guarantee | Design by contract, precondition, postcondition, refinement, behavioral subtyping | Is this interface change semantically compatible? |
| `topologygraphen` | Shape, hole, coverage | Topology, homology, persistent homology, connected component, boundary | Where are holes, isolated regions, or persistent structures? |

## Tool Composition

Intermediate tools should be composable. For example:

```text
liftgraphen
  -> casegraphen
  -> invariantgraphen
  -> obstructiongraphen
  -> completiongraphen
  -> projectiongraphen
```

Architecture analysis may combine:

```text
casegraphen
morphographen
contextgraphen
invariantgraphen
obstructiongraphen
completiongraphen
evidencegraphen
projectiongraphen
```

An audit product may combine:

```text
evidencegraphen
tracegraphen
decisiongraphen
projectiongraphen
```

## Naming Policy

Use `higher-graphen-*` for core packages and `*graphen` for intermediate tools.

Examples:

```text
higher-graphen-structure::context -> contextgraphen
higher-graphen-evidence           -> evidencegraphen
higher-graphen-runtime            -> no direct intermediate tool; it hosts tool workflows.
```

Do not name a business product directly as `*graphen` unless it is intended to
be a reusable intermediate abstraction. Domain products should be named by their
domain:

```text
Architecture Product
Contract Product
Project Product
AI Governance Product
Incident Analysis Product
Research Support Product
```

## Implementation Guidance

Each intermediate tool should define a package-level contract before
implementation:

| Contract item | Description |
| --- | --- |
| Central object | The abstract object the tool organizes around. |
| Core dependencies | Required HigherGraphen packages. |
| Theory dependencies | Mathematical, formal-methods, or CS concepts used. |
| Schema | Tool-specific structural schema. |
| Invariants | Properties the tool can check. |
| Obstructions | Failure objects the tool can produce. |
| Completions | Missing structures the tool can propose. |
| Projections | Human, AI, audit, or system views the tool can emit. |
| Workflows | Common operations and review flows. |
| Reference scenarios | Minimal examples proving the tool works. |

This prevents a `*graphen` tool from becoming only a naming wrapper around a
core package.
