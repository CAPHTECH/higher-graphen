# highergraphen CLI

The `highergraphen` command is the operational CLI for HigherGraphen runtime
workflows. It exposes the deterministic Architecture Product direct database
access smoke workflow, the bounded architecture input lift workflow, the
bounded Feed Product reader workflow, the bounded PR review target
recommendation workflow, the bounded test-gap detector workflow, the semantic
proof artifact adapter and verifier, and the explicit completion review
workflow as stable JSON reports.

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
./target/debug/highergraphen version
./target/debug/highergraphen architecture smoke direct-db-access --format json
```

You can also run the package through Cargo:

```sh
cargo run -p highergraphen-cli -- architecture smoke direct-db-access --format json
```

## Commands

```sh
highergraphen version
highergraphen --version
```

These commands print the CLI binary name and package version.

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
highergraphen test-gap input from-git --base <ref> --head <ref> --format json [--repo <path>] [--binding-rules <path>] [--output <path>]
```

This command deterministically converts a local git commit range into a bounded
`highergraphen.test_gap.input.v1` snapshot. It shells out to local git for the
diff, changed files, numstat, and commit summaries, then applies fixed adapter
rules for changed files, file-level changed-behavior symbols,
policy-accepted verification requirements, changed test files, evidence,
contexts, risk signals, and a detector verification policy. When the git range
includes changed integration tests, that policy can accept both `unit` and
`integration` test kinds without
rewriting the observed test type. It does not crawl the full repository,
execute tests, infer semantic coverage from source bodies, or accept generated
missing-test candidates.

For HigherGraphen-owned test-gap surfaces, the adapter also lifts higher-order
structure: CLI command cells, runtime runner cells, public export cells,
workflow registry cells, schema and fixture contract cells, report projection
cells, base/head Rust AST and JSON Schema semantic cells, semantic delta
morphisms, and incidence edges between them. The detector then evaluates
missing tests as verification gaps for those morphisms instead of treating
every changed file as an isolated obligation. Semantic delta morphisms expose
preservation, addition, and deletion at the parsed AST/schema level; typed
MIR-level equivalence and full behavior proofs remain explicit information
loss.

For changed Rust test files, the adapter also parses test bodies and lifts
test functions, assertions, observed CLI invocations, and observed JSON fields
as semantic evidence cells. Those cells can create
`rust_test_content_evidence` morphisms to the law or morphism that the test
body names, such as a `test-gap input from-path` command or the test-gap input
schema ID. This makes the detector distinguish content-backed verification
from path-only test-file presence, while still avoiding test execution or
unbounded proof of behavioral equivalence.

The Rust test semantic extraction step is intentionally project-neutral: it
recognizes Rust test functions, assertion macros, CLI-like token arrays, JSON
field indexing, and schema-shaped string identifiers without knowing
HigherGraphen IDs. The HigherGraphen adapter then binds those extracted
observations to this repository's known commands, adapters, morphisms, and
laws. This keeps the semantic lift reusable while making the repository-local
target mapping explicit.

The default binding rules are built into the CLI for this repository. A bounded
`highergraphen.test_gap.binding_rules.input.v1` file can replace those rules
with `--binding-rules <path>` for `test-gap input from-git` and `test-gap input
from-path`. The binding file maps extracted trigger terms to optional CLI
labels and target IDs; it does not change the generic Rust test semantic
extractor.

For HigherGraphen semantic-proof artifact adapter changes, the adapter lifts
the change into theorem/law/morphism structure rather than leaving helper
functions as isolated obligations. It creates semantic-proof artifact adapter
correctness cells, status-totality and proof/counterexample preservation laws,
artifact-to-input and artifact-to-proof/refutation morphisms, and maps existing
CLI roundtrip tests to those high-order obligations. Low-level helper semantic
deltas remain observable structure, but the verification decision is carried by
the semantic-proof theorem and morphisms.

```sh
highergraphen test-gap input from-path --path <path> [--path <path> ...] [--include-tests] --format json [--repo <path>] [--binding-rules <path>] [--output <path>]
```

This command deterministically converts selected current-tree files or
directories into the same bounded `highergraphen.test_gap.input.v1` snapshot
shape without reading git history. It resolves every `--path` inside the local
repository, recursively scans supported source, test, schema, fixture, and
documentation files, and optionally adds all repository test files when
`--include-tests` is present. The snapshot records `base_ref` and `head_ref` as
`current-tree`, uses `test-gap-from-path.v1` as its adapter marker, and keeps
`.git/` and `target/` outside the boundary.

The from-path adapter reuses the same HigherGraphen lift as from-git:
selected files become changed-behavior symbols, HigherGraphen surfaces become
command/adapter/schema/runner cells, current Rust and JSON Schema contents
become `head` semantic cells, and path-only semantic additions become explicit
morphisms. It is useful for broad folder or file audits when no meaningful git
range exists, but it remains a bounded snapshot: it does not execute tests,
prove full behavior equivalence, or accept generated candidates.
When selected Rust tests are present directly or through `--include-tests`,
their test functions, assertions, CLI observations, and JSON observations are
also lifted into content-backed verification evidence. The same generic Rust
test semantic extractor is used here; only the final mapping from extracted
observations to HigherGraphen test-gap targets is repository-specific.

```sh
highergraphen rust-test semantics from-path --path <path> [--path <path> ...] --format json [--repo <path>] [--test-run <path>] [--output <path>]
```

This command emits the generic `highergraphen.rust_test_semantics.input.v1` document
for selected Rust files or directories without applying HigherGraphen test-gap
binding rules. It records only the selected-path boundary, parsed Rust test
functions, assertion macros, CLI-like token arrays, JSON field observations,
and schema-shaped string identifiers. `.git/` and `target/` are excluded when
directories are scanned. The output is intended as a reusable semantic
extraction layer for agents and downstream adapters; repository-specific
mapping to commands, laws, morphisms, or verification targets is deliberately
left to a later binding step. When `--test-run <path>` is provided, the same
generic document also records parsed execution cases and static function-name
matches, still without adding HigherGraphen target IDs.

The repository also defines `highergraphen.test_semantics.input.v1` as a
language-neutral super-contract for future Jest, pytest, ExUnit, or other test
semantic adapters. The current CLI command emits the Rust-specific contract.

```sh
highergraphen test-semantics interpret --input <path> --format json [--interpreter <id>] [--output <path>]
```

This command reads a bounded Rust or language-neutral test semantics document
and emits `highergraphen.test_semantics.interpretation.v1`. The interpretation
document is an AI-agent candidate structure: interpreted cells, interpreted
morphisms, candidate laws, binding candidates, evidence links, and explicit
information-loss notes are all emitted with `review_status: "unreviewed"`.
It does not accept coverage, approve a binding, or turn semantic candidates
into proof objects.

```sh
highergraphen test-semantics review accept \
  --input <path> \
  --candidate <id> \
  --reviewer <id> \
  --reason <text> \
  --format json \
  [--output <path>]

highergraphen test-semantics review reject \
  --input <path> \
  --candidate <id> \
  --reviewer <id> \
  --reason <text> \
  --format json \
  [--output <path>]
```

These commands read `highergraphen.test_semantics.interpretation.v1`, select an
AI-created interpreted cell, interpreted morphism, candidate law, binding
candidate, or evidence link by ID, and emit a separate
`highergraphen.test_semantics.interpretation_review.report.v1` report. The
source interpretation is not edited. Accepted candidates are still not coverage
or proof objects; they are review decisions that a later verification workflow
can consume.

```sh
highergraphen test-gap evidence from-test-run --input <path> --test-run <path> --format json [--output <path>]
```

This command augments a bounded `highergraphen.test_gap.input.v1` snapshot with
test execution evidence. It reads either JSON/JSONL test case records or stable
`cargo test` text lines such as `test name ... ok`, then adds a
`test-run-evidence.v1` adapter marker, a `test_result` evidence record,
test-run artifact and executed test-case cells, and incidences connecting
passed test cases to parsed Rust test functions. For passed cases, it creates
`executed_automated_test` verification cells that mirror the matching
content-derived verification target and include the executed test-case cell as
evidence. Failed tests are represented as failed execution cells and high
severity risk signals instead of accepted verification.

```sh
highergraphen test-gap detect --input <path> --format json [--output <path>]
```

This command reads a bounded `highergraphen.test_gap.input.v1` snapshot and
emits a missing-unit-test detector report. The workflow lifts supplied files,
symbols, requirements, tests, coverage, evidence, and risk signals as bounded
input facts, then emits missing-test obstructions and `missing_test`
completion candidates as reviewable report data. Obstructions and completion
candidates remain `review_status: "unreviewed"` until a later explicit review
workflow accepts or rejects them.

```sh
highergraphen semantic-proof backend run \
  --backend <name> \
  --backend-version <version> \
  --command <path> \
  [--arg <text> ...] \
  [--input <path>] \
  --format json \
  [--output <path>]
```

This command runs a local proof backend command without a shell and emits a
bounded backend artifact. A zero exit status becomes `status: "proved"` with
`review_status: "accepted"`; a non-zero exit status becomes
`status: "counterexample_found"` with `review_status: "unreviewed"`. The
artifact records command path, args, exit code, stdout/stderr excerpts, and
deterministic hashes behind a local-process trust boundary. Pass the artifact to
`semantic-proof input from-artifact` before `semantic-proof verify`; HG policy,
not the process exit alone, decides whether proof or counterexample structure is
accepted.

```sh
highergraphen semantic-proof input from-artifact \
  --artifact <path> \
  --backend <name> \
  --backend-version <version> \
  --theorem-id <id> \
  --theorem-summary <text> \
  --law-id <id> \
  --law-summary <text> \
  --morphism-id <id> \
  --morphism-type <text> \
  --base-cell <id> \
  --base-label <text> \
  --head-cell <id> \
  --head-label <text> \
  --format json \
  [--output <path>]
```

This command converts a local proof-backend artifact into a bounded
`highergraphen.semantic_proof.input.v1` snapshot. The artifact must declare
`status: "proved"`, `status: "counterexample"`, or
`status: "counterexample_found"`, and may supply hashes, witness/path IDs,
review status, severity, and confidence. The CLI arguments supply the HG
theorem, law, morphism, and base/head semantic cell identities, so the generated
input can be passed directly to `highergraphen semantic-proof verify`. This
adapter does not run Kani, Prusti, SMT solving, model checking, symbolic
execution, or MIR extraction; use `semantic-proof backend run` when a local
process should be executed first. The generated policy requires accepted proof
certificates and accepted counterexamples before they are trusted at the HG
boundary.

```sh
highergraphen semantic-proof input from-report \
  --report <path> \
  --format json \
  [--output <path>]
```

This command reads an `insufficient_proof` semantic-proof report and produces a
new `highergraphen.semantic_proof.input.v1` snapshot containing the open law and
morphism obligations, with certificates and counterexamples cleared. This is the
bounded reinput path for agents: unresolved proof obligations can be routed back
to backend execution or artifact attachment without treating suggested or failed
structure as accepted.

```sh
highergraphen semantic-proof verify --input <path> --format json [--output <path>]
```

This command verifies a bounded `highergraphen.semantic_proof.input.v1`
certificate bundle. It is the HG-facing layer for formal verification
backends: Kani, Prusti, Creusot, SMT, symbolic execution, model checking, or
property-test adapters can emit proof certificates or counterexamples, and this
workflow validates their references and verification policy before emitting
`proof_objects`, `counterexamples`, and issues. It does not itself run rustc
MIR extraction, SMT solving, model checking, or symbolic execution; those
backend runs occur before the bounded input is supplied, either externally or
through the bounded `semantic-proof backend run` adapter. The report preserves
that boundary as projection information loss, and `semantic-proof input
from-report` can requeue unproved laws and morphisms.

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
| `--input <path>` | For `test-semantics review` | Reads a test semantics interpretation document. |
| `--candidate <id>` | For `test-semantics review` | Selects the interpreted cell, interpreted morphism, candidate law, binding candidate, or evidence link to accept or reject. |
| `--reviewer <id>` | For `test-semantics review` | Records the explicit reviewer or workflow identifier. |
| `--reason <text>` | For `test-semantics review` | Records the explicit acceptance or rejection rationale. |
| `--base <ref>` | For `pr-review input from-git` and `test-gap input from-git` | Git base ref for the deterministic diff range. |
| `--head <ref>` | For `pr-review input from-git` and `test-gap input from-git` | Git head ref for the deterministic diff range. |
| `--repo <path>` | No | Repository path for git input commands; defaults to the current directory. |
| `--path <path>` | For `test-gap input from-path` and `rust-test semantics from-path` | Selects a current-tree file or directory to scan; repeat for multiple roots. |
| `--include-tests` | No | For `test-gap input from-path`, adds repository test files to the bounded snapshot. |
| `--binding-rules <path>` | No | For `test-gap input from-git` and `test-gap input from-path`, replaces the built-in Rust test semantic binding rules with a bounded `highergraphen.test_gap.binding_rules.input.v1` document. |
| `--test-run <path>` | For `test-gap evidence from-test-run` and `rust-test semantics from-path` | Reads bounded JSON/JSONL or plain `cargo test` output. For test-gap evidence it attaches execution evidence to the input snapshot; for rust-test semantics it records generic execution cases and matched test functions. |
| `--interpreter <id>` | No | For `test-semantics interpret`, names the AI agent or process that authored the unreviewed interpretation candidates. |
| `--input <path>` | For `pr-review targets recommend` | Reads the bounded PR review target JSON input snapshot. |
| `--input <path>` | For `test-gap detect` and `test-gap evidence from-test-run` | Reads the bounded test-gap JSON input snapshot. |
| `--command <path>` | For `semantic-proof backend run` | Runs the local proof backend process without a shell. |
| `--arg <text>` | For `semantic-proof backend run` | Adds one backend process argument; repeat for multiple arguments. |
| `--input <path>` | For `semantic-proof backend run` | Optional backend input material included in the artifact input hash. |
| `--artifact <path>` | For `semantic-proof input from-artifact` | Reads a local bounded backend artifact. |
| `--backend <name>` | For `semantic-proof backend run` and `semantic-proof input from-artifact` | Records the proof backend name and adds it to the generated verification policy. |
| `--backend-version <version>` | For `semantic-proof backend run` and `semantic-proof input from-artifact` | Records the proof backend version on generated artifacts and certificates. |
| `--theorem-id <id>` / `--theorem-summary <text>` | For `semantic-proof input from-artifact` | Defines the HG theorem obligation represented by the artifact. |
| `--law-id <id>` / `--law-summary <text>` | For `semantic-proof input from-artifact` | Defines the semantic law that must be preserved or refuted. |
| `--morphism-id <id>` / `--morphism-type <text>` | For `semantic-proof input from-artifact` | Defines the semantic morphism checked by the proof artifact. |
| `--base-cell <id>` / `--base-label <text>` | For `semantic-proof input from-artifact` | Defines the source semantic endpoint for the morphism. |
| `--head-cell <id>` / `--head-label <text>` | For `semantic-proof input from-artifact` | Defines the target semantic endpoint for the morphism. |
| `--report <path>` | For `semantic-proof input from-report` | Reads an insufficient semantic-proof report and requeues open obligations. |
| `--input <path>` | For `semantic-proof verify` | Reads the bounded semantic proof certificate snapshot. |
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

Run the checked-in test-gap fixture:

```sh
./target/debug/highergraphen test-gap detect \
  --input schemas/inputs/test-gap.input.example.json \
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

Generate a test-gap input from a local git range:

```sh
./target/debug/highergraphen test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json
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

Write a test-gap report to a file:

```sh
./target/debug/highergraphen test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

./target/debug/highergraphen test-gap detect \
  --input test-gap.input.json \
  --format json \
  --output test-gap.report.json
```

Verify a semantic proof certificate bundle:

```sh
./target/debug/highergraphen semantic-proof verify \
  --input schemas/inputs/semantic-proof.input.example.json \
  --format json \
  --output semantic-proof.report.json
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
target and test-gap input and report contracts:

```sh
python3 scripts/validate-json-contracts.py
```

Run the focused PR review target runtime and CLI coverage:

```sh
cargo test -p higher-graphen-runtime --test pr_review_target
cargo test -p highergraphen-cli pr_review_input_from_git
cargo test -p highergraphen-cli pr_review_targets_recommend
```

Run the focused test-gap runtime and CLI coverage:

```sh
cargo test -p higher-graphen-runtime --test test_gap
cargo test -p highergraphen-cli test_gap_detect
```

## Exit Behavior

Exit code `0` means the workflow ran and emitted a report. The current workflow
is expected to detect a direct database access architecture violation, and that
domain finding is still a successful CLI result. For PR review targeting,
`result.status` values such as `"targets_recommended"` and `"no_targets"` are
also successful domain results.
For test-gap detection, `result.status` values such as `"gaps_detected"` and
`"no_gaps_in_snapshot"` are successful domain results. A
`"no_gaps_in_snapshot"` result is bounded to the supplied snapshot and is not
global proof that the repository has complete tests.

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

The test-gap detector report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.test_gap.report.v1` |
| Report type | `test_gap` |
| Report version | `1` |
| Input schema | [`test-gap.input.schema.json`](../../schemas/inputs/test-gap.input.schema.json) |
| Input fixture | [`test-gap.input.example.json`](../../schemas/inputs/test-gap.input.example.json) |
| Report schema | [`test-gap.report.schema.json`](../../schemas/reports/test-gap.report.schema.json) |
| Example fixture | [`test-gap.report.example.json`](../../schemas/reports/test-gap.report.example.json) |
| Runtime runner | `higher_graphen_runtime::run_test_gap_detect` |

The test-gap report has `result.status` set to `"gaps_detected"` for the
checked-in fixture, records lifted bounded input facts under
`result.accepted_fact_ids`, records missing-test obstructions with severity,
confidence, source IDs, and `review_status: "unreviewed"`, and records
`completion_candidates` with `candidate_type: "missing_test"`, suggested test
shape, provenance, confidence, and `review_status: "unreviewed"`.
Projection views must preserve `information_loss`; summaries must not hide
omitted source bodies, summarized diffs, absent coverage dimensions, inferred
candidate status, or the bounded snapshot boundary.

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

The test semantics interpretation review report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.test_semantics.interpretation_review.report.v1` |
| Report type | `test_semantics_interpretation_review` |
| Report version | `1` |
| Report schema | [`test-semantics-interpretation-review.report.schema.json`](../../schemas/reports/test-semantics-interpretation-review.report.schema.json) |
| Runtime runner | CLI-local review adapter |

The review report records source interpretation metadata under
`scenario.source_interpretation`, preserves the selected unreviewed source
candidate under `scenario.candidate` and `result.review_record.candidate`, and
records the explicit decision under `result.review_record.request`.
`result.review_record.reviewed_candidate` shows the reviewed status, while
`result.accepted_fact_ids`, `result.coverage_ids`, and
`result.proof_object_ids` remain empty because review is not verification.

## Semantic Rules

Consumers must preserve these semantics:

- A detected architecture violation is report data, not a CLI failure.
- The billing status API is a completion candidate, not accepted structure.
- The completion candidate must remain `review_status: "unreviewed"` until a
  later explicit review workflow accepts or rejects it.
- Accepting or rejecting a completion candidate emits a separate auditable
  review report and never edits or silently promotes the source candidate.
- Accepting or rejecting a test semantics interpretation candidate emits a
  separate auditable review report and never creates coverage or proof objects.
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
- The test-gap path consumes bounded `highergraphen.test_gap.input.v1`
  snapshots, including `schemas/inputs/test-gap.input.example.json`.
- The test-gap git input adapter may create those snapshots from local commit
  history, but it only supplies deterministic file-level obligations and
  evidence. For HigherGraphen-owned test-gap surfaces it also supplies parsed
  base/head Rust AST and JSON Schema semantic delta morphisms, but it does not
  prove typed semantic equivalence or full behavior coverage.
- The semantic-proof path consumes bounded proof certificate snapshots. It
  checks theorem/law/morphism/certificate/counterexample references and
  certificate and counterexample review policy, then emits accepted
  `proof_objects`, accepted counterexamples, or unreviewed issues. The optional
  backend runner records local process output as artifact material; HG
  verification still owns acceptance, and report reinput preserves open
  obligations without accepting them.
- Test-gap accepted facts are limited to the supplied snapshot. Domain findings
  such as missing-test obstructions, insufficient evidence, `gaps_detected`,
  and `no_gaps_in_snapshot` are successful report data, not CLI failures.
- Test-gap `completion_candidates` with `candidate_type: "missing_test"` and
  inferred obstructions remain `review_status: "unreviewed"` until a later
  explicit review workflow accepts or rejects them.
- Test-gap projections must expose `information_loss` and preserve source IDs,
  severity, confidence, obstruction witnesses, suggested test shape, and review
  status for agent and audit use.
- `no_gaps_in_snapshot` means the supplied snapshot did not violate the
  configured invariants. It is not proof that all tests are complete across the
  repository.
- Agent skills and future tool surfaces should consume the CLI output or runtime
  runner and validate against the schema instead of reimplementing the workflow.

## Agent Reporting

When an agent reports a test-gap result, include:

- the exact command run and whether validation passed;
- `result.status` and whether it is a bounded success result;
- obstructions with severity, confidence, source IDs, target IDs, and review
  status;
- completion candidates with candidate IDs, suggested test shape, confidence,
  provenance/source IDs, and `review_status`;
- projection `information_loss` from human, AI, and audit views when present;
- unsupported or deferred scope, especially full repository crawling, generated
  test acceptance, external proof backend execution, typed semantic equivalence
  beyond supplied certificates, or global proof of complete tests.

## Unsupported Usage

These are intentionally unsupported in the current CLI:

- Human-readable output formats.
- Architecture input formats beyond the bounded JSON v1 document.
- Feed input formats beyond the bounded JSON v1 fixture.
- PR review input formats beyond the bounded PR review target JSON v1 snapshot.
- Test-gap input formats beyond bounded JSON v1 snapshots, deterministic
  local git range input, deterministic current-tree path input, and bounded
  test-run evidence augmentation.
- Network fetching, scheduling, database persistence, read state, UI rendering,
  and production RSS/Atom parsing.
- Pull request approval, provider comment posting, reviewer assignment, or
  automatic promotion of AI recommendations into accepted review coverage.
- Test generation, running test commands, coverage approval, or automatic
  promotion of missing-test candidates into accepted tests.
- Global assertions that no test gaps exist outside the bounded input snapshot.
- MCP server behavior.
- Provider-specific plugin, marketplace, or manifest behavior.
- Provider-specific skills beyond the repository-owned
  `skills/highergraphen/SKILL.md` CLI skill.
- Additional `highergraphen` subcommands beyond `architecture smoke
  direct-db-access`, `architecture input lift`, `feed reader run`,
  `pr-review input from-git`, `pr-review targets recommend`, `test-gap
  input from-git`, `test-gap input from-path`, `test-gap evidence
  from-test-run`, `test-gap detect`, `semantic-proof verify`, and
  `completion review`.
