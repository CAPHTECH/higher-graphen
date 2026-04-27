# PR Review Target Report Contract

This document defines the first bounded contract for a PR review target
recommender. The contract is report-first: it describes the input shape and
stable JSON report that a future runtime or CLI command can emit without
changing Rust runtime code in this task.

The structural lift from a bounded PR snapshot into HigherGraphen Space, Cell,
Incidence, Context, Provenance, and ReviewStatus records is defined in
[`pr-review-target-lift-model.md`](pr-review-target-lift-model.md).

The contract reuses the same public patterns as the Architecture Product
reports:

- `ReportEnvelope` fields: `schema`, `report_type`, `report_version`,
  `metadata`, `scenario`, `result`, and `projection`.
- `ProjectionViewSet` fields: `human_review`, `ai_view`, and `audit_trace`.
- `Obstruction` records for review blockers or structural risks.
- `CompletionCandidate` records for missing review structure.
- `ReviewStatus::Unreviewed` for every AI-proposed review target until an
  explicit human review decision is recorded elsewhere.

## Scope

The recommender accepts a bounded PR summary, not raw Git hosting provider
payloads. Provider adapters may create this input later.

The contract covers:

- repository and pull request identity;
- changed files and optional symbols;
- optional risk signals from static analysis, dependency analysis, tests, or
  prior agent inspection;
- review target recommendations for files, symbols, tests, dependencies, or
  docs;
- obstructions that explain why review risk remains unresolved;
- completion candidates for missing review structure;
- human, AI-agent, and audit projections.

The contract does not approve PRs, post GitHub comments, assign reviewers, or
promote AI recommendations to accepted facts.

## Input Shape

Schema ID: `highergraphen.pr_review_target.input.v1`

Required fields:

| Field | Contract |
| --- | --- |
| `schema` | Must equal `highergraphen.pr_review_target.input.v1`. |
| `source` | SourceRef-like metadata for the bounded input document. |
| `repository` | Repository ID, name, and optional URI/default branch. |
| `pull_request` | PR ID, number, title, source branch, target branch, and optional URI/author. |
| `changed_files` | One or more changed-file records. |

Optional fields:

| Field | Contract |
| --- | --- |
| `symbols` | Symbols or modules associated with changed files. |
| `owners` | People, teams, or systems that own changed files or symbols. |
| `contexts` | Repository, PR, ownership, test, dependency, review-focus, or custom contexts. |
| `tests` | Tests associated with changed files or symbols. |
| `dependency_edges` | Directed or undirected dependency relations between files, symbols, tests, or owners. |
| `evidence` | Diff hunks, static analysis output, test results, coverage, ownership metadata, dependency scans, notes, or custom evidence. |
| `signals` | Precomputed risk signals with source IDs, severity, and confidence. |
| `reviewer_context` | Review focus, required expertise, and excluded paths supplied by a workflow. |

Changed files, symbols, owners, contexts, tests, dependency edges, evidence,
and risk signals are accepted input observations when they come from the source
adapter or supplied workflow input. Accepted observations are not final review
decisions.

## Report Envelope

Schema ID: `highergraphen.pr_review_target.report.v1`

The report uses the existing runtime-style envelope:

```json
{
  "schema": "highergraphen.pr_review_target.report.v1",
  "report_type": "pr_review_target",
  "report_version": 1,
  "metadata": {},
  "scenario": {},
  "result": {},
  "projection": {}
}
```

`scenario` preserves the bounded input and may include `lifted_structure`, the
Space/Cell/Incidence/Context form defined by the lift model. `result` carries
the machine-readable recommendations. `projection` renders the same IDs into
human, AI-agent, and audit views.

## Result Fields

| Field | Contract |
| --- | --- |
| `status` | `targets_recommended`, `no_targets`, or `unsupported_input`. |
| `accepted_change_ids` | Changed-file, symbol, and other lifted input fact IDs treated as accepted observations. |
| `review_targets` | AI-proposed targets. Every target must have `review_status: "unreviewed"`. |
| `obstructions` | Review risks or blockers represented as structured obstruction records. |
| `completion_candidates` | Missing review structure represented as unreviewed completion candidates. |
| `source_ids` | IDs used to produce the result. Non-empty when targets, obstructions, or candidates are present. |

`no_targets` is a successful domain result. Malformed input, invalid enum
values, invalid confidence, and schema validation failures are tool errors.

## Review Target Record

A review target is an AI recommendation for a human to inspect something. It
is not an accepted review result.

Required fields:

| Field | Contract |
| --- | --- |
| `id` | Stable target ID within the report. |
| `target_type` | `file`, `symbol`, `test`, `dependency`, `documentation`, or `cross_cutting`. |
| `target_ref` | The referenced file path, symbol ID, test ID, dependency ID, or topic. |
| `title` | Short human-readable target label. |
| `rationale` | Why the target deserves review. |
| `evidence_ids` | Source IDs supporting the recommendation. |
| `severity` | Core severity value: `low`, `medium`, `high`, or `critical`. |
| `confidence` | Inference confidence in the recommendation, from `0.0` to `1.0`. |
| `review_status` | Must be `unreviewed` for AI-proposed targets. |

Optional fields:

| Field | Contract |
| --- | --- |
| `location` | File path, line range, and optional symbol ID. |
| `suggested_questions` | Review prompts for the human reviewer. |
| `related_target_ids` | Other target IDs that should be reviewed together. |

## Severity And Confidence

Severity and confidence are intentionally separate:

- `severity` is impact if the target hides a real defect or missing review.
- `confidence` is model or rule confidence that the target is relevant.
- A high-severity target can have low confidence and still be useful as a
  human review prompt.
- Confidence never implies acceptance.
- Severity never implies confidence.

## Review Safety Rule

Every AI-proposed review target, completion candidate, inferred risk, and
projection record derived from those proposals remains `unreviewed` until a
separate explicit human review decision exists.

Projection views may summarize or prioritize targets, but they must not change
`review_status` and must not describe unreviewed targets as accepted review
coverage.

## Projection Contract

The report projection follows the current `ProjectionViewSet` pattern:

| View | Contract |
| --- | --- |
| `human_review` | Summarizes target count, major risks, and recommended human review actions. |
| `ai_view` | Preserves stable target, obstruction, completion candidate, lifted cell/incidence/context, file, and symbol records with IDs, severity, confidence, and review status. |
| `audit_trace` | Records source IDs, roles, represented views, and information loss. |

Every view must carry non-empty `source_ids` and at least one
`information_loss` entry. AI and audit views must preserve stable IDs for
represented records.

## Schema And Fixture Artifacts

- Input schema: `schemas/inputs/pr-review-target.input.schema.json`
- Input fixture: `schemas/inputs/pr-review-target.input.example.json`
- Report schema: `schemas/reports/pr-review-target.report.schema.json`
- Report fixture: `schemas/reports/pr-review-target.report.example.json`
