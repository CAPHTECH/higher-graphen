# evidencegraphen Contract

This document defines the implementable contract for `evidencegraphen`, the
claim and evidence centered intermediate tool in the primary HigherGraphen tool
family. It refines the `higher-graphen-evidence` row in
[`../intermediate-tools-map.md`](../intermediate-tools-map.md) without changing
core package responsibilities.

## Scope

`evidencegraphen` organizes claims, supporting evidence, counter-evidence,
provenance, confidence, contradiction handling, and auditability.

The tool must answer these questions:

- Which claims are being asserted?
- Which evidence supports, weakens, contradicts, or qualifies each claim?
- Which claims are unsupported, disputed, contradicted, accepted, or rejected?
- Where did every claim, evidence item, relation, and review action come from?
- Which facts are accepted, which are AI-inferred, and which still require
  review?
- What evidence and contradiction information must be preserved in human,
  AI-agent, and audit projections?

Out of scope for this contract:

- MCP server behavior.
- UI workflows.
- Provider SDK integration.
- General source ingestion. Bounded lift adapters may create evidence graphs,
  but the evidence contract starts after source material has become structured
  claims, evidence items, and relations.
- Silent promotion of AI-inferred claims or completion candidates into accepted
  fact.

## Conceptual Basis

`evidencegraphen` is based on a small set of concepts:

| Concept | Contract role |
| --- | --- |
| Argumentation graph | Claims and relations such as supports, weakens, rebuts, and undercuts. |
| Provenance graph | Every record carries source, extraction, confidence, and review state. |
| Defeasible reasoning | Support can be defeated by counter-evidence or contextual qualification. |
| Bayesian-inspired update | Confidence is updated by explicit support and counter-support, but it is not a substitute for review. |
| Proof object | A report should expose the path from claim to evidence, not only the conclusion. |
| Audit trace | Projections must list represented source IDs and declared information loss. |

The tool separates three ideas that must not collapse into each other:

- `Confidence` estimates extraction or inference reliability.
- `ReviewStatus` records explicit human or workflow review state.
- Evidence evaluation records whether the current graph supports or defeats a
  claim under declared thresholds.

## Package And CLI Surface

The intended package split is:

| Surface | Contract |
| --- | --- |
| Core evidence crate | `crates/higher-graphen-evidence/` |
| Rust crate name | `higher_graphen_evidence` |
| Intermediate tool package | `tools/evidencegraphen/` |
| CLI binary | `evidencegraphen` |
| Tool skill name | `evidencegraphen` |

The `higher-graphen-evidence` crate owns structural types and deterministic
evaluators:

```rust
pub struct EvidenceGraph;
pub struct Claim;
pub struct EvidenceItem;
pub struct EvidenceRelation;
pub struct Contradiction;
pub struct ClaimEvaluation;
pub struct EvidenceReport;

pub fn validate_graph(graph: &EvidenceGraph) -> EvidenceResult<()>;
pub fn evaluate_claims(graph: &EvidenceGraph, policy: EvaluationPolicy)
    -> EvidenceResult<Vec<ClaimEvaluation>>;
pub fn detect_contradictions(graph: &EvidenceGraph, policy: ContradictionPolicy)
    -> EvidenceResult<Vec<Contradiction>>;
pub fn build_report(graph: &EvidenceGraph, policy: EvaluationPolicy)
    -> EvidenceResult<EvidenceReport>;
```

The initial CLI surface should be file-based and deterministic:

```sh
evidencegraphen validate --input evidence.graph.json --format json
evidencegraphen report --input evidence.graph.json --format json
evidencegraphen unsupported --input evidence.graph.json --format json
evidencegraphen contradictions --input evidence.graph.json --format json
evidencegraphen audit --input evidence.graph.json --format json
evidencegraphen review accept-claim --input evidence.graph.json --claim <id> --reviewer <id> --reason <text> --format json
evidencegraphen review reject-claim --input evidence.graph.json --claim <id> --reviewer <id> --reason <text> --format json
evidencegraphen review resolve-contradiction --input evidence.graph.json --contradiction <id> --resolution <text> --reviewer <id> --format json
```

All CLI commands must accept `--format json`. Commands that emit reports should
also accept `--output <path>` once the first report schema exists.

Domain findings are successful command results. For example, unsupported claims
and unresolved contradictions should produce `ok` reports and exit `0`.
Malformed input, invalid primitive values, unreadable files, schema mismatches,
or output failures are tool failures.

## Core Dependencies

`higher-graphen-evidence` must reuse core primitives:

- `Id`
- `SourceRef`
- `Provenance`
- `Confidence`
- `Severity`
- `ReviewStatus`
- core-owned structured errors where primitive construction fails

The evidence crate may depend on `higher-graphen-space` only if the graph is
implemented as cells and incidences. It must not depend on runtime packages,
CLI packages, tools, apps, provider SDKs, or MCP packages.

The `evidencegraphen` tool may use:

- `higher-graphen-evidence` for graph validation and evaluation.
- `higher-graphen-projection` for human, AI-agent, and audit views.
- `higher-graphen-obstruction` when unresolved contradictions need to be
  exposed as obstruction-like failure objects.
- Runtime report envelope helpers only if the runtime package intentionally
  owns the shared report shape for this workflow.

## Input Contract

The initial graph input is a JSON document with this top-level shape:

```json
{
  "schema": "highergraphen.evidence.graph.v1",
  "graph_id": "evidence_graph:example",
  "space_id": "space:example",
  "claims": [],
  "evidence": [],
  "relations": [],
  "review_records": [],
  "metadata": {}
}
```

Required graph fields:

| Field | Contract |
| --- | --- |
| `schema` | Exact schema identifier for the graph format. |
| `graph_id` | Stable `Id` for this evidence graph. |
| `space_id` | Stable `Id` for the structural universe being evaluated. |
| `claims` | Claim records. |
| `evidence` | Evidence and counter-evidence records. |
| `relations` | Typed edges between claims, evidence items, and source structures. |
| `review_records` | Explicit review actions. Empty when no review has occurred. |
| `metadata` | Downstream-owned object; must not carry required semantics. |

### Claim

```json
{
  "id": "claim:order-service-reads-billing-db",
  "space_id": "space:architecture-product-smoke",
  "statement": "Order Service reads Billing DB.",
  "claim_type": "fact",
  "scope": {
    "cell_ids": ["cell:order-service", "cell:billing-db"],
    "context_ids": ["context:orders", "context:billing"],
    "morphism_ids": [],
    "time_range": null
  },
  "provenance": {
    "source": {"kind": "document"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`claim_type` values:

- `fact`
- `inference`
- `requirement`
- `risk`
- `recommendation`
- `decision`
- `custom:<extension>`

Claims are assertions, not proof. A claim may be present with no support, but
evaluation must report that state.

### Evidence Item

```json
{
  "id": "evidence:architecture-input-line-3",
  "space_id": "space:architecture-product-smoke",
  "evidence_type": "observation",
  "summary": "Input states that Order Service reads Billing DB.",
  "source_ids": ["source:architecture-input"],
  "payload_ref": null,
  "provenance": {
    "source": {"kind": "document", "source_local_id": "line:3"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`evidence_type` values:

- `observation`
- `document`
- `log`
- `api_response`
- `code_reference`
- `test_result`
- `human_review`
- `ai_inference`
- `counter_evidence`
- `custom:<extension>`

The payload may be omitted or stored by reference. If a payload is omitted, the
summary and provenance must still be sufficient for a reviewer to locate the
source.

### Evidence Relation

```json
{
  "id": "relation:evidence-line-3-supports-claim-read",
  "from_id": "evidence:architecture-input-line-3",
  "to_id": "claim:order-service-reads-billing-db",
  "relation_type": "supports",
  "strength": 1.0,
  "provenance": {
    "source": {"kind": "document"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`relation_type` values:

- `supports`
- `weakens`
- `contradicts`
- `rebuts`
- `undercuts`
- `qualifies`
- `derived_from`
- `same_as`
- `custom:<extension>`

`strength` is a `Confidence` value that weights the relation. It must not be
used as review acceptance.

## Output Contract

The initial report output should use a stable envelope:

```json
{
  "schema": "highergraphen.evidence.report.v1",
  "report_type": "evidence_report",
  "report_version": 1,
  "metadata": {},
  "scenario": {},
  "result": {},
  "projection": {}
}
```

`scenario` must preserve the input graph identifiers and enough counts to
identify the evaluated graph:

| Field | Contract |
| --- | --- |
| `graph_id` | Input evidence graph ID. |
| `space_id` | Input space ID. |
| `claim_count` | Number of claims evaluated. |
| `evidence_count` | Number of evidence items evaluated. |
| `relation_count` | Number of relations evaluated. |
| `policy` | Evaluation thresholds and contradiction policy used. |

`result` must include:

| Field | Contract |
| --- | --- |
| `status` | `evaluated`, `unsupported_claims_found`, `contradictions_found`, or `invalid_graph`. |
| `claim_evaluations` | Per-claim support, counter-support, confidence, review status, and evidence paths. |
| `unsupported_claim_ids` | Claims below support threshold. |
| `contradictions` | Unresolved and resolved contradiction records. |
| `accepted_claim_ids` | Claims explicitly accepted by review and still valid under evidence invariants. |
| `rejected_claim_ids` | Claims explicitly rejected by review. |

`projection` must include:

- `human_review`: concise claim status, contradictions, and review actions.
- `ai_view`: machine-friendly records with IDs, confidence, review status,
  provenance, and source IDs.
- `audit_trace`: represented source IDs, evidence paths, review records, and
  information-loss declarations.

The report must not omit counter-evidence or unresolved contradictions from the
AI and audit projections. A human summary may be concise, but it must disclose
that a contradiction or unsupported claim exists.

## Confidence Evaluation

The default evaluator should be deterministic and explainable.

For each relation connected to a claim:

```text
term = evidence.provenance.confidence * relation.strength
```

Supporting terms are combined with noisy-or:

```text
support_score = 1 - product(1 - support_term)
```

Counter terms from `weakens`, `contradicts`, `rebuts`, and `undercuts` are
combined the same way:

```text
counter_score = 1 - product(1 - counter_term)
```

The evaluator may expose `evaluated_confidence` for ranking:

```text
supported_upper_bound = prior_confidence + support_score * (1 - prior_confidence)
evaluated_confidence = supported_upper_bound * (1 - counter_score)
```

The separate `support_score` and `counter_score` remain contractual. Review
workflows and contradiction handling must not depend only on the composite
confidence value.

Default policy values:

| Policy field | Default |
| --- | --- |
| `support_threshold` | `0.7` |
| `counter_threshold` | `0.5` |
| `critical_contradiction_severity` | `high` |
| `accepted_claim_requires_review` | `true` |

## Contradiction Handling

Contradictions are first-class records, not logging side effects.

```json
{
  "id": "contradiction:claim-a-claim-b",
  "space_id": "space:example",
  "claim_ids": ["claim:a", "claim:b"],
  "evidence_ids": ["evidence:x", "evidence:y"],
  "relation_ids": ["relation:x-contradicts-a"],
  "severity": "high",
  "status": "unresolved",
  "explanation": "The two claims cannot both hold in the same scope.",
  "required_resolution": "Reject one claim, qualify one claim by context, or add resolving evidence.",
  "provenance": {
    "source": {"kind": "ai"},
    "confidence": 0.9,
    "review_status": "unreviewed"
  }
}
```

`status` values:

- `unresolved`
- `resolved`
- `superseded`
- `accepted_exception`

The MVP detector must handle explicit `contradicts`, `rebuts`, and `undercuts`
relations. Later detectors may add semantic or structural contradiction
detection, but they must emit the same contradiction record shape.

Resolution must be explicit. Valid resolution patterns include:

- Reject a claim.
- Reject or qualify an evidence item.
- Split a claim by context, scope, or time.
- Add supporting evidence that resolves an apparent contradiction.
- Mark an accepted exception with reviewer, reason, and audit record.

No evaluator may silently remove a contradiction because a confidence score is
lower than another score.

## Invariants

Validation must enforce these invariants:

- Every `id` is a valid core `Id` and is unique within its record type.
- Every relation endpoint exists in `claims`, `evidence`, or declared external
  `source_ids`.
- Every claim, evidence item, relation, contradiction, and review record carries
  provenance with `source`, `confidence`, and `review_status`.
- Confidence and relation strength values are finite and in `0.0..=1.0`.
- AI-inferred claims, evidence, and contradiction detections default to
  `review_status: "unreviewed"`.
- Rejected claims and rejected evidence must not increase support scores for
  accepted claims.
- A claim with `review_status: "accepted"` must meet the support threshold and
  must have no unresolved high or critical contradiction in the same scope.
- A claim with unresolved high or critical contradiction must not appear in
  `accepted_claim_ids`.
- Counter-evidence and contradiction records represented in the result must
  also appear in AI and audit projections.
- Projection information loss must declare any omitted claim payload, evidence
  payload, relation details, or source material.
- Review actions append `review_records`; they do not rewrite the original
  provenance or delete the original evidence path.
- Evaluation over the same input and policy is deterministic.

## Failure Modes

Tool failures:

| Failure | Required behavior |
| --- | --- |
| Malformed JSON or unknown schema | Return structured validation error. |
| Invalid core primitive | Surface stable primitive error code where available. |
| Dangling relation endpoint | Return graph validation error with relation ID and missing endpoint. |
| Missing required provenance | Return graph validation error with record ID and field name. |
| Invalid confidence or strength | Return validation error before evaluation. |
| Unknown review target | Return review error and do not emit an accepted or rejected record. |
| Output write failure | Return CLI failure; do not report a successful evaluation. |

Domain findings:

| Finding | Required behavior |
| --- | --- |
| Unsupported claim | Emit report with unsupported claim IDs. |
| Counter-evidence present | Emit claim evaluation with counter score and evidence path. |
| Explicit contradiction | Emit contradiction record. |
| Unresolved critical contradiction | Emit report and prevent accepted-claim status for affected claims. |
| Evidence projection loses detail | Declare information loss and keep source IDs. |

Unsupported claims and contradictions are not runtime failures. They are the
normal output of the evidence workflow.

## Validation Expectations

Implementation work for this contract should add focused checks:

- Serde round-trip tests for graph input and report output.
- Constructor and deserialization tests for invalid IDs, missing provenance,
  invalid confidence, invalid relation strength, and dangling relation
  endpoints.
- Evaluation tests for supported, unsupported, disputed, contradicted, accepted,
  and rejected claims.
- Tests proving rejected evidence does not increase support.
- Tests proving unresolved high or critical contradictions prevent a claim from
  appearing in `accepted_claim_ids`.
- Tests proving AI-inferred structures default to `unreviewed`.
- Golden JSON fixture validation for `highergraphen.evidence.graph.v1` and
  `highergraphen.evidence.report.v1`.
- CLI tests for `validate`, `report`, `unsupported`, `contradictions`, `audit`,
  and explicit review commands.
- Projection tests proving `human_review`, `ai_view`, and `audit_trace` include
  source IDs and information-loss declarations.
- Dependency tests proving lower crates do not depend on runtime, CLI, apps,
  provider SDKs, or MCP packages.

Expected command set once implemented:

```sh
cargo test -p higher-graphen-evidence
cargo test -p evidencegraphen
evidencegraphen validate --input fixtures/evidence.graph.json --format json
evidencegraphen report --input fixtures/evidence.graph.json --format json
```

## Relationship To Trace And Review Workflows

`evidencegraphen` is not the long-term owner of all temporal trace semantics.
A future `tracegraphen` may own event sourcing, temporal causality, and
cross-report lineage. Until then, evidence reports must expose audit anchors:

- stable IDs for every represented claim, evidence item, relation, source, and
  review record;
- `source_ids` in human, AI, and audit projections;
- `information_loss` entries for omitted evidence payloads or summarized paths;
- `audit_trace.traces` entries showing where each source ID appears.

Review workflows consume evidence reports but do not rewrite source evidence.
For claim review, `evidencegraphen review` emits a new review record and a new
report. For completion review, completion workflows may cite evidence graph
claim IDs and evidence paths, but accepting a completion candidate remains a
completion review action rather than an evidence action.

Architecture and agent workflows must preserve these rules:

- Accepted facts, AI inferences, reviewable candidates, and rejected claims stay
  separate.
- Evidence reports can justify or challenge a recommendation, but they do not
  promote the recommendation by themselves.
- Projections may summarize evidence for humans, but AI and audit views must
  keep enough IDs, provenance, confidence, and review status for downstream
  checking.

