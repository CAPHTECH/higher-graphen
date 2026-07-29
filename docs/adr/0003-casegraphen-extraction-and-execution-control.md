# ADR 0003: CaseGraphen Extraction And Execution Control

## Status

Accepted on 2026-07-30.

## Context

`tools/casegraphen` is the HigherGraphen intermediate tool for lifting bounded
source snapshots into case spaces, deriving obstructions and completions,
checking invariants, applying reviewed morphisms, and projecting lossy views.
Two documented boundaries constrained its evolution:

- `docs/specs/intermediate-tools/casegraphen.md` lists "Executing scenarios
  against external systems" as an explicit non-goal.
- `docs/specs/intermediate-tools/casegraphen-native-case-management.md` lists
  "Changing the external `/Users/rizumita/Workspace/casegraphen` repository"
  as an explicit non-goal.

The next planned capability is execution control: dispatching accepted work
items to external workers with side effects, validating worker-proposed state
transitions as morphisms, and committing verified changes as new revisions.
That capability requires exactly what the first non-goal forbids, and the
artifacts it produces (worker bindings, operational runbooks, credentials for
worker environments) are material `COMMERCIAL_BOUNDARY.md` says should not
live in this public repository.

The prior standalone implementation was renamed to
`CAPHTECH/casegraphen-legacy`, freeing the `casegraphen` repository name.

## Decision

1. `tools/casegraphen` moves to the standalone repository
   `CAPHTECH/casegraphen`, together with `schemas/casegraphen/**` and the
   reference fixtures its tests require. The casegraphen specs under
   `docs/specs/intermediate-tools/casegraphen*.md` transfer ownership to that
   repository; this repository keeps pointers.
2. The two non-goal clauses named above are amended: execution control against
   external systems becomes an explicit goal of the extracted repository, and
   the extracted repository is the place where that change happens. Inside
   this repository nothing executes external scenarios; the boundary moves,
   it does not silently erode.
3. Dependency direction is unchanged and remains one-way:
   `casegraphen -> published higher-graphen crates`. The extracted tool must
   not depend on `higher-graphen-runtime`, and no crate in this workspace may
   depend on the extracted repository. HigherGraphen may keep using a released
   `casegraphen` CLI binary for its own development management; that is not a
   Cargo or release dependency.
4. crates.io publication of the `casegraphen` crate transfers to the extracted
   repository starting at version 0.8.0, with the manifest `repository` field
   updated. Versions up to 0.7.x were published from this workspace.

## Consequences

- `tools/casegraphen` leaves the workspace members; `examples/architecture`
  drops its path dependency on it.
- `scripts/check-static-limits.py` and `scripts/validate-json-contracts.py`
  no longer cover casegraphen; equivalent gates live in the extracted
  repository.
- The AI-does-not-own-state invariants (generated structure stays unreviewed
  until explicit review, inferred evidence does not satisfy hard requirements,
  readiness is derived rather than stored) are inherited by the extracted
  repository as contract, not convention.
- Execution-control operational material (worker bindings, approval policies,
  runbooks) is kept out of this public repository, consistent with
  `COMMERCIAL_BOUNDARY.md`.
