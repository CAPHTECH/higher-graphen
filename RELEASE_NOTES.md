# Release Notes

## v0.2.0

Release scope:

- Repository release for the HigherGraphen Rust workspace at `0.2.0`.
- CLI binaries: `casegraphen` and `highergraphen`.
- Provider-neutral CLI skill bundle at `0.5.0`.
- JSON schemas, fixtures, examples, docs, repository-owned skills, public `.casegraphen` traces, license, and commercial boundary.

Highlights:

- Added deterministic PR review target workflows for deriving review inputs from Git changes and recommending structural review targets.
- Added abstract coverage, weighted coverage, topology, and persistent homology concepts to generic `higher-graphen-*` packages for reuse beyond PR review.
- Added structural boundary signals that catch parent module wiring and command dispatch changes during PR review target selection.
- Expanded CaseGraphen topology surfaces, including focused topology and topology diff reports for workflow and native case history.
- Updated docs, skills, schemas, fixtures, and the provider-neutral CLI skill bundle to reflect the current CLI integration surface.

Publication decisions:

- Cargo packages are not published to crates.io in this release because workspace packages currently set `publish = false`.
- Provider marketplace publication, MCP server publication, and provider-specific manifests remain out of scope.
- This release publishes a Git tag, GitHub Release, and Darwin arm64 binary artifacts after explicit maintainer approval.

## v0.1.0

Release scope:

- Repository release for the HigherGraphen Rust workspace at `0.1.0`.
- CLI binaries: `casegraphen` and `highergraphen`.
- Provider-neutral CLI skill bundle at `0.4.0`.
- JSON schemas, fixtures, examples, docs, repository-owned skills, public `.casegraphen` traces, license, and commercial boundary.

Highlights:

- Added higher-order topology and homology summaries for CaseGraphen graph, workflow, and native CaseSpace surfaces.
- Hardened release quality gates with workspace format/check/test/clippy/doc, static limits, CLI report validation, JSON schema contract validation, and CLI skill bundle validation.
- Added CI release-quality workflow.
- Added native CaseGraphen report schemas, workflow operation report schemas, and schema alias coverage.
- Preserved explicit review boundaries: proposal and generated completion flows keep inferred structures unreviewed until an explicit review action.
- Added `release-runner` as a repository-owned skill for release preparation through publication.

Publication decisions:

- Cargo packages are not published to crates.io in this release because workspace packages currently set `publish = false`.
- Provider marketplace publication, MCP server publication, and provider-specific manifests remain out of scope.
- Publishing a Git tag, GitHub Release, or binary artifacts still requires explicit maintainer approval at release time.
