# higher-graphen Release Surface

Read this before selecting release scope, bumping versions, drafting release notes, or publishing artifacts. If this inventory conflicts with `Cargo.toml`, `integrations/cli-skill-bundle/bundle.json`, or current repository files, the repository files are authoritative and this reference must be updated.

## Current Release Model

- The repository uses a single Cargo workspace version in root `Cargo.toml`.
- Current Cargo packages, including tools and examples, set `publish = false`; do not publish to crates.io unless release policy changes explicitly.
- Default release scope is the whole repository through a Git tag and optional GitHub Release.
- Optional release artifacts can include CLI binaries, schema archives, and the CLI skill bundle.
- `LICENSE`, `COMMERCIAL_BOUNDARY.md`, and public `.casegraphen/` traces are release-facing public artifacts.

## Rust Workspace Crates

These are versioned library crates in `crates/`. They are part of the repository release and workspace API surface, but are not currently registry-published:

- `higher-graphen-core`
- `higher-graphen-space`
- `higher-graphen-context`
- `higher-graphen-morphism`
- `higher-graphen-invariant`
- `higher-graphen-obstruction`
- `higher-graphen-completion`
- `higher-graphen-model-checking`
- `higher-graphen-abstract-interpretation`
- `higher-graphen-causal`
- `higher-graphen-confidence-model`
- `higher-graphen-topology`
- `higher-graphen-prover`
- `higher-graphen-projection`
- `higher-graphen-interpretation`
- `higher-graphen-runtime`

Before release, confirm the list with:

```sh
cargo metadata --locked --format-version 1 --no-deps
```

## CLI Tools

These tool packages are part of the release surface. They are not currently registry-published, but their binaries may be attached to a GitHub Release if the release plan includes binary artifacts:

- `tools/casegraphen` package `casegraphen`, binary `casegraphen`
- `tools/highergraphen-cli` package `highergraphen-cli`, binary `highergraphen`

If publishing binaries, run a release build in addition to the mandatory gate:

```sh
cargo build --workspace --release --locked
```

## Skill and Integration Bundles

These are releaseable agent integration surfaces:

- `integrations/cli-skill-bundle/bundle.json`
- `integrations/cli-skill-bundle/README.md`
- `integrations/cli-skill-bundle/check-bundle.py`
- `integrations/cli-skill-bundle/skills/highergraphen/SKILL.md`
- `integrations/cli-skill-bundle/skills/casegraphen/SKILL.md`
- `integrations/cli-skill-bundle/skills/casegraphen-ddd-diagnostics/SKILL.md`
- `integrations/cli-skill-bundle/skills/architecture-review/SKILL.md`
- `integrations/cli-skill-bundle/references/cli-contract.md`
- repository source skills under `skills/highergraphen`, `skills/casegraphen`, `skills/casegraphen-ddd-diagnostics`, and `skills/release-runner`

The CLI skill bundle has its own `version` in `bundle.json`. Decide explicitly whether a repository release also bumps the bundle version.

`skills/release-runner` is release process support. It is not part of the CLI skill bundle unless a future release explicitly adds it.

## Schemas, Fixtures, and Report Contracts

These directories are stable machine-readable contracts and examples. Include them in release notes when changed:

- `schemas/casegraphen/`
- `schemas/inputs/`
- `schemas/reports/`
- reference reports and fixtures under `examples/architecture/reference/`, `examples/feed/reference/`, and `examples/casegraphen/`

When schema IDs change or new report schemas are added, update direct schema files or `schemas/casegraphen/report-schema-aliases.json` and run the JSON contract validation gate.

## Example and Smoke Packages

These workspace members are validation examples, not separately published packages:

- `examples/architecture`, package `higher-graphen-architecture-smoke`
- `examples/feed`, package `higher-graphen-feed-example`

They must remain green through the workspace test gate because they validate release-facing reference workflows.

## Docs and CI Surfaces

Treat these as release-facing when behavior, CLI contracts, schemas, or packaging changes:

- `README.md`
- `LICENSE`
- `COMMERCIAL_BOUNDARY.md`
- `.casegraphen/README.md`
- public `.casegraphen/cases/`
- `docs/cli/highergraphen.md`
- `docs/specs/`
- `docs/product-packages/`
- `.github/workflows/release-quality.yml`
- `scripts/static-analysis.sh`
- `scripts/validate-cli-report-contract.py`
- `scripts/validate-json-contracts.py`

## Scope Checklist

For each release, state which of these are included:

- Git tag and GitHub Release
- Rust workspace source/API release
- CLI binaries for `casegraphen` and `highergraphen`
- CLI skill bundle
- JSON schemas, fixtures, and report contracts
- Documentation and reference examples
- License, commercial boundary, and public CaseGraphen traces
- Any explicitly out-of-scope package publication, marketplace publication, or provider-specific bundle
