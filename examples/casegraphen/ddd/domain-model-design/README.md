# DDD Domain Model Design Diagnostic Example

This retained fixture models a proposed Sales/Billing `Customer` unification
for HigherGraphen's DDD review workflow. It records the domain review structure
as a `CaseSpace` plus `MorphismLog`.

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

Create a HigherGraphen DDD input from the retained case-space fixture:

```sh
highergraphen ddd input from-case-space \
  --case-space examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json \
  --format json \
  --output ddd-review.input.json
```

Run the bounded review:

```sh
highergraphen ddd review --input ddd-review.input.json --format json
```

Current CaseGraphen-native commands and schemas live in
[`CAPHTECH/casegraphen`](https://github.com/CAPHTECH/casegraphen).

## Expected Findings

Expected report data after import:

- `space validate` returns `valid: true`.
- `space reason` returns `result.evaluation.status == "blocked"`.
- `obstruction list` includes:
  - `contradiction` for `relation:risk-blocks-unified-customer`;
  - `missing_evidence` for `evidence:customer-equivalence-proof`;
  - `review_required` for `review:domain-model-acceptance`.
- `completion candidates` includes
  `completion:missing-sales-billing-acl` and generated candidates for evidence,
  review, and contradiction resolution.
- `invariant check` separates accepted workshop evidence from unreviewed AI
  inference.
- `projection apply` reports information loss for requested projections.
- `invariant close-check` returns `closeable: false`.

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
