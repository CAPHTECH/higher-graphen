# AI Operator Paradigm

## Purpose

HigherGraphen is based on a shift in the assumed operator of software.

Most products and services have been designed for human operators. They expose
objects, screens, workflows, permissions, reports, and dashboards at a level
that fits human attention, memory, vocabulary, and manual decision-making.
HigherGraphen starts from a different premise: when AI agents become primary
operators, the product surface can move closer to the structural concepts that
the system actually needs to reason about.

This does not remove humans from the loop. It changes where human-facing
interfaces sit in the architecture. A human interface becomes a projection from
the structural system, not the system's only or primary model.

## Human-Operated Product Design

Human-operated software is shaped by human cognitive limits.

Typical product surfaces are organized around:

- Screens that keep a small amount of state visible at once.
- CRUD operations that let a person inspect and modify one object at a time.
- Reports that compress complex state into readable summaries.
- Workflows that sequence decisions into manageable steps.
- Tickets, comments, and approvals that preserve social accountability.
- Dashboards that turn continuous system behavior into human-scannable signals.

These are valuable design patterns when the user is a person. They reduce
cognitive load, provide navigable context, and make work accountable. However,
they also impose a ceiling on the structural concepts that the product can
directly expose. Many deeper relationships are flattened into text, fields,
links, labels, and manual interpretation.

For example, an architecture review product designed only for humans might
show documents, component lists, dependency diagrams, risks, and recommended
actions. The human reviewer is expected to mentally assemble the higher
structure: which requirement maps to which design element, which dependency
breaks a boundary invariant, which missing interface is only a completion
candidate, and which conclusion is based on evidence rather than inference.

## AI-Operated Product Design

AI operators can work with a different product surface.

An AI agent does not need every capability to be reduced first into a screen,
form, dashboard, or prose report. It can call tools, inspect structured output,
maintain larger working context, compare alternatives, and operate over
explicit symbolic structures. This makes it possible to expose concepts that
would be too abstract, dense, or operationally awkward for a human-first UI.

HigherGraphen therefore treats concepts such as the following as first-class
operational objects:

- `Space`: the bounded structural world under analysis.
- `Cell`: an entity, relation, higher-order relation, observation, constraint,
  or consistency condition.
- `Complex`: an organized structure of cells and incidences.
- `Context`: a local region where vocabulary, validity, or rules apply.
- `Morphism`: a transformation, mapping, lift, projection, interpretation, or
  migration between structures.
- `Invariant`: a property that must be preserved.
- `Obstruction`: a structured reason that something cannot hold or proceed
  safely.
- `CompletionCandidate`: a proposed missing structure that remains reviewable.
- `Projection`: a view produced for a specific audience and purpose, with
  declared information loss.
- `InterpretationPackage`: the domain-specific meaning layer placed over the
  shared structural core.

In this model, the AI does not merely automate clicks in a human product. It
operates directly on the concepts that define the target world.

## The Central Shift

The central shift is:

```text
Human-operated product:
  Product model is constrained by what humans can manually inspect and operate.

AI-operated product:
  Product model can expose higher-order structure directly, then project it
  into human views when needed.
```

This shift changes product architecture.

In a human-first system, a report is often the final product. In HigherGraphen,
a report is a projection from a richer structure. In a human-first system, a
workflow step may be the unit of progress. In HigherGraphen, progress may be
expressed as a morphism preserving invariants, an obstruction blocking a
transformation, or a completion candidate waiting for review. In a human-first
system, ambiguity often stays in prose. In HigherGraphen, ambiguity should be
represented as provenance, confidence, review status, alternative structure, or
an unresolved local/global consistency problem.

## Product Principle

HigherGraphen's product principle is:

```text
Product = Interpretation Package over Higher Structure
```

The shared higher-structure core provides the operational primitives. A domain
product supplies the interpretation:

- Vocabulary for cells and relationships.
- Domain-specific contexts.
- Domain-specific invariants.
- Completion rules.
- Obstruction classifications.
- Projection templates.
- Lift adapters from source material into structure.
- Review and provenance policy.

This lets multiple products share the same reasoning foundation without
forcing them to share the same human-facing UI or domain vocabulary.

Examples:

```text
Architecture Product = Architecture Interpretation over HigherGraphen
Feed Product         = Feed Interpretation over HigherGraphen
Contract Product     = Contract Interpretation over HigherGraphen
Project Product      = Project Interpretation over HigherGraphen
Evidence Product     = Evidence Interpretation over HigherGraphen
```

## Why Ordinary Graphs Are Not Enough

Ordinary graphs are useful, but they usually center on nodes and binary edges.
That is not enough for AI-operated structural products.

Many important product questions are higher-order:

- A problem may arise only from the joint relationship between three or more
  entities.
- Two local structures may be valid independently but inconsistent globally.
- A transformation may preserve some invariants while losing others.
- A proposed completion may be plausible but not yet accepted as fact.
- A projection may be useful for a human while omitting material detail.
- A concept may have different meaning in different contexts.
- An AI inference may need to be kept separate from an observed fact.

These questions require more than node-edge storage. They need cells of
different dimensions, context-sensitive interpretation, morphisms, invariants,
obstructions, completions, provenance, and projections.

## Relationship to Human Interfaces

HigherGraphen does not reject human-facing products. It changes their role.

Human interfaces remain necessary for review, accountability, decision-making,
exploration, and communication. The difference is that they should be treated
as projections from a structural substrate. A UI, report, or dashboard is one
view of the structure, optimized for a human audience and a specific purpose.

This has two important consequences:

1. A human view should declare meaningful information loss when it compresses
   higher structure.
2. The source structure should remain available for AI agents, audits, and
   alternative projections.

The human interface is no longer the only durable representation of product
state.

## Safety and Review

AI-operated products need stronger boundaries between observation, inference,
candidate structure, and accepted conclusion.

HigherGraphen therefore treats provenance and review status as core concerns.
An AI agent may propose a completion candidate, classify an obstruction, infer
a correspondence, or generate a projection. That does not make the result a
fact. The structure must preserve where the claim came from, how confident the
system is, which evidence it depends on, and whether a human or trusted process
has accepted it.

This is why `CompletionCandidate` is not just an implementation detail. It is a
product-level safety concept. HigherGraphen should make it difficult to
silently convert plausible missing structure into accepted structure.

## Architecture Product Example

In a human-first architecture review tool, a reviewer might read documents and
write a risk report:

```text
Order Service reads Billing DB directly. This violates a boundary rule.
Recommended action: add a Billing API.
```

HigherGraphen represents the same situation structurally:

- `Order Service`, `Billing Service`, and `Billing DB` are cells.
- Ownership and access are incidences or morphisms.
- "No cross-context direct database access" is an invariant.
- The direct read is an obstruction.
- "Add a Billing API" is a completion candidate.
- The human-readable review is a projection.

The projection is useful, but it is not the full product state. The AI operator
can inspect the invariant, obstruction, candidate, provenance, and information
loss directly.

## Feed Product Example

In a human-first feed reader, the product may show a timeline, unread counts,
folders, and article cards. Those are useful human views. For an AI operator,
the deeper product surface can be a source-indexed observation space:

- Feed entries are observation cells with source and time provenance.
- Topics and incidents can be inferred as grouping cells.
- Duplicates, follow-ups, and counterpoints are correspondences.
- Missing official sources or missing counterpoints are completion candidates.
- Conflicting source claims are obstructions.
- Timeline, topic digest, and audit views are projections.

The AI can operate over the structure before producing a human-readable digest.

## Design Implications

This paradigm leads to several design requirements:

- Model the target world before designing the human report.
- Treat UI, CLI output, dashboards, and prose as projections.
- Preserve provenance for observations, claims, inferences, and accepted
  conclusions.
- Represent missing structure as candidates, not silent facts.
- Make invariants and obstructions explicit product concepts.
- Make transformations inspectable through morphisms and preservation rules.
- Allow multiple domain products to share the same structural core.
- Provide agent-facing tools, schemas, and skills in addition to human
  documentation.

## Non-Goals

HigherGraphen is not a claim that humans should read high-dimensional structure
directly. Humans still need clear projections.

It is also not a claim that AI output should be trusted by default. The point
is the opposite: AI-operated products need explicit structure for provenance,
review, candidates, confidence, and information loss.

Finally, HigherGraphen is not only an automation layer over existing products.
Automation keeps the human-shaped product model and asks AI to operate it.
HigherGraphen asks what the product model should become when AI can operate
higher-order structure directly.
