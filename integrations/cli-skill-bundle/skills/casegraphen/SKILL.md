---
name: casegraphen
description: Use when an agent needs to create, inspect, reason over, review, patch, or validate CaseGraphen cases and workflow graphs through the installed `cg` workspace CLI and the repository-owned `casegraphen` CLI.
---

# CaseGraphen CLI Skill

Use this skill when a task asks for CaseGraphen workspace operation, workflow
reasoning, readiness, blockers, missing work, completion candidates, evidence
boundaries, projection loss, review workflows, patch workflows, or
validation-before-close.

This repository skill is CLI-only. MCP servers, provider SDK integrations,
provider marketplace metadata, and the external
`casegraphen reference workspace` repository are outside this surface.

## Source Of Truth

- Feature completion contract:
  `docs/specs/intermediate-tools/casegraphen-feature-completion-contract.md`
- Workflow contract: `docs/specs/intermediate-tools/casegraphen-workflow-contracts.md`
- Workflow graph schema: `schemas/casegraphen/workflow.graph.schema.json`
- Workflow report schema: `schemas/casegraphen/workflow.report.schema.json`
- Workflow graph example: `schemas/casegraphen/workflow.graph.example.json`
- Workflow report example: `schemas/casegraphen/workflow.report.example.json`
- Reference workflow: `examples/casegraphen/reference/workflow.graph.json`
- Native case management design:
  `docs/specs/intermediate-tools/casegraphen-native-case-management.md`
- Native case schema: `schemas/casegraphen/native.case.space.schema.json`
- Native case example: `schemas/casegraphen/native.case.space.example.json`
- Native reference examples: `examples/casegraphen/native/README.md`
- CLI implementation: `tools/casegraphen/src/cli.rs`
- Native CLI implementation: `tools/casegraphen/src/native_cli.rs`

Do not restate the schemas as competing contracts. Consume the schema, fixture,
and CLI output.

## Command Surfaces

There are two command surfaces. Pick the surface by ownership, not by command
name similarity.

### Installed `cg`

Use installed `cg` for the native `.casegraphen/` workspace:

- create or inspect cases: `cg init`, `cg case new`, `cg case list`,
  `cg case show`;
- edit native case graphs: `cg node add`, `cg node update`, `cg edge add`,
  `cg edge remove`;
- record native progress: `cg task start|done|wait|resume|cancel|fail`,
  `cg decision decide`, `cg event record`, `cg evidence add`;
- inspect actionable work and blockers: `cg frontier`, `cg blockers`;
- validate and diagnose the workspace: `cg validate --case <case_id>`,
  `cg validate storage`, `cg history topology`.

Installed `cg` owns append-only `.casegraphen` events and derived projections.
Do not hand-edit `.casegraphen/cases/<case_id>/events.jsonl`, cache files, or
locks. Run `cg evidence add` and `cg task done` only when the user or parent
task explicitly allows case mutation.

### Repo-Owned `casegraphen`

Use repo-owned `casegraphen` for strict file-based workflow graph reasoning and
the `cg`-compatible workflow bridge implemented in this repository. If the
binary is unavailable, run it through Cargo from the repository root:

```sh
cargo run -q -p casegraphen -- <args>
```

Repo-owned `casegraphen` reports use `schema`, `metadata`, `input`, `result`,
`projection`, and, for reasoning reports that can project HigherGraphen core
extension structure, `core_extensions` fields. They are not native installed-`cg`
events and do not replace `cg frontier`, `cg blockers`, or `cg validate --case`.

`core_extensions` is an explicit bridge to HigherGraphen core extension objects
such as `Witness`, `Derivation`, `Policy`, `Capability`, `Scenario`,
`SchemaMorphism`, `EquivalenceClaim`, and `Valuation`. Generated extensions are
report evidence, but supplied extensions can also gate decisions. If a workflow
graph, native case space, or native morphism has
`metadata.higher_graphen_extensions`, repo-owned `casegraphen` deserializes that
payload, validates the relevant core extension acceptance/decision contracts,
and reports failures in `core_extensions.validation`.

Decision-gate effects:

- `casegraphen workflow reason` sets `result.status` to `review_required` when
  supplied core extensions are blocked.
- `casegraphen case close-check` emits `result.core_extension_blocked`; when it
  is true, `result.close_check.closeable` is false.
- `casegraphen morphism check` keeps malformed-shape validation separate from
  policy/valuation gating: a structurally valid morphism can report
  `valid: true` and `applicable: false` when supplied core extensions are
  blocked.

### Native CaseGraphen

Use native repo-owned `casegraphen case ...` and `casegraphen morphism ...`
commands for CaseSpace plus MorphismLog work. Native CaseGraphen is not a
`cg` clone: task-like work is only one `CaseCell` type, readiness/frontier are
derived by replay, and accepted evidence or review state requires explicit
native records or morphisms.

Create or import a native case space:

```sh
casegraphen case new --store <dir> --case-space-id <id> --space-id <id> --title "<title>" --revision-id <revision_id> --format json
casegraphen case import --store <dir> --input native.case.space.json --revision-id <revision_id> --format json
casegraphen case list --store <dir> --format json
casegraphen case inspect --store <dir> --case-space-id <id> --format json
casegraphen case history --store <dir> --case-space-id <id> --format json
casegraphen case history topology --store <dir> --case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen case history topology diff --left-store <dir> --left-case-space-id <id> --right-store <dir> --right-case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen case replay --store <dir> --case-space-id <id> --format json
casegraphen case validate --store <dir> --case-space-id <id> --format json
```

Reason over replayed native state:

```sh
casegraphen case reason --store <dir> --case-space-id <id> --format json
casegraphen case frontier --store <dir> --case-space-id <id> --format json
casegraphen case obstructions --store <dir> --case-space-id <id> --format json
casegraphen case completions --store <dir> --case-space-id <id> --format json
casegraphen case evidence --store <dir> --case-space-id <id> --format json
casegraphen case project --store <dir> --case-space-id <id> --format json
casegraphen case close-check --store <dir> --case-space-id <id> --base-revision-id <revision_id> --validation-evidence-id <evidence_id> --format json
```

Propose, check, apply, or reject native morphisms:

```sh
casegraphen morphism propose --store <dir> --case-space-id <id> --input case_morphism.json --format json
casegraphen morphism check --store <dir> --case-space-id <id> --morphism-id <morphism_id> --format json
casegraphen morphism apply --store <dir> --case-space-id <id> --morphism-id <morphism_id> --base-revision-id <revision_id> --reviewer-id <reviewer_id> --reason "<reason>" --format json
casegraphen morphism reject --store <dir> --case-space-id <id> --morphism-id <morphism_id> --reviewer-id <reviewer_id> --reason "<reason>" --revision-id <revision_id> --format json
```

Native CLI limitations are part of the contract. Current morphism mutation is
metadata-only; unmaterialized payload changes are rejected instead of silently
rewriting a case space. There is `case close-check`, but no native `case close`
command yet. Document residual limitations when publishing examples or
operator reports.

Native case spaces and morphism proposal metadata can carry
`metadata.higher_graphen_extensions`. `case close-check` merges supplied
extensions from the case-space metadata with generated close-check extensions.
`morphism check` merges supplied extensions from the case-space metadata and the
candidate morphism metadata with generated scenario/schema/equivalence/valuation
extensions. Blocked supplied extensions are reported in
`core_extensions.validation`; they make close-check not closeable or morphism
check not applicable without treating the JSON shape itself as invalid.

## Native Case Workflow

For a native `.casegraphen` case:

1. Inspect the case and work state:

   ```sh
   cg case show --case <case_id> --format json
   cg frontier --case <case_id> --format json
   cg blockers --case <case_id> --format json
   ```

2. Create or edit the case only through installed `cg`:

   ```sh
   cg case new --id <case_id> --title "<title>"
   cg node add --case <case_id> --id <node_id> --kind task --title "<title>"
   cg edge add --case <case_id> --id <edge_id> --type depends_on --from <from_id> --to <to_id>
   ```

3. Record explicit state transitions only after the work happened:

   ```sh
   cg task start --case <case_id> <task_id>
   cg task wait --case <case_id> <task_id> --reason "<reason>" --for <event_id>
   cg task done --case <case_id> <task_id>
   ```

4. Record task evidence only when case mutation is allowed:

   ```sh
   cg evidence add --case <case_id> --id <evidence_id> --target <task_id> --title "<title>"
   ```

## Workflow Reasoning Commands

Run `casegraphen workflow reason` for the aggregate machine-readable workflow
reasoning report. The report includes top-level `core_extensions` when the
workflow can be projected into HigherGraphen core extension objects. Supplying
`metadata.higher_graphen_extensions` in the workflow graph makes those
extensions part of the decision gate; blocked supplied extensions force
`result.status` to `review_required`.

```sh
casegraphen workflow reason --input workflow.graph.json --format json
```

Cargo form:

```sh
cargo run -q -p casegraphen -- \
  workflow reason \
  --input schemas/casegraphen/workflow.graph.example.json \
  --format json
```

Use focused commands when the user asks for one section:

```sh
casegraphen workflow validate --input workflow.graph.json --format json
casegraphen workflow readiness --input workflow.graph.json --format json [--projection projection.json]
casegraphen workflow obstructions --input workflow.graph.json --format json
casegraphen workflow completions --input workflow.graph.json --format json
casegraphen workflow evidence --input workflow.graph.json --format json
casegraphen workflow history topology --input workflow.graph.json --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen workflow history topology diff --left left.workflow.json --right right.workflow.json --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen workflow project --input workflow.graph.json --projection projection.json --format json
casegraphen workflow correspond --left left.workflow.json --right right.workflow.json --format json
casegraphen workflow evolution --input workflow.graph.json --format json
```

Every report-producing command supports `--output <path>`. Domain findings such
as blocked work, missing proof, review-required completion candidates, failed
semantic validation, non-equivalent correspondence, and projection loss are
successful JSON report data unless the command itself fails.

There is no standalone `casegraphen workflow transition check` command in the
implemented CLI. Check reviewable graph transitions through
`casegraphen cg workflow patch check`.

## Workflow Store Bridge

Use `casegraphen cg workflow ...` when a workflow graph needs a durable
repo-owned store, history, replay, readiness over stored state, or explicit
review transitions:

```sh
casegraphen cg workflow import --store <dir> --input workflow.graph.json --revision-id <revision_id> --format json
casegraphen cg workflow list --store <dir> --format json
casegraphen cg workflow inspect --store <dir> --workflow-graph-id <id> --format json
casegraphen cg workflow history --store <dir> --workflow-graph-id <id> --format json
casegraphen cg workflow history topology --store <dir> --workflow-graph-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen cg workflow replay --store <dir> --workflow-graph-id <id> --format json
casegraphen cg workflow validate --store <dir> --workflow-graph-id <id> --format json
casegraphen cg workflow readiness --store <dir> --workflow-graph-id <id> --format json [--projection projection.json]
casegraphen cg workflow readiness --input workflow.graph.json --format json [--projection projection.json]
```

The bridge writes workflow graph snapshots and JSONL history through
`WorkflowWorkspaceStore` at the explicit `--store <dir>`. It does not append
native `.casegraphen` events.

## Topology Diagnostics

Use `--higher-order` when the task asks for persistent homology, higher-order
topology, or shape changes across history. Baseline topology reports omit
`higher_order`; higher-order reports include `filtration_source`,
`stage_sources`, compact `summary`, and raw `persistence`.

- File-based `history topology` and `workflow history topology` use
  `filtration_source: "deterministic_cell_order"` because the input is a
  snapshot without durable revision history.
- Store-backed `casegraphen cg workflow history topology` uses
  `filtration_source: "workflow_history"` and maps stages to workflow
  revisions.
- Native `casegraphen case history topology` uses
  `filtration_source: "native_morphism_log"` and maps stages to morphism log
  entries.
- `history topology diff`, `workflow history topology diff`, and native
  `case history topology diff` compare lifted topology summaries and
  source-mapping deltas. They are diagnostic comparison reports, not JSON
  patches and not blockers by themselves.

## Completion Review And Patch Flow

Completion candidates are proposed structure. They remain `unreviewed` until an
explicit bridge review records reviewer metadata, reason, revision, and optional
evidence or decision links.

Review a candidate by choosing one action command, such as `accept`, `reject`,
or `reopen`:

```sh
casegraphen cg workflow completion accept \
  --store <dir> \
  --workflow-graph-id <id> \
  --candidate-id <candidate_id> \
  --reviewer-id <reviewer_id> \
  --reason "<reason>" \
  --revision-id <revision_id> \
  --format json \
  [--evidence-id <evidence_id> ...] \
  [--decision-id <decision_id> ...]

casegraphen cg workflow completion reject \
  --store <dir> \
  --workflow-graph-id <id> \
  --candidate-id <candidate_id> \
  --reviewer-id <reviewer_id> \
  --reason "<reason>" \
  --revision-id <revision_id> \
  --format json

casegraphen cg workflow completion reopen \
  --store <dir> \
  --workflow-graph-id <id> \
  --candidate-id <candidate_id> \
  --reviewer-id <reviewer_id> \
  --reason "<reason>" \
  --revision-id <revision_id> \
  --format json
```

Convert an accepted completion candidate into a reviewable patch transition:

```sh
casegraphen cg workflow completion patch \
  --store <dir> \
  --workflow-graph-id <id> \
  --candidate-id <candidate_id> \
  --reviewer-id <reviewer_id> \
  --reason "<reason>" \
  --revision-id <revision_id> \
  --format json \
  [--transition-id <transition_id>]
```

Check the patch transition, then choose either the `apply` or `reject` review
command:

```sh
casegraphen cg workflow patch check \
  --store <dir> \
  --workflow-graph-id <id> \
  --transition-id <transition_id> \
  --format json

casegraphen cg workflow patch apply \
  --store <dir> \
  --workflow-graph-id <id> \
  --transition-id <transition_id> \
  --reviewer-id <reviewer_id> \
  --reason "<reason>" \
  --revision-id <revision_id> \
  --format json

casegraphen cg workflow patch reject \
  --store <dir> \
  --workflow-graph-id <id> \
  --transition-id <transition_id> \
  --reviewer-id <reviewer_id> \
  --reason "<reason>" \
  --revision-id <revision_id> \
  --format json
```

Patch review is bounded. It audits the transition record and records review
state in the workflow store; it does not silently materialize arbitrary
free-form payloads into full workflow records.

## Legacy Commands

Existing non-workflow commands keep their compatibility surface:

```sh
casegraphen version
casegraphen create --case-graph-id <id> --space-id <id> --store <dir> --format json
casegraphen inspect --input <case.graph.json> --format json
casegraphen list --store <dir> --format json
casegraphen validate --input <case.graph.json> --format json
casegraphen coverage --input <case.graph.json> --coverage <coverage.policy.json> --format json
casegraphen missing --input <case.graph.json> --coverage <coverage.policy.json> --format json
casegraphen conflicts --input <case.graph.json> --format json
casegraphen project --input <case.graph.json> --projection <projection.json> --format json
casegraphen compare --left <case.graph.json> --right <case.graph.json> --format json
casegraphen history topology --input <case.graph.json> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen history topology diff --left <left.case.graph.json> --right <right.case.graph.json> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
```

## Evidence And Projection Boundaries

- AI inference records do not become accepted evidence because they appear in a
  report.
- Source-backed or explicitly accepted evidence is the boundary for satisfying
  evidence and proof requirements.
- `casegraphen cg workflow completion accept --evidence-id ...` links evidence
  IDs in the workflow store; it does not create native `cg` evidence and does
  not promote unrelated inference records.
- Projection commands and projection views are read-only. They must keep
  `projection.information_loss`, omitted IDs, source IDs, confidence, severity,
  and review status visible.
- Do not use projection output to accept a completion, resolve a blocker, or
  satisfy a proof requirement.

## Validation Before Close

Before marking native work done, run the gates that match the task scope:

```sh
cg validate --case <case_id>
```

If `.casegraphen` state changed, or this is final release verification, also
run:

```sh
cg validate storage
```

For file-based workflow graphs, run:

```sh
casegraphen workflow validate --input workflow.graph.json --format json
```

For bridge stores, run:

```sh
casegraphen cg workflow validate --store <dir> --workflow-graph-id <id> --format json
casegraphen cg workflow history --store <dir> --workflow-graph-id <id> --format json
casegraphen cg workflow replay --store <dir> --workflow-graph-id <id> --format json
```

For native CaseSpace stores, run:

```sh
casegraphen case validate --store <dir> --case-space-id <id> --format json
casegraphen case history --store <dir> --case-space-id <id> --format json
casegraphen case replay --store <dir> --case-space-id <id> --format json
casegraphen case close-check --store <dir> --case-space-id <id> --base-revision-id <revision_id> --validation-evidence-id <evidence_id> --format json
```

After CLI or model changes, prefer:

```sh
cargo fmt --all --check
cargo test -p casegraphen --test command
cargo test -p casegraphen --lib
cargo check -p casegraphen
```

## Interpretation Rules

- Exit code `0` means the command emitted a structurally valid report.
- `result.status` values such as `blocked`, `obstructions_detected`, or
  `review_required` are successful domain findings, not failed CLI runs.
- Completion candidates are proposed structure. Keep
  `review_status: "unreviewed"` unless an explicit review workflow accepts or
  rejects them.
- AI inference records do not become accepted evidence because they appear in a
  report.
- Source-backed or accepted evidence is the boundary for satisfying evidence
  and proof requirements unless a future contract explicitly says otherwise.
- Keep `projection.information_loss`, source IDs, and audit records visible in
  summaries.
- Do not mutate input workflow graphs when interpreting reports.
- Do not treat `casegraphen cg workflow ...` history as native `.casegraphen`
  event history.
- Do not treat installed `cg` as the native CaseGraphen product model.
- Native `casegraphen case ...` reports derive readiness/frontier/blockers from
  replayed `CaseSpace` plus `MorphismLog`, not stored task state.

## Agent Output Shape

When reporting results to a user, include:

- The command that was run.
- Whether the command and contract validation passed.
- `result.status`, ready item IDs, and not-ready item IDs.
- Blocking obstructions with witness IDs.
- Completion candidates with confidence and review status.
- Evidence boundary findings, especially inference records that remain
  unaccepted.
- Projection loss or omitted IDs when relevant.
- Review actions taken, reviewer/revision IDs, and linked evidence or decision
  IDs when relevant.
- Validation commands run before close.
- Native residual limitations when relevant, especially metadata-only morphism
  application and absence of a native `case close` command.

## Safety Rules

- Do not edit `.casegraphen` files directly.
- Do not promote inferred records to evidence without an explicit review
  transition.
- Do not accept or reject completion candidates without an explicit review
  workflow.
- Do not apply a patch transition without checking it first.
- Do not hide projection loss in human, AI, or audit summaries.
- Do not introduce MCP, provider SDKs, or external repository dependencies for
  this CLI skill path.
- Do not change existing `highergraphen.case.graph.v1` command semantics when
  working on workflow reasoning.
- Do not document `casegraphen cg workflow ...` as the native CaseGraphen
  product surface; it is a compatibility bridge for workflow graphs.
