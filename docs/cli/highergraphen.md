# highergraphen CLI

The `highergraphen` command is the operational CLI for HigherGraphen runtime
workflows. It exposes the deterministic Architecture Product direct database
access smoke workflow, the bounded architecture input lift workflow, the
bounded Feed Product reader workflow, the bounded PR review target
recommendation workflow, and the explicit completion review workflow as stable
JSON reports.

For the underlying implementation contract, see
[`runtime-cli-scope.md`](../specs/runtime-cli-scope.md) and
[`runtime-workflow-contract.md`](../specs/runtime-workflow-contract.md). For
agent-specific packaging guidance, see
[`agent-tooling-handoff.md`](../specs/agent-tooling-handoff.md).

## Build or Run Locally

From the repository root, build the CLI with the workspace:

```sh
cargo build -p highergraphen-cli
```

After building, invoke the binary from `target/debug`:

```sh
./target/debug/highergraphen architecture smoke direct-db-access --format json
```

You can also run the package through Cargo:

```sh
cargo run -p highergraphen-cli -- architecture smoke direct-db-access --format json
```

## Commands

```sh
highergraphen architecture smoke direct-db-access --format json [--output <path>]
```

This command runs the Architecture Product direct database access smoke
workflow. The workflow is deterministic in the current implementation and does
not read external architecture files, databases, tickets, ADRs, or source code.

```sh
highergraphen architecture input lift --input <path> --format json [--output <path>]
```

This command reads a bounded architecture JSON v1 document and lifts accepted
component and relation facts into HigherGraphen cells and incidences. Inferred
structures from the input are preserved as unreviewed completion candidates and
are not promoted into accepted cells.

```sh
highergraphen feed reader run --input <path> --format json [--output <path>]
```

This command reads a bounded Feed Product JSON v1 fixture and emits a compact
RSS-reader-style analysis report. It lifts source feeds and entries into a
source-indexed observation space, preserves correspondence hints, emits
completion and obstruction candidates, and projects the result into timeline,
topic digest, and audit views. It does not fetch network feeds, parse raw RSS or
Atom XML, schedule refreshes, persist read state, or render a UI.

```sh
highergraphen pr-review input from-git --base <ref> --head <ref> --format json [--repo <path>] [--output <path>]
```

This command deterministically converts a local git commit range into a bounded
`highergraphen.pr_review_target.input.v1` snapshot. It shells out to local git
for changed files, numstat, and commit summaries, then applies fixed path rules
for owners, contexts, tests, dependency edges, evidence, and risk signals. It
also maps Rust boundary, incidence, and composition diff observations through
`higher-graphen-space` structural analysis so parent-module wiring changes can
be reviewed as deterministic dependency risks. It does not use LLM inference,
GitHub API payloads, or working-tree heuristics. Use the generated input with
`highergraphen pr-review targets recommend`.

```sh
highergraphen pr-review targets recommend --input <path> --format json [--output <path>]
```

This command reads a bounded PR review target JSON v1 snapshot and emits a
review-targeting report. The checked-in fixture is
`schemas/inputs/pr-review-target.input.example.json`, and compatible inputs
must use `schema: "highergraphen.pr_review_target.input.v1"`. The workflow
lifts supplied PR change facts as accepted input observations, then presents
AI-created review targets, obstructions, and completion candidates as
suggestions with `review_status: "unreviewed"`. It does not approve PRs,
record review decisions, post provider comments, or promote recommendations
into accepted review coverage. Humans should inspect the recommended targets
and record explicit accept/reject/waive decisions in a separate review system
or later explicit workflow.

```sh
highergraphen completion review accept \
  --input <path> \
  --candidate <id> \
  --reviewer <id> \
  --reason <text> \
  --format json \
  [--reviewed-at <text>] \
  [--output <path>]

highergraphen completion review reject \
  --input <path> \
  --candidate <id> \
  --reviewer <id> \
  --reason <text> \
  --format json \
  [--reviewed-at <text>] \
  [--output <path>]
```

These commands read a workflow report containing `result.completion_candidates`
or a review snapshot containing `source_report` and `completion_candidates`.
They emit a separate completion review report with the source candidate
snapshot, reviewer request, and accepted or rejected outcome. They do not edit
the source report and do not promote the candidate into accepted facts.

## Options

| Option | Required | Description |
| --- | --- | --- |
| `--format json` | Yes | Emits the stable JSON report. No human text format is supported yet. |
| `--input <path>` | For `architecture input lift` | Reads the bounded architecture JSON input document. |
| `--input <path>` | For `feed reader run` | Reads the bounded Feed Product JSON input fixture. |
| `--base <ref>` | For `pr-review input from-git` | Git base ref for the deterministic diff range. |
| `--head <ref>` | For `pr-review input from-git` | Git head ref for the deterministic diff range. |
| `--repo <path>` | No | Repository path for `pr-review input from-git`; defaults to the current directory. |
| `--input <path>` | For `pr-review targets recommend` | Reads the bounded PR review target JSON input snapshot. |
| `--input <path>` | For `completion review` | Reads a report or review snapshot containing completion candidates. |
| `--candidate <id>` | For `completion review` | Selects the candidate to accept or reject. |
| `--reviewer <id>` | For `completion review` | Records the explicit reviewer or workflow identifier. |
| `--reason <text>` | For `completion review` | Records the explicit acceptance or rejection rationale. |
| `--reviewed-at <text>` | No | Adds externally supplied review time metadata to the audit record. |
| `--output <path>` | No | Writes the JSON report to the requested file path instead of stdout. |

When `--output` is omitted, the command writes exactly one JSON report to
stdout. When `--output` is supplied, the command writes exactly one JSON report
file and keeps stdout empty.

## Agent Skill

The repository-owned CLI skill lives at
[`skills/highergraphen/SKILL.md`](../../skills/highergraphen/SKILL.md). It is
the immediate agent integration path for this report: agents should run the CLI,
validate the report contract, and interpret the JSON according to the schema.

MCP servers, provider-specific plugin bundles, marketplace metadata, and
provider-specific manifests are future optional work. They are not required for
the current CLI plus skill integration path.

## Examples

Emit the report to stdout:

```sh
./target/debug/highergraphen architecture smoke direct-db-access --format json
```

Lift the checked-in architecture input fixture:

```sh
./target/debug/highergraphen architecture input lift \
  --input schemas/inputs/architecture-lift.input.example.json \
  --format json
```

Run the checked-in Feed Product reader fixture:

```sh
./target/debug/highergraphen feed reader run \
  --input schemas/inputs/feed-lift.input.example.json \
  --format json
```

Run the checked-in PR review target fixture:

```sh
./target/debug/highergraphen pr-review targets recommend \
  --input schemas/inputs/pr-review-target.input.example.json \
  --format json
```

Generate a PR review target input from a local git range:

```sh
./target/debug/highergraphen pr-review input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output pr-review.input.json
```

Write the report to a file:

```sh
./target/debug/highergraphen architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Write a lifted input report to a file:

```sh
./target/debug/highergraphen architecture input lift \
  --input schemas/inputs/architecture-lift.input.example.json \
  --format json \
  --output architecture-input-lift.report.json
```

Write a Feed Product reader report to a file:

```sh
./target/debug/highergraphen feed reader run \
  --input schemas/inputs/feed-lift.input.example.json \
  --format json \
  --output feed-reader.report.json
```

Write a PR review target report to a file:

```sh
./target/debug/highergraphen pr-review input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output pr-review.input.json

./target/debug/highergraphen pr-review targets recommend \
  --input pr-review.input.json \
  --format json \
  --output pr-review-target.report.json
```

Accept a completion candidate from a generated report:

```sh
./target/debug/highergraphen completion review accept \
  --input architecture-direct-db-access-smoke.report.json \
  --candidate candidate:billing-status-api \
  --reviewer reviewer:architecture-lead \
  --reason "Billing Service owns the API boundary." \
  --format json \
  --output completion-review.report.json
```

Validate the generated report with the repository-owned no-network validator:

```sh
python3 scripts/validate-cli-report-contract.py
```

Validate an existing report file:

```sh
python3 scripts/validate-cli-report-contract.py \
  --report architecture-direct-db-access-smoke.report.json
```

Validate all checked-in JSON schemas and fixtures, including the PR review
target input and report contracts:

```sh
python3 scripts/validate-json-contracts.py
```

Run the focused PR review target runtime and CLI coverage:

```sh
cargo test -p higher-graphen-runtime --test pr_review_target
cargo test -p highergraphen-cli pr_review_input_from_git
cargo test -p highergraphen-cli pr_review_targets_recommend
```

## Exit Behavior

Exit code `0` means the workflow ran and emitted a report. The current workflow
is expected to detect a direct database access architecture violation, and that
domain finding is still a successful CLI result. For PR review targeting,
`result.status` values such as `"targets_recommended"` and `"no_targets"` are
also successful domain results.

Nonzero exits are reserved for command usage errors, runtime construction
failures, report serialization failures, or file output failures.

## Report Contract

The emitted report uses this stable contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.architecture.direct_db_access_smoke.report.v1` |
| Report type | `architecture_direct_db_access_smoke` |
| Report version | `1` |
| Schema file | [`architecture-direct-db-access-smoke.report.schema.json`](../../schemas/reports/architecture-direct-db-access-smoke.report.schema.json) |
| Example fixture | [`architecture-direct-db-access-smoke.report.example.json`](../../schemas/reports/architecture-direct-db-access-smoke.report.example.json) |
| Contract validator | [`validate-cli-report-contract.py`](../../scripts/validate-cli-report-contract.py) |
| Runtime runner | `higher_graphen_runtime::run_architecture_direct_db_access_smoke` |

The top-level JSON object contains:

- `schema`
- `report_type`
- `report_version`
- `metadata`
- `scenario`
- `result`
- `projection`

The current deterministic report has `result.status` set to
`"violation_detected"`, exactly one direct database access obstruction, and
exactly one billing status API completion candidate.

The architecture input lift report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.architecture.input_lift.report.v1` |
| Report type | `architecture_input_lift` |
| Report version | `1` |
| Input schema | [`architecture-lift.input.schema.json`](../../schemas/inputs/architecture-lift.input.schema.json) |
| Input fixture | [`architecture-lift.input.example.json`](../../schemas/inputs/architecture-lift.input.example.json) |
| Report schema | [`architecture-input-lift.report.schema.json`](../../schemas/reports/architecture-input-lift.report.schema.json) |
| Example fixture | [`architecture-input-lift.report.example.json`](../../schemas/reports/architecture-input-lift.report.example.json) |
| Runtime runner | `higher_graphen_runtime::run_architecture_input_lift` |

The input lift report has `result.status` set to `"lifted"`, records accepted
cell and incidence IDs under `result.accepted_fact_ids`, and records unreviewed
completion candidate IDs under `result.inferred_structure_ids`.

The Feed Product reader report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.feed.reader.report.v1` |
| Report type | `feed_reader` |
| Report version | `1` |
| Input schema | [`feed-lift.input.schema.json`](../../schemas/inputs/feed-lift.input.schema.json) |
| Input fixture | [`feed-lift.input.example.json`](../../schemas/inputs/feed-lift.input.example.json) |
| Report schema | [`feed-reader.report.schema.json`](../../schemas/reports/feed-reader.report.schema.json) |
| Example fixture | [`feed-reader.report.example.json`](../../schemas/reports/feed-reader.report.example.json) |
| Runtime runner | `higher_graphen_runtime::run_feed_reader` |

The Feed Product reader report has `result.status` set to
`"obstructions_detected"` for the checked-in fixture, records observed entry
IDs, inferred topic/event IDs, correspondences, completion candidates, and
obstructions, then projects them into `timeline`, `topic_digest`, and
`audit_trace` views with explicit `information_loss`.

The PR review target report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.pr_review_target.report.v1` |
| Report type | `pr_review_target` |
| Report version | `1` |
| Input schema | [`pr-review-target.input.schema.json`](../../schemas/inputs/pr-review-target.input.schema.json) |
| Input fixture | [`pr-review-target.input.example.json`](../../schemas/inputs/pr-review-target.input.example.json) |
| Report schema | [`pr-review-target.report.schema.json`](../../schemas/reports/pr-review-target.report.schema.json) |
| Example fixture | [`pr-review-target.report.example.json`](../../schemas/reports/pr-review-target.report.example.json) |
| Runtime runner | `higher_graphen_runtime::run_pr_review_target_recommend` |

The PR review target report has `result.status` set to
`"targets_recommended"` for the checked-in fixture, records accepted PR change
fact IDs under `result.accepted_change_ids`, and records AI-created review
targets, obstructions, and completion candidates as suggestions with
`review_status: "unreviewed"`. These records are review guidance only; the
workflow does not approve a pull request or record the human decision.

The git input adapter emits the same input schema from local commit history:

| Surface | Value |
| --- | --- |
| Command | `highergraphen pr-review input from-git --base <ref> --head <ref> --format json` |
| Output schema | `highergraphen.pr_review_target.input.v1` |
| Deterministic facts | Changed files, additions/deletions, commit evidence, path-derived owners/contexts/tests/dependency edges |
| Deterministic signals | Large change, ownership boundary, dependency coupling, schema validation coverage, docs/agent guidance boundary, security-sensitive paths, structural boundary change |

The adapter intentionally creates a bounded snapshot rather than a review
report. Run `pr-review targets recommend` afterward to produce unreviewed
review targets and obstructions.

The completion review report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.completion.review.report.v1` |
| Report type | `completion_review` |
| Report version | `1` |
| Report schema | [`completion-review.report.schema.json`](../../schemas/reports/completion-review.report.schema.json) |
| Runtime runner | `higher_graphen_runtime::run_completion_review` |

The review report records source report metadata under
`scenario.source_report`, preserves the selected source candidate under
`scenario.candidate` and `result.review_record.candidate`, and records the
explicit request under `result.review_record.request`. Accepted reports include
`result.review_record.accepted_completion`; rejected reports include
`result.review_record.rejected_completion`.

## Semantic Rules

Consumers must preserve these semantics:

- A detected architecture violation is report data, not a CLI failure.
- The billing status API is a completion candidate, not accepted structure.
- The completion candidate must remain `review_status: "unreviewed"` until a
  later explicit review workflow accepts or rejects it.
- Accepting or rejecting a completion candidate emits a separate auditable
  review report and never edits or silently promotes the source candidate.
- The input lift path treats JSON `components` and `relations` as accepted
  facts and JSON `inferred_structures` as unreviewed candidates.
- The feed reader path treats JSON `source_feeds` and `entries` as accepted
  local fixture facts, while completion and obstruction hints remain report
  findings for review.
- The PR review target path consumes bounded
  `highergraphen.pr_review_target.input.v1` snapshots, including
  `schemas/inputs/pr-review-target.input.example.json`.
- The git input adapter may create those snapshots from local commit history,
  but its facts and signals are still deterministic inputs to the recommender,
  not accepted review decisions.
- The PR review target path treats supplied changed files, symbols, tests,
  evidence, signals, and reviewer context as input observations; generated
  review targets, obstructions, and completion candidates remain unreviewed
  suggestions.
- PR review recommendations must be reviewed by humans, and explicit decisions
  must be recorded outside this recommender report.
- Agent skills and future tool surfaces should consume the CLI output or runtime
  runner and validate against the schema instead of reimplementing the workflow.

## Unsupported Usage

These are intentionally unsupported in the current CLI:

- Human-readable output formats.
- Architecture input formats beyond the bounded JSON v1 document.
- Feed input formats beyond the bounded JSON v1 fixture.
- PR review input formats beyond the bounded PR review target JSON v1 snapshot.
- Network fetching, scheduling, database persistence, read state, UI rendering,
  and production RSS/Atom parsing.
- Pull request approval, provider comment posting, reviewer assignment, or
  automatic promotion of AI recommendations into accepted review coverage.
- MCP server behavior.
- Provider-specific plugin, marketplace, or manifest behavior.
- Provider-specific skills beyond the repository-owned
  `skills/highergraphen/SKILL.md` CLI skill.
- Additional `highergraphen` subcommands beyond `architecture smoke
  direct-db-access`, `architecture input lift`, `feed reader run`,
  `pr-review targets recommend`, and `completion review`.
