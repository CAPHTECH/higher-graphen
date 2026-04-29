---
name: highergraphen
description: Use when an agent needs to run or interpret repository-owned HigherGraphen CLI workflow reports, including Architecture Product smoke, Feed reader, completion review, PR review target recommendation, and test-gap detector contracts.
---

# HigherGraphen CLI Skill

Use this skill when a task asks for HigherGraphen agent-facing workflow output,
Architecture Product smoke validation, bounded Feed reader output, completion
review output, PR review target recommendations, bounded missing-unit-test
gap detection, or interpretation of a `highergraphen` JSON report.

This repository skill is CLI-only. MCP servers, provider plugin bundles,
marketplace metadata, and provider-specific manifests are outside the immediate
path.

## Source Of Truth

- CLI reference: `docs/cli/highergraphen.md`
- Agent handoff: `docs/specs/agent-tooling-handoff.md`
- Report schema: `schemas/reports/architecture-direct-db-access-smoke.report.schema.json`
- Example report: `schemas/reports/architecture-direct-db-access-smoke.report.example.json`
- PR review target input schema: `schemas/inputs/pr-review-target.input.schema.json`
- PR review target report schema: `schemas/reports/pr-review-target.report.schema.json`
- PR review target fixture: `schemas/inputs/pr-review-target.input.example.json`
- Test-gap input schema: `schemas/inputs/test-gap.input.schema.json`
- Test-gap report schema: `schemas/reports/test-gap.report.schema.json`
- Test-gap fixture: `schemas/inputs/test-gap.input.example.json`
- Local contract validator: `scripts/validate-cli-report-contract.py`
- JSON contract validator: `scripts/validate-json-contracts.py`

Do not restate the report schema as a competing contract. Consume the schema,
fixture, and CLI output.

## When To Run The CLI

Run the CLI when the user asks for a current HigherGraphen workflow report,
including the Architecture Product smoke workflow, direct database access
architecture report, bounded feed reader report, completion review report, or
PR review target recommendation report. Run it for test-gap work when the user
has a bounded `highergraphen.test_gap.input.v1` snapshot and wants missing
unit-test obstructions or completion candidates.

Preferred local validation:

```sh
python3 scripts/validate-cli-report-contract.py
```

Check the installed CLI version:

```sh
highergraphen version
```

Generate the report to stdout:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access --format json
```

Generate the report to a file:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Validate an existing report file:

```sh
python3 scripts/validate-cli-report-contract.py \
  --report architecture-direct-db-access-smoke.report.json
```

Run the bounded PR review target recommender:

```sh
cargo run -q -p highergraphen-cli -- \
  pr-review targets recommend \
  --input schemas/inputs/pr-review-target.input.example.json \
  --format json
```

Generate a bounded PR review target input from local git history:

```sh
cargo run -q -p highergraphen-cli -- \
  pr-review input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output pr-review.input.json
```

Generate a PR review target report to a file:

```sh
cargo run -q -p highergraphen-cli -- \
  pr-review targets recommend \
  --input pr-review.input.json \
  --format json \
  --output pr-review-target.report.json
```

Run the bounded test-gap detector:

```sh
cargo run -q -p highergraphen-cli -- \
  test-gap detect \
  --input schemas/inputs/test-gap.input.example.json \
  --format json
```

Generate a bounded test-gap input from local git history:

```sh
cargo run -q -p highergraphen-cli -- \
  test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json
```

Generate a test-gap report to a file:

```sh
cargo run -q -p highergraphen-cli -- \
  test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

cargo run -q -p highergraphen-cli -- \
  test-gap detect \
  --input test-gap.input.json \
  --format json \
  --output test-gap.report.json
```

Verify a semantic proof certificate bundle:

```sh
cargo run -q -p highergraphen-cli -- \
  semantic-proof backend run \
  --backend kani \
  --backend-version 1.0.0 \
  --command /path/to/backend \
  --arg verify \
  --input proof-obligation.json \
  --format json \
  --output proof-artifact.json

cargo run -q -p highergraphen-cli -- \
  semantic-proof input from-artifact \
  --artifact proof-artifact.json \
  --backend kani \
  --backend-version 1.0.0 \
  --theorem-id theorem:semantic:pricing \
  --theorem-summary "Pricing typed signature is preserved." \
  --law-id law:semantic:signature-preserved \
  --law-summary "Public typed signature is preserved." \
  --morphism-id morphism:semantic:pricing-signature \
  --morphism-type typed_signature_preservation \
  --base-cell cell:semantic:pricing:base \
  --base-label "base calculate_discount MIR" \
  --head-cell cell:semantic:pricing:head \
  --head-label "head calculate_discount MIR" \
  --format json \
  --output semantic-proof.input.json

cargo run -q -p highergraphen-cli -- \
  semantic-proof verify \
  --input semantic-proof.input.json \
  --format json \
  --output semantic-proof.report.json
```

Requeue unproved semantic proof obligations from an insufficient report:

```sh
cargo run -q -p highergraphen-cli -- \
  semantic-proof input from-report \
  --report semantic-proof.report.json \
  --format json \
  --output semantic-proof.reinput.json
```

Validate all checked-in schema-bearing fixtures:

```sh
python3 scripts/validate-json-contracts.py
```

Run focused PR review target runtime and CLI coverage:

```sh
cargo test -p higher-graphen-runtime --test pr_review_target
cargo test -p highergraphen-cli pr_review_input_from_git
cargo test -p highergraphen-cli pr_review_targets_recommend
```

Run focused test-gap runtime and CLI coverage:

```sh
cargo test -p higher-graphen-runtime --test test_gap
cargo test -p higher-graphen-runtime --test semantic_proof
cargo test -p highergraphen-cli test_gap_input_from_git
cargo test -p highergraphen-cli test_gap_detect
cargo test -p highergraphen-cli semantic_proof
```

## Interpretation Rules

- Exit code `0` means the workflow ran and emitted a report.
- `result.status == "violation_detected"` is a successful domain finding, not
  a failed CLI run.
- The report should contain exactly one direct database access obstruction for
  `obstruction:order-service-direct-billing-db-access`.
- The suggested billing status API is a completion candidate, not accepted
  structure.
- Preserve `review_status: "unreviewed"` for the obstruction provenance and the
  completion candidate unless a later explicit review workflow accepts or
  rejects it.
- Present `projection.recommended_actions` as recommendations, and keep
  `projection.information_loss` visible in summaries.
- State that this workflow is deterministic smoke coverage, not full ingestion
  of real architecture documents, source code, ADRs, tickets, databases, or
  OpenAPI files.
- For `highergraphen pr-review targets recommend`, consume only bounded
  `highergraphen.pr_review_target.input.v1` snapshots such as
  `schemas/inputs/pr-review-target.input.example.json`.
- For local repositories, prefer `highergraphen pr-review input from-git` to
  create the bounded snapshot deterministically from commit history before
  running `pr-review targets recommend`.
- Interpret `signal:structural-boundary-change` as a deterministic dependency
  prompt derived from finite boundary, incidence, or composition observations;
  use it to inspect parent-module wiring and command dispatch targets.
- Treat git-derived risk signals as deterministic review prompts, not as final
  review decisions.
- Treat PR review targets, obstructions, and completion candidates created by
  the workflow as suggestions with `review_status: "unreviewed"`.
- State that PR review target reports do not approve pull requests or record
  final review decisions. Humans must review recommended targets and record
  explicit decisions elsewhere.
- For `highergraphen test-gap detect`, consume only bounded
  `highergraphen.test_gap.input.v1` snapshots such as
  `schemas/inputs/test-gap.input.example.json`.
- For local repositories, prefer `highergraphen test-gap input from-git` to
  create the bounded test-gap snapshot deterministically from commit history
  before running `test-gap detect`.
- Treat test-gap git-derived symbols, requirements, evidence, risk signals,
  base/head semantic cells, and semantic delta morphisms as deterministic
  bounded structure, not typed proof of full behavior coverage.
- Interpret `detector_context.test_kinds` as the verification policy for the
  bounded snapshot. A git-derived snapshot may accept both `unit` and
  `integration` tests while preserving each observed test's actual type.
- For HigherGraphen-owned test-gap surfaces, interpret generated command,
  runner, export, registry, schema, fixture, projection, base/head Rust AST and
  JSON Schema semantic cells, semantic delta morphisms, incidence, and
  `requirement:morphism:*` records as the primary high-order verification
  structure.
- For HigherGraphen semantic-proof changes, interpret
  `theorem:semantic-proof:artifact-adapter-correctness`,
  `theorem:semantic-proof:backend-run-trust-boundary`,
  `theorem:semantic-proof:obligation-reinput-correctness`,
  `law:semantic-proof:*`, and `morphism:semantic-proof:*` as the primary
  verification structure. Helper-level Rust semantic deltas are observable
  structure, but missing-test decisions should be read through the high-order
  theorem/law/morphism proof objects.
- For `highergraphen semantic-proof verify`, treat accepted proof certificates
  and accepted counterexamples as formal verification cells only inside the
  bounded certificate snapshot and verification policy. The command checks
  certificate, counterexample, reference, and review policy.
- Prefer `highergraphen semantic-proof backend run` when the agent needs to
  execute a local proof command. It records command path, args, exit status,
  hashes, and excerpts as a bounded artifact; zero exit is accepted proof
  material, while non-zero exit remains an unreviewed counterexample artifact
  until HG policy accepts it.
- Prefer `highergraphen semantic-proof input from-artifact` when an external
  proof backend has already produced a local artifact. The adapter converts
  the artifact into theorem, law, morphism, semantic cell, certificate, or
  counterexample structure for HG.
- Use `highergraphen semantic-proof input from-report` after
  `insufficient_proof` to generate a new bounded input containing the open laws
  and morphisms with proof certificates and counterexamples cleared.
- Treat test-gap statuses such as `gaps_detected` and
  `no_gaps_in_snapshot` as successful report data. `no_gaps_in_snapshot` is
  bounded to the supplied snapshot and is not global proof that the repository
  has complete tests.
- Treat missing-test obstructions as successful detector findings. Preserve
  their severity, confidence, target IDs, source IDs, witness data, and
  `review_status: "unreviewed"`.
- Treat completion candidates with `candidate_type: "missing_test"` as
  suggested test work only. Preserve suggested test shape, provenance/source
  IDs, confidence, and `review_status: "unreviewed"`.
- Keep projection `information_loss` visible for every test-gap summary,
  especially omitted source bodies, summarized diffs, absent coverage
  dimensions, unreviewed inference, and the bounded snapshot boundary.

## Agent Output Shape

When reporting results to a user, include:

- The command or validator that was run.
- Whether contract validation passed.
- The invariant or obstruction that was found.
- The recommended actions from the projection.
- Any completion candidates with confidence and review status.
- For PR review target reports, recommended targets with severity, confidence,
  evidence IDs, and review status.
- For test-gap reports, obstructions with severity, confidence, source IDs,
  target IDs, witness summary, and review status.
- For test-gap reports, missing-test completion candidates with suggested test
  shape, confidence, provenance/source IDs, and review status.
- Projection information loss for human, AI-agent, and audit views when
  present.
- Any unsupported scope the user requested, especially full repository
  crawling, backend-specific proof semantics beyond local process execution,
  semantic coverage inference, candidate acceptance, MCP, plugin packaging, or
  marketplace work.

## Safety Rules

- Do not treat AI-inferred or suggested structure as accepted fact.
- Do not treat AI-created PR review targets, obstructions, or completion
  candidates as approved review coverage.
- Do not approve PRs or record review decisions from the recommender report.
- Do not accept or reject completion candidates without an explicit review
  workflow.
- Do not treat missing-test candidates or detector obstructions as accepted
  tests or reviewed coverage.
- Do not present `no_gaps_in_snapshot` as proof that all repository tests are
  complete.
- Do not claim `highergraphen test-gap input from-git` executes tests, crawls
  the full repository, proves typed semantic equivalence, or proves complete
  behavior coverage.
- Do not claim `highergraphen semantic-proof verify` runs external proof
  backends. It verifies supplied proof certificates and counterexamples.
- Do not treat `highergraphen semantic-proof backend run` output as accepted HG
  proof until it has passed `semantic-proof input from-artifact` and
  `semantic-proof verify` under the review policy.
- Do not claim `highergraphen semantic-proof input from-artifact` proves
  anything by itself. It normalizes already-produced backend artifacts into the
  bounded HG proof-input contract.
- Do not hide information loss in projections.
- Do not introduce MCP implementation or dependencies for this CLI skill path.
- Do not modify lower-level crates to change the report contract unless the user
  explicitly asks for a new runtime or schema version.
