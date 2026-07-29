# CLI Contract Reference

This bundle consumes the repository-owned HigherGraphen CLI report contract. It
does not define a competing schema or workflow.

## Commands

Architecture smoke report:

```sh
highergraphen architecture smoke direct-db-access --format json
```

Cargo form:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access --format json
```

Bounded test-gap detector:

```sh
highergraphen test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

highergraphen test-gap detect \
  --input test-gap.input.json \
  --format json
```

`highergraphen test-gap input from-git` creates a deterministic bounded
`highergraphen.test_gap.input.v1` snapshot from a local Git range. It does not
execute tests, crawl the full repository, prove typed semantic equivalence, or
accept missing-test candidates.

Bounded semantic proof certificate validation:

```sh
highergraphen semantic-proof verify \
  --input schemas/inputs/semantic-proof.input.example.json \
  --format json
```

`semantic-proof verify` validates supplied proof certificates and
counterexamples against theorem, law, morphism, backend, hash, and review
policy. It does not run external proof backends.

HigherGraphen DDD review:

```sh
highergraphen ddd input from-case-space \
  --case-space examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json \
  --format json \
  --output ddd-review.input.json

highergraphen ddd review \
  --input ddd-review.input.json \
  --format json
```

The retained case-space fixture is part of the HigherGraphen DDD test surface.
CaseGraphen’s own CLI, schemas, skills, and operator contract are maintained in
[`CAPHTECH/casegraphen`](https://github.com/CAPHTECH/casegraphen).

## Stable Files

| Surface | Path |
| --- | --- |
| CLI reference | `docs/cli/highergraphen.md` |
| Agent handoff | `docs/specs/agent-tooling-handoff.md` |
| Architecture report schema | `schemas/reports/architecture-direct-db-access-smoke.report.schema.json` |
| Architecture report fixture | `schemas/reports/architecture-direct-db-access-smoke.report.example.json` |
| Test-gap input schema | `schemas/inputs/test-gap.input.schema.json` |
| Test-gap input fixture | `schemas/inputs/test-gap.input.example.json` |
| Test-gap report schema | `schemas/reports/test-gap.report.schema.json` |
| Test-gap report fixture | `schemas/reports/test-gap.report.example.json` |
| DDD review contract | `docs/specs/ddd-review-cli-contract.md` |
| DDD review input schema | `schemas/inputs/ddd-review.input.schema.json` |
| DDD review report schema | `schemas/reports/ddd-review.report.schema.json` |
| Contract validator | `scripts/validate-cli-report-contract.py` |
| HigherGraphen source skill | `skills/highergraphen/SKILL.md` |
| HigherGraphen DDD skill | `skills/highergraphen-ddd/SKILL.md` |

## Required Semantics

- CLI exit code `0` means the workflow ran and emitted a report.
- `result.status == "violation_detected"` is successful report data.
- Test-gap statuses such as `gaps_detected` and `no_gaps_in_snapshot` are
  successful report data bounded to the supplied snapshot.
- The deterministic smoke report contains exactly one direct database access
  obstruction.
- Completion candidates remain `review_status: "unreviewed"` until an explicit
  review transition records a decision.
- Test-gap missing-test obstructions and candidates retain severity,
  confidence, source IDs, witnesses, suggested test shape, and projection
  `information_loss`.
- DDD boundary semantic loss, missing evidence, AI inference, unreviewed
  `semantic_case` records, and `evidence_boundary` findings are report data.

## Validation

Run:

```sh
python3 scripts/validate-cli-report-contract.py
```

To validate a report file:

```sh
python3 scripts/validate-cli-report-contract.py \
  --report architecture-direct-db-access-smoke.report.json
```
