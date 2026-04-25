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

Lift the fixture with:

```sh
highergraphen architecture input lift \
  --input schemas/inputs/architecture-lift.input.example.json \
  --format json
```
