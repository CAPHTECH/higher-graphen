# casegraphen Workflow Contracts

Status: implementable model and report contract for case
`casegraphen-highergraphen-rewrite`, task
`task_casegraphen_reasoning_contract`.

This document defines the next implementation slice for the `casegraphen`
workflow reasoning engine inside the `higher-graphen` workspace. It is an
implementation-planning contract that pairs with the workflow JSON schemas in
`schemas/casegraphen/`. It describes model records, report records, CLI surface
expectations, compatibility rules, and validation expectations before Rust
implementation begins.

It extends these existing documents:

- [`casegraphen.md`](casegraphen.md)
- [`casegraphen-workflow-reasoning-engine.md`](casegraphen-workflow-reasoning-engine.md)
- [`casegraphen-current-surface-inventory.md`](casegraphen-current-surface-inventory.md)

The external `casegraphen reference workspace` repository is out of
scope. Implementation belongs in the `higher-graphen` repository, primarily
under future `tools/casegraphen/` and `schemas/casegraphen/` changes.

## Contract Goals

The workflow reasoning slice must make these concepts explicit:

- workflow case graph versioning and additive compatibility with
  `highergraphen.case.graph.v1`;
- work items and workflow relations;
- readiness rules and readiness reports;
- blockers as obstruction reports with witnesses;
- missing proof and missing task completions as generalized completion
  candidates;
- evidence records, evidence requirements, and the AI inference boundary;
- patch or transition records as morphisms between workflow spaces or
  revisions;
- projection definitions, projection profiles, and machine-readable projection
  loss;
- correspondence records for equivalent, similar, conflicting, and
  non-comparable workflows;
- evolution, revision, and transition trace records;
- CLI and report naming strategy that keeps existing commands compatible;
- validation and test expectations for the later implementation slice.

## Versioning Strategy

The existing v1 case graph contract remains valid and unchanged:

```text
highergraphen.case.graph.v1
highergraphen.case.coverage_policy.v1
highergraphen.case.projection.v1
highergraphen.case.<operation>.report.v1
```

The current v1 graph schema is strict. The workflow slice must not add
workflow-only fields to `highergraphen.case.graph.v1` records. New workflow
records use additive workflow schema identifiers and can reference an existing
case graph by ID.

The initial workflow graph schema identifier should be:

```text
highergraphen.case.workflow.graph.v1
```

The workflow graph is a sidecar or wrapper model, not a breaking replacement
for the baseline case graph. It may embed a baseline graph in a future schema,
but the first implementation should keep the contract simple:

```json
{
  "schema": "highergraphen.case.workflow.graph.v1",
  "workflow_graph_id": "workflow:casegraphen-rewrite",
  "case_graph_id": "case_graph:casegraphen-rewrite",
  "space_id": "space:casegraphen-rewrite",
  "base_schema": "highergraphen.case.graph.v1",
  "workflow_version": 1,
  "work_items": [],
  "workflow_relations": [],
  "readiness_rules": [],
  "evidence_requirements": [],
  "evidence_records": [],
  "completion_candidates": [],
  "transitions": [],
  "projection_profiles": [],
  "correspondences": [],
  "revisions": [],
  "transition_traces": [],
  "metadata": {}
}
```

Compatibility rules:

- Existing `casegraphen` commands and report shapes keep their current
  behavior.
- Existing v1 schemas remain strict and are not overloaded with workflow
  fields.
- New workflow reports use `highergraphen.case.workflow.<operation>.report.v1`.
- New workflow input files may reference current v1 case graphs by
  `case_graph_id`.
- Breaking changes to workflow records require a new schema suffix, such as
  `.v2`.
- Unknown workflow record fields are rejected in v1 unless a specific
  `metadata` object is documented as downstream-owned.
- Domain findings remain successful command results. Tool failures are limited
  to malformed input, unsupported schema versions, invalid primitive values,
  unreadable paths, unsupported options, or serialization failures.

## Shared Primitives

Workflow records should reuse the core primitives already required by the
baseline contract:

| Primitive | Use |
| --- | --- |
| `Id` | Stable identifiers for graph, item, relation, rule, evidence, candidate, transition, revision, and report records. |
| `SourceRef` | Source-backed evidence and traceability to external material. |
| `Provenance` | Origin, actor, confidence, and review information. |
| `Confidence` | Confidence in generated findings, candidates, evidence interpretation, and correspondences. |
| `Severity` | Resolution priority for obstructions, missing requirements, projection loss, and conflicts. |
| `ReviewStatus` | Review boundary for inferred or generated records. |

Record IDs must be stable across projections. A report may omit a record, but
it must not rewrite IDs for convenience.

## Workflow Case Graph

`WorkflowCaseGraph` is the bounded reasoning universe for workflow reasoning.
It links the baseline case graph to additional workflow records.

Required fields:

| Field | Contract |
| --- | --- |
| `schema` | Exact value `highergraphen.case.workflow.graph.v1`. |
| `workflow_graph_id` | Stable ID for the workflow graph. |
| `case_graph_id` | Baseline case graph ID being extended. |
| `space_id` | HigherGraphen space for the workflow. It should match the referenced case graph unless a later schema supports cross-space workflows. |
| `base_schema` | Baseline graph schema being extended, initially `highergraphen.case.graph.v1`. |
| `workflow_version` | Integer model version inside the workflow schema. Starts at `1`. |
| `work_items` | First-class work, proof, decision, wait, and review records. |
| `workflow_relations` | Typed workflow incidences between records and external structures. |
| `readiness_rules` | Deterministic rules used to derive readiness. |
| `evidence_requirements` | Proof obligations that must be satisfied by acceptable evidence. |
| `evidence_records` | Source-backed or explicitly labeled inference records. |
| `completion_candidates` | Reviewable proposed missing structure. |
| `transitions` | Patch, state-transition, review-transition, or morphism records. |
| `projection_profiles` | Named projection contracts for reports and skills. |
| `correspondences` | Structural matches or mismatches against other workflow graphs. |
| `revisions` | Optional revision records for file-based evolution. |
| `transition_traces` | Optional traces explaining changes across revisions. |
| `metadata` | Downstream-owned object with no required semantics. |

The model can be represented as JSON in the first implementation. It should
remain independent of provider SDKs, MCP servers, and domain product runtime
packages.

## WorkItem

`WorkItem` is the first-class workflow node. It generalizes task, goal,
decision, event, evidence, proof, external wait, and review action records.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable work item ID. |
| `space_id` | Owning HigherGraphen space. |
| `work_type` | Workflow role. |
| `state` | Current workflow state. |
| `title` | Human-readable title. |
| `summary` | Optional concise description. |
| `case_ids` | Related baseline case IDs. Empty when the item is not case-specific. |
| `scenario_ids` | Related scenario IDs. |
| `structure_ids` | Related cells, incidences, contexts, invariants, morphisms, or external stable structure IDs. |
| `evidence_requirement_ids` | Proof obligations attached directly to the item. |
| `source_ids` | Source IDs represented by the item. |
| `tags` | Search and projection tags. |
| `provenance` | Source, confidence, and review status. |
| `metadata` | Downstream-owned object with no required semantics. |

Allowed `work_type` values:

- `task`
- `goal`
- `decision`
- `event`
- `evidence`
- `proof`
- `test`
- `external_wait`
- `review_action`
- `checkpoint`
- `custom:<extension>`

Allowed `state` values:

- `todo`
- `doing`
- `waiting`
- `blocked`
- `done`
- `cancelled`
- `failed`
- `accepted`
- `rejected`
- `recorded`
- `custom:<extension>`

State is an input fact. Readiness is not. Readiness must be derived by
evaluating `ReadinessRule` records over `WorkItem`, `WorkflowRelation`,
`EvidenceRequirement`, and `EvidenceRecord` records.

## WorkflowRelation

`WorkflowRelation` is the first-class workflow incidence. It connects work
items, cases, scenarios, evidence, completions, projections, correspondences,
transitions, and external HigherGraphen structures.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable relation ID. |
| `relation_type` | Workflow relation semantics. |
| `from_id` | Source record or external structure ID. |
| `to_id` | Target record or external structure ID. |
| `required_for_readiness` | Boolean stating whether an unsatisfied relation can block readiness. |
| `strength` | `hard`, `soft`, or `diagnostic`. |
| `evidence_ids` | Evidence records supporting the relation. |
| `rule_ids` | Readiness or invariant rules that consume the relation. |
| `provenance` | Source, confidence, and review status. |
| `metadata` | Downstream-owned object with no required semantics. |

Allowed `relation_type` values:

- `depends_on`
- `waits_for`
- `requires_evidence`
- `satisfies_evidence_requirement`
- `blocks`
- `unblocks`
- `derives_from`
- `covers`
- `verifies`
- `invalidates`
- `contradicts`
- `refines`
- `projects_to`
- `transitions_to`
- `corresponds_to`
- `accepts`
- `rejects`
- `supersedes`
- `custom:<extension>`

Endpoint validation must accept records inside the workflow graph, records in
the referenced baseline case graph, and stable external HigherGraphen structure
IDs allowed by the current prefix policy. Dangling internal endpoints are
validation errors.

## ReadinessRule

`ReadinessRule` defines a deterministic condition for deciding whether a work
item is ready.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable rule ID. |
| `rule_type` | Deterministic readiness condition. |
| `applies_to` | Work item IDs or selectors by `work_type`, tag, state, or structure ID. |
| `required_relation_types` | Relation types considered by the rule. |
| `required_states` | Acceptable states for referenced work items when applicable. |
| `evidence_requirement_ids` | Evidence requirements consumed by the rule. |
| `severity_if_failed` | Severity of the obstruction emitted when the rule fails. |
| `rationale` | Human-readable reason for the rule. |
| `provenance` | Source, confidence, and review status. |
| `metadata` | Downstream-owned object with no required semantics. |

Allowed `rule_type` values:

- `all_dependencies_done`
- `external_wait_resolved`
- `evidence_requirement_satisfied`
- `no_unresolved_obstructions`
- `state_allows_work`
- `review_status_allows_work`
- `invariant_preserved`
- `no_blocking_contradiction`
- `custom:<extension>`

Rules are model records, not code snippets. The first implementation should
support a closed set of rule evaluators and reject unsupported `rule_type`
values unless they follow `custom:<extension>` and are treated as diagnostic
or unevaluated.

## ReadinessReport

`ReadinessReport` is a derived projection. It must not be serialized back into
the input graph as a stored ready flag.

Report schema identifier:

```text
highergraphen.case.workflow.readiness.report.v1
```

Report type:

```text
case_workflow_readiness
```

Required result fields:

| Field | Contract |
| --- | --- |
| `workflow_graph_id` | Evaluated workflow graph. |
| `case_graph_id` | Referenced baseline case graph. |
| `frontier_item_ids` | Ready items in stable order. |
| `blocked_item_ids` | Not-ready items with at least one blocking obstruction. |
| `ready_count` | Count of ready items. |
| `blocked_count` | Count of blocked items. |
| `item_results` | Per-item readiness results. |
| `obstructions` | Obstruction records generated or referenced by readiness evaluation. |
| `completion_candidates` | Missing proof, task, evidence, or review candidates emitted by evaluation. |
| `projection_loss` | Loss records for omitted workflow information. |

Per-item readiness result fields:

| Field | Contract |
| --- | --- |
| `work_item_id` | Evaluated item. |
| `state` | Input state at evaluation time. |
| `ready` | Derived boolean. |
| `rule_results` | Rule-by-rule pass, fail, skipped, or unevaluated results. |
| `hard_dependency_ids` | Dependencies that must be satisfied. |
| `unresolved_dependency_ids` | Hard dependencies not satisfied. |
| `external_wait_ids` | External wait records considered. |
| `unresolved_wait_ids` | External waits not resolved. |
| `evidence_requirement_ids` | Evidence requirements considered. |
| `unsatisfied_evidence_requirement_ids` | Evidence requirements not satisfied by acceptable evidence. |
| `obstruction_ids` | Blocking or diagnostic obstruction IDs. |
| `inference_record_ids` | Inferred records considered without treating them as accepted evidence. |

Domain findings such as no ready work, partial readiness, unresolved waits,
missing proof, and blocked work are successful report results.

## ObstructionReport And Witnesses

An obstruction is the workflow contract for blockers, contradictions, invalid
transitions, impossible closure, missing evidence, missing proof, and missing
task completions.

Report schema identifier:

```text
highergraphen.case.workflow.obstructions.report.v1
```

Report type:

```text
case_workflow_obstructions
```

`ObstructionRecord` required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable obstruction ID. |
| `obstruction_type` | Blocking condition. |
| `affected_ids` | Work items, cases, scenarios, structures, transitions, or projections affected. |
| `source_rule_id` | Readiness rule, invariant, or constraint that produced the obstruction. |
| `witness_ids` | Witness records proving or explaining the obstruction. |
| `evidence_ids` | Evidence records supporting the obstruction. |
| `completion_candidate_ids` | Candidate resolutions emitted for this obstruction. |
| `severity` | Resolution priority. |
| `explanation` | Concise explanation suitable for human review. |
| `required_resolution` | Known resolution path, or `null` when unknown. |
| `provenance` | Source, confidence, and review status. |

Allowed `obstruction_type` values:

- `unmet_dependency`
- `external_wait_unresolved`
- `missing_evidence`
- `missing_proof`
- `missing_task_completion`
- `failed_readiness_rule`
- `invalid_transition`
- `contradiction`
- `cycle`
- `invariant_violation`
- `impossible_closure`
- `projection_loss_exceeds_profile`
- `stale_evidence`
- `custom:<extension>`

`ObstructionWitness` required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable witness ID. |
| `witness_type` | Kind of witness. |
| `record_id` | Internal record ID when the witness points to a graph record. |
| `source_ref` | Source reference when the witness points outside the graph. |
| `path_ids` | Optional path or chain of relations that explains the witness. |
| `observed_value` | Observed state, relation, review status, evidence status, or projection loss. |
| `expected_value` | Expected value under the rule or constraint. |
| `explanation` | Human-readable witness explanation. |
| `provenance` | Source, confidence, and review status. |

Allowed `witness_type` values:

- `work_item_state`
- `workflow_relation`
- `readiness_rule_result`
- `evidence_record`
- `evidence_requirement`
- `completion_candidate`
- `transition`
- `projection_loss`
- `correspondence_mismatch`
- `source_ref`
- `external_structure`
- `custom:<extension>`

Witnesses must be precise enough for tests to assert why a blocker was
reported. An obstruction without a witness is invalid unless its
`obstruction_type` is a documented aggregate summary.

## CompletionCandidate

`CompletionCandidate` generalizes the existing `MissingCase` review boundary.
It represents proposed missing structure and must remain unaccepted until an
explicit review action accepts or rejects it.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable candidate ID. |
| `candidate_type` | Missing structure type. |
| `target_ids` | Work items, cases, scenarios, structures, rules, projections, or transitions affected. |
| `proposed_record_kind` | Kind of record to create or update if accepted. |
| `proposed_record` | Optional structured draft. May be omitted when only a prompt is available. |
| `blocking_obstruction_ids` | Obstructions this candidate could resolve. |
| `evidence_requirement_ids` | Evidence requirements this candidate relates to. |
| `rationale` | Why the candidate is needed. |
| `confidence` | Confidence in the proposal. |
| `severity` | Priority if not completed. |
| `source_ids` | Source IDs that motivated the candidate. |
| `provenance` | Source, confidence, and review status. |
| `review_status` | Must start as `unreviewed`. |
| `review_record_id` | Review action that accepted or rejected it, or `null`. |

Allowed `candidate_type` values:

- `missing_task`
- `missing_task_completion`
- `missing_evidence`
- `missing_proof`
- `missing_test`
- `missing_decision`
- `missing_dependency_relation`
- `missing_case`
- `missing_projection`
- `missing_review_action`
- `missing_transition`
- `custom:<extension>`

Missing proof and missing task completion are separate:

- `missing_proof` means the workflow lacks acceptable evidence that a claim,
  decision, invariant, or completion is true.
- `missing_task_completion` means a required work item has not reached an
  acceptable terminal state, or its terminal state lacks required evidence.

Projection output must not convert a completion candidate into a work item,
case, relation, or evidence record. Acceptance requires an explicit review
record or transition.

## EvidenceRecord And EvidenceRequirement

Evidence is separate from inference. A report may contain inferred findings,
but those findings do not become accepted evidence merely because they appear
in a report or projection.

### EvidenceRecord

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable evidence ID. |
| `evidence_type` | Evidence kind. |
| `evidence_origin` | `source_backed`, `inferred`, or `review_promoted`. |
| `source_ref` | Source reference for source-backed records. Required unless origin is `inferred`. |
| `content_hash` | Optional hash for command output, files, or source excerpts. |
| `summary` | Concise description of what the evidence says. |
| `supports_ids` | Claims, work items, relations, requirements, or transitions supported. |
| `contradicts_ids` | Claims, work items, relations, requirements, or transitions contradicted. |
| `captured_at` | Optional timestamp when evidence was captured. |
| `captured_by` | Optional actor or tool identifier. |
| `confidence` | Confidence in interpretation of the evidence. |
| `provenance` | Source, confidence, and review status. |
| `metadata` | Downstream-owned object with no required semantics. |

Allowed `evidence_type` values:

- `source_document`
- `test_result`
- `command_output`
- `review_decision`
- `human_attestation`
- `external_event`
- `schema_validation`
- `transition_trace`
- `inference_record`
- `custom:<extension>`

### EvidenceRequirement

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable requirement ID. |
| `requirement_type` | Proof obligation kind. |
| `target_ids` | Records this requirement must prove, verify, or review. |
| `required_evidence_types` | Acceptable evidence kinds. |
| `allowed_evidence_origins` | Acceptable origins. Defaults to `source_backed` and `review_promoted`. |
| `min_review_status` | Minimum review status needed to satisfy the requirement. |
| `satisfaction_mode` | `all`, `any`, or `threshold`. |
| `threshold` | Required count or ratio when `satisfaction_mode` is `threshold`. |
| `severity_if_missing` | Severity when not satisfied. |
| `rationale` | Why this evidence is required. |
| `provenance` | Source, confidence, and review status. |

Allowed `requirement_type` values:

- `proof`
- `test`
- `review`
- `source`
- `decision`
- `completion`
- `external_wait_resolution`
- `custom:<extension>`

### Inference Boundary

The default evidence rule is:

```text
EvidenceRecord.evidence_origin == "inferred" does not satisfy an
EvidenceRequirement unless a later explicit review transition promotes it or
the requirement explicitly allows inferred evidence for a non-binding
diagnostic.
```

Reports that include inferred records must expose an `inference_boundary`
section with:

- inferred record IDs;
- source-backed evidence IDs used by the inference;
- evidence requirements satisfied by accepted evidence;
- evidence requirements still unsatisfied;
- completion candidates proposed because evidence is missing;
- review transitions that promoted or rejected inferred records.

This boundary must appear in AI-agent and audit projections. A human projection
may summarize it, but it must not hide unresolved evidence requirements.

## Transition And Morphism Records

`TransitionRecord` represents a reviewable change between workflow states,
workflow graphs, revisions, or projections. It is the workflow-level morphism
record used for patches, state changes, review actions, schema migrations, and
structure-preserving transformations.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable transition ID. |
| `transition_type` | Change type. |
| `from_revision_id` | Source revision, or `null` for an initial transition. |
| `to_revision_id` | Target revision, or `null` until applied. |
| `from_workflow_graph_id` | Source workflow graph. |
| `to_workflow_graph_id` | Target workflow graph, if different. |
| `domain_ids` | Source records participating in the morphism. |
| `codomain_ids` | Target records participating in the morphism. |
| `mapping` | Source-to-target ID pairs. |
| `added_ids` | Records added by the transition. |
| `removed_ids` | Records removed by the transition. |
| `changed_ids` | Records changed by the transition. |
| `preserved_ids` | Records or structures declared preserved. |
| `lost_ids` | Records or structures declared lost. |
| `precondition_rule_ids` | Rules that must hold before the transition. |
| `postcondition_rule_ids` | Rules that must hold after the transition. |
| `preservation_checks` | Check results for invariants, evidence requirements, and projection loss. |
| `obstruction_ids` | Obstructions produced by invalid or blocked transitions. |
| `completion_candidate_ids` | Candidates produced by transition analysis. |
| `evidence_ids` | Evidence supporting the transition. |
| `provenance` | Source, confidence, and review status. |
| `review_status` | Review state of the transition itself. |

Allowed `transition_type` values:

- `patch`
- `state_transition`
- `review_transition`
- `projection_transition`
- `schema_migration`
- `merge`
- `split`
- `supersession`
- `custom:<extension>`

Transition validation should answer whether a proposed change preserves the
declared structure, loses information, violates an invariant, resolves an
obstruction, or creates new obstructions. A patch transition is not applied
merely because it is valid; application requires an explicit command or review
workflow in a later implementation slice.

## ProjectionDefinition, ProjectionProfile, And Loss

The baseline `highergraphen.case.projection.v1` contract remains valid for
current case projections. Workflow projections add named profiles and
machine-readable loss records.

### ProjectionProfile

`ProjectionProfile` defines a reusable view contract for a specific audience or
agent surface.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable profile ID. |
| `audience` | Intended consumer. |
| `include_record_types` | Record types included by default. |
| `omit_record_types` | Record types omitted by default. |
| `include_states` | Work item states included by default. |
| `include_evidence_detail` | `full`, `summary`, `ids_only`, or `none`. |
| `include_inference_records` | Whether inferred records appear in the projection. |
| `include_sources` | Whether source references appear in the projection. |
| `allowed_operations` | Operations an agent using this profile may perform or propose. |
| `loss_policy` | Required loss disclosure rules. |
| `max_allowed_loss_severity` | Highest allowed loss severity before the projection reports an obstruction. |
| `provenance` | Source, confidence, and review status. |

Allowed `audience` values:

- `human`
- `ai_agent`
- `audit`
- `system`
- `skill:<name>`
- `custom:<extension>`

### ProjectionDefinition

Workflow `ProjectionDefinition` is an operation request. It selects a profile
and may add operation-specific filters.

Required fields:

| Field | Contract |
| --- | --- |
| `schema` | Workflow projection request schema identifier when separated from the graph. |
| `profile_id` | Projection profile to apply. |
| `target_ids` | Optional item, case, scenario, revision, or transition IDs to project. |
| `filters` | Optional work type, state, tag, severity, or source filters. |
| `requested_by` | Optional actor or tool identifier. |
| `metadata` | Downstream-owned object with no required semantics. |

### ProjectionLoss

Every report that presents a subset or summary of workflow space must include
loss records.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable loss ID inside the report. |
| `loss_type` | Kind of omitted, redacted, summarized, or coarsened information. |
| `omitted_ids` | Record IDs omitted by the projection. |
| `omitted_record_types` | Record types omitted. |
| `source_ids` | Source IDs affected by the loss. |
| `severity` | Impact of the loss for the selected audience. |
| `rationale` | Why the loss occurred. |
| `recoverable_by` | Profile or command that can recover the omitted information, if known. |

Allowed `loss_type` values:

- `omitted_non_ready_work`
- `omitted_ready_work`
- `omitted_evidence_detail`
- `summarized_source`
- `redacted_source`
- `dropped_relation`
- `coarsened_state`
- `hidden_inference`
- `omitted_revision_history`
- `custom:<extension>`

Projection must not change record truth, review status, or evidence status. It
may hide or summarize records only when the loss is declared.

## CorrespondenceRecord

`CorrespondenceRecord` generalizes compare output from case-level diffs into
workflow-level structural correspondence.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable correspondence ID. |
| `correspondence_type` | Relationship between the compared structures. |
| `left_graph_id` | Left workflow or case graph. |
| `right_graph_id` | Right workflow or case graph. |
| `left_revision_id` | Optional left revision. |
| `right_revision_id` | Optional right revision. |
| `left_ids` | Records or structures on the left side. |
| `right_ids` | Records or structures on the right side. |
| `mapping` | Left-to-right ID pairs when a structural mapping exists. |
| `mismatch_witness_ids` | Witnesses showing mismatch, conflict, or loss. |
| `projection_loss_ids` | Loss records involved in the comparison. |
| `transferable_pattern` | Optional mitigation, completion, or scenario pattern that can transfer. |
| `transfer_constraints` | Conditions that must hold before transfer. |
| `confidence` | Confidence in the correspondence. |
| `provenance` | Source, confidence, and review status. |

Allowed `correspondence_type` values:

- `equivalent`
- `similar_with_loss`
- `scenario_pattern_match`
- `conflicting`
- `not_comparable`
- `transferable_completion_pattern`
- `transferable_mitigation_pattern`
- `custom:<extension>`

Similarity is not identity. Any `similar_with_loss` or transferable pattern
record must carry mismatch witnesses or projection loss records that explain
what does not match.

Report schema identifier:

```text
highergraphen.case.workflow.correspondence.report.v1
```

Report type:

```text
case_workflow_correspondence
```

## Evolution, Revision, And Transition Trace Records

The first workflow implementation can remain file-based, but the model must
support revision-indexed reasoning.

### RevisionRecord

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable revision ID. |
| `workflow_graph_id` | Workflow graph revised. |
| `parent_revision_ids` | Parent revisions. Empty for an initial revision. |
| `content_hash` | Optional hash of the workflow graph content. |
| `created_at` | Optional timestamp. |
| `created_by` | Optional actor or tool identifier. |
| `summary` | Concise revision summary. |
| `transition_ids` | Transitions included in the revision. |
| `provenance` | Source, confidence, and review status. |

### TransitionTraceRecord

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable trace ID. |
| `transition_id` | Transition being traced. |
| `from_revision_id` | Source revision. |
| `to_revision_id` | Target revision. |
| `added_ids` | Records first appearing in the target revision. |
| `removed_ids` | Records removed from the source revision. |
| `changed_ids` | Records changed between revisions. |
| `appeared_obstruction_ids` | Obstructions that first appeared. |
| `resolved_obstruction_ids` | Obstructions that disappeared or were resolved. |
| `attached_evidence_ids` | Evidence attached during the transition. |
| `accepted_completion_ids` | Completion candidates accepted. |
| `rejected_completion_ids` | Completion candidates rejected. |
| `projection_loss_ids` | Loss introduced or removed by the transition. |
| `correspondence_ids` | Correspondences that persisted or changed. |
| `provenance` | Source, confidence, and review status. |

Evolution reports should answer:

- when a blocker appeared;
- when proof was attached;
- when a completion candidate was accepted or rejected;
- which workflow shape persisted across revisions;
- which transition broke an invariant;
- which projection started omitting or revealing important information.

Report schema identifier:

```text
highergraphen.case.workflow.evolution.report.v1
```

Report type:

```text
case_workflow_evolution
```

## Report Envelope

Workflow reports should use the same broad envelope style as current
`casegraphen` reports:

```json
{
  "schema": "highergraphen.case.workflow.<operation>.report.v1",
  "report_type": "case_workflow_<operation>",
  "report_version": 1,
  "metadata": {
    "command": "casegraphen workflow <operation> ...",
    "tool_package": "tools/casegraphen"
  },
  "input": {},
  "result": {},
  "projection": {
    "profile_id": "projection:ai-agent",
    "projection_loss": []
  }
}
```

Additional report sections are allowed when documented by operation:

- `inference_boundary` for reports that include inferred records;
- `obstructions` for reports that explain blockers or invalid transitions;
- `completion_candidates` for reports that propose missing structure;
- `correspondences` for compare or transfer reports;
- `evolution` for revision-indexed reports.

AI and audit projections must preserve stable IDs, source IDs when available,
review status, evidence status, inference boundary, and projection loss.

## CLI Naming Strategy

Existing commands keep their current names and semantics:

```sh
casegraphen validate ...
casegraphen coverage ...
casegraphen missing ...
casegraphen conflicts ...
casegraphen project ...
casegraphen compare ...
```

Workflow commands should live under a `workflow` namespace to avoid overloading
current report contracts:

```sh
casegraphen workflow reason --input workflow.graph.json --format json [--output report.json]
casegraphen workflow validate --input workflow.graph.json --format json
casegraphen workflow readiness --input workflow.graph.json --format json [--projection projection.json] [--output report.json]
casegraphen workflow obstructions --input workflow.graph.json --format json [--output report.json]
casegraphen workflow completions --input workflow.graph.json --format json [--output report.json]
casegraphen workflow evidence --input workflow.graph.json --format json [--output report.json]
casegraphen workflow transition check --from before.workflow.json --to after.workflow.json --format json [--output report.json]
casegraphen workflow project --input workflow.graph.json --projection projection.json --format json [--output report.json]
casegraphen workflow correspond --left left.workflow.json --right right.workflow.json --format json [--output report.json]
casegraphen workflow evolution --input workflow.graph.json --format json [--output report.json]
```

The first implemented workflow CLI surface is:

```sh
casegraphen workflow reason --input <workflow.graph.json> --format json [--output <path>]
```

It reads `highergraphen.case.workflow.graph.v1` inputs with
`read_workflow_graph`, derives readiness, obstruction, completion, evidence
boundary, projection, correspondence, and evolution results, and emits the
`highergraphen.case.workflow.report.v1` report. The command is read-only:
domain findings such as blocked work, missing proof, unreviewed completion
candidates, or projection loss remain successful JSON report results and do not
mutate the workflow graph.

All workflow commands must support `--format json`. `--output` should write the
same JSON report that would otherwise be printed to stdout.

Exit behavior:

- `0` for successful reports, including blocked work, missing proof, missing
  task completion, unresolved obstructions, non-equivalent correspondence, and
  projection loss;
- nonzero only for tool failures such as invalid input, invalid schema,
  invalid primitive values, unreadable files, unsupported options, or output
  errors.

## Compatibility Constraints

- Do not edit or reinterpret `highergraphen.case.graph.v1` fields for workflow
  behavior.
- Do not change existing report schema identifiers or result field meanings.
- Do not treat missing cases, workflow completions, conflicts, blockers,
  partial readiness, or projection loss as CLI failures.
- Do not allow projection output to promote evidence, accept a completion, or
  resolve an obstruction.
- Do not allow inferred records to satisfy evidence requirements unless the
  requirement explicitly allows inferred diagnostics or a review transition
  promotes the inference.
- Do not make workflow model or evaluator code depend on runtime product
  packages, provider SDKs, MCP packages, or agent integration packages.
- Do not require a persistent event store for the first implementation. File
  inputs and explicit revision records are sufficient.
- Do not mutate the external `casegraphen reference workspace`
  repository as part of this rewrite.

## Validation Expectations

The later implementation should include validation before reasoning:

- exact schema identifier checks for workflow input and projection requests;
- stable non-empty IDs for every workflow record;
- same-space validation between workflow graph, baseline case graph, work
  items, rules, evidence requirements, and transitions unless a later schema
  explicitly supports cross-space references;
- endpoint checks for `WorkflowRelation` records;
- rule selector checks for `ReadinessRule` records;
- evidence requirement references from work items and rules;
- evidence origin checks for source-backed, inferred, and review-promoted
  records;
- completion candidate review status checks requiring generated candidates to
  start as `unreviewed`;
- transition mapping checks for duplicate, dangling, or contradictory mapping
  pairs;
- projection profile checks for required loss disclosure;
- correspondence checks requiring mismatch witnesses for similarity with loss,
  conflict, or non-comparability;
- revision and transition trace checks for parent revision references and
  transition references.

Validation should distinguish structural invalidity from domain findings. For
example, a cycle in hard dependencies may be a reported obstruction when the
input is otherwise well-formed, while a dangling internal relation endpoint is
invalid input.

## Test Expectations

The implementation slice should add tests before relying on the workflow
reasoning engine in product examples:

- serde or constructor tests for every record listed in this contract;
- JSON schema and example tests for the future workflow graph and workflow
  report schemas;
- backward compatibility tests proving existing v1 case graph examples and
  current commands still pass;
- CLI tests for `--format json`, `--output`, report schema identifiers, and
  tool failure exit behavior;
- readiness tests for satisfied dependencies, unresolved dependencies,
  external waits, missing evidence, blocked states, and no-ready-work results;
- obstruction tests proving every blocker has at least one useful witness;
- completion tests proving missing proof and missing task completion stay
  `review_status: "unreviewed"` until an explicit review transition;
- evidence boundary tests proving inferred records do not satisfy evidence
  requirements by default;
- transition tests proving preservation checks, loss checks, invalid
  transitions, and review transitions are reported correctly;
- projection tests proving every subset report declares projection loss and
  preserves AI/audit traceability;
- correspondence tests proving equivalent, similar-with-loss, conflicting,
  not-comparable, and transferable-pattern outputs are distinct;
- evolution tests proving blockers, evidence, accepted completions, rejected
  completions, and projection loss can be traced across revisions.

## First Implementation Slice Boundary

The first implementation after this documentation should prioritize:

1. workflow graph and record structs;
2. workflow graph validation;
3. readiness evaluation over dependencies, waits, states, and evidence
   requirements;
4. obstruction and witness generation for readiness failures;
5. generalized completion candidates for missing proof and missing task
   completion;
6. evidence origin checks and inference boundary reporting;
7. workflow report envelopes and CLI smoke tests.

Correspondence, projection profile tailoring, and evolution can be skeletal in
the first slice as long as their records are accepted, validated, and reported
with explicit unsupported or unevaluated statuses where appropriate.
