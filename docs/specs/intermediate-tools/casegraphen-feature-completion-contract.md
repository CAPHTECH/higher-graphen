# CaseGraphen Feature Completion Contract

Status: feature completion contract and gap inventory for case
`casegraphen-complete-feature-surface`, task
`task_casegraphen_feature_contract_gap_inventory`.

This document defines the completed CaseGraphen surface expected inside the
`higher-graphen` repository. It inventories the current `cg` workspace
commands, the repo-owned `casegraphen` CLI and workflow reasoning slice,
schemas, skills, examples, storage behavior, missing gaps, verification gates,
sequencing guidance, and out-of-scope boundaries.

The completed feature surface is CLI and skill only. MCP, provider SDK
integrations, provider marketplace publication, and direct changes to
`casegraphen reference workspace` remain out of scope.

## Source Material

This contract is grounded in:

- `docs/specs/intermediate-tools/casegraphen.md`
- `docs/specs/intermediate-tools/casegraphen-current-surface-inventory.md`
- `docs/specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`
- `docs/specs/intermediate-tools/casegraphen-workflow-contracts.md`
- `tools/casegraphen/`
- `schemas/casegraphen/`
- `examples/casegraphen/reference/`
- `examples/architecture/reference/`
- `skills/casegraphen/SKILL.md`
- `integrations/cli-skill-bundle/`
- `.casegraphen/cases/casegraphen-complete-feature-surface/`

## Current Surface

### `cg` Workspace Commands

The current workspace is managed through `cg 0.1.1`. The visible workspace
surface is:

| Command group | Current command surface | Current role |
| --- | --- | --- |
| Workspace setup | `cg init --title <title>` | Initializes a local `.casegraphen/` workspace. |
| Case lifecycle | `cg case new`, `cg case list`, `cg case show --case <id>` | Creates and reads event-sourced cases. |
| Manual graph editing | `cg node add`, `cg node update`, `cg edge add`, `cg edge remove` | Mutates task, decision, event, evidence, and goal graphs through the CLI. |
| State transitions | `cg task start|done|wait|resume|cancel|fail`, `cg decision decide`, `cg event record`, `cg evidence add` | Records workflow progress and evidence without hand-editing events. |
| Readiness inspection | `cg frontier --case <id>`, `cg blockers --case <id>` | Derives ready work and blocked work from the workspace graph. |
| Validation | `cg validate --case <id>`, `cg validate storage` | Validates one case or the workspace projections/storage. |
| History topology | `cg history topology --case <id> [--higher-order]` | Reports structural history and optional higher-order topology diagnostics. |
| Recovery | `cg cache rebuild` | Rebuilds projections when event history and cache diverge. |

`cg` owns the append-only `.casegraphen/cases/<case_id>/events.jsonl` event
stream and derived projections. Manual edits to `events.jsonl`, cache files, or
locks are not part of the completed surface.

### Repo-Owned `casegraphen` Package

The current package is `tools/casegraphen`, Rust package and binary
`casegraphen`. It is file-based and independent of the external CaseGraphen
repository.

| Surface | Current implementation |
| --- | --- |
| Entry point | `tools/casegraphen/src/main.rs` delegates to `casegraphen::cli::main_entry()`. |
| CLI parser | `tools/casegraphen/src/cli.rs` requires `--format json` and supports `--output` on every command. |
| Baseline model | `tools/casegraphen/src/model.rs` defines `CaseGraph`, `CaseRecord`, `Scenario`, `CoverageGoal`, `CaseRelation`, `ReviewRecord`, `CoveragePolicy`, `ProjectionDefinition`, `MissingCase`, and `ConflictingCase`. |
| Baseline evaluator | `tools/casegraphen/src/eval.rs` validates case graphs, evaluates coverage, detects missing cases, detects conflicts, compares graphs, and builds projection results. |
| Baseline reports | `tools/casegraphen/src/report.rs` emits `highergraphen.case.<operation>.report.v1` reports. |
| Store helpers | `tools/casegraphen/src/store.rs` reads strict JSON, checks schema identifiers and workflow schema version, writes reports, creates local graph files, and lists local graph files. |
| Workflow model | `tools/casegraphen/src/workflow_model.rs` defines `WorkflowCaseGraph`, work items, workflow relations, readiness rules, evidence records, transition records, projection profiles, correspondence records, and workflow provenance. |
| Workflow evaluator | `tools/casegraphen/src/workflow_eval.rs` and submodules derive readiness, obstructions, completion candidates, evidence findings, projection results, correspondence results, and evolution results. |
| Workflow report | `tools/casegraphen/src/workflow_report.rs` emits `highergraphen.case.workflow.report.v1` with human, AI, and audit projection views. |

Current non-workflow commands:

```sh
casegraphen create --case-graph-id <id> --space-id <id> --store <dir> --format json [--output <path>]
casegraphen inspect --input <case.graph.json> --format json [--output <path>]
casegraphen list --store <dir> --format json [--output <path>]
casegraphen validate --input <case.graph.json> --format json [--output <path>]
casegraphen coverage --input <case.graph.json> --coverage <coverage.policy.json> --format json [--output <path>]
casegraphen missing --input <case.graph.json> --coverage <coverage.policy.json> --format json [--output <path>]
casegraphen conflicts --input <case.graph.json> --format json [--output <path>]
casegraphen project --input <case.graph.json> --projection <projection.json> --format json [--output <path>]
casegraphen compare --left <case.graph.json> --right <case.graph.json> --format json [--output <path>]
```

Current workflow commands:

```sh
casegraphen workflow reason --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow validate --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow readiness --input <workflow.graph.json> --format json [--projection <projection.json>] [--output <path>]
casegraphen workflow obstructions --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow completions --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow evidence --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow project --input <workflow.graph.json> --projection <projection.json> --format json [--output <path>]
casegraphen workflow correspond --left <left.workflow.json> --right <right.workflow.json> --format json [--output <path>]
casegraphen workflow evolution --input <workflow.graph.json> --format json [--output <path>]
```

Current repo-owned `cg` compatibility bridge commands:

```sh
casegraphen cg workflow import --store <dir> --input <workflow.graph.json> --revision-id <id> --format json [--output <path>]
casegraphen cg workflow list --store <dir> --format json [--output <path>]
casegraphen cg workflow inspect --store <dir> --workflow-graph-id <id> --format json [--output <path>]
casegraphen cg workflow history --store <dir> --workflow-graph-id <id> --format json [--output <path>]
casegraphen cg workflow replay --store <dir> --workflow-graph-id <id> --format json [--output <path>]
casegraphen cg workflow validate --store <dir> --workflow-graph-id <id> --format json [--output <path>]
casegraphen cg workflow readiness (--input <workflow.graph.json> | --store <dir> --workflow-graph-id <id>) --format json [--projection <projection.json>] [--output <path>]
casegraphen cg workflow completion accept|reject|reopen --store <dir> --workflow-graph-id <id> --candidate-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--reviewed-at <text>] [--evidence-id <id> ...] [--decision-id <id> ...] [--output <path>]
casegraphen cg workflow completion patch --store <dir> --workflow-graph-id <id> --candidate-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--transition-id <id>] [--reviewed-at <text>] [--output <path>]
casegraphen cg workflow patch check --store <dir> --workflow-graph-id <id> --transition-id <id> --format json [--output <path>]
casegraphen cg workflow patch apply|reject --store <dir> --workflow-graph-id <id> --transition-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--reviewed-at <text>] [--output <path>]
```

This bridge is implemented in the repository-owned `casegraphen` package
because this repository cannot modify the installed external `cg` binary. It
uses `WorkflowWorkspaceStore` for durable workflow graph snapshots and JSONL
history. It does not append native `.casegraphen` events, does not replace
`cg case`, `cg frontier`, `cg blockers`, `cg validate`, or `cg history
topology`. Completion review and patch review commands mutate only the explicit
`--store <dir>` workflow store and keep AI-inferred completions separate until
an explicit review transition records the reviewer, reason, and linked evidence
or decisions.

Domain findings such as partial coverage, missing cases, conflicts, blocked
work, unresolved obstructions, missing proof, unreviewed completion candidates,
non-equivalent correspondence, and projection loss are successful report data.
Tool failures are malformed input, unsupported schema/version, invalid
primitive values, unreadable paths, unsupported arguments, output failures, or
serialization failures.

### Schemas And Examples

Current schema files:

| File | Contract |
| --- | --- |
| `schemas/casegraphen/case.graph.schema.json` | `highergraphen.case.graph.v1` input. |
| `schemas/casegraphen/coverage.policy.schema.json` | `highergraphen.case.coverage_policy.v1` input. |
| `schemas/casegraphen/projection.schema.json` | `highergraphen.case.projection.v1` input. |
| `schemas/casegraphen/case.report.schema.json` | Shared `highergraphen.case.*.report.v1` report envelope. |
| `schemas/casegraphen/workflow.graph.schema.json` | `highergraphen.case.workflow.graph.v1` input. |
| `schemas/casegraphen/workflow.report.schema.json` | `highergraphen.case.workflow.report.v1` output. |

Current examples:

| Path | Role |
| --- | --- |
| `schemas/casegraphen/*.example.json` | Minimal schema fixtures used by package tests. |
| `examples/casegraphen/reference/workflow.graph.json` | Deterministic workflow reasoning input. |
| `examples/casegraphen/reference/reports/workflow.reason.report.json` | Checked-in expected `workflow reason` report. |
| `examples/architecture/reference/casegraphen-reference.*.json` | Baseline architecture case graph, coverage policy, projection, and generated reports. |

### Skill And Bundle Surface

The repo now has a `casegraphen` skill surface:

| Surface | Current behavior |
| --- | --- |
| `skills/casegraphen/SKILL.md` | CLI-only agent workflow for installed `cg`, repo-owned `casegraphen workflow ...`, repo-owned `casegraphen cg workflow ...`, evidence boundaries, projection loss, review workflows, patch workflows, and validation-before-close. |
| `integrations/cli-skill-bundle/skills/casegraphen/SKILL.md` | Byte-for-byte bundled copy of the repo skill. |
| `integrations/cli-skill-bundle/bundle.json` | Declares CaseGraphen workflow command entrypoints, schema references, and bundle checks. |
| `integrations/cli-skill-bundle/references/cli-contract.md` | Provider-neutral contract reference for HigherGraphen CLI and CaseGraphen workflow reasoning. |

The skill path is intentionally CLI-only. It does not define a competing schema
and does not promote inferred records to accepted evidence.

## Target Completed Surface

The completed surface must let an AI operator manage, reason about, project,
patch, persist, and verify workflow cases end-to-end inside this repository.

Required completed capabilities:

1. Preserve all current `cg` workspace commands and `casegraphen` baseline
   commands.
2. Stabilize the workflow substrate APIs around strict JSON contracts,
   deterministic reasoning, structured errors, and reusable Rust modules.
3. Integrate workflow graph storage/history with the `.casegraphen` workspace
   model instead of relying only on standalone JSON files.
4. Expose a clear `cg`-compatible operator command path for creation,
   validation, reasoning, review, patch application, projection, and close-time
   verification.
5. Split the aggregate `casegraphen workflow reason` report into focused
   workflow commands.
6. Add explicit review and patch workflows for completion candidates and other
   suggested graph changes.
7. Keep skill/operator documentation synchronized with the implemented command
   path and schemas.
8. Prove the full feature surface through reference fixtures, schema checks,
   package tests, workspace validation, static analysis, bundle checks, and
   CaseGraphen case evidence.

## Feature Matrix

| Feature | Current status | Completion contract | Gap |
| --- | --- | --- | --- |
| `cg` case graph authoring | Present through `cg` workspace commands. | Keep CLI-only mutations for cases, nodes, edges, task states, decisions, events, and evidence. | Need bridge guidance so operators know when to use `cg` versus repo-owned `casegraphen`. |
| `cg` readiness | Present through `frontier` and `blockers`. | Preserve readiness for workspace task graphs and expose it in operator workflows. | Need completed bridge between `cg` readiness and workflow reasoning reports. |
| `cg` validation/history | Present through `validate`, storage validation, and history topology. | Keep validation before task close and use history/topology as diagnostics, not blockers by themselves. | Need final verification gate that combines workspace validation with package/report checks. |
| Baseline `casegraphen` graph commands | Present. | Keep `create`, `inspect`, `list`, `validate`, `coverage`, `missing`, `conflicts`, `project`, and `compare` stable. | No breaking changes allowed; add regression tests when workflow commands change shared code. |
| Workflow aggregate reasoning | Present as `casegraphen workflow reason`. | Keep as the umbrella command that emits full readiness, obstruction, completion, evidence, projection, correspondence, and evolution results. | Final E2E verification should confirm the aggregate report still matches the checked-in reference. |
| Workflow focused commands | Implemented as `workflow validate`, `readiness`, `obstructions`, `completions`, `evidence`, `project`, `correspond`, and `evolution`. | Keep focused commands read-only and derived from shared evaluator/report logic. | No standalone `casegraphen workflow transition check` exists; transition checks are bridge patch checks. |
| Workflow schemas | Present for graph, aggregate report, and operation-specific report identifiers emitted by focused commands. | Keep strict v1 schemas and document focused report sections. | Final E2E verification should cover schema/example expectations for focused reports. |
| Workflow package APIs | First slice present. | Expose stable model, validation, evaluator, report, store, and error APIs for later commands. | Need explicit `CaseResult` or equivalent error boundary and command-independent validation API. |
| Completion review | Implemented in the repo-owned bridge. | Keep accept/reject/reopen workflows with evidence and decision links, plus completion-to-patch conversion for reviewable transitions. | No separate `promote` command is implemented; accepted candidates and patch transitions remain explicit review records. |
| Patch/morphism workflow | Implemented in the repo-owned bridge for reviewable transition records. | Add reviewable patches that can be checked, applied, rejected, and audited. | Free-form patch payload materialization remains intentionally bounded. |
| Evidence boundary | Present in workflow records, focused evidence reports, bridge review guidance, and skills. | Enforce that accepted/source-backed evidence satisfies requirements while AI inference stays separate unless explicitly reviewed. | Final E2E verification should prove inference cannot silently satisfy evidence. |
| Projection loss | Present in workflow report views and focused project/readiness reports. | Every subset view and operator workflow must declare represented IDs, omitted IDs, and information loss. | Final E2E verification should check projection loss is visible in aggregate, focused, and bridge reports. |
| Correspondence | Present as aggregate and focused report records. | Distinguish equivalent, similar-with-loss, scenario-pattern match, conflicting, not-comparable, and transferable patterns. | Final E2E verification should include mismatch witness expectations and reference examples. |
| Evolution/history | Present as file-based transition records in workflow report. | Connect revision-indexed workflow reasoning to `.casegraphen` history and durable event replay. | Current local store is a file collection, not workspace history integration. |
| Skills | Repo skill and bundled copy are present and cover bridge, review, patch, focused commands, and validation-before-close. | Keep source and bundled skill synchronized with the implemented commands and safety rules. | Bundle smoke check enforces sync and key operator command terms. |
| Examples | Baseline and workflow references are present. | Cover normal operation, blocked work, missing evidence, missing proof, completion review, patch review, storage replay, projection loss, and `cg` bridge. | Current reference proves the aggregate reason command only. |
| Verification | Focused tests exist for current slice. | Full release gate covers docs, schemas, package tests, workspace validation, bundle checks, static analysis, and CaseGraphen evidence. | Need final release verification task after implementation tasks complete. |

## Package API Expectations

The completed `tools/casegraphen` package must expose command-independent APIs
that future commands can share instead of duplicating report logic.

Required compatibility APIs:

- `CaseGraph`, `CaseRecord`, `Scenario`, `CoverageGoal`, `CaseRelation`,
  `ReviewRecord`, `CoveragePolicy`, `ProjectionDefinition`, `MissingCase`, and
  `ConflictingCase` remain serializable and compatible with
  `highergraphen.case.graph.v1` inputs.
- `validate_case_graph`, `evaluate_coverage`, `detect_missing_cases`,
  `detect_conflicts`, `compare_graphs`, `projection_result`, and
  `graph_counts` remain command-independent.
- Existing report builders keep `highergraphen.case.<operation>.report.v1`
  schema identifiers and field meanings.
- `LocalCaseStore::create_graph`, `LocalCaseStore::list_graphs`,
  `read_case_graph`, `read_coverage_policy`, `read_projection`,
  `read_workflow_graph`, and `write_report` keep strict schema and version
  checks.

Required workflow APIs:

- `WorkflowCaseGraph` and all workflow record types remain strict serde
  contracts for `highergraphen.case.workflow.graph.v1`.
- `evaluate_workflow` remains the aggregate evaluator and must not depend on
  CLI parsing, runtime products, provider SDKs, MCP packages, or agent
  integration packages.
- Focused evaluators must be factored so `workflow reason` and the focused
  workflow commands share the same derivation code.
- Workflow validation must be available before reasoning and must distinguish
  invalid input from successful domain findings.
- Workflow report builders must preserve stable IDs, review status,
  confidence, severity, source IDs, inference boundaries, and projection loss.
- Error types must separate usage errors, I/O errors, JSON/schema errors,
  primitive construction errors, validation failures, and output write errors.

Implementation note for `task_casegraphen_core_substrate_hardening`
(2026-04-26): the first hardening slice exposes reusable
`validate_workflow_graph`, checked aggregate evaluation, and checked
section-level workflow evaluator helpers. It wires semantic workflow validation
into `read_workflow_graph` without adding new CLI commands; focused commands
remain assigned to later tasks and should reuse these package APIs.

Implementation note for `task_casegraphen_workspace_history_integration`
(2026-04-26): `tools/casegraphen/src/workflow_workspace.rs` adds a
provider-neutral, file-backed workflow workspace adapter. It stores validated
workflow graph snapshots under deterministic revision paths and appends JSONL
history entries without writing `.casegraphen` events. The later `cg` bridge
should mount this store under a workspace-local durable directory or translate
the same history entry contract into native `cg` events.

Implementation note for `task_casegraphen_cg_compatibility_bridge`
(2026-04-26): the compatibility bridge is additive and lives under
`casegraphen cg workflow ...`. The installed `cg` binary remains the durable
task backbone for cases, nodes, task states, evidence, frontier, blockers,
validation, and topology. The repo-owned bridge exposes workflow workspace
import/list/inspect/history/replay/validate and focused readiness reasoning
over either a file input or a stored workflow graph. The bridge writes only to
the explicit `--store <dir>` supplied by the operator.

Implementation note for `task_casegraphen_focused_workflow_commands`
(2026-04-26): `casegraphen workflow validate`, `readiness`, `obstructions`,
`completions`, `evidence`, `project`, `correspond`, and `evolution` are the
implemented focused file-based report commands. Patch or transition checks are
not file-to-file `workflow transition check` commands; they live in the
repo-owned bridge as `casegraphen cg workflow patch check`.

Implementation note for `task_casegraphen_skill_docs_operator_surface`
(2026-04-26): the source skill and bundled skill now document installed `cg`
versus repo-owned `casegraphen`, focused workflow commands, bridge workspace
commands, completion review, patch review/apply/reject, evidence and projection
boundaries, and validation-before-close. Bundle validation checks that these
operator command terms remain present.

Dependency direction expectations:

- Lower model/evaluator code may use `higher-graphen-core`,
  `higher-graphen-space`, and projection primitives where needed.
- Workflow-specific code must not depend on `higher-graphen-runtime`, product
  examples, skills, provider packages, MCP packages, or the external
  CaseGraphen repository.
- Any future use of obstruction, completion, evidence, invariant, morphism, or
  projection crates must keep the tool boundary domain-neutral and
  provider-neutral.

## CLI Command Expectations

### Must Preserve

The completed feature surface must preserve all current `casegraphen` baseline
commands and the current aggregate workflow command:

```sh
casegraphen create ...
casegraphen inspect ...
casegraphen list ...
casegraphen validate ...
casegraphen coverage ...
casegraphen missing ...
casegraphen conflicts ...
casegraphen project ...
casegraphen compare ...
casegraphen workflow reason --input <workflow.graph.json> --format json [--output <path>]
```

### Implemented Or Finalized Surface

The completed workflow command suite includes:

```sh
casegraphen workflow validate --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow readiness --input <workflow.graph.json> --format json [--projection <projection.json>] [--output <path>]
casegraphen workflow obstructions --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow completions --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow evidence --input <workflow.graph.json> --format json [--output <path>]
casegraphen workflow project --input <workflow.graph.json> --projection <projection.json> --format json [--output <path>]
casegraphen workflow correspond --left <left.workflow.json> --right <right.workflow.json> --format json [--output <path>]
casegraphen workflow evolution --input <workflow.graph.json> --format json [--output <path>]
```

There is no implemented standalone `casegraphen workflow transition check`
command. Reviewable graph transitions are produced and checked through
`casegraphen cg workflow completion patch` and
`casegraphen cg workflow patch check`.

The completed `cg`-compatible operator path must cover:

- installed `cg`: creating and editing native cases, state transitions,
  evidence records, frontier/blocker inspection, workspace validation, history
  topology, and final task evidence;
- repo-owned `casegraphen cg workflow ...`: importing workflow graphs into a
  workflow store, listing and inspecting stored workflow graphs, reading
  workflow history, replaying the current graph, validating stored workflow
  history, and running focused readiness reasoning over stored or file-based
  workflow graphs;
- existing `casegraphen workflow ...`: aggregate and focused file-based
  workflow reports for readiness, obstructions, completions, evidence,
  projection, correspondence, and evolution;
- repo-owned bridge review commands: accepting, rejecting, reopening, and
  converting completion candidates into reviewable patch transitions; checking,
  applying, or rejecting patch transitions against the workflow store history;
- source and bundled skills: operator guidance for all of the above, including
  evidence/projection boundaries and validation-before-close.

The exact command spelling can be implemented in `cg`, in `casegraphen`, or as
a documented bridge between them, but the operator must not have to guess which
surface is authoritative for each workflow step.

### Command Invariants

- Every command supports `--format json`.
- Every report-producing command supports `--output <path>`.
- Successful domain findings exit successfully and are represented in JSON.
- Tool failures are nonzero and explain usage, schema, I/O, primitive, or
  output errors.
- Reports preserve stable IDs and never hide review status or projection loss.
- A projection command must never promote evidence, accept a completion, or
  resolve an obstruction.

## Skill And Operator Workflows

The completed skill surface must give agents a repeatable operator workflow:

1. Inspect the case with `cg case show`, `cg frontier`, and `cg blockers`.
2. Read the relevant docs, schemas, examples, and skill source of truth.
3. Run `casegraphen workflow reason` or a focused workflow command when asked
   for readiness, blockers, missing work, evidence boundaries, projection
   loss, correspondence, or evolution.
4. Treat completion candidates and inferred records as review-required.
5. Use explicit review or patch commands before changing workflow structure.
6. Run the validation gates required by the task.
7. Record evidence with `cg evidence add` only when the task allows native case
   mutation.
8. Mark only the assigned task done with `cg task done` only when the parent or
   user assigned that closeout step.
9. Do not close the case unless the user explicitly assigned case closure.

Skill requirements:

- `skills/casegraphen/SKILL.md` remains the source skill.
- `integrations/cli-skill-bundle/skills/casegraphen/SKILL.md` remains
  byte-for-byte synchronized when the source skill changes.
- The skill must reference schemas, fixtures, and CLI output instead of
  restating them as a competing contract.
- The skill must keep MCP, provider SDKs, provider marketplace metadata, and
  the external `casegraphen reference workspace` repository out of scope.
- The skill must teach agents to expose command results, validation status,
  ready and not-ready IDs, blocking obstructions, completion review status,
  evidence boundary findings, and projection loss.

## Storage And History Expectations

Current storage is split:

- `cg` owns the event-sourced `.casegraphen/` workspace, append-only events,
  derived case projections, readiness, blockers, validation, and history
  topology.
- `casegraphen` owns strict file-based case graph and workflow graph inputs,
  local graph files under `LocalCaseStore`, and generated JSON reports.

The completed surface must integrate these without corrupting either boundary.

Required completed behavior:

- workflow graphs can be imported into or associated with a `.casegraphen`
  case;
- workflow graph updates are represented as durable events or replayable
  transition records;
- `cg` validation and `casegraphen` schema validation can both be run before
  close;
- storage validation detects dangling references, unsupported schema versions,
  invalid primitive IDs, stale projections, and incompatible history state;
- history can answer when blockers appeared, when proof/evidence was attached,
  when completions were accepted or rejected, which projections lost
  information, and which workflow shape persisted across revisions;
- cache/projection rebuilds remain explicit recovery operations, not silent
  rewrites of append-only history.

Missing storage/history gaps:

- no current command imports workflow graphs into `.casegraphen`;
- no current command updates workflow graphs through durable workspace events;
- reviewed completion candidates can be converted into workflow patch
  transitions in the repo-owned bridge, but arbitrary suggested payloads are
  not materialized as full workflow records;
- patch commands check, apply, and reject transition records against the
  workflow store history without appending native `.casegraphen` events;
- `LocalCaseStore` stores graph files but does not provide revision-indexed
  workflow replay.

## Remaining Verification And E2E Limitations

The implemented CLI and skill surface now covers validation, focused workflow
reports, file-backed workflow store import/history/replay, completion
accept/reject/reopen review, completion-to-patch conversion, patch
check/apply/reject review, evidence boundary guidance, projection loss
guidance, and validation-before-close guidance.

Remaining limitations for the final E2E verification task are:

1. The bridge workflow store is file-backed `WorkflowWorkspaceStore` history,
   not native `.casegraphen` events.
2. Reviewed completion candidates can be converted into workflow patch
   transitions, but arbitrary free-form patch payloads are not materialized into
   full workflow records.
3. The checked-in reference example proves aggregate workflow reasoning, while
   the command integration suite now exercises focused commands, store
   import/replay, completion accept/reject/reopen, completion patch conversion,
   patch check/apply/reject, explicit projection loss, invalid targets, and
   close-time validation coverage together.
4. Final release verification must still record the full gate results across
   docs, schemas, package APIs, CLI behavior, skill/bundle sync, workspace
   validation, static analysis, and higher-order topology smoke checks.

## Verification Gates

Every implementation slice must run the narrowest meaningful gates for its
blast radius. The final feature surface must pass all of these:

| Gate | Required when |
| --- | --- |
| `git diff --check` | Every task. |
| `cg validate --case casegraphen-complete-feature-surface` | Every task in this case. |
| `cg validate storage` | Any task that changes `.casegraphen` state or final release verification. |
| `cargo fmt --all --check` | Any Rust or generated schema/report code change. |
| `cargo check -p casegraphen` | Any `tools/casegraphen` API or command change. |
| `cargo test -p casegraphen --lib` | Any workflow model/evaluator/report/store change. |
| `cargo test -p casegraphen --test command` | Any CLI, schema, fixture, report, or command contract change. |
| `cargo test --workspace` | Shared API, dependency, or final release verification changes. |
| `sh scripts/static-analysis.sh` | Shared implementation changes and final release verification. |
| `python3 integrations/cli-skill-bundle/check-bundle.py` | Any skill or bundle metadata change. |
| Schema/example validation | Any schema or checked-in report fixture change. |
| `cg history topology --case casegraphen-complete-feature-surface --higher-order` | Final release diagnostic smoke or when topology/history behavior changes. |

Each completed case task must record evidence with `cg evidence add` targeting
that task before `cg task done`.

## Sequencing And Parallelization

Recommended task order:

1. Harden workflow substrate package APIs.
2. Integrate workspace storage and workflow history.
3. Expose the `cg`-compatible command bridge.
4. Implement individual reasoning commands in parallel where they share a
   stable evaluator API.
5. Implement completion review and patch workflows after storage/history and
   focused commands exist.
6. Finish skill and operator documentation after command names and review
   semantics are stable.
7. Add final reference scenarios and release verification last.

Parallelization rules:

- API hardening is a serial dependency for most later work.
- Readiness, obstructions, completions, evidence, projection,
  correspondence, and evolution command implementations can be split across
  subagents after the shared evaluator/report API is stable.
- Storage/history and command bridge work must coordinate on command names and
  mutation semantics.
- Skill docs should not be finalized until command behavior exists.
- Final verification must run after all feature slices are merged.
- Subagents must stay within their assigned write scopes and never revert
  concurrent edits.

## Explicit Out Of Scope

These items are not part of this feature completion case:

- building an MCP server;
- adding provider SDK integrations;
- publishing provider marketplace metadata;
- direct edits to `casegraphen reference workspace`;
- replacing the append-only `.casegraphen` event stream with manual edits;
- changing existing `highergraphen.case.graph.v1`,
  `highergraphen.case.coverage_policy.v1`, or
  `highergraphen.case.projection.v1` semantics without a version bump;
- treating blockers, conflicts, missing cases, partial readiness, completion
  candidates, projection loss, or non-equivalence as tool failures;
- promoting AI inference to accepted evidence through projection alone;
- accepting or rejecting completion candidates without an explicit review
  transition;
- making `tools/casegraphen` depend on runtime product packages, provider
  packages, MCP packages, or external repositories.

## Completion Definition

This case is complete only when the repository contains:

- stable package APIs for baseline and workflow CaseGraphen reasoning;
- preserved baseline `casegraphen` commands;
- aggregate and focused workflow commands with JSON reports;
- a clear `cg`-compatible operator path;
- durable workflow storage/history integration;
- explicit completion review and patch workflows;
- synchronized source and bundled skills;
- reference examples for the full operator workflow;
- validation gates recorded as case evidence;
- explicit out-of-scope boundaries still intact.

This task only defines the contract and gap inventory. It does not implement
Rust code and does not close the case.
