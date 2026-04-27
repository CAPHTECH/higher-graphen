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
