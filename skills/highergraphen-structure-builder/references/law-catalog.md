# HigherGraphen Law Catalog

Use these laws as prompts for extracting `requirement:law:*` obligations. Keep
only laws relevant to the changed surface.

## Review Boundary

- `law:accepted-facts-remain-accepted`: accepted input facts may be lifted, but
  AI-generated candidates and obstructions remain unreviewed.
- `law:candidates-remain-unreviewed`: generated completion candidates must not
  be promoted without explicit review.
- `law:review-decision-is-separate`: accept/reject workflows emit separate
  auditable reports and do not mutate the source report.

## Test Gap

- `law:test-gap-is-bounded`: `no_gaps_in_snapshot` applies only to the supplied
  snapshot and detector policy.
- `law:verification-policy-controls-test-kind`: a test closes an obligation
  only when its observed `test_type` is accepted by `detector_context.test_kinds`.
- `law:coverage-status-controls-branch`: branch coverage closes a branch gap
  only when status is covered or partial and linked to an accepted test.
- `law:requirements-map-to-implementation-and-test`: in-scope requirements need
  implementation targets and accepted verification cells.
- `law:regressions-need-regression-verification`: bug-fix requirements require
  accepted regression verification.

## Schema And Serialization

- `law:schema-id-preserved`: emitted documents use the expected schema ID.
- `law:enum-casing-round-trips`: serialized enum values use the schema's casing
  and round-trip through runtime types.
- `law:fixtures-validate-against-schema`: checked-in examples validate against
  their declared schemas.
- `law:runtime-shapes-preserve-schema`: runtime structs preserve required schema
  fields, optional fields, and deny-unknown-field boundaries where applicable.

## CLI Semantics

- `law:command-routes-to-runner`: CLI command parsing dispatches to the intended
  runtime runner or input adapter.
- `law:json-format-required`: workflow commands require `--format json`.
- `law:output-file-suppresses-stdout`: `--output` writes a file and does not
  emit JSON to stdout.
- `law:input-from-git-is-deterministic`: git adapters use local git range data
  and deterministic path/structure rules.
- `law:input-from-git-does-not-prove-semantic-coverage`: git adapters do not
  execute tests, crawl the full repository, or infer full semantic coverage.
- `law:semantic-delta-is-explicit`: parsed base/head semantic cells expose
  preservation, addition, and deletion morphisms instead of hiding source
  changes behind file-level obligations.
- `law:semantic-delta-has-verification`: changed semantic delta morphisms
  require accepted verification cells under the detector policy.

## Projection And Information Loss

- `law:projection-declares-information-loss`: human, AI, and audit views must
  preserve source IDs and state what was omitted.
- `law:audit-trace-preserves-source-roles`: audit projections preserve stable
  source IDs, roles, review statuses, severity, and confidence.
- `law:ai-view-does-not-hide-review-status`: AI-oriented summaries must keep
  unreviewed status visible.
