# Product Integration for AI Agents

This guide teaches an AI agent how to lift a target product into
HigherGraphen structure.

It is not an API reference and not a theory introduction. The goal is to help
an agent decide which HigherGraphen objects to create, which objects must stay
reviewable, and which crate or runtime surface should be used when embedding
HigherGraphen into a real product.

## Operating Contract

Before modeling a product, preserve these boundaries:

| Rule | Meaning |
| --- | --- |
| Observed input is not complete truth. | A bounded source snapshot may be incomplete. Do not infer absence from missing input. |
| Inferred structure is not accepted fact. | AI-created cells, mappings, candidates, or explanations start as unreviewed unless an explicit review workflow accepts them. |
| A completion candidate is not an approved change. | It is a reviewable proposal for missing structure. |
| An equivalence claim is not identity. | It says that two structures may be treated as equivalent under stated criteria, scope, evidence, and loss. |
| A scenario is not the real world. | It is a hypothetical, reachable, counterfactual, or proposed state. |
| Confidence is not review acceptance. | A high confidence score does not replace `ReviewStatus` or policy approval. |
| A projection is lossy unless its loss is declared. | Human, AI, audit, and CLI views should preserve what was omitted or compressed. |
| A policy or capability check is not optional for agent action. | If an agent is expected to execute, approve, export, or mutate structure, the product must model who can do what under which policy. |

When uncertain, keep the structure reviewable instead of promoting it.

## Modeling Pipeline

Use this pipeline for new product integrations:

```text
source material
  -> bounded source snapshot
  -> Space
  -> accepted observation cells
  -> contexts and incidences
  -> morphisms and interpretation mapping
  -> invariants and checks
  -> obstructions
  -> completion candidates
  -> evidence, witnesses, confidence, review status
  -> projections
  -> runtime workflow or embedded product API
```

The important shift is that the report, UI, or CLI output is not the model. It
is a projection from a richer structure that an AI agent can inspect and
operate.

## Step 1: Bound The Source Snapshot

Start by defining what the product is allowed to treat as input fact.

Examples:

| Product | Bounded source snapshot |
| --- | --- |
| Architecture review | Submitted architecture input, selected code ownership records, declared rules, and source metadata. |
| Test gap detection | Git range, changed files, parsed symbols, supplied coverage, known tests, and optional requirements. |
| Case workflow | Case graph file, revisions, transition records, evidence records, and review decisions. |
| Feed intelligence | Feed entries, source metadata, timestamps, trust levels, and supplied grouping hints. |
| Contract review | Contract text excerpts, clause IDs, policy references, reviewer notes, and source document metadata. |

Record what was included, what was excluded, and what information was lost
during extraction. This source boundary should become part of the scenario or
report, not a hidden implementation detail.

Do not let an agent read an entire repository, document set, mailbox, or web
corpus and silently treat everything it noticed as accepted product state. If a
source adapter adds facts, mark them as adapter-supplied. If an agent infers
structure, keep it unreviewed.

## Step 2: Create The Space

Create one `Space` for the bounded target world.

The `Space` answers:

- What world is under analysis?
- Which source boundary produced it?
- Which cells, contexts, morphisms, invariants, obstructions, completion
  candidates, projections, and review records belong together?

Examples:

| Product | Space |
| --- | --- |
| Architecture review | `space:architecture:<system>:<snapshot>` |
| Test gap detection | `space:test-gap:<repo>:<base>..<head>` |
| Case workflow | `space:case:<case-id>:<revision>` |
| Feed intelligence | `space:feed:<source-set>:<window>` |

The `Space` is a structural view of a bounded snapshot. It is not a claim that
the product has captured the complete real world.

## Step 3: Lift Accepted Observations Into Cells

Lift source records into `Cell` objects.

Use cells for durable entities, observations, relationships, requirements,
claims, rules, tests, evidence artifacts, and higher-order relations. A cell is
not limited to a node in an ordinary graph.

| Source record | Possible cell |
| --- | --- |
| Service, database, API, module | Entity or component cell |
| Direct access, ownership, dependency | Relation cell |
| Requirement, clause, law, invariant text | Requirement or constraint cell |
| Test, coverage record, command output | Evidence or test cell |
| Claim from a document | Claim cell with provenance |
| Relation between several relations | Higher-order cell |

Accepted source records can be marked as accepted observations when they come
from the bounded source adapter or supplied input. AI-created cells should not
start accepted.

## Step 4: Connect Cells With Incidences

Use incidences to preserve why cells should be considered together.

Examples:

| Incidence | Meaning |
| --- | --- |
| `owns` | A context, team, or component owns another cell. |
| `depends_on` | One cell structurally depends on another. |
| `accesses` | A service, actor, or operation accesses a resource. |
| `implements` | A function, component, or clause implements a requirement. |
| `covered_by` | A behavior, branch, requirement, or function is covered by a test or evidence cell. |
| `supports` | Evidence supports a claim, obstruction, derivation, or candidate. |
| `contradicts` | Evidence or a claim conflicts with another structure. |
| `in_context` | A cell belongs to a local vocabulary, rule scope, review scope, or product context. |

Avoid flattening all relations into generic links. Relation type, provenance,
review status, and context are part of the product model.

## Step 5: Define Contexts

Use `Context` when meaning, validity, vocabulary, policy, ownership, or review
scope is local.

Examples:

| Context | Why it matters |
| --- | --- |
| Bounded context | A term such as `Customer` or `Order` may have different meaning across domains. |
| Module or package | A dependency may be acceptable inside a package and forbidden across package boundaries. |
| Test scope | A unit test, integration test, and smoke test can support different obligations. |
| Policy scope | Internal, external, regulated, or privileged material may have different allowed operations. |
| Review focus | A PR review may only make claims about files and dependencies in a bounded snapshot. |

Do not collapse context-specific terms into one global meaning without a
mapping. If an agent wants to treat two contextual meanings as the same, model
that as an `EquivalenceClaim`.

## Step 6: Model Transformations As Morphisms

Use `Morphism` for lifts, projections, migrations, translations, comparisons,
and other structure-preserving or structure-transforming maps.

Common product morphisms:

| Morphism | Meaning |
| --- | --- |
| `source -> structure` | Lift source material into cells, incidences, contexts, and provenance. |
| `domain -> core` | Map product vocabulary into HigherGraphen concepts. |
| `before -> after` | Compare a changed system state with an earlier one. |
| `requirement -> implementation` | Map an obligation to implementation cells. |
| `implementation -> test` | Map behavior to tests or coverage evidence. |
| `structure -> projection` | Produce a human, AI, audit, CLI, or external view. |
| `schema_v1 -> schema_v2` | Migrate an interpretation package or report schema. |

A morphism should record preserved structure, lost structure, distortion, and
composition constraints when those are meaningful. If the mapping is partial,
say so. A partial mapping is not automatically an obstruction unless an
invariant requires the missing part.

## Step 7: Define Invariants

Use `Invariant` for properties that must remain true inside a scope or across a
transformation.

Examples:

| Product | Invariant |
| --- | --- |
| Architecture review | A service must not directly read a database owned by another bounded context. |
| Test gap detection | A changed public function with a new branch should have unit-level evidence for that branch. |
| Case workflow | A case cannot close while blocking obstructions remain unreviewed. |
| Feed intelligence | A projection that summarizes conflicting claims must expose the conflict and source loss. |
| Contract review | A required clause must have either accepted coverage, an explicit exception, or an obstruction. |

An invariant is not only prose. It should be checkable enough to produce a
result, obstruction, or reviewable gap.

## Step 8: Emit Obstructions For Structured Failure

Use `Obstruction` when a condition cannot hold, a transformation cannot proceed
safely, a local structure cannot be glued into a global structure, or an
invariant fails.

An obstruction should answer:

- What invariant, constraint, morphism, or policy failed?
- Which cells and contexts are involved?
- What evidence or witness supports the failure?
- Is it a blocker, warning, ambiguity, contradiction, missing evidence, or
  unresolved review boundary?
- What completion candidates or review actions could resolve it?

Do not emit an obstruction for every missing field. Use it when the missing or
conflicting structure matters to a product rule.

## Step 9: Propose Completion Candidates

Use `CompletionCandidate` for missing structure that may resolve an
obstruction or improve the model.

Examples:

| Product | Completion candidate |
| --- | --- |
| Architecture review | Add a Billing API instead of direct database access. |
| Test gap detection | Add a unit test covering a changed error branch. |
| Case workflow | Add missing evidence before closing a case. |
| Feed intelligence | Add an official source, counterpoint, or duplicate link. |
| Contract review | Add a missing exception clause or reviewer decision. |

Completion candidates should carry rationale, source evidence, confidence, and
review status. They remain candidates until a review workflow accepts or
rejects them.

## Step 10: Preserve Justification

Use provenance, confidence, review status, evidence modules, and core extension
objects to keep reasoning inspectable.

| Need | Object or field |
| --- | --- |
| Where did this come from? | `Provenance`, `SourceRef` |
| How strong is the numeric support? | `Confidence` or evidence confidence records |
| Has a human or trusted workflow accepted it? | `ReviewStatus` |
| What supports this claim? | `Witness`, evidence cell, support incidence |
| How did premises lead to a conclusion? | `Derivation` |
| Is correlation being treated as causation? | `higher_graphen_evidence::causal` |
| Is a proof or solver result involved? | `higher_graphen_evidence::prover` |

Do not replace review with confidence. A highly confident unreviewed candidate
is still unreviewed.

## Step 11: Use Extension Objects Deliberately

HigherGraphen v0.3.0 exposes several core extension objects. Use them when the
product has the corresponding failure mode. Do not add them as decorative
metadata.

| Object | Use when | Do not use as |
| --- | --- | --- |
| `EquivalenceClaim` | The product wants to treat two cells or structures as the same under criteria, context, evidence, and loss. | A silent merge or final identity. |
| `Derivation` | The product must explain how premises, rules, and evidence produced a conclusion. | A generic comment field. |
| `Witness` | A claim, invariant violation, equivalence, or derivation needs observable support or counterexample material. | A raw dump of unrelated evidence. |
| `Scenario` | The product compares actual, proposed, reachable, counterfactual, or what-if worlds. | The accepted current state. |
| `Capability` | The product needs to know which actor or agent can perform an operation on a target. | An authorization bypass. |
| `Policy` | The product has permissions, prohibitions, review requirements, export rules, or safety constraints. | Informal prose detached from checks. |
| `Valuation` | The product compares alternatives by value, cost, risk, priority, or tradeoff. | A substitute for invariant satisfaction. |
| `SchemaMorphism` | A schema, ontology, interpretation package, or report contract changes and existing structure must be migrated or compared. | A normal data mapping with no compatibility concern. |

If an extension object has no validation check, projection, review workflow, or
agent operation in the product, keep it out of the initial integration.

## Step 12: Define Projections

Use `Projection` to produce views for a specific audience and purpose.

Common projection audiences:

- `human_review`: concise report for a reviewer or operator;
- `ai_view`: structured details for an agent to continue work;
- `audit_trace`: provenance, evidence, loss, and review state;
- `cli`: deterministic command output;
- `external_api`: stable integration payload.

Every projection should declare meaningful information loss, such as omitted
source text, summarized evidence, dropped unsupported fields, hidden internal
policy, or collapsed contexts.

## Implemented Surface In v0.3.0

Use this map when choosing crates.

| Crate | Mental model | Use for |
| --- | --- | --- |
| `higher-graphen-core` | Shared primitives and core extension records. | IDs, provenance, confidence, review status, source refs, extension objects such as `Scenario`, `Policy`, `Witness`, and `SchemaMorphism`. |
| `higher-graphen-structure` | Shape the target world. | `space`, `context`, `morphism`, and `topology` modules. |
| `higher-graphen-reasoning` | Judge whether structure is acceptable. | `invariant`, `obstruction`, `completion`, `model_checking`, and `abstract_interpretation` modules. |
| `higher-graphen-evidence` | Support, doubt, or bridge claims. | `confidence`, `causal`, and `prover` modules. |
| `higher-graphen-projection` | Define audience-specific views. | Projection definitions, selectors, results, and view metadata. |
| `higher-graphen-interpretation` | Give domain meaning to shared structure. | Domain packages, architecture interpretation, templates, and lift boundaries. |
| `higher-graphen-runtime` | Run workflows and emit reports. | Architecture, feed, PR review target, test gap, semantic proof, and completion review workflows. |

The conceptual primitives remain first-class. They are modules, records,
checks, and workflows; they are not necessarily one crate each.

## Integration Modes

Choose the integration mode before designing the product surface.

| Mode | Use when | Product responsibility |
| --- | --- | --- |
| Library embedding | The product is Rust code that owns its own workflow and storage. | Build spaces, cells, invariants, obstructions, completions, and projections directly from crates. |
| Runtime workflow | The product can consume a bounded HigherGraphen report. | Prepare stable input snapshots, call runtime or CLI workflows, validate report contracts, and render projections. |
| Agent integration | The product wants AI agents to use HigherGraphen correctly. | Provide skills, schemas, examples, safety rules, and commands that preserve review and provenance boundaries. |
| Intermediate tool | The product needs an operational tool such as `casegraphen`. | Define central objects, commands, schemas, invariant checks, obstruction outputs, completion rules, projections, and skill instructions. |

Do not start by creating a new `*graphen` tool. First prove that the product has
a central object, schema, invariant checks, obstruction outputs, completion
rules, projection templates, reference scenario, and agent skill protocol.

## Worked Walkthrough: Architecture Review

Input sentence:

```text
Order Service reads Billing DB directly.
```

Lift:

| Step | Structure |
| --- | --- |
| Source boundary | Architecture snapshot containing Order Service, Billing Service, Billing DB, and declared ownership/access facts. |
| Space | `space:architecture:direct-db-access-smoke` |
| Cells | `order_service`, `billing_service`, `billing_db`, `direct_db_access` |
| Contexts | `order_context`, `billing_context` |
| Incidences | `order_service accesses billing_db`, `billing_service owns billing_db` |
| Invariant | `no_cross_context_direct_database_access` |
| Check result | Violated because `order_service` crosses into the billing context by direct DB access. |
| Obstruction | `obstruction:order-service-direct-billing-db-access` |
| Completion candidate | Add or use a Billing Service API for billing status instead of direct DB reads. |
| Projection | Human architecture review report plus AI view and audit trace. |

Review boundary:

- The direct access fact may be accepted if it came from the source snapshot.
- The recommended API remains an unreviewed completion candidate until accepted.
- The human report must not hide the source boundary or projection loss.

## Worked Walkthrough: Test Gap Detection

Input:

```text
Git range changes a public function and adds a new error branch.
Coverage data shows no unit test exercises that branch.
```

Lift:

| Step | Structure |
| --- | --- |
| Source boundary | Git range, changed files, parsed symbols, branch metadata, supplied coverage, existing tests. |
| Space | `space:test-gap:<repo>:<base>..<head>` |
| Cells | changed file, public function, new error branch, existing tests, coverage evidence |
| Contexts | repository, module, unit-test scope, review focus |
| Morphisms | `before -> after`, `implementation -> test` |
| Invariant | Changed public branch should have unit-level evidence when the detector context requires it. |
| Obstruction | Missing unit-level evidence for the changed error branch. |
| Completion candidate | Add a unit test for the error branch. |
| Projection | Developer review report, AI view for candidate generation, audit trace with source loss. |

Review boundary:

- Coverage data is accepted only if supplied by the input snapshot.
- A proposed missing test is not a committed test and not approved coverage.
- Integration or smoke coverage may support confidence without satisfying a
  unit-scope invariant.

## Worked Walkthrough: CaseGraphen Workflow

Input:

```text
A case has tasks, evidence, blockers, review decisions, and a requested close
operation.
```

Lift:

| Step | Structure |
| --- | --- |
| Source boundary | Case space, revision, workflow graph, evidence records, review records. |
| Space | `space:case:<case-id>:<revision>` |
| Cells | case, task, evidence, decision, blocker, transition, close request |
| Contexts | workflow scope, review scope, evidence scope |
| Morphisms | revision transition, workflow transition, candidate-to-reviewed state |
| Invariants | A case cannot close while required evidence or blocking review decisions are missing. |
| Obstructions | Missing evidence, unresolved blocker, invalid transition, unreviewed completion. |
| Completion candidates | Add evidence, request review, resolve blocker, split task. |
| Projection | Case readiness report, AI workflow view, audit trace. |

Review boundary:

- An AI recommendation to close a case is not a close decision.
- A transition should preserve provenance and review status.
- A report can recommend actions without mutating case state.

## New Product Integration Checklist

Use this checklist before embedding HigherGraphen into a product.

### Source Boundary

- [ ] The bounded source snapshot is defined.
- [ ] Accepted facts are separated from AI inference.
- [ ] Excluded sources and information loss are recorded.
- [ ] Provenance exists for each accepted observation.

### Structural Lift

- [ ] A `Space` exists for the target world.
- [ ] Durable objects and observations lift to cells.
- [ ] Incidences preserve typed relationships.
- [ ] Contexts preserve local meaning, policy, ownership, and review scope.
- [ ] Morphisms record transformations, lifts, projections, and loss.

### Reasoning

- [ ] Product invariants are explicit and checkable.
- [ ] Failed checks produce obstructions with witnesses or evidence.
- [ ] Missing structure becomes completion candidates, not accepted facts.
- [ ] Confidence and review status remain separate.

### Integration Surface

- [ ] The crate or workflow entry point is chosen.
- [ ] The product knows whether it embeds libraries, consumes runtime reports,
  or exposes an agent skill.
- [ ] Projections are defined by audience and purpose.
- [ ] Projection loss is declared.
- [ ] Agent actions are governed by policy and capability rules when needed.

### Review

- [ ] AI-created objects start unreviewed.
- [ ] Accepted, rejected, superseded, and candidate states are explicit.
- [ ] Review workflows can promote or reject completion candidates.
- [ ] Audit projections preserve source, evidence, and decision trails.

## Decision Shortcut

When an AI agent is unsure which object to use, apply this shortcut:

| Question | Object |
| --- | --- |
| What world is under analysis? | `Space` |
| What durable things, observations, or relations exist? | `Cell` and incidence |
| Where does meaning or validity change? | `Context` |
| What maps, transforms, lifts, migrates, or projects? | `Morphism` |
| What must stay true? | `Invariant` |
| Why can this not proceed safely? | `Obstruction` |
| What missing structure could fix or complete it? | `CompletionCandidate` |
| What supports or refutes it? | `Witness`, evidence, provenance |
| How did premises lead to the conclusion? | `Derivation` |
| Is this only a proposed or hypothetical world? | `Scenario` |
| Who can do what? | `Capability` |
| What rule permits, forbids, or requires review? | `Policy` |
| Which alternative is preferable under tradeoffs? | `Valuation` |
| Can two structures be treated as the same under criteria? | `EquivalenceClaim` |
| Did the schema or interpretation itself change? | `SchemaMorphism` |
| Who is this view for and what did it lose? | `Projection` |

The correct product integration is usually not the one with the most object
types. It is the one that preserves the source boundary, review boundary, and
projection boundary while making the product's actual invariants inspectable.
