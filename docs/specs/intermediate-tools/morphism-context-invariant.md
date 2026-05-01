# morphographen, contextgraphen, and invariantgraphen Contracts

This document defines the first implementable contract for three primary
HigherGraphen intermediate tools. These tools sit above the Higher Structure OS
packages and below domain products. They are CLI plus skill surfaces; MCP,
marketplace packaging, and provider-specific plugin manifests are out of scope.

## Scope

`morphographen` manages transformations between structures. It answers what is
preserved, lost, composed, or broken when one structure is translated into
another.

`contextgraphen` manages local meanings and gluing across contexts. It answers
where a model is valid, how it restricts to a local context, and whether local
sections can be assembled into a consistent global structure.

`invariantgraphen` manages properties that must survive changes, projections,
and workflow steps. It answers whether an operation preserved required
conditions and which counterexample demonstrates a violation.

## Conceptual Basis

The three tools share a common mathematical surface:

- Morphisms and composition for transformations.
- Functor-like mappings between interpreted domains.
- Contexts, covers, restrictions, sections, and gluing from sheaf-style
  modeling.
- Invariants, preservation checks, equivalence classes, and counterexamples.
- Information-loss declarations for any projection or quotient used by a view.

The tools should expose these concepts as explicit records instead of prose-only
diagnostics.

## Package And CLI Surface

Initial packages should be independently usable crates and CLI command groups:

- `morphographen`
- `contextgraphen`
- `invariantgraphen`

Suggested CLI groups:

```text
highergraphen morphism check --input <path> --format json
highergraphen morphism compose --input <path> --format json
highergraphen context glue --input <path> --format json
highergraphen context restrict --input <path> --context <id> --format json
highergraphen invariant check --input <path> --format json
```

Every command emits a versioned JSON report and keeps deterministic examples in
`schemas/reports/`.

## Core Dependencies

These tools depend on existing lower packages:

- `higher-graphen-core` for identifiers, provenance, confidence, review status,
  severity, and structured errors.
- `higher-graphen-structure::space` for spaces, cells, incidences, and complexes.
- `higher-graphen-structure::morphism` for mappings, composition, and preservation checks.
- `higher-graphen-reasoning::invariant` for invariant definitions, check inputs, and
  violations.
- `higher-graphen-reasoning::obstruction` for non-composability, non-gluability, and
  invariant violation explanations.
- `higher-graphen-projection` for audience-specific views with declared
  information loss.

## Input Contract

Inputs must be bounded JSON documents with explicit schemas. A document may
contain:

- Source and target spaces.
- Morphism definitions with source IDs, target IDs, mapping kind, and declared
  preservation expectations.
- Context definitions with local vocabulary, restrictions, and candidate
  gluing rules.
- Invariant definitions with scope, severity, target kind, and source IDs.
- Optional projections that must declare information loss.
- Provenance and confidence for facts extracted from documents or tools.

Inferred mappings, inferred gluing candidates, and inferred invariants must not
be accepted facts until a later explicit review workflow accepts them.

## Output Contract

Reports must include:

- `schema`, `report_type`, and `report_version`.
- The accepted input structure that was checked.
- A result status such as `preserved`, `not_preserved`, `composable`,
  `not_composable`, `gluable`, `not_gluable`, `satisfied`, or `violated`.
- Preservation results with preserved IDs, lost IDs, and unmapped IDs.
- Context results with restriction summaries and gluing diagnostics.
- Invariant results with violations and counterexample references.
- Obstructions for failed composition, failed gluing, or violated invariants.
- Projection views for human, AI-agent, and audit consumers.
- Source IDs and information-loss declarations for every view.

## Invariants

The implementation must enforce:

- A morphism cannot claim preservation without listing the relevant source and
  target IDs.
- Composition cannot silently ignore an unmatched intermediate mapping.
- A context gluing report cannot be successful if two local sections assign
  incompatible values to the same global object.
- An invariant violation must include a target, severity, and source evidence.
- Projection views cannot omit source IDs or declared information loss.
- Review status must remain `unreviewed` for inferred structure until an
  explicit completion review workflow changes it.

## Failure Modes

Expected failure modes are domain findings, not CLI crashes:

- `not_composable`: two morphisms cannot be composed because the intermediate
  structure does not align.
- `not_preserved`: a required invariant is not preserved by the transformation.
- `not_gluable`: local context sections are internally valid but globally
  inconsistent.
- `unsupported_input_schema`: the input version is not supported.
- `insufficient_provenance`: a mapping or invariant has no traceable source.
- `projection_loss_missing`: a view omitted required loss declarations.

Usage, parse, and unsupported-schema errors should exit nonzero. Domain findings
inside a successfully generated report should exit zero.

## Validation Expectations

Each implementation case must include:

- JSON schema fixtures for valid and invalid inputs.
- Runtime tests for preservation, composition, restriction, gluing, and
  invariant violation paths.
- CLI tests for stdout, `--output`, missing `--format json`, and unsupported
  arguments.
- Static-analysis checks through `sh scripts/static-analysis.sh`.
- Regression checks proving existing Architecture Product reports still pass.

## Composition Rules

`morphographen` produces preservation records that `invariantgraphen` can check.
`contextgraphen` produces gluing results that can become obstructions or
completion candidates. `invariantgraphen` must be able to consume both morphism
and context reports as check targets.

The three tools should share identifiers and report fragments rather than
duplicating semantics. If two tools view the same source structure at different
granularities, the projection layer must declare the lost detail.

## Non-Goals

The first implementation does not include:

- General theorem proving.
- Full sheaf-theoretic automation.
- Arbitrary programming-language migration analysis.
- Natural-language extraction beyond already lifted structure.
- MCP servers, marketplace publication, or provider-specific plugins.

## First Implementation Tasks

1. Add schema documents for morphism, context, and invariant tool inputs.
2. Add runtime report types with stable source IDs and loss declarations.
3. Implement a deterministic fixture where a requirement-to-design morphism
   loses an approval invariant.
4. Implement a deterministic context fixture where two local customer meanings
   fail to glue into one global customer.
5. Add CLI commands and tests for the deterministic fixtures.
6. Add skill guidance that tells agents to treat failed preservation or failed
   gluing as successful structural findings, not tool failures.
