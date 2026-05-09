# Core Extension Operation Contract

This document fixes the operation contract for the HigherGraphen core extension
objects accepted by
[`ADR 0002`](../adr/0002-ai-operated-structure-spine.md). It complements
[`core-contracts.md`](core-contracts.md) and the Japanese draft
[`../highergraphen_core_extension_basic_design.md`](../highergraphen_core_extension_basic_design.md).

## Scope

The core extension objects are shared structural primitives, not complete
domain workflows:

- `EquivalenceClaim`
- `Derivation`
- `Witness`
- `Scenario`
- `Capability`
- `Policy`
- `Valuation`
- `SchemaMorphism`

Each object must remain portable across runtime reports, CLI JSON, future SDK
bindings, and agent projections. Domain-specific semantics belong in
interpretation packages, runtime workflows, tools, or product packages.

## Contract Rules

Every core extension object must define:

- data model and stable serialization;
- constructor and deserialization validation;
- lifecycle or review status semantics;
- at least one engine, runtime, or tool operation that consumes it;
- projection behavior for human, AI-agent, or audit views;
- failure modes that produce structured findings rather than silent promotion;
- source or witness trace when it supports a claim, decision, or action.

AI-inferred extension objects start as candidates or unreviewed records unless
an explicit workflow supplies accepted provenance. Review state must not be
encoded as confidence.

## Operation Matrix

| Object | Core role | Required validation | Runtime/tool operation | Projection | Failure mode |
| --- | --- | --- | --- | --- | --- |
| `EquivalenceClaim` | Claims that structures may be treated as equivalent under a criterion, scope, and loss model. | Non-empty subjects, declared scope for accepted claims, criterion, confidence, provenance, and review status. | Equivalence check, quotient preview, schema or interpretation alignment. | Identity risk report, quotient preview, audit trace. | Unsupported equivalence, context-loss conflict, affected invariant not checked. |
| `Derivation` | Records how premises lead to a conclusion under inference rules. | Non-empty premises and conclusion, inference rule, verifier status, provenance. | Proof or reasoning verification, semantic-proof report, unsupported inference detection. | Audit trail, unsupported inference report. | Missing premise, unverified rule, failed verifier, conclusion not derived. |
| `Witness` | Provides observable support or counterexample payload for claims, derivations, constraints, and obstructions. | Payload reference or observed structure, witness type, status, provenance. | Invariant check, obstruction explanation, proof/counterexample attachment. | Evidence summary, counterexample report. | Missing payload, unverifiable witness, stale source, counter-witness conflict. |
| `Scenario` | Represents candidate, hypothetical, reachable, counterfactual, or accepted worlds. | Declared kind, status, changed structures, source boundary, review state. | Runtime smoke scenarios, what-if workflows, model checking, completion dry-runs. | What-if report, reachability report, dry-run preview. | Candidate treated as reality, reachability unverified, scenario changes lack witnesses. |
| `Capability` | Describes actor, target, operation, context, and current permission state. | Actor, target, operation, status, applicable policy references. | Agent permission view, command gating, reviewed morphism application. | Agent operation contract, permission report. | Agent attempts unsupported operation, missing owner, stale capability. |
| `Policy` | Encodes allow, deny, require, and review conditions for operations or projections. | Policy kind, applicability, rules, required review, lifecycle status. | Review gates, projection/export safety checks, completion promotion policy. | Policy compliance report, review requirement summary. | Policy ambiguity, forbidden operation, missing required review. |
| `Valuation` | Ranks structures, scenarios, morphisms, or candidates under explicit criteria. | At least one criterion, direction/order type, subject, value or tradeoff, provenance. | Candidate ranking, tradeoff comparison, decision support. | Decision comparison, tradeoff report. | Value criteria conflict, unsupported score, desirable confused with true. |
| `SchemaMorphism` | Maps schema, ontology, or interpretation package versions with declared compatibility and loss. | Source and target schemas, mapping kind, mapping entries, compatibility, verification state, loss claims. | Lift adapter validation, migration report, compatibility check. | Migration report, compatibility/loss report. | Undeclared loss, incompatible schema change, unverified mapping. |

## Ownership

Core owns only the portable object contracts and primitive validation rules.

The expected owning surfaces are:

- `higher-graphen-core`: object definitions, lifecycle/status types,
  validation, serialization.
- `higher-graphen-structure`: structural references and morphism/context
  relationships that extension objects point to.
- `higher-graphen-evidence`: richer evidence, causal, confidence, and prover
  records that can be referenced by `Witness` and `Derivation`.
- `higher-graphen-reasoning`: invariant, obstruction, completion, model
  checking, abstract interpretation, and uncertainty operations that consume
  the extension objects.
- `higher-graphen-runtime`: report envelopes and workflow-specific views.
- `tools/*graphen`: CLI parsing, JSON output, review commands, and report
  validation.
- skills and agent integrations: procedural guidance and projection
  interpretation.

## Acceptance Gate

A new or changed extension object is complete only when the pull request or
design change includes:

- a documented row in this matrix;
- constructor and deserialization tests for invalid states;
- at least one runtime, reasoning, or tool witness showing how the object is
  consumed;
- projection behavior that declares information loss where applicable;
- review semantics showing that candidates are not silently promoted.
