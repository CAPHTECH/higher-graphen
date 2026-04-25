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
