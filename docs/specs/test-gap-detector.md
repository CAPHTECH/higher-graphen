# Test Gap Detector Contract

This document defines the pre-implementation contract for a HigherGraphen
missing unit test detector. It is a design contract for later runtime, CLI,
schema, and fixture work. It does not require Rust implementation changes.

The detector is not a raw coverage threshold tool. It treats missing unit tests
as structural gaps: a bounded source snapshot is lifted into HigherGraphen
cells, incidences, contexts, morphisms, invariants, obstructions, completion
candidates, evidence, and projections. AI-created gaps remain reviewable
suggestions until an explicit completion review or later workflow accepts or
rejects them.

The contract follows the existing report-first patterns used by the PR review
target recommender and Architecture Product reports:

- `ReportEnvelope` fields: `schema`, `report_type`, `report_version`,
  `metadata`, `scenario`, `result`, and `projection`.
- `ProjectionViewSet` fields: `human_review`, `ai_view`, and `audit_trace`.
- `Obstruction` records for structural reasons test coverage is incomplete.
- `CompletionCandidate` records with `candidate_type: "missing_test"`.
- `ReviewStatus::Unreviewed` for every AI-proposed missing test until an
  explicit review decision exists.

## Scope

The detector accepts a bounded test-gap snapshot, not an unbounded repository
crawl. A source adapter may produce the snapshot from local git, test metadata,
coverage output, requirements, or supplied workflow evidence, but the detector
may only treat records inside the snapshot as accepted input facts.

The contract covers:

- changed files and changed symbols;
- functions, methods, branches, conditions, and public behavior boundaries;
- existing tests and optional test-to-symbol or test-to-condition metadata;
- optional line, branch, function, or condition coverage evidence;
- requirements, bug-fix notes, review context, and declared test obligations;
- dependency edges, risk signals, static-analysis notes, and prior evidence;
- missing unit test obstructions and reviewable missing-test completion
  candidates;
- human, AI-agent, and audit projections with declared information loss.

The contract does not:

- scan the full repository and treat discovered facts as complete by default;
- infer accepted requirements, ownership, coverage, or test intent from source
  text without provenance;
- write tests, edit code, approve coverage, or mark candidates accepted;
- replace language-specific test runners, mutation testing, or code coverage
  tools;
- claim that no test gap exists outside the bounded snapshot.

## CLI Sketch

The initial CLI should align with the existing `highergraphen` style:

```sh
highergraphen test-gap input from-git \
  --base <ref> \
  --head <ref> \
  --format json \
  [--repo <path>] \
  [--coverage <path>] \
  [--requirements <path>] \
  [--output <path>]

highergraphen test-gap detect \
  --input <path> \
  --format json \
  [--output <path>]
```

`test-gap input from-git` deterministically converts a local git range plus
optional bounded artifacts into `highergraphen.test_gap.input.v1`. It may shell
out to local git and parse supplied coverage or requirement files, but it must
not use LLM inference, GitHub API payloads, or working-tree heuristics as
accepted facts.

`test-gap detect` reads the bounded input and emits
`highergraphen.test_gap.report.v1`. It may produce obstructions and completion
candidates, but every AI-created or rule-inferred missing-test candidate must
remain `review_status: "unreviewed"`.

When `--output` is omitted, commands write exactly one JSON report to stdout.
When `--output` is supplied, commands write exactly one JSON file and keep
stdout empty.

## Input Snapshot

Schema ID: `highergraphen.test_gap.input.v1`

Required fields:

| Field | Contract |
| --- | --- |
| `schema` | Must equal `highergraphen.test_gap.input.v1`. |
| `source` | SourceRef-like metadata for the bounded input document. |
| `repository` | Repository ID, name, and optional URI/default branch. |
| `change_set` | Base ref, head ref, commit IDs when available, and snapshot boundary. |
| `changed_files` | One or more changed-file records. |

Optional fields:

| Field | Contract |
| --- | --- |
| `symbols` | Changed or relevant symbols, functions, methods, types, modules, or public API elements. |
| `branches` | Branches, guards, conditions, pattern arms, error paths, or state transitions associated with symbols. |
| `requirements` | Requirement, bug-fix, issue, ADR, or specification records that should be verified by tests. |
| `tests` | Existing unit tests and optional integration, smoke, or property tests when relevant to the unit boundary. |
| `coverage` | Bounded coverage records copied from supplied tooling output. |
| `dependency_edges` | Directed or undirected relations between files, symbols, tests, requirements, or evidence. |
| `contexts` | Repository, module, package, test scope, domain, requirement, or review-focus contexts. |
| `evidence` | Diff hunks, coverage records, test results, static analysis output, mutation output, requirement links, notes, or custom evidence. |
| `signals` | Risk observations with severity and confidence. |
| `detector_context` | Required focus, excluded paths, test kinds to consider, and declared obligations supplied by the workflow. |

Accepted input facts are records supplied by the source adapter or input file.
They may be incomplete. The absence of a record is not evidence that the
repository has no such structure; it only means the bounded snapshot did not
provide it.

## Source Boundary

The source boundary is part of the scenario, not a footnote. The detector must
record:

- base and head refs, commit IDs when available, repository path or URI, and
  excluded paths;
- which adapters contributed accepted facts, such as `git_diff.v1`,
  `coverage_lcov.v1`, `requirements_json.v1`, `test_metadata.v1`, or
  `static_analysis.v1`;
- whether branches, symbols, requirement mappings, or test mappings were
  adapter-supplied facts or detector inference;
- which coverage dimensions are present: line, branch, function, condition,
  mutation, or none;
- information loss from summarizing diffs, omitting full source text, omitting
  full test bodies, or dropping unsupported coverage fields.

An adapter may add deterministic records for changed files, changed symbols,
existing tests, and coverage. It must not mark an inferred requirement-to-test
mapping as accepted unless the source artifact explicitly supplied that link.

## Structural Lift

Each test-gap snapshot lifts into one Space.

| Field | Contract |
| --- | --- |
| `id` | Stable ID such as `space:test-gap:<repository.id>:<change_set.id>`. |
| `name` | Human-readable test-gap scenario label. |
| `description` | Summary of the repository, change set, and source boundary. |
| `cell_ids` | Lifted file, symbol, function, branch, requirement, test, evidence, coverage, and risk-signal cells. |
| `incidence_ids` | Lifted containment, coverage, requirement, dependency, exercise, evidence, and support relations. |
| `context_ids` | Repository, module, package, test-scope, domain, requirement, and review-focus contexts. |

The Space is a structural view of the bounded snapshot. It is not a complete
repository graph.

### Cells

Accepted input records lift to cells with `review_status: "accepted"` when
they come from the source adapter or supplied workflow input. Acceptance means
the record may be used as an observed input fact. It does not mean test
coverage is sufficient.

| Input record | Cell type | Notes |
| --- | --- | --- |
| Changed file | `test_gap.changed_file` | Carries path, change type, language, additions, deletions, optional owner/context IDs. |
| Symbol | `test_gap.symbol` | Carries name, kind, file ID, path, visibility, optional line range, and public API marker. |
| Function or method | `test_gap.function` | May be a symbol subtype or separate cell when branch/test obligations attach at function granularity. |
| Branch or condition | `test_gap.branch` | Represents branch, guard, condition, pattern arm, error path, boundary condition, or state transition. |
| Requirement | `test_gap.requirement` | Represents requirement, bug fix, issue acceptance criterion, ADR constraint, or declared behavior. |
| Existing test | `test_gap.test` | Represents unit, property, integration, smoke, or unknown test. Unit tests are the primary target. |
| Coverage | `test_gap.coverage` | Represents line, branch, function, condition, or mutation coverage evidence from supplied tooling. |
| Evidence | `test_gap.evidence` | Represents diff hunks, test results, coverage artifacts, static analysis, requirement links, notes, or custom evidence. |
| Risk signal | `test_gap.risk_signal` | Represents supplied risk observations and preserves severity/confidence. |

AI-proposed missing tests, inferred obstructions, and inferred mappings are not
accepted cells. They remain result records with `review_status: "unreviewed"`
unless a later explicit review workflow accepts or rejects them.

### Incidences

Incidences connect lifted cells and preserve why the detector should consider
them together.

| Relation | From | To | Review status |
| --- | --- | --- | --- |
| `contains_symbol` | changed-file cell | symbol/function cell | Accepted when supplied by adapter symbol metadata. |
| `has_branch` | symbol/function cell | branch/condition cell | Accepted when supplied by parser/static analysis; unreviewed if inferred. |
| `implements_requirement` | symbol/function/file cell | requirement cell | Accepted only when supplied by requirement metadata. |
| `covered_by_test` | file/symbol/function/requirement/branch cell | test cell | Accepted when supplied by coverage or test metadata. |
| `exercises_condition` | test cell | branch/condition cell | Accepted when supplied by coverage/test metadata. |
| `depends_on` | file/symbol/function/test cell | file/symbol/function/test cell | Accepted when supplied by dependency analysis. |
| `supports` | evidence, coverage, or risk cell | file, symbol, branch, requirement, test, obstruction, or candidate ID | Accepted for supplied evidence; unreviewed when AI-created. |
| `in_context` | cell | context ID | Accepted when copied from input context membership. |

Coverage is represented as evidence plus incidences. A `covered_by_test`
incidence is not automatically a sufficient unit-test proof. Invariants decide
whether the represented coverage satisfies the declared obligation.

### Contexts

Contexts describe where the lifted cells are meaningful:

- `repository`
- `module`
- `package`
- `symbol_scope`
- `test_scope`
- `domain`
- `requirement_scope`
- `coverage_scope`
- `review_focus`
- `custom`

Cells may participate in multiple contexts. A function can belong to a module
context, a domain context, a public API context, and a unit-test scope at the
same time. The detector must not collapse these contexts when deciding whether
a test is sufficient. For example, an integration smoke test in a package
context may support audit confidence but may still fail the unit-scope
invariant for a public function.

## Morphisms

The detector models test sufficiency through morphisms and preservation rules.

| Morphism | Meaning | Required preservation | Loss signal |
| --- | --- | --- | --- |
| `requirement -> implementation` | Requirement or bug-fix behavior maps to files, symbols, functions, or branches. | Requirement identity, behavior summary, acceptance criterion, source IDs. | Requirement has no implementation target, target is inferred only, or mapping omits acceptance criteria. |
| `implementation -> test` | File, symbol, function, branch, or condition maps to existing tests and coverage. | Target ID, test ID, test kind, exercised condition when available, evidence IDs. | Public behavior, branch, boundary, error path, or side effect is not represented by a test. |
| `before -> after` | Changed structure maps pre-change behavior to post-change behavior. | Added/modified/removed target identity, changed conditions, changed requirement links. | New behavior has no new/updated test, removed behavior still has stale test mapping, or changed condition lost test coverage. |
| `candidate -> accepted test` | Missing-test completion candidate maps to an implemented/reviewed test. | Candidate ID, test ID, review decision, test result evidence, source IDs. | Candidate remains unreviewed, was rejected without replacement, or added test does not exercise the witness. |

Morphisms may be partial. The report must distinguish a partial morphism from a
failed invariant. A partial mapping becomes an obstruction only when an
invariant requires the missing structure.

## Invariants

The first detector should support these invariant templates:

| Invariant | Contract |
| --- | --- |
| Requirement verified | Every changed or explicitly in-scope requirement must map to at least one implementation target and at least one accepted verification method. Unit tests are preferred when the target is unit-testable. |
| Public behavior covered | Public functions, public methods, exported modules, and behavior-bearing public API changes should have accepted unit-test coverage or an explicit accepted waiver. |
| Boundary cases represented | Changed numeric, collection, enum, optional/null, empty, maximum, minimum, parsing, serialization, and state-boundary conditions should be represented by unit tests when those conditions are visible in the snapshot. |
| Error cases represented | Changed failure paths, validation errors, dependency failures, permission failures, parse errors, and exceptional outcomes should be represented by unit tests or accepted negative-test evidence. |
| Regression test for bug fix | A bug-fix requirement or issue-linked change should have a regression test that would fail before the fix and pass after the fix, or an accepted waiver explaining why not. |
| Projection declares information loss | Every projection view must state what structure was omitted, summarized, inferred, or not available in the bounded snapshot. |

Invariant evaluation must use the bounded snapshot only. If branch metadata or
coverage is absent, the detector may emit an obstruction such as
`insufficient_test_evidence`; it must not claim that branches are fully covered.

## Obstructions

Obstructions explain why the current structure cannot justify "unit testing is
sufficient" for the bounded target.

Required obstruction fields:

| Field | Contract |
| --- | --- |
| `id` | Stable obstruction ID within the report. |
| `obstruction_type` | One of the detector obstruction types. |
| `title` | Short human-readable label. |
| `target_ids` | File, symbol, function, branch, requirement, or condition IDs affected. |
| `witness` | Structured payload describing the missing or conflicting structure. |
| `invariant_ids` | Invariants that produced the obstruction. |
| `evidence_ids` | Accepted evidence or input records supporting the obstruction. |
| `severity` | `low`, `medium`, `high`, or `critical`. |
| `confidence` | Inference confidence from `0.0` to `1.0`. |
| `review_status` | Must be `unreviewed` for detector-inferred obstructions. |

Initial obstruction types:

| Type | Witness payload |
| --- | --- |
| `missing_requirement_verification` | `requirement_id`, optional `implementation_ids`, missing `test_ids`, requirement source, expected verification kind. |
| `missing_public_behavior_unit_test` | `symbol_id` or `function_id`, visibility, changed behavior summary, existing related tests, expected unit-test obligation. |
| `missing_branch_unit_test` | `branch_id`, parent symbol/function, condition summary, observed branch/coverage evidence, missing test relation. |
| `missing_boundary_case_unit_test` | target ID, boundary type, representative value or class, existing nearby tests, expected behavior. |
| `missing_error_case_unit_test` | target ID, error path, trigger condition, expected error behavior, existing nearby tests. |
| `missing_regression_test` | bug-fix requirement ID, changed implementation IDs, failing-before/passing-after expectation, related issue or evidence IDs. |
| `stale_or_mismatched_test_mapping` | test ID, mapped target ID, changed target identity, reason the mapping may no longer preserve behavior. |
| `insufficient_test_evidence` | target ID, missing evidence kind, source boundary reason, information needed before a stronger decision. |
| `projection_information_loss_missing` | projection ID, omitted required loss declaration, affected source IDs. |

Obstructions are not proof that a bug exists. They are proof that the bounded
structure does not satisfy a declared invariant.

## Completion Candidate

Missing unit tests are emitted as completion candidates.

Required candidate fields:

| Field | Contract |
| --- | --- |
| `id` | Stable candidate ID within the report. |
| `candidate_type` | Must be `missing_test`. |
| `missing_type` | Must be `unit_test` for the initial detector. |
| `target_ids` | Requirement, symbol, function, branch, condition, or evidence IDs the test should cover. |
| `obstruction_ids` | Obstructions this candidate would resolve if implemented and accepted. |
| `suggested_test_shape` | Test name, test kind, setup, inputs, expected behavior, assertions, and optional fixture notes. |
| `rationale` | Why this test closes the structural gap. |
| `provenance` | SourceRef-like records and evidence IDs used to infer the candidate. |
| `severity` | Impact if the missing test hides a real defect. |
| `confidence` | Inference confidence from `0.0` to `1.0`. |
| `review_status` | Must be `unreviewed` when emitted by `test-gap detect`. |

Example shape:

```json
{
  "id": "candidate:test-gap:calculate-discount-zero-boundary",
  "candidate_type": "missing_test",
  "missing_type": "unit_test",
  "target_ids": [
    "function:pricing:calculate_discount",
    "branch:pricing:calculate_discount:discount_percent_zero"
  ],
  "obstruction_ids": [
    "obstruction:missing-boundary-test:calculate-discount-zero"
  ],
  "suggested_test_shape": {
    "test_name": "returns_original_price_when_discount_percent_is_zero",
    "test_kind": "unit",
    "setup": "construct a price with a zero discount percent",
    "inputs": {
      "price": "nonzero positive price",
      "discount_percent": 0
    },
    "expected_behavior": "returns the original price without rounding drift",
    "assertions": [
      "discounted price equals original price"
    ]
  },
  "rationale": "The zero boundary is represented as a changed condition but no accepted unit test exercises it.",
  "provenance": {
    "source_ids": [
      "function:pricing:calculate_discount",
      "branch:pricing:calculate_discount:discount_percent_zero",
      "coverage:pricing:calculate_discount"
    ],
    "extraction_method": "test_gap_detect.v1"
  },
  "severity": "medium",
  "confidence": 0.84,
  "review_status": "unreviewed"
}
```

## Evidence, Provenance, And Review Boundary

The detector uses this boundary:

| Category | Examples | Review status |
| --- | --- | --- |
| Accepted input facts | Repository identity, change set, changed files, declared symbols, parser-supplied branches, supplied requirements, existing tests, supplied coverage records, dependency edges, evidence, risk signals. | `accepted` |
| Detector inference | Missing-test obstructions, inferred requirement/test mappings, inferred branch obligations, suggested test shapes, severity/confidence scoring. | `unreviewed` |
| Completion review | Explicit accept/reject decision for a candidate by a human or authorized workflow. | Separate review report or later workflow output. |
| Implemented test evidence | Test file/symbol added later, test runner result, coverage result, review decision. | Accepted only when supplied by the later source adapter or review workflow. |

Confidence never implies acceptance. Severity never implies confidence.
Projection summaries must not describe unreviewed candidates as accepted unit
test coverage. A candidate can become accepted only through an explicit review
or a later bounded snapshot that includes the implemented test and accepted
evidence linking it to the original candidate.

The detector should be compatible with the existing `highergraphen completion
review accept|reject` style. Candidate review emits a separate audit record; it
does not mutate the original `test-gap detect` report.

## Report Envelope

Schema ID: `highergraphen.test_gap.report.v1`

The report uses the existing runtime-style envelope:

```json
{
  "schema": "highergraphen.test_gap.report.v1",
  "report_type": "test_gap",
  "report_version": 1,
  "metadata": {},
  "scenario": {},
  "result": {},
  "projection": {}
}
```

`scenario` preserves the bounded input, source boundary, and lifted structure.
`result` carries machine-readable obstructions and missing-test candidates.
`projection` renders stable IDs into human, AI-agent, and audit views.

## Result Fields

| Field | Contract |
| --- | --- |
| `status` | `gaps_detected`, `no_gaps_in_snapshot`, or `unsupported_input`. |
| `accepted_fact_ids` | Lifted input fact IDs treated as accepted observations. |
| `evaluated_invariant_ids` | Invariants evaluated against the bounded snapshot. |
| `morphism_summaries` | Requirement-to-implementation, implementation-to-test, before-to-after, and candidate-to-accepted-test preservation/loss summaries. |
| `obstructions` | Structural reasons the snapshot does not prove sufficient unit testing. |
| `completion_candidates` | Missing-test candidates with `review_status: "unreviewed"`. |
| `source_ids` | IDs used to produce the result. Non-empty when obstructions or candidates are present. |

`no_gaps_in_snapshot` is a successful bounded result. It means the supplied
snapshot did not violate the configured invariants. It does not mean the entire
repository has complete tests.

## Projection Contract

The report projection follows the current `ProjectionViewSet` pattern:

| View | Contract |
| --- | --- |
| `human_review` | Summarizes missing unit test count, highest-severity gaps, and recommended review or implementation actions. It may group candidates by file, symbol, or requirement. |
| `ai_view` | Preserves stable cells, incidences, contexts, morphism summaries, invariants, obstructions, completion candidates, source IDs, confidence, severity, and review status for machine inspection. |
| `audit_trace` | Records source IDs, adapter roles, represented views, review boundary, and information loss. |

Every view must carry non-empty `source_ids` and at least one
`information_loss` entry. AI and audit views must preserve stable IDs for
represented records. Projection must not change `review_status`.

Required information-loss examples:

- raw source bodies were omitted;
- full diffs were summarized to changed files and symbols;
- coverage data was absent or only line-level;
- branch extraction was parser-supplied, inferred, or unavailable;
- integration tests were represented but unit-scope intent was unknown;
- candidate suggestions are unreviewed inference.

## Schema And Fixture Artifacts

Later implementation tasks should add:

- Input schema: `schemas/inputs/test-gap.input.schema.json`
- Input fixture: `schemas/inputs/test-gap.input.example.json`
- Report schema: `schemas/reports/test-gap.report.schema.json`
- Report fixture: `schemas/reports/test-gap.report.example.json`

The schemas should lock:

- the v1 input and report schema IDs;
- stable report envelope fields;
- source boundary and information-loss fields;
- accepted facts versus detector inference;
- obstruction witness payloads;
- missing-test completion candidate fields;
- projection view fields and source IDs;
- `review_status: "unreviewed"` for emitted detector suggestions.

## Verification Plan

Later implementation tasks should verify the detector with:

1. Fixtures covering a minimal gap, no gap in bounded snapshot, missing
   requirement verification, missing branch test, missing boundary test,
   missing error test, missing regression test, insufficient evidence, and
   projection information-loss failure.
2. JSON Schema validation for input and report examples, integrated with the
   existing contract validation script or an equivalent schema validation gate.
3. CLI tests for `highergraphen test-gap input from-git` and
   `highergraphen test-gap detect`, including stdout/output-file behavior.
4. Cargo tests for lift rules, morphism preservation/loss summaries, invariant
   evaluation, obstruction witness construction, completion candidate
   construction, and projection source-ID preservation.
5. Static analysis and formatting gates used by the workspace.
6. `cg history topology --case hg_missing_unit_test_detector --higher-order
   --format json` as a read-only diagnostic while implementing and verifying
   the case. Topology findings are evidence for review; they are not readiness
   blockers by themselves.

## Minimal Reference Scenario

Input:

```text
Function calculate_discount changed to handle discount_percent = 0.
Existing tests cover normal discount values.
Branch coverage reports the zero-discount condition as not exercised.
The change is linked to a bug-fix requirement.
```

Expected structure:

```text
Cells:
  calculate_discount function
  zero-discount branch
  bug-fix requirement
  existing normal-case test
  branch coverage evidence

Incidences:
  function has_branch zero-discount branch
  function implements_requirement bug-fix requirement
  existing test covered_by_test function
  coverage supports zero-discount branch

Invariant:
  Regression test for bug fix
  Boundary cases represented

Obstruction:
  Missing boundary/regression unit test for discount_percent = 0.

CompletionCandidate:
  Add a unit test asserting zero percent discount returns the original price.
```

Expected projection:

```text
Human review:
  One medium-severity missing unit test candidate for calculate_discount.

AI view:
  Stable target IDs, obstruction witness, suggested test shape, confidence,
  severity, source IDs, and review_status unreviewed.

Audit trace:
  Source boundary, represented IDs, and information loss from summarized source
  and branch-level coverage.
```
