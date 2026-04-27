# PR Review Target Lift Model

This document defines how a bounded PR snapshot becomes HigherGraphen
structure for the PR review target recommender. It complements
[`pr-review-target-report-contract.md`](pr-review-target-report-contract.md)
and does not require runtime or CLI code changes.

## Scope

The lift starts from `highergraphen.pr_review_target.input.v1`, a
provider-neutral snapshot. The snapshot may include changed files, symbols,
owners, review contexts, tests, dependency edges, evidence, and risk signals.
It must not require raw Git hosting payloads or full diff contents.

The lifted structure is inspectable by AI agents as `Space`, `Cell`,
`Incidence`, `Context`, and `Provenance` records, then projected into the
human, AI-agent, and audit report views.

## Space

Each PR snapshot lifts into one Space:

| Field | Contract |
| --- | --- |
| `id` | Stable ID such as `space:pr-review-target:<pull_request.id>`. |
| `name` | Human-readable PR review target label. |
| `description` | Summary of the repository, PR, and snapshot boundary. |
| `cell_ids` | Lifted file, symbol, owner, test, evidence, and signal cells. |
| `incidence_ids` | Lifted containment, ownership, coverage, dependency, and evidence edges. |
| `context_ids` | Repository, PR, ownership, test, dependency, and review-focus contexts. |

The Space is a structural view of the bounded snapshot, not a complete
repository graph.

## Cells

Accepted input records lift to cells with `review_status: "accepted"` when they
come from the source adapter or supplied workflow input. Acceptance means the
record may be used as an observed input fact; it does not mean that the PR has
been reviewed or that a risk has been resolved.

| Input record | Cell type | Notes |
| --- | --- | --- |
| Changed file | `pr.changed_file` | Carries path, change type, language, additions, deletions, optional owner/context IDs. |
| Symbol | `pr.symbol` | Carries name, kind, file ID, path, and optional line range. |
| Owner | `pr.owner` | Represents a person, team, or system owner supplied by ownership metadata. |
| Test | `pr.test` | Represents a unit, integration, e2e, smoke, manual, or unknown test. |
| Evidence | `pr.evidence` | Represents diff hunks, static-analysis output, test results, coverage, ownership metadata, dependency scans, notes, or custom evidence. |
| Risk signal | `pr.risk_signal` | Represents supplied risk observations and preserves severity/confidence. |

AI-proposed review targets, obstructions, and completion candidates are not
accepted cells. They remain report result records with
`review_status: "unreviewed"` unless a later explicit review workflow promotes
or rejects them.

## Contexts

Contexts describe where the lifted cells are meaningful. Common context types
are:

- `repository`
- `pull_request`
- `review_focus`
- `ownership`
- `test_scope`
- `dependency_scope`
- `custom`

Cells may participate in multiple contexts. A changed file can be in the
repository, PR, ownership, and review-focus contexts at the same time.

## Incidences

Incidences connect lifted cells and preserve the reason an AI agent should
consider them together.

| Relation | From | To | Review status |
| --- | --- | --- | --- |
| `contains_symbol` | changed-file cell | symbol cell | Accepted when supplied by the snapshot. |
| `owned_by` | changed-file or symbol cell | owner cell | Accepted when supplied by ownership metadata. |
| `covered_by_test` | changed-file or symbol cell | test cell | Accepted when supplied by the snapshot or test metadata. |
| `depends_on` | changed-file or symbol cell | changed-file or symbol cell | Accepted when supplied by dependency analysis. |
| `supports` | evidence or signal cell | file, symbol, test, dependency, target, obstruction, or candidate ID | Accepted for supplied evidence; unreviewed when AI-created. |
| `in_context` | cell | context ID | Accepted when copied from input context membership. |

Dependency edges are incidences, not cells. If a dependency edge itself needs
to be inspected, the report can also create an unreviewed review target whose
`target_type` is `dependency`.

## Provenance

Every lifted cell and incidence carries provenance:

| Field | Contract |
| --- | --- |
| `source` | SourceRef-like object copied from the snapshot source with optional `source_local_id`. |
| `confidence` | Input confidence when supplied, otherwise the snapshot source confidence. |
| `review_status` | `accepted` for accepted input facts; `unreviewed` for AI inference. |
| `extraction_method` | `pr_review_target_lift.v1`. |

`confidence` expresses extraction or source confidence only. It never promotes
an inferred review target to an accepted fact.

## Accepted Facts vs AI Inference

The lift uses this boundary:

| Category | Examples | ReviewStatus |
| --- | --- | --- |
| Accepted input facts | Repository/PR identity, changed files, declared symbols, owners, contexts, tests, dependency edges, supplied evidence, supplied risk signals. | `accepted` |
| AI inference | Review targets, inferred dependency concerns, obstructions, completion candidates, inferred checklist structure, AI-authored evidence links. | `unreviewed` |

Accepted risk signals are accepted as observations only. A signal can support an
unreviewed obstruction or review target, but it does not make that obstruction
or target accepted.

## Projection Rules

The report may embed the lifted structure under `scenario.lifted_structure` and
must project stable IDs into:

- `human_review`, which summarizes review actions and intentionally loses raw
  structural detail;
- `ai_view`, which preserves cells, incidences, contexts, evidence IDs,
  confidence, and review status for machine inspection;
- `audit_trace`, which records which source IDs were represented in each view.

Projection must not change `review_status`. Human-facing summaries must not
describe unreviewed targets, obstructions, or candidates as accepted review
coverage.
