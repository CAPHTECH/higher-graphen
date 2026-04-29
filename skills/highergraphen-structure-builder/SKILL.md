---
name: highergraphen-structure-builder
description: Use when an agent must turn code diffs, tests, schemas, or HigherGraphen workflow surfaces into high-order HG structure before judging test gaps, reviews, or completion candidates.
---

# HigherGraphen Structure Builder

Use this skill when the task asks whether tests, reviews, or implementation
coverage are sufficient for a HigherGraphen change. Do not start from "changed
file needs test". First construct the high-order structure that the change
creates or modifies.

This skill complements `highergraphen`: use `highergraphen` to run or interpret
CLI reports, and use this skill to decide whether the agent has built enough
HG structure before trusting a report or proposing missing tests.

## Core Rule

Represent the change as cells, incidences, morphisms, laws, verification cells,
obstructions, completion candidates, projections, and information loss.

Prefer this wording:

```text
changed morphism has no accepted verification cell
```

Avoid this wording unless no structure can be extracted:

```text
changed file has no test
```

## Workflow

1. Identify the changed surface.
   - CLI command
   - runtime runner
   - public export
   - workflow registry
   - schema
   - fixture
   - projection
   - review boundary
   - domain law
   - test or validation harness

2. Build cells.
   Create stable IDs for every meaningful structure. Examples:
   - `command:highergraphen:test-gap:detect`
   - `runner:test-gap:detect`
   - `schema:test-gap:input`
   - `fixture:test-gap:report-example`
   - `law:test-gap:candidates-remain-unreviewed`
   - `test:runtime:test-gap-contract`

3. Build incidences.
   Connect cells with directed relationships:
   - command supports runner
   - export supports runner
   - registry contains runner
   - runtime shape supports schema
   - fixture supports schema
   - projection preserves report shape
   - test verifies law or morphism

4. Build morphism obligations.
   Convert important incidences into requirements. Use IDs like:
   - `requirement:morphism:<surface>:<from>-to-<to>`
   - `requirement:law:<surface>:<law-name>`

5. Map verification cells.
   For each test, fixture validation, schema validation, or CLI smoke, list the
   exact cells, incidences, morphisms, and laws it verifies. Preserve observed
   test type; use `detector_context.test_kinds` as the verification policy.

6. Detect obstructions.
   A gap exists only when an in-scope morphism or law has no accepted
   verification cell under the current policy.

7. Report information loss.
   Always state what the structure does not prove: semantic branch coverage,
   full repository coverage, unparsed source bodies, path/name heuristic
   mappings, or any laws not extracted.

## Minimum Output

When reporting a structural assessment, include:

- cells created or reused;
- incidence edges created or reused;
- morphism or law obligations;
- verification cells and what each closes;
- remaining obstructions, if any;
- information loss and bounded scope.

## Law Catalog

Read [law-catalog.md](references/law-catalog.md) when the task touches runtime
behavior, schema/report contracts, projection, review status, test-gap
detection, or CLI semantics.

## Structure Patterns

Read [structure-patterns.md](references/structure-patterns.md) when extracting
structure from a git diff, designing `from-git` behavior, or deciding which
cells/incidences to create for HigherGraphen-owned surfaces.

## Safety Rules

- Do not accept AI-inferred cells as facts without source IDs.
- Do not treat completion candidates as accepted structure.
- Do not treat `no_gaps_in_snapshot` as global proof.
- Do not rewrite observed test type to fit a policy.
- Do not hide file-level fallback when no high-order structure can be built.
- Do not remove information-loss notes just because all extracted obligations
  are closed.
