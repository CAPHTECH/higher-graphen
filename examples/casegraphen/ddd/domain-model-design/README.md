# DDD Domain Model Design Diagnostic Example

This example shows how to use native CaseGraphen as a DDD design review
substrate with the `casegraphen-ddd-diagnostics` skill. The fixture models a
proposed Sales/Billing `Customer` unification and records the domain review
structure as a `CaseSpace` plus `MorphismLog`.

The example is intentionally small but it exercises the main diagnostic
signals:

- boundary semantic loss between bounded contexts;
- an AI-inferred proof that cannot satisfy an evidence requirement;
- a required domain model review that is still unaccepted;
- an unreviewed completion candidate for an anti-corruption mapping;
- projection loss in an implementation-focused view.

The fixture uses `semantic_case` records for design risks and
`evidence_boundary` metadata to keep source-backed evidence separate from
AI inference.

## Fixture

- `sales-billing-customer.case.space.json`

Important cells:

| Cell | Meaning |
| --- | --- |
| `context:sales` | Sales bounded context, where Customer means a prospect or deal participant. |
| `context:billing` | Billing bounded context, where Customer means the legal billing counterparty. |
| `decision:unified-customer-model` | Proposed shared Customer model across contexts. |
| `semantic_case:customer-identity-loss` | Boundary issue for collapsing the two Customer meanings. |
| `completion:missing-sales-billing-acl` | Candidate to add an explicit Sales-to-Billing mapping. |
| `evidence:customer-equivalence-proof` | AI-inferred and unreviewed equivalence proof. |
| `evidence:workshop-notes` | Accepted source-backed workshop evidence. |

## Run

Use a temporary native store:

```sh
cargo run -q -p casegraphen -- \
  case import \
  --store /tmp/casegraphen-ddd-store \
  --input examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json \
  --revision-id revision:ddd-sales-billing-imported \
  --format json
```

Inspect focused diagnostic views:

Command shorthand: `casegraphen case reason`,
`casegraphen case obstructions`, `casegraphen case completions`,
`casegraphen case evidence`, `casegraphen case project`, and
`casegraphen case close-check`.

```sh
cargo run -q -p casegraphen -- case validate --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
cargo run -q -p casegraphen -- case reason --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
cargo run -q -p casegraphen -- case obstructions --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
cargo run -q -p casegraphen -- case completions --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
cargo run -q -p casegraphen -- case evidence --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
cargo run -q -p casegraphen -- case project --store /tmp/casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
```

Run the close gate:

```sh
cargo run -q -p casegraphen -- \
  case close-check \
  --store /tmp/casegraphen-ddd-store \
  --case-space-id case_space:ddd-sales-billing-demo \
  --base-revision-id revision:ddd-sales-billing-imported \
  --validation-evidence-id evidence:workshop-notes \
  --format json
```

## Expected Findings

Expected report data after import:

- `case validate` returns `valid: true`.
- `case reason` returns `result.evaluation.status == "blocked"`.
- `case obstructions` includes:
  - `contradiction` for `relation:risk-blocks-unified-customer`;
  - `missing_evidence` for `evidence:customer-equivalence-proof`;
  - `review_required` for `review:domain-model-acceptance`.
- `case completions` includes
  `completion:missing-sales-billing-acl` and generated candidates for evidence,
  review, and contradiction resolution.
- `case evidence` separates accepted workshop evidence from unreviewed AI
  inference.
- `case project` reports information loss for `projection:implementation-view`.
- `case close-check` returns `closeable: false`.

These findings are successful domain report data. They do not mean the CLI
failed.

## Interpretation

The design should not be accepted as-is. The next domain modeling actions are:

1. Review whether Sales Customer and Billing Customer can be represented by a
   single model without losing legal-counterparty semantics.
2. Add an explicit boundary or anti-corruption mapping if the meanings differ.
3. Replace or promote the AI-inferred equivalence proof with source-backed,
   accepted evidence.
4. Record a human domain model review before treating the design as closeable.
