# Input Schemas

This directory contains bounded source input contracts for HigherGraphen runtime
workflows.

`architecture-lift.input.schema.json` defines the first architecture input
contract, `highergraphen.architecture.input.v1`. The scope is intentionally
narrow: one structured JSON document with a space, explicit contexts, accepted
component/relation facts, source provenance, confidence values, and separate
unreviewed inferred structures.

The matching fixture is:

```sh
schemas/inputs/architecture-lift.input.example.json
```

A reuse fixture exercises non-database Architecture Product vocabulary:

```sh
schemas/inputs/architecture-lift.reuse.input.example.json
```

`feed-lift.input.schema.json` defines the first Feed Product input contract,
`highergraphen.feed.input.v1`. The scope is intentionally narrow: one
structured JSON fixture with source feeds, feed entries, correspondence hints,
completion hints, obstruction hints, and requested projections. It does not
accept raw RSS or Atom XML; feed fetching and parsing stay outside this first
contract.

The matching fixture is:

```sh
schemas/inputs/feed-lift.input.example.json
```

`pr-review-target.input.schema.json` defines the first bounded PR review target
input contract, `highergraphen.pr_review_target.input.v1`. It accepts a
provider-neutral PR summary with repository identity, pull request identity,
changed files, optional symbols, optional risk signals, and reviewer context.
It does not accept raw provider webhook payloads or full diffs.

The matching fixture is:

```sh
schemas/inputs/pr-review-target.input.example.json
```

Run the PR review target recommender fixture with:

```sh
highergraphen pr-review targets recommend \
  --input schemas/inputs/pr-review-target.input.example.json \
  --format json
```

The command treats the bounded snapshot as input observations. AI-created
review targets, obstructions, and completion candidates in the emitted report
remain suggestions with `review_status: "unreviewed"`; the input contract does
not carry PR approval or final review decisions.

Create the same input schema from local commit history with:

```sh
highergraphen pr-review input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output pr-review.input.json
```

The git adapter emits deterministic changed-file facts, commit/diff evidence,
path-derived owners and contexts, and risk signals. It does not approve PRs or
record human review decisions.

`ddd-review.input.schema.json` defines the bounded DDD review input contract,
`highergraphen.ddd_review.input.v1`. It accepts source-backed DDD facts,
constraints, reviews, unreviewed inferred claims, completion hints, projection
requests, and an explicit source boundary. Accepted source facts are separate
from AI-inferred or unreviewed claims.

The matching fixture is:

```sh
schemas/inputs/ddd-review.input.example.json
```

The fixture mirrors the Sales/Billing Customer CaseGraphen example:

```sh
examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json
```

Run the DDD review fixture with:

```sh
highergraphen ddd review \
  --input schemas/inputs/ddd-review.input.example.json \
  --format json
```

`rust-test-semantics.input.schema.json` defines a bounded, project-neutral Rust
test semantic extraction contract, `highergraphen.rust_test_semantics.input.v1`. It
captures selected paths, Rust test functions, assertion macros, CLI-like token
arrays, JSON field observations, and schema-shaped string identifiers without
binding them to HigherGraphen-specific laws or morphisms.

`test-semantics.input.schema.json` defines the language-neutral super-contract,
`highergraphen.test_semantics.input.v1`, for adapters that normalize Rust,
Jest, pytest, ExUnit, or other test frameworks into common test, command,
data, and execution observations. The Rust-specific contract remains the
repository-owned concrete adapter shape.

`test-semantics-interpretation.input.schema.json` defines the unreviewed
AI-agent interpretation contract,
`highergraphen.test_semantics.interpretation.v1`. It preserves interpreted
cells, interpreted morphisms, candidate laws, binding candidates, evidence
links, and explicit information-loss notes as candidate structure rather than
accepted coverage.

The matching fixture is:

```sh
schemas/inputs/rust-test-semantics.input.example.json
```

Create the same document from local files with:

```sh
highergraphen rust-test semantics from-path \
  --path tools/highergraphen-cli/tests/command.rs \
  --test-run test-run.txt \
  --format json
```

Create unreviewed AI-agent interpretation candidates from that document with:

```sh
highergraphen test-semantics interpret \
  --input rust-test-semantics.input.json \
  --interpreter codex \
  --format json
```

`test-semantics-expected-obligations.input.schema.json` defines accepted
semantic test obligations for gap detection,
`highergraphen.test_semantics.expected_obligations.input.v1`. It is compared
with one or more verified test semantics reports by:

```sh
highergraphen test-semantics gap \
  --expected test-semantics-expected-obligations.input.json \
  --verified test-semantics-verification.report.json \
  --format json
```

`test-gap-binding-rules.input.schema.json` defines the project-specific binding
contract, `highergraphen.test_gap.binding_rules.input.v1`. It maps extracted
Rust test semantic trigger terms to CLI labels and HigherGraphen target IDs.
When omitted, the CLI uses the built-in HigherGraphen binding rules. When
provided, the file replaces the built-in rules for that input generation run.

The matching fixture is:

```sh
schemas/inputs/test-gap-binding-rules.input.example.json
```

Use a binding file with local test-gap input generation:

```sh
highergraphen test-gap input from-path \
  --path tools/highergraphen-cli/tests/command.rs \
  --binding-rules schemas/inputs/test-gap-binding-rules.input.example.json \
  --format json
```

Lift the architecture fixture with:

```sh
highergraphen architecture input lift \
  --input schemas/inputs/architecture-lift.input.example.json \
  --format json
```

Run the feed fixture with:

```sh
highergraphen feed reader run \
  --input schemas/inputs/feed-lift.input.example.json \
  --format json
```

Validate schema-bearing fixtures with:

```sh
python3 scripts/validate-json-contracts.py
```
