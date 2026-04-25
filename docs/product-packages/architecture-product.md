# Architecture Product

The Architecture Product is the first recommended reference product for
HigherGraphen.

## Purpose

The Architecture Product represents architecture documents, APIs, databases,
events, requirements, and tests as higher structure. It detects design
inconsistency, missing structure, and unsafe transformations.

## Interpretation

The product maps architecture vocabulary onto HigherGraphen primitives.

| HigherGraphen primitive | Architecture interpretation |
| --- | --- |
| Cell | Component, API, database, event, requirement, test. |
| Morphism | Dependency, interface, requirement-to-design mapping, design-to-test mapping. |
| Invariant | Boundary rule, verification rule, ownership rule, consistency rule. |
| Obstruction | Boundary violation, missing interface, missing test, inconsistent mapping. |
| Completion Candidate | Proposed API, proposed test, proposed interface, proposed ownership clarification. |
| Projection | Architecture review report, design risk summary, developer action plan. |

## Inputs

Initial inputs may include:

- Architecture documents.
- OpenAPI specifications.
- Database schemas.
- ADRs.
- Test specifications.
- Tickets or issue descriptions.

## Core Invariants

The first reference scenario should include these invariants:

| Invariant | Meaning |
| --- | --- |
| No cross-context direct database access | A component must not directly access a database owned by another context. |
| Requirement must be verified | A requirement must map to at least one design element and at least one test or accepted verification method. |
| Projection must declare information loss | A report or view must state what was omitted or simplified. |

## MVP Scenario

Input:

```text
Order Service manages orders.
Billing Service manages billing.
Order Service reads Billing DB to check billing status.
Billing DB is owned by Billing Service.
```

Expected structure:

```text
Cells:
  Order Service
  Billing Service
  Billing DB

Incidences:
  Order Service -> Billing DB
  Billing Service -> Billing DB

Invariant:
  No Cross-context Direct Database Access

Obstruction:
  Order Service directly accesses Billing DB.

CompletionCandidate:
  Billing Service should expose an API for billing status queries.
```

Expected projection:

```text
Architecture inconsistency detected:

Order Service directly accesses Billing DB, which is owned by Billing Service.
This violates the cross-context database access invariant.

Recommended action:
  - Add a billing status query API to Billing Service.
  - Change Order Service to use the Billing API instead of Billing DB.
```

## MVP Validation Goal

This scenario is successful when HigherGraphen can:

1. Build the structural model from the input.
2. Represent ownership and access as cells and incidences.
3. Evaluate the boundary invariant.
4. Produce an obstruction.
5. Produce a completion candidate.
6. Project the result into a human-readable architecture review.

