---
name: highergraphen-ddd
description: Use when an agent needs to run or interpret the bounded `highergraphen ddd` review workflow for domain model, bounded context, aggregate, anti-corruption mapping, evidence-boundary, projection-loss, or closeability diagnostics.
---

# HigherGraphen DDD Skill

Use this skill for the repository-owned `highergraphen ddd ...` workflow. This
skill replaces the old CaseGraphen-specific DDD diagnostic skill for product
workflow usage: DDD interpretation belongs in the bounded HigherGraphen DDD
workflow contract, not in `higher-graphen-core`.

## Source Of Truth

- CLI reference: `docs/cli/highergraphen.md`
- DDD review contract: `docs/specs/ddd-review-cli-contract.md`
- Input schema: `schemas/inputs/ddd-review.input.schema.json`
- Input fixture: `schemas/inputs/ddd-review.input.example.json`
- Report schema: `schemas/reports/ddd-review.report.schema.json`
- Report fixture: `schemas/reports/ddd-review.report.example.json`
- Legacy motivating example:
  `examples/casegraphen/ddd/domain-model-design/README.md`

Do not restate the schemas as competing contracts. Consume the contract,
fixtures, and CLI output.

## Commands

Convert a native CaseGraphen DDD case-space fixture into bounded DDD review
input:

```sh
cargo run -q -p highergraphen-cli -- \
  ddd input from-case-space \
  --case-space examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json \
  --format json \
  --output ddd-review.input.json
```

Run DDD review on bounded input:

```sh
cargo run -q -p highergraphen-cli -- \
  ddd review \
  --input schemas/inputs/ddd-review.input.example.json \
  --format json
```

Write a report:

```sh
cargo run -q -p highergraphen-cli -- \
  ddd review \
  --input schemas/inputs/ddd-review.input.example.json \
  --format json \
  --output ddd-review.report.json
```

Validate checked-in schema-bearing fixtures:

```sh
python3 scripts/validate-json-contracts.py
```

## Interpretation Rules

- Exit code `0` means the workflow emitted a report. Domain findings such as
  blocked decisions, missing accepted evidence, unreviewed completion
  candidates, projection loss, review gaps, and non-closeable state are
  successful report data, not CLI failures.
- Accepted source facts and AI-inferred claims must stay separated. Do not
  promote inferred equivalence proofs, generated diagnostics, or suggested
  mappings into accepted evidence unless an explicit review workflow does so.
- DDD-specific concepts such as bounded context, aggregate, domain event,
  invariant, anti-corruption mapping, and semantic-case risk are interpretation
  records over HigherGraphen structure. Do not add them to
  `higher-graphen-core`.
- Prefer `highergraphen ddd review` for product-facing DDD diagnostics. Use
  `casegraphen` only when the task is explicitly about native CaseGraphen
  CaseSpace/MorphismLog inspection, migration, or low-level case reasoning.

## Expected Report Signals

For the Sales/Billing Customer fixture, expect these evidence boundaries and
review signals:

- the unified Customer decision is blocked;
- Sales and Billing Customer identity collapse is represented as a boundary or
  semantic-loss risk;
- the equivalence proof remains inferred or unreviewed and does not satisfy
  accepted evidence requirements;
- a Sales-to-Billing anti-corruption mapping is proposed as a reviewable
  completion candidate;
- implementation-focused projection loss remains visible;
- closeability is false until hard obstructions, evidence gaps, review gaps,
  and projection-loss disclosure are resolved.
